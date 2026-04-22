//! Governance V2 — modular on-chain governance wiring.
//!
//! Supersedes the legacy `pallet_governance` monolith (which is no longer wired into the
//! runtime). See `DESIGN.md` in the repository root for the architecture.

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    BoundedVec, parameter_types,
    sp_runtime::Perbill,
    traits::{AsEnsureOriginWithArg, ConstU32},
};
use frame_system::EnsureRoot;
use pallet_multi_collective::{
    Collective, CollectiveInfo, CollectiveInspect, CollectiveName, CollectivesInfo,
};
use pallet_referenda::{DecisionStrategy, Track, TrackInfo, TrackName, TracksInfo};
use scale_info::TypeInfo;
use subtensor_runtime_common::SetLike;

use crate::{
    AccountId, BlockNumber, MultiCollective, Preimage, Referenda, RuntimeCall, RuntimeOrigin,
    Scheduler, System,
};

/// Identifiers for each collective.
///
/// Adding a variant requires an exhaustive-match update in `SubtensorCollectives::collectives()`.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum CollectiveId {
    /// Accounts authorized to submit proposals.
    Proposers = 0,
    /// Triumvirate — 3 members; PassOrFail signed voting.
    Triumvirate = 1,
    /// Economic collective — top 16 validators by stake.
    Economic = 2,
    /// Building collective — top 16 subnet owners.
    Building = 3,
}

/// Voting scheme indicator — matched by voting pallets against their own `Scheme` constant.
#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub enum VotingScheme {
    /// Signed votes recorded directly. Implemented by `pallet-signed-voting`.
    Signed,
    /// bLSAG ring-signature anonymous votes. Stubbed for v1.
    Anonymous,
}

/// Voter/proposer set composed of one or two collectives, read at query time.
#[derive(Clone)]
pub enum MemberSet {
    One(CollectiveId),
    Two(CollectiveId, CollectiveId),
}

impl SetLike<AccountId> for MemberSet {
    fn contains(&self, account: &AccountId) -> bool {
        match self {
            MemberSet::One(id) => MultiCollective::is_member(*id, account),
            MemberSet::Two(a, b) => {
                MultiCollective::is_member(*a, account) || MultiCollective::is_member(*b, account)
            }
        }
    }

    /// Number of **unique** members. Computed via `members()` to stay consistent with it —
    /// both `len()` and `members().len()` must agree, otherwise the tally denominator
    /// (`total`) drifts from the actual eligible voter count.
    fn len(&self) -> u32 {
        self.members().len() as u32
    }

    /// Snapshot of **unique** members. In `MemberSet::Two`, the two collectives may overlap
    /// (e.g. a top-stake validator who is also a top-price subnet owner); such an account
    /// must count once, not twice — otherwise `total` inflates and thresholds become
    /// unreachable even at unanimous support.
    fn members(&self) -> sp_std::vec::Vec<AccountId> {
        match self {
            MemberSet::One(id) => MultiCollective::members_of(*id),
            MemberSet::Two(a, b) => {
                let mut v = MultiCollective::members_of(*a);
                for m in MultiCollective::members_of(*b) {
                    if !v.contains(&m) {
                        v.push(m);
                    }
                }
                v
            }
        }
    }
}

fn fixed_name(s: &str) -> [u8; 32] {
    let mut n = [0u8; 32];
    let bytes = s.as_bytes();
    let len = bytes.len().min(32);
    if let (Some(dst), Some(src)) = (n.get_mut(..len), bytes.get(..len)) {
        dst.copy_from_slice(src);
    }
    n
}

/// Static list of all collectives. Adding a `CollectiveId` variant forces an update here.
pub struct SubtensorCollectives;

impl CollectivesInfo<BlockNumber, CollectiveName> for SubtensorCollectives {
    type Id = CollectiveId;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, BlockNumber, CollectiveName>> {
        [
            Collective {
                id: CollectiveId::Proposers,
                info: CollectiveInfo {
                    name: fixed_name("proposers"),
                    min_members: 0,
                    max_members: Some(20),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Triumvirate,
                info: CollectiveInfo {
                    name: fixed_name("triumvirate"),
                    min_members: 0,
                    max_members: Some(3),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Economic,
                info: CollectiveInfo {
                    name: fixed_name("economic"),
                    min_members: 0,
                    max_members: Some(16),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Building,
                info: CollectiveInfo {
                    name: fixed_name("building"),
                    min_members: 0,
                    max_members: Some(16),
                    term_duration: None,
                },
            },
        ]
        .into_iter()
    }
}

/// Static track definitions (v1).
///
/// - Track 0: triumvirate review — Signed PassOrFail, 67% approve threshold.
/// - Track 1: collective oversight — Signed Adjustable, 75% fast-track / 51% reject.
///   (Anonymous voting postponed to a later release; track 1 uses Signed for now.)
pub struct SubtensorTracks;

impl TracksInfo<TrackName, AccountId, RuntimeCall, BlockNumber> for SubtensorTracks {
    type Id = u16;
    type ProposerSet = MemberSet;
    type VotingScheme = VotingScheme;
    type VoterSet = MemberSet;

    fn tracks()
    -> impl Iterator<Item = Track<Self::Id, TrackName, BlockNumber, MemberSet, MemberSet, VotingScheme>>
    {
        [
            Track {
                id: 0u16,
                info: TrackInfo {
                    name: fixed_name("triumvirate"),
                    proposer_set: MemberSet::One(CollectiveId::Proposers),
                    voter_set: MemberSet::One(CollectiveId::Triumvirate),
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: runtime_common::prod_or_fast!(50_400, 50),
                        // Use exact 2/3 rationals — `from_percent(67)` rounds to 670_000_000
                        // parts, while a 2-of-3 tally is `from_rational(2, 3)` =
                        // 666_666_666 parts. The latter would be `< threshold` and force
                        // a full 3/3 vote, contradicting DESIGN.md (worked example: 2/3 passes).
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                    },
                },
            },
            Track {
                id: 1u16,
                info: TrackInfo {
                    name: fixed_name("collective"),
                    proposer_set: MemberSet::One(CollectiveId::Proposers),
                    voter_set: MemberSet::Two(CollectiveId::Economic, CollectiveId::Building),
                    // Signed for now — Anonymous (bLSAG) is on the roadmap for a later release.
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::Adjustable {
                        // Max extra delay at 0% approval. Dev: ~45s (30 blocks × 1.5s).
                        // Prod: ~1h (300 blocks × 12s).
                        initial_delay: runtime_common::prod_or_fast!(300, 30),
                        fast_track_threshold: Perbill::from_percent(75),
                        reject_threshold: Perbill::from_percent(51),
                    },
                },
            },
        ]
        .into_iter()
    }
}

parameter_types! {
    pub const MaxCollectiveMembers: u32 = 20;
    pub const MaxVotesToClear: u32 = 100;
    pub const MaxAnonymousRingSize: u32 = 64;
    pub const AnonymousPowDifficulty: u32 = 16;
    pub const ReferendaMaxQueuedPerTrack: u32 = 50;
    /// Headroom over the largest possible voter set in v1 (Triumvirate=3, Economic+Building=32).
    pub const MaxSnapshotMembers: u32 = 64;
    pub SignedSchemeKind: VotingScheme = VotingScheme::Signed;
    pub AnonymousSchemeKind: VotingScheme = VotingScheme::Anonymous;
}

impl pallet_multi_collective::Config for crate::Runtime {
    type CollectiveId = CollectiveId;
    type Collectives = SubtensorCollectives;
    type AddOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type SwapOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type ResetOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type OnMembersChanged = ();
    type OnNewTerm = ();
    type MaxMembers = MaxCollectiveMembers;
}

impl pallet_signed_voting::Config for crate::Runtime {
    type Scheme = SignedSchemeKind;
    type Polls = Referenda;
    type MaxVotesToClear = MaxVotesToClear;
    type MaxSnapshotMembers = MaxSnapshotMembers;
}

impl pallet_anonymous_voting::Config for crate::Runtime {
    type Scheme = AnonymousSchemeKind;
    type Polls = Referenda;
    type PowDifficulty = AnonymousPowDifficulty;
    type MaxRingSize = MaxAnonymousRingSize;
}

impl pallet_referenda::Config for crate::Runtime {
    type RuntimeCall = RuntimeCall;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type CancelOrigin = EnsureRoot<AccountId>;
    type Tracks = SubtensorTracks;
    type BlockNumberProvider = System;
    type PollHooks = (crate::SignedVoting, crate::AnonymousVoting);
    type MaxQueued = ReferendaMaxQueuedPerTrack;
}

// Keep referenced to avoid unused-import warnings when only some items are used.
#[allow(dead_code)]
fn _ensure_types_used() {
    let _: BoundedVec<AccountId, ConstU32<3>> = BoundedVec::new();
    let _ = RuntimeOrigin::none();
}
