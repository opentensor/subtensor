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
pub struct Tally {
    ayes: u32,
    nays: u32,
    total: u32,
}

impl VoteTally for Tally {
    fn approval(&self) -> Perbill {
        Perbill::from_rational(self.ayes, self.total)
    }
    fn rejection(&self) -> Perbill {
        Perbill::from_rational(self.nays, self.total)
    }
    fn abstention(&self) -> Perbill {
        let voted = self.ayes.saturating_add(self.nays);
        Perbill::from_rational(self.total.saturating_sub(voted), self.total)
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
        type Polls: Polls<Self::AccountId, Tally = Tally>;

        type MaxVotesToClear: Get<u32>;
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
    pub type TallyOf<T: Config> = StorageMap<_, Twox64Concat, PollIndexOf<T>, Tally, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Voted {
            who: T::AccountId,
            poll_index: PollIndexOf<T>,
            approve: bool,
            tally: Tally,
        },

        VoteRemoved {
            who: T::AccountId,
            poll_index: PollIndexOf<T>,
            tally: Tally,
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
            T::Polls::on_tally_updated(poll_index, tally.clone());

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
            Self::ensure_part_of_voter_set(poll_index, &who)?;

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
            T::Polls::on_tally_updated(poll_index, tally.clone());

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
}

impl<T: Config> PollHooks<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_created(poll_index: PollIndexOf<T>) {
        let total = T::Polls::voter_set_of(poll_index)
            .map(|voter_set| voter_set.len())
            .unwrap_or(0);

        TallyOf::<T>::insert(
            poll_index,
            Tally {
                ayes: 0,
                nays: 0,
                total,
            },
        );
    }

    fn on_poll_completed(poll_index: PollIndexOf<T>) {
        let max = T::MaxVotesToClear::get().into();
        let _ = VotingFor::<T>::clear_prefix(poll_index, max, None);
        TallyOf::<T>::remove(poll_index);
    }
}
