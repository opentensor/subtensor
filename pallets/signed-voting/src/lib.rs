#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{pallet_prelude::*, sp_runtime::Perbill};
use frame_system::pallet_prelude::*;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;
pub use weights::WeightInfo;

mod benchmarking;
pub mod weights;

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PollIndexOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::Index;
type VotingSchemeOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::VotingScheme;

#[freeze_struct("769f064d4b346846")]
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, PartialEq, Eq, Clone, TypeInfo, Debug,
)]
pub struct SignedVoteTally {
    pub ayes: u32,
    pub nays: u32,
    pub total: u32,
}

impl From<SignedVoteTally> for VoteTally {
    fn from(t: SignedVoteTally) -> Self {
        let voted = t.ayes.saturating_add(t.nays);
        let abstention = t.total.saturating_sub(voted);
        VoteTally {
            approval: Perbill::from_rational(t.ayes, t.total),
            rejection: Perbill::from_rational(t.nays, t.total),
            abstention: Perbill::from_rational(abstention, t.total),
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The voting scheme this pallet handles.
        type Scheme: Get<VotingSchemeOf<Self>>;

        /// The referenda pallet. Provides poll queries and receives tally updates.
        type Polls: Polls<Self::AccountId>;

        /// Maximum number of votes to clear in a single `on_poll_completed` cleanup.
        #[pallet::constant]
        type MaxVotesToClear: Get<u32>;

        /// Upper bound on the number of voters captured in the per-poll snapshot. Must be
        /// greater than or equal to the largest possible voter set across all tracks using
        /// the Signed scheme; otherwise the snapshot would silently truncate.
        #[pallet::constant]
        type MaxSnapshotMembers: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
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

    /// Snapshot of eligible voters captured at `on_poll_created`. Membership checks for
    /// `vote`/`remove_vote` use this snapshot, **not** the live collective — so adding
    /// members to the collective mid-poll cannot fabricate `ayes` past `total`.
    #[pallet::storage]
    pub type VoterSnapshot<T: Config> = StorageMap<
        _,
        Twox64Concat,
        PollIndexOf<T>,
        BoundedVec<T::AccountId, T::MaxSnapshotMembers>,
        OptionQuery,
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
        #[pallet::weight(T::WeightInfo::vote())]
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
        #[pallet::weight(T::WeightInfo::remove_vote())]
        pub fn remove_vote(origin: OriginFor<T>, poll_index: PollIndexOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::Polls::is_ongoing(poll_index), Error::<T>::PollNotOngoing);
            Self::ensure_valid_voting_scheme(poll_index)?;
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
    pub(crate) fn try_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
        approve: bool,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate(poll_index, who, |vote| -> DispatchResult {
            match vote {
                Some(prev) => match (*prev, approve) {
                    (true, false) => {
                        tally.ayes = tally.ayes.saturating_sub(1);
                        tally.nays = tally.nays.saturating_add(1);
                    }
                    (false, true) => {
                        tally.nays = tally.nays.saturating_sub(1);
                        tally.ayes = tally.ayes.saturating_add(1);
                    }
                    _ => return Err(Error::<T>::DuplicateVote.into()),
                },
                None => {
                    if approve {
                        tally.ayes = tally.ayes.saturating_add(1);
                    } else {
                        tally.nays = tally.nays.saturating_add(1);
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

    pub(crate) fn try_remove_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate_exists(poll_index, who, |vote| -> DispatchResult {
            match vote {
                Some(prev) => {
                    if *prev {
                        tally.ayes = tally.ayes.saturating_sub(1);
                    } else {
                        tally.nays = tally.nays.saturating_sub(1);
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
        // Membership is checked against the frozen snapshot — NOT the live collective —
        // so that mid-poll additions to the collective can't fabricate eligible votes.
        let snapshot = VoterSnapshot::<T>::get(poll_index).ok_or(Error::<T>::PollNotFound)?;
        ensure!(snapshot.contains(who), Error::<T>::NotInVoterSet);
        Ok(())
    }

    fn scheme_matches(poll_index: PollIndexOf<T>) -> bool {
        T::Polls::voting_scheme_of(poll_index)
            .map(|s| s == T::Scheme::get())
            .unwrap_or(false)
    }
}

impl<T: Config> PollHooks<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_created(poll_index: PollIndexOf<T>) {
        if !Self::scheme_matches(poll_index) {
            return;
        }
        let voter_set = match T::Polls::voter_set_of(poll_index) {
            Some(v) => v,
            None => return,
        };
        // Snapshot members at this exact moment — basis of all subsequent eligibility
        // checks and the `total` denominator in the tally.
        let members = voter_set.members();
        let snapshot: BoundedVec<T::AccountId, T::MaxSnapshotMembers> =
            BoundedVec::try_from(members).unwrap_or_default();
        let total = snapshot.len() as u32;

        VoterSnapshot::<T>::insert(poll_index, snapshot);
        TallyOf::<T>::insert(
            poll_index,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total,
            },
        );
    }

    fn on_poll_completed(poll_index: PollIndexOf<T>) {
        if TallyOf::<T>::contains_key(poll_index) {
            let max = T::MaxVotesToClear::get();
            let _ = VotingFor::<T>::clear_prefix(poll_index, max, None);
            TallyOf::<T>::remove(poll_index);
            VoterSnapshot::<T>::remove(poll_index);
        }
    }
}
