#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{
    pallet_prelude::*,
    sp_runtime::{Perbill, Saturating},
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{OnPollCompleted, OnPollCreated, Polls, SetLike, VoteTally};

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PollIndexOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::Index;
type VotingSchemeOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::VotingScheme;

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, PartialEq, Eq, Clone, TypeInfo, Debug,
)]
#[subtensor_macros::freeze_struct("635a41a083f013e5")]
pub struct SignedVoteTally {
    pub ayes: u32,
    pub nays: u32,
    pub total: u32,
}

impl From<SignedVoteTally> for VoteTally {
    fn from(value: SignedVoteTally) -> Self {
        // Empty voter set → everyone implicitly abstains. Bypass
        // `Perbill::from_rational(_, 0)` which substrate returns as 100% and
        // would otherwise yield 300% total across approval+rejection+abstention.
        if value.total == 0 {
            return VoteTally::default();
        }
        let voted = value.ayes.saturating_add(value.nays);
        let abstention = value.total.saturating_sub(voted);
        VoteTally {
            approval: Perbill::from_rational(value.ayes, value.total),
            rejection: Perbill::from_rational(value.nays, value.total),
            abstention: Perbill::from_rational(abstention, value.total),
        }
    }
}

#[frame_support::pallet(dev_mode)]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Scheme: Get<VotingSchemeOf<Self>>;

        type Polls: Polls<Self::AccountId>;
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

    /// Per-poll tally. Doubles as the index of *active* polls — every
    /// poll has an entry between `on_poll_created` and `on_poll_completed`,
    /// and nowhere else. `remove_votes_for` iterates `TallyOf::iter_keys()`
    /// to find the polls a member voted on, so we don't need a parallel
    /// `ActivePolls` list. The cap on simultaneously-live polls comes from
    /// the `Polls` provider — `pallet-referenda::MaxQueued` in the runtime —
    /// which is the only producer of `on_poll_created` events.
    #[pallet::storage]
    pub type TallyOf<T: Config> =
        StorageMap<_, Twox64Concat, PollIndexOf<T>, SignedVoteTally, OptionQuery>;

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
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate(poll_index, who, |vote| -> DispatchResult {
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
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::PollNotFound)?;

        VotingFor::<T>::try_mutate_exists(poll_index, who, |vote| -> DispatchResult {
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
    ///
    /// `total` is intentionally left unchanged: the runtime is expected to
    /// replace departing voters via `swap_member` or `set_members`, which
    /// preserve voter-set size. The `outgoing`-only iteration in typical
    /// `OnMembersChanged` wiring (e.g. referenda's `VoteCleanup`) has no
    /// symmetric counterpart for incoming members, so decrementing `total`
    /// here would make the denominator diverge from the actual voter-set
    /// size on swap or set. Pure `remove_member` of a voter in an active
    /// poll is therefore a known operational limitation — leaves `total`
    /// stale (denominator too high, conservative for thresholds).
    pub fn remove_votes_for(who: &T::AccountId) {
        // Snapshot keys first: `T::Polls::on_tally_updated` could in
        // principle reach back into us via `on_poll_completed` (e.g. if
        // a vote-driven hook concluded the poll), and modifying a
        // storage map during iteration is unsafe. Today removal can
        // only *decrease* approval / rejection so no threshold gets
        // crossed downward, but we don't want correctness to depend on
        // that invariant holding through future hook changes.
        let polls: Vec<PollIndexOf<T>> = TallyOf::<T>::iter_keys().collect();
        for poll_index in polls {
            if let Some(approve) = VotingFor::<T>::take(poll_index, who)
                && let Some(mut tally) = TallyOf::<T>::get(poll_index)
            {
                if approve {
                    tally.ayes.saturating_dec();
                } else {
                    tally.nays.saturating_dec();
                }
                TallyOf::<T>::insert(poll_index, tally.clone());
                T::Polls::on_tally_updated(poll_index, &tally.clone().into());

                Self::deposit_event(Event::<T>::VoteInvalidated {
                    who: who.clone(),
                    poll_index,
                    tally,
                });
            }
        }
    }
}

impl<T: Config> OnPollCreated<PollIndexOf<T>> for Pallet<T> {
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
    }

    fn weight() -> Weight {
        Weight::zero()
    }
}

impl<T: Config> OnPollCompleted<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_completed(poll_index: PollIndexOf<T>) {
        // `u32::MAX` is effectively unbounded. `VotingFor` entries per poll
        // are bounded by the voter-set size, so one call clears everything.
        let _ = VotingFor::<T>::clear_prefix(poll_index, u32::MAX, None);
        TallyOf::<T>::remove(poll_index);
    }

    fn weight() -> Weight {
        Weight::zero()
    }
}
