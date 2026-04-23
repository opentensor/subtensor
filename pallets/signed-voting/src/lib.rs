#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    pallet_prelude::*,
    sp_runtime::{Perbill, Saturating},
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PollIndexOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::Index;
type VotingSchemeOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::VotingScheme;

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, PartialEq, Eq, Clone, TypeInfo, Debug,
)]
pub struct SignedVoteTally {
    pub ayes: u32,
    pub nays: u32,
    pub total: u32,
}

impl Into<VoteTally> for SignedVoteTally {
    fn into(self: SignedVoteTally) -> VoteTally {
        let voted = self.ayes.saturating_add(self.nays);
        let abstention = self.total.saturating_sub(voted);
        VoteTally {
            approval: Perbill::from_rational(self.ayes, self.total),
            rejection: Perbill::from_rational(self.nays, self.total),
            abstention: Perbill::from_rational(abstention, self.total),
        }
    }
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Scheme: Get<VotingSchemeOf<Self>>;

        type Polls: Polls<Self::AccountId>;

        type MaxVotesToClear: Get<u32>;

        /// Maximum number of active polls this pallet can track simultaneously.
        type MaxActivePolls: Get<u32>;
    }

    #[pallet::storage]
    pub type VotingFor<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        PollIndexOf<T>,
        Twox64Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type TallyOf<T: Config> =
        StorageMap<_, Twox64Concat, PollIndexOf<T>, SignedVoteTally, OptionQuery>;

    #[pallet::storage]
    pub type ActivePolls<T: Config> = StorageValue<
        _,
        BoundedVec<PollIndexOf<T>, T::MaxActivePolls>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Voted {
            who: T::AccountId,
            poll_index: PollIndexOf<T>,
            approve: bool,
            tally: SignedVoteTally,
        },

        VoteRemoved {
            who: T::AccountId,
            poll_index: PollIndexOf<T>,
            tally: SignedVoteTally,
        },

        VoteInvalidated {
            who: T::AccountId,
            poll_index: PollIndexOf<T>,
            tally: SignedVoteTally,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        PollNotOngoing,
        PollNotFound,
        InvalidVotingScheme,
        NotInVoterSet,
        DuplicateVote,
        VoteNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn vote(
            origin: OriginFor<T>,
            poll_index: PollIndexOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::Polls::is_ongoing(poll_index), Error::<T>::PollNotOngoing);
            Self::ensure_valid_voting_scheme(poll_index)?;
            Self::ensure_part_of_voter_set(poll_index, &who)?;

            let tally = Self::try_vote(poll_index, &who, approve)?;

            Self::deposit_event(Event::<T>::Voted {
                who,
                poll_index,
                approve,
                tally,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn remove_vote(origin: OriginFor<T>, poll_index: PollIndexOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::Polls::is_ongoing(poll_index), Error::<T>::PollNotOngoing);
            Self::ensure_valid_voting_scheme(poll_index)?;
            // TODO: blocks self-removal post-rotation
            Self::ensure_part_of_voter_set(poll_index, &who)?;

            let tally = Self::try_remove_vote(poll_index, &who)?;

            Self::deposit_event(Event::<T>::VoteRemoved {
                who,
                poll_index,
                tally,
            });
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn try_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
        approve: bool,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(&poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate(&poll_index, &who, |vote| -> DispatchResult {
            match vote {
                Some(vote) => match (vote, approve) {
                    (true, false) => {
                        tally.ayes.saturating_dec();
                        tally.nays.saturating_inc();
                    }
                    (false, true) => {
                        tally.nays.saturating_dec();
                        tally.ayes.saturating_inc();
                    }
                    _ => return Err(Error::<T>::DuplicateVote.into()),
                },
                None => {
                    if approve {
                        tally.ayes.saturating_inc();
                    } else {
                        tally.nays.saturating_inc();
                    }
                }
            }
            *vote = Some(approve);
            Ok(())
        })?;

        TallyOf::<T>::insert(poll_index, tally.clone());
        T::Polls::on_tally_updated(poll_index, &tally.clone().into());

        Ok(tally)
    }

    fn try_remove_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(&poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate_exists(&poll_index, &who, |vote| -> DispatchResult {
            match vote {
                Some(vote) => {
                    if *vote {
                        tally.ayes.saturating_dec();
                    } else {
                        tally.nays.saturating_dec();
                    }
                }
                None => return Err(Error::<T>::VoteNotFound.into()),
            }
            *vote = None;
            Ok(())
        })?;

        TallyOf::<T>::insert(poll_index, tally.clone());
        T::Polls::on_tally_updated(poll_index, &tally.clone().into());

        Ok(tally)
    }

    fn ensure_valid_voting_scheme(poll_index: PollIndexOf<T>) -> DispatchResult {
        let scheme = T::Polls::voting_scheme_of(poll_index).ok_or(Error::<T>::PollNotFound)?;
        ensure!(T::Scheme::get() == scheme, Error::<T>::InvalidVotingScheme);
        Ok(())
    }

    fn ensure_part_of_voter_set(poll_index: PollIndexOf<T>, who: &T::AccountId) -> DispatchResult {
        let voter_set = T::Polls::voter_set_of(poll_index).ok_or(Error::<T>::PollNotFound)?;
        ensure!(voter_set.contains(who), Error::<T>::NotInVoterSet);
        Ok(())
    }

    /// Remove all votes by `who` across all active polls, adjusting tallies.
    /// Called when a member is rotated out of a collective.
    pub fn remove_votes_for(who: &T::AccountId) {
        for poll_index in ActivePolls::<T>::get().iter() {
            if let Some(approve) = VotingFor::<T>::take(poll_index, who) {
                if let Some(mut tally) = TallyOf::<T>::get(poll_index) {
                    if approve {
                        tally.ayes.saturating_dec();
                    } else {
                        tally.nays.saturating_dec();
                    }
                    TallyOf::<T>::insert(poll_index, tally.clone());
                    T::Polls::on_tally_updated(*poll_index, &tally.clone().into());

                    Self::deposit_event(Event::<T>::VoteInvalidated {
                        who: who.clone(),
                        poll_index: *poll_index,
                        tally,
                    });
                }
            }
        }
    }
}

impl<T: Config> PollHooks<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_created(poll_index: PollIndexOf<T>) {
        let total = T::Polls::voter_set_of(poll_index)
            .map(|voter_set| voter_set.len())
            .unwrap_or(0);

        TallyOf::<T>::insert(
            poll_index,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total,
            },
        );

        // TODO: silent error
        ActivePolls::<T>::mutate(|polls| {
            let _ = polls.try_push(poll_index);
        });
    }

    fn on_poll_completed(poll_index: PollIndexOf<T>) {
        let max = T::MaxVotesToClear::get().into();
        // TODO: potential cursor loss and storage leak
        let _ = VotingFor::<T>::clear_prefix(poll_index, max, None);
        TallyOf::<T>::remove(poll_index);

        ActivePolls::<T>::mutate(|polls| {
            polls.retain(|idx| *idx != poll_index);
        });
    }
}
