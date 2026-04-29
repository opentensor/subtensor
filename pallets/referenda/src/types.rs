//! Type definitions for the referenda pallet.
//!
//! Split into a separate module so the pallet logic in `lib.rs` stays
//! focused on behavior. The runtime-facing trait [`TracksInfo`] and its
//! associated types live here; pallet-side aliases over `Config` follow at
//! the bottom of the file.

use frame_support::{
    pallet_prelude::*,
    sp_runtime::Perbill,
    traits::{
        Bounded, LockIdentifier,
        schedule::v3::{Anon as ScheduleAnon, TaskName},
    },
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{SetLike, VoteTally};

use crate::Config;

/// Maximum length of a track's display name.
pub const MAX_TRACK_NAME_LEN: usize = 32;

/// Fixed-width track name. Padded with zeros if shorter than the maximum.
pub type TrackName = [u8; MAX_TRACK_NAME_LEN];

/// Monotonic referendum identifier. Issued by `submit`.
pub type ReferendumIndex = u32;

/// Hash-keyed name used to identify a scheduler entry.
pub type ProposalTaskName = [u8; 32];

/// Lock identifier reserved by this pallet for any locks placed by the
/// voting layer on behalf of a referendum.
pub const REFERENDA_ID: LockIdentifier = *b"referend";

/// `PalletsOrigin` re-exported from the runtime for use in scheduler calls.
pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// The runtime call type used for proposed calls and the pallet's own
/// scheduled `advance_referendum` invocations.
pub type CallOf<T> = <T as Config>::RuntimeCall;

/// Bounded reference to a runtime call. Stored on-chain as the preimage
/// hash plus length; the actual call bytes live in the preimage pallet.
pub type BoundedCallOf<T> = Bounded<CallOf<T>, <T as frame_system::Config>::Hashing>;

/// Address type returned by anonymous scheduler entries. Currently unused
/// by the pallet logic but kept so runtimes can implement
/// [`Config::Scheduler`] with either the anon or named scheduler.
pub type ScheduleAddressOf<T> = <<T as Config>::Scheduler as ScheduleAnon<
    BlockNumberFor<T>,
    CallOf<T>,
    PalletsOriginOf<T>,
>>::Address;

/// The runtime's track table type.
pub type TracksOf<T> = <T as Config>::Tracks;

/// The id type used to identify tracks in the runtime configuration.
pub type TrackIdOf<T> =
    <TracksOf<T> as TracksInfo<TrackName, AccountIdOf<T>, CallOf<T>, BlockNumberFor<T>>>::Id;

/// The voting scheme tag carried on each track. The voting pallet uses it
/// to dispatch tally updates to the correct backend.
pub type VotingSchemeOf<T> = <TracksOf<T> as TracksInfo<
    TrackName,
    AccountIdOf<T>,
    CallOf<T>,
    BlockNumberFor<T>,
>>::VotingScheme;

/// The set of accounts allowed to vote on a track.
pub type VoterSetOf<T> =
    <TracksOf<T> as TracksInfo<TrackName, AccountIdOf<T>, CallOf<T>, BlockNumberFor<T>>>::VoterSet;

/// Convenience alias for [`ReferendumStatus`] specialized to the runtime.
pub type ReferendumStatusOf<T> =
    ReferendumStatus<AccountIdOf<T>, TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;

/// Convenience alias for [`ReferendumInfo`] specialized to the runtime.
pub type ReferendumInfoOf<T> =
    ReferendumInfo<AccountIdOf<T>, TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;

/// What a referendum proposes. Determined by the track's strategy at
/// submit time.
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum Proposal<Call> {
    /// A call to dispatch on approval. Used by `PassOrFail` tracks.
    Action(Call),
    /// A scheduled call whose timing is governed by votes. Used by
    /// `Adjustable` tracks. The actual call lives on the scheduler under
    /// the referendum's `task_name`; the proposal carries no payload.
    Review,
}

/// How a track decides outcomes for the referenda filed against it.
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum DecisionStrategy<TrackId, BlockNumber> {
    /// Binary decision before a deadline. The referendum is approved if
    /// `tally.approval` reaches `approve_threshold`, rejected if
    /// `tally.rejection` reaches `reject_threshold`, and expired if neither
    /// happens by `submitted + decision_period`. On approval, the action
    /// in `on_approval` runs.
    PassOrFail {
        /// Number of blocks after submission within which a decision must
        /// be reached. Past this point the referendum expires.
        decision_period: BlockNumber,
        /// Approval ratio needed to pass.
        approve_threshold: Perbill,
        /// Rejection ratio needed to fail.
        reject_threshold: Perbill,
        /// What to do once the proposal is approved.
        on_approval: ApprovalAction<TrackId>,
    },
    /// Timing decision over a call already scheduled at submit time. The
    /// call runs after `initial_delay` by default. Voters can fast-track,
    /// cancel, or shift the dispatch time via linear interpolation between
    /// those extremes (target moves earlier as approval rises, never later).
    Adjustable {
        /// Default delay between submission and dispatch.
        initial_delay: BlockNumber,
        /// Approval ratio at which the task is rescheduled to next block
        /// and the referendum concludes as `FastTracked`.
        fast_track_threshold: Perbill,
        /// Rejection ratio at which the scheduled task is cancelled and the
        /// referendum concludes as `Cancelled`.
        cancel_threshold: Perbill,
    },
}

/// What happens when a `PassOrFail` referendum is approved.
#[derive(Clone, Debug, PartialEq, Eq, TypeInfo)]
pub enum ApprovalAction<TrackId> {
    /// Schedule the call for next-block dispatch on this referendum's index.
    Execute,
    /// Hand the call off to a fresh `Adjustable` referendum on `track`.
    /// The parent concludes as `Delegated` and the new referendum drives
    /// the rest of the lifecycle.
    Review {
        /// Target track for the review referendum. Must be `Adjustable`;
        /// validated by [`Pallet::integrity_test`].
        track: TrackId,
    },
}

/// Per-track configuration carried in the runtime.
#[derive(Clone, Debug)]
pub struct TrackInfo<TrackId, Name, BlockNumber, ProposerSet, VoterSet, VotingScheme> {
    /// Display name. Padded to fixed width.
    pub name: Name,
    /// Set of accounts allowed to submit referenda on this track. `None`
    /// means the track is currently closed to new submissions; existing
    /// referenda continue their lifecycle normally.
    pub proposer_set: Option<ProposerSet>,
    /// Voting scheme tag. Used by the voting layer to route tally updates.
    pub voting_scheme: VotingScheme,
    /// Set of accounts entitled to vote on referenda on this track.
    pub voter_set: VoterSet,
    /// How outcomes are decided on this track.
    pub decision_strategy: DecisionStrategy<TrackId, BlockNumber>,
}

/// A track entry in the runtime track table. Pairs an id with its
/// configuration.
#[derive(Clone, Debug)]
pub struct Track<Id, Name, BlockNumber, ProposerSet, VoterSet, VotingScheme> {
    /// Stable id used to reference this track from referenda and from
    /// `ApprovalAction::Review { track }`.
    pub id: Id,
    /// Track configuration.
    pub info: TrackInfo<Id, Name, BlockNumber, ProposerSet, VoterSet, VotingScheme>,
}

/// Runtime configuration of available tracks. Implementors define the
/// available tracks at compile time; the pallet queries this trait at
/// submit time and during state-machine evaluation.
pub trait TracksInfo<Name, AccountId, Call, BlockNumber> {
    /// Stable identifier for a track.
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;
    /// Set of accounts allowed to submit referenda.
    type ProposerSet: SetLike<AccountId>;
    /// Voting scheme tag carried on each track.
    type VotingScheme: PartialEq;
    /// Set of accounts entitled to vote.
    type VoterSet: SetLike<AccountId>;

    /// Iterate over every track defined in the runtime.
    fn tracks() -> impl Iterator<
        Item = Track<
            Self::Id,
            Name,
            BlockNumber,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
    >;

    /// Iterate over the ids of every defined track.
    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|x| x.id)
    }

    /// Look up the configuration for a single track id.
    fn info(
        id: Self::Id,
    ) -> Option<
        TrackInfo<
            Self::Id,
            Name,
            BlockNumber,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
    > {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    /// Optional per-track authorization of a proposed call. Defaults to
    /// allow-all. Runtimes can override to filter calls based on track.
    fn authorize_proposal(
        _track_info: &TrackInfo<
            Self::Id,
            Name,
            BlockNumber,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
        _call: &Call,
    ) -> bool {
        true
    }

    /// Validate the runtime track table once at startup.
    ///
    /// Returns `Err` with a static message if either invariant is broken:
    ///
    /// 1. Track ids are unique. Lookups by id silently pick the first
    ///    match, so duplicates would mask later entries.
    /// 2. Every `ApprovalAction::Review { track }` references a track
    ///    that exists and uses the `Adjustable` strategy. Otherwise an
    ///    approval that delegates would either find no track or hand off
    ///    to a track that cannot model a review.
    fn check_integrity() -> Result<(), &'static str> {
        let tracks: alloc::vec::Vec<_> = Self::tracks().collect();

        let mut ids: alloc::vec::Vec<_> = tracks.iter().map(|t| t.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        if ids.len() != total {
            return Err("track ids must be unique");
        }

        for track in &tracks {
            if let DecisionStrategy::PassOrFail {
                on_approval:
                    ApprovalAction::Review {
                        track: review_track,
                    },
                ..
            } = &track.info.decision_strategy
            {
                let referenced = Self::info(*review_track)
                    .ok_or("ApprovalAction::Review references unknown track")?;
                if !matches!(
                    referenced.decision_strategy,
                    DecisionStrategy::Adjustable { .. }
                ) {
                    return Err("ApprovalAction::Review target track must be Adjustable");
                }
            }
        }

        Ok(())
    }
}

/// Per-referendum data captured at submit time and updated as votes arrive.
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub struct ReferendumInfo<AccountId, TrackId, Call, BlockNumber> {
    /// Track this referendum was filed against.
    pub track: TrackId,
    /// What this referendum proposes.
    pub proposal: Proposal<Call>,
    /// The signed account that submitted the referendum.
    pub proposer: AccountId,
    /// Block at which the referendum was submitted. Used to anchor
    /// timing computations in `Adjustable` strategies.
    pub submitted: BlockNumber,
    /// Latest tally observed from the voting pallet.
    pub tally: VoteTally,
}

/// Lifecycle status of a referendum. Each terminal variant carries the
/// block number at which it was reached.
#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum ReferendumStatus<AccountId, Id, Call, BlockNumber> {
    /// Voting is in progress.
    Ongoing(ReferendumInfo<AccountId, Id, Call, BlockNumber>),
    /// Approval threshold reached on a `PassOrFail` track. The call has
    /// been scheduled for dispatch on this referendum's index. Transitions
    /// to [`Enacted`](Self::Enacted) once the scheduled task has run.
    Approved(BlockNumber),
    /// Approval reached with `ApprovalAction::Review`. The call now lives
    /// on a fresh referendum on the configured review track; this index
    /// is a terminal audit trail.
    Delegated(BlockNumber),
    /// Rejection threshold reached on a `PassOrFail` track.
    Rejected(BlockNumber),
    /// Decision period elapsed without crossing approve or reject
    /// thresholds.
    Expired(BlockNumber),
    /// Fast-track threshold reached on an `Adjustable` track. The
    /// scheduled task was rescheduled to next block. Transitions to
    /// [`Enacted`](Self::Enacted).
    FastTracked(BlockNumber),
    /// Cancel threshold reached on an `Adjustable` track. The scheduled
    /// task was cancelled.
    Cancelled(BlockNumber),
    /// The referendum's call has been dispatched. Terminal.
    Enacted(BlockNumber),
    /// Terminated by [`Config::KillOrigin`](crate::Config::KillOrigin)
    /// before reaching a vote-driven outcome.
    Killed(BlockNumber),
}

/// Stable scheduler name for a referendum's enactment task.
pub fn task_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "enactment", index).using_encoded(sp_io::hashing::blake2_256)
}

/// Stable scheduler name for a referendum's alarm.
pub fn alarm_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "alarm", index).using_encoded(sp_io::hashing::blake2_256)
}
