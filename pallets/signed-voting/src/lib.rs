#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    pallet_prelude::*,
    sp_runtime::{Perbill, Saturating},
    weights::WeightMeter,
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{OnPollCompleted, OnPollCreated, Polls, SetLike, VoteTally};

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PollIndexOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::Index;
type VotingSchemeOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::VotingScheme;

/// Raw counts of votes cast on a poll. Converted to the producer's
/// `VoteTally` (Perbill ratios) on every tally update; storing counts
/// on-chain keeps the math exact and makes the `Voted` event payload
/// directly auditable.
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, PartialEq, Eq, Clone, TypeInfo, Debug,
)]
#[subtensor_macros::freeze_struct("523f104c4bf2ada2")]
pub struct SignedVoteTally {
    /// Aye votes cast so far.
    pub ayes: u32,
    /// Nay votes cast so far.
    pub nays: u32,
    /// Size of the voter-set snapshot at poll creation. The denominator
    /// for `approval` / `rejection` / `abstention` ratios; fixed for
    /// the poll's lifetime so thresholds cannot shift mid-poll.
    pub total: u32,
}

impl From<SignedVoteTally> for VoteTally {
    // Empty voter set: everyone implicitly abstains.
    fn from(value: SignedVoteTally) -> Self {
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

/// Resume cursor returned by `clear_prefix` and persisted across idle
/// blocks so a poll's cleanup can span multiple drain passes without
/// re-iterating already-removed entries.
pub type CleanupCursorOf<T> = BoundedVec<u8, <T as Config>::CleanupCursorMaxLen>;

#[frame_support::pallet]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Scheme: Get<VotingSchemeOf<Self>>;

        type Polls: Polls<Self::AccountId>;

        /// Upper bound on the size of any track's voter set, used as the
        /// storage bound for [`VoterSetOf`]. Must be ≥ the largest set
        /// the runtime can produce via [`Polls::voter_set_of`]; runtimes
        /// should derive it from their collective `max_members`.
        #[pallet::constant]
        type MaxVoterSetSize: Get<u32>;

        /// Maximum number of polls that can sit in [`PendingCleanup`] at
        /// once. Should be ≥ the [`Polls`] provider's cap on
        /// simultaneously active polls; a smaller bound risks rejecting
        /// cleanup work and leaking storage.
        #[pallet::constant]
        type MaxPendingCleanup: Get<u32>;

        /// Number of `VotingFor` entries cleared per [`Hooks::on_idle`]
        /// drain step. Tunes the trade-off between idle-block weight cost
        /// and the latency of fully draining a completed poll.
        #[pallet::constant]
        type CleanupChunkSize: Get<u32>;

        /// Storage bound on the resume cursor. The cursor is a partial
        /// trie key whose length depends on the storage layout; expose
        /// the bound as a constant so it shows up in metadata. 128 is
        /// comfortable for any `(poll, account)` shape.
        #[pallet::constant]
        type CleanupCursorMaxLen: Get<u32>;

        type WeightInfo: WeightInfo;

        /// Benchmark setup hook. The runtime supplies an ongoing poll
        /// index whose voting scheme matches `Self::Scheme::get()`.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: crate::benchmarking::BenchmarkHelper<Self>;
    }

    /// Per-`(poll, voter)` vote direction. `true` is an aye, `false` a
    /// nay; absence means the voter has not cast a vote on this poll.
    /// Drained lazily by `on_idle` after `on_poll_completed` enqueues
    /// the poll for cleanup.
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

    /// Per-poll tally. Doubles as the index of *active* polls: every
    /// poll has an entry between `on_poll_created` and `on_poll_completed`,
    /// and nowhere else. The cap on simultaneously-live polls comes from
    /// the [`Polls`] provider, which is the only producer of
    /// `on_poll_created` events.
    #[pallet::storage]
    pub type TallyOf<T: Config> =
        StorageMap<_, Twox64Concat, PollIndexOf<T>, SignedVoteTally, OptionQuery>;

    /// Voter-set snapshot taken at `on_poll_created` and used as the
    /// authoritative eligibility roster for the poll's lifetime. Frozen
    /// at creation: members rotated in or out of the underlying collective
    /// during the poll do not change who can vote here. Cleared by
    /// `on_poll_completed` alongside `TallyOf`.
    #[pallet::storage]
    pub type VoterSetOf<T: Config> = StorageMap<
        _,
        Twox64Concat,
        PollIndexOf<T>,
        BoundedVec<T::AccountId, T::MaxVoterSetSize>,
        OptionQuery,
    >;

    /// FIFO queue of polls awaiting `VotingFor` cleanup. `on_poll_completed`
    /// pushes to the back; `on_idle` drains from the front in chunks of
    /// `T::CleanupChunkSize`. The optional cursor lets a poll's cleanup
    /// span multiple idle blocks without re-iterating already-removed
    /// entries.
    #[pallet::storage]
    pub type PendingCleanup<T: Config> = StorageValue<
        _,
        BoundedVec<(PollIndexOf<T>, Option<CleanupCursorOf<T>>), T::MaxPendingCleanup>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A vote was cast or changed.
        Voted {
            /// Account that cast the vote.
            who: T::AccountId,
            /// Poll the vote was cast on.
            poll_index: PollIndexOf<T>,
            /// `true` for an aye, `false` for a nay.
            approve: bool,
            /// Tally after applying the vote.
            tally: SignedVoteTally,
        },

        /// A previously-cast vote was withdrawn.
        VoteRemoved {
            /// Account that withdrew the vote.
            who: T::AccountId,
            /// Poll the vote was withdrawn from.
            poll_index: PollIndexOf<T>,
            /// Tally after the vote was removed.
            tally: SignedVoteTally,
        },

        /// A poll concluded but the cleanup queue was already full, so
        /// its per-voter records were left in storage. The records do
        /// not affect correctness but will not be reclaimed unless the
        /// queue cap is raised. Indicates a runtime misconfiguration
        /// where the cap is smaller than the maximum number of polls
        /// that can complete simultaneously.
        CleanupQueueFull {
            /// Poll whose per-voter records were not enqueued.
            poll_index: PollIndexOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The poll either never existed or has already concluded.
        PollNotOngoing,
        /// No poll with this index is registered.
        PollNotFound,
        /// This poll uses a different voting scheme.
        InvalidVotingScheme,
        /// The caller is not eligible to vote on this poll.
        NotInVoterSet,
        /// The caller has already cast a vote in the same direction.
        DuplicateVote,
        /// The caller has not cast a vote on this poll.
        VoteNotFound,
        /// The poll's voter-set snapshot is missing. The poll is
        /// reported as ongoing but its eligibility roster was never
        /// recorded or has been cleared early. Internal inconsistency
        /// that should be unreachable in production.
        VoterSetMissing,
        /// The poll's tally is missing. The poll is reported as ongoing
        /// but its tally was never recorded or has been cleared early.
        /// Internal inconsistency that should be unreachable in
        /// production.
        TallyMissing,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // `on_poll_completed` only enqueues per-voter cleanup; this
        // hook is what actually frees the storage. Spreading the work
        // across idle blocks keeps the synchronous completion path
        // O(1) regardless of voter-set size.
        fn on_idle(_n: BlockNumberFor<T>, remaining: Weight) -> Weight {
            Pallet::<T>::drain_pending_cleanup(remaining)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Cast or change a vote on an ongoing poll. Calling again with
        /// the opposite direction flips the vote and updates the tally;
        /// calling with the same direction is rejected as a duplicate.
        ///
        /// The caller must be in the poll's voter-set snapshot taken at
        /// creation; eligibility is not affected by membership changes
        /// after the poll started.
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::vote(T::MaxVoterSetSize::get())
                .saturating_add(T::Polls::on_tally_updated_weight())
        )]
        pub fn vote(
            origin: OriginFor<T>,
            poll_index: PollIndexOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::Polls::is_ongoing(poll_index), Error::<T>::PollNotOngoing);
            Self::ensure_valid_voting_scheme(poll_index)?;
            Self::ensure_in_voter_set(poll_index, &who)?;

            let tally = Self::try_vote(poll_index, &who, approve)?;

            Self::deposit_event(Event::<T>::Voted {
                who,
                poll_index,
                approve,
                tally,
            });
            Ok(())
        }

        /// Withdraw a previously-cast vote on an ongoing poll. The
        /// tally is rolled back as if the caller had never voted, and
        /// the caller may cast a new vote afterwards.
        #[pallet::call_index(1)]
        #[pallet::weight(
            T::WeightInfo::remove_vote(T::MaxVoterSetSize::get())
                .saturating_add(T::Polls::on_tally_updated_weight())
        )]
        pub fn remove_vote(origin: OriginFor<T>, poll_index: PollIndexOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(T::Polls::is_ongoing(poll_index), Error::<T>::PollNotOngoing);
            Self::ensure_valid_voting_scheme(poll_index)?;
            Self::ensure_in_voter_set(poll_index, &who)?;

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
    // Apply a fresh or flipped vote to the tally and persist the
    // direction. The match arms cover the three reachable states:
    // first vote, flip aye/nay, and the rejected duplicate.
    fn try_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
        approve: bool,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::TallyMissing)?;

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

    // Roll back the caller's vote and clear their `VotingFor` entry.
    // The tally counter to decrement is decided by the stored direction,
    // not by anything the caller passes in.
    fn try_remove_vote(
        poll_index: PollIndexOf<T>,
        who: &T::AccountId,
    ) -> Result<SignedVoteTally, DispatchError> {
        let mut tally = TallyOf::<T>::get(poll_index).ok_or(Error::<T>::TallyMissing)?;

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

    // The producer can host multiple voting backends keyed by scheme;
    // refuse polls owned by another backend so their tallies can't be
    // mutated through this pallet.
    fn ensure_valid_voting_scheme(poll_index: PollIndexOf<T>) -> DispatchResult {
        let scheme = T::Polls::voting_scheme_of(poll_index).ok_or(Error::<T>::PollNotFound)?;
        ensure!(T::Scheme::get() == scheme, Error::<T>::InvalidVotingScheme);
        Ok(())
    }

    // O(log n) thanks to the snapshot being sorted at `on_poll_created`.
    // The sort cost is paid once; eligibility is read on every vote.
    fn ensure_in_voter_set(poll_index: PollIndexOf<T>, who: &T::AccountId) -> DispatchResult {
        let voter_set = VoterSetOf::<T>::get(poll_index).ok_or(Error::<T>::VoterSetMissing)?;
        voter_set
            .binary_search(who)
            .map_err(|_| Error::<T>::NotInVoterSet)?;
        Ok(())
    }

    // Drains the head of `PendingCleanup` in `CleanupChunkSize` chunks
    // until either the queue is empty or the meter is exhausted. A poll
    // stays at the head until `clear_prefix` returns no resume cursor,
    // at which point its prefix is empty and it is popped.
    //
    // The queue is read once and written once. The entry budget covers
    // both atomically: we will not read the queue if we cannot also
    // afford to write any progress back. Mutation between iterations
    // happens in memory.
    fn drain_pending_cleanup(remaining: Weight) -> Weight {
        let chunk = T::CleanupChunkSize::get();
        if chunk == 0 {
            return Weight::zero();
        }
        let per_step = T::WeightInfo::idle_cleanup_chunk(chunk);
        let entry_cost = T::DbWeight::get().reads_writes(1, 1);
        let body_cost = per_step.saturating_sub(entry_cost);
        let mut meter = WeightMeter::with_limit(remaining);

        if meter.try_consume(entry_cost).is_err() {
            return meter.consumed();
        }
        let mut queue = PendingCleanup::<T>::get();
        if queue.is_empty() {
            return meter.consumed();
        }

        let mut dirty = false;
        loop {
            if meter.try_consume(body_cost).is_err() {
                break;
            }
            let Some((poll, prev_cursor)) = queue.first().cloned() else {
                break;
            };
            let result = VotingFor::<T>::clear_prefix(
                poll,
                chunk,
                prev_cursor.as_ref().map(|c| c.as_slice()),
            );
            match result.maybe_cursor {
                None => {
                    if !queue.is_empty() {
                        let _ = queue.remove(0);
                    }
                }
                Some(c) => {
                    // If the cursor exceeds `CleanupCursorMaxLen`, drop it:
                    // the next pass restarts the prefix and re-iterates
                    // already-removed entries: slower but correct.
                    let bounded = BoundedVec::<u8, T::CleanupCursorMaxLen>::try_from(c).ok();
                    if let Some(head) = queue.iter_mut().next() {
                        *head = (poll, bounded);
                    }
                }
            }
            dirty = true;
            if queue.is_empty() {
                break;
            }
        }

        if dirty {
            PendingCleanup::<T>::put(queue);
        }
        meter.consumed()
    }
}

impl<T: Config> OnPollCreated<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_created(poll_index: PollIndexOf<T>) {
        // Sort once so `ensure_in_voter_set` can use `binary_search`.
        // `SetLike::to_vec` doesn't guarantee ordering, and the snapshot
        // is read on every vote, so paying the sort once is worth it.
        //
        // A `None` from the producer or a set bigger than
        // `MaxVoterSetSize` collapses to an empty snapshot. With
        // `total = 0` every threshold fails closed and the poll lapses
        // through its timeout: a safe failure mode if a misconfigured
        // runtime ever reaches this path.
        let snapshot: BoundedVec<T::AccountId, T::MaxVoterSetSize> =
            T::Polls::voter_set_of(poll_index)
                .map(|s| {
                    let mut v = s.to_vec();
                    v.sort();
                    v
                })
                .and_then(|v| BoundedVec::try_from(v).ok())
                .unwrap_or_default();

        let total = snapshot.len() as u32;
        VoterSetOf::<T>::insert(poll_index, snapshot);
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
        T::WeightInfo::on_poll_created()
    }
}

impl<T: Config> OnPollCompleted<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_completed(poll_index: PollIndexOf<T>) {
        // Keep this path O(1): the `VotingFor` prefix grows with voter
        // count, so clearing it synchronously would put unbounded work
        // on the producer's call. `on_idle` drains it instead.
        TallyOf::<T>::remove(poll_index);
        VoterSetOf::<T>::remove(poll_index);

        let pushed = PendingCleanup::<T>::mutate(|q| q.try_push((poll_index, None)).is_ok());
        if !pushed {
            // Don't fail the hook on overflow: that would tear down the
            // producer's call. The orphaned `VotingFor` entries are a
            // storage leak (unread after `TallyOf` is gone), not a
            // correctness issue; the event surfaces the misconfiguration.
            Self::deposit_event(Event::<T>::CleanupQueueFull { poll_index });
        }
    }

    fn weight() -> Weight {
        T::WeightInfo::on_poll_completed()
    }
}
