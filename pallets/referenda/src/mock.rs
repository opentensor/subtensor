#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::expect_used
)]

use core::cell::RefCell;

use frame_support::{derive_impl, pallet_prelude::*, parameter_types, traits::EqualPrivilegeOnly};
use frame_system::{EnsureRoot, limits};
use sp_core::U256;
use sp_runtime::{BuildStorage, Perbill, traits::IdentityLookup};
use subtensor_runtime_common::pad_name;

use crate::{self as pallet_referenda, *};
use pallet_multi_collective::{
    self, Collective, CollectiveInfo, CollectiveInspect, CollectivesInfo,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system = 1,
        Balances: pallet_balances = 2,
        Preimage: pallet_preimage = 3,
        Scheduler: pallet_scheduler = 4,
        Referenda: pallet_referenda = 5,
        SignedVoting: pallet_signed_voting = 6,
        MultiCollective: pallet_multi_collective = 7,
    }
);

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum CollectiveId {
    Proposers,
    Triumvirate,
    Economic,
    Building,
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum VotingScheme {
    Signed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberSet {
    Single(CollectiveId),
    Union(Vec<CollectiveId>),
}

impl subtensor_runtime_common::SetLike<U256> for MemberSet {
    fn contains(&self, who: &U256) -> bool {
        match self {
            MemberSet::Single(id) => <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                U256,
                CollectiveId,
            >>::is_member(*id, who),
            MemberSet::Union(ids) => ids.iter().any(|id| {
                <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                        U256,
                        CollectiveId,
                    >>::is_member(*id, who)
            }),
        }
    }
    fn len(&self) -> u32 {
        self.to_vec().len() as u32
    }
    fn to_vec(&self) -> Vec<U256> {
        match self {
            MemberSet::Single(id) => <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                U256,
                CollectiveId,
            >>::members_of(*id),
            // Mirrors the production `GovernanceMemberSet` impl: members can
            // overlap across collectives but a dual member can only vote
            // once. Sum-of-`member_count` would inflate `total` and bias
            // thresholds upward; dedup so the returned set has the true
            // cardinality.
            MemberSet::Union(ids) => {
                let mut accounts: Vec<U256> = Vec::new();
                for id in ids {
                    accounts.extend(
                        <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                            U256,
                            CollectiveId,
                        >>::members_of(*id),
                    );
                }
                accounts.sort();
                accounts.dedup();
                accounts
            }
        }
    }
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = U256;
    type AccountData = pallet_balances::AccountData<u64>;
    type Lookup = IdentityLookup<Self::AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

impl pallet_preimage::Config for Test {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Test>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<U256>;
    type Consideration = ();
}

parameter_types! {
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::with_sensible_defaults(
        Weight::from_parts(2_000_000_000_000, u64::MAX),
        Perbill::from_percent(75),
    );
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<U256>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Test>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}

pub struct TestTracks;

pub type MockTrack = Track<u8, TrackName, u64, MemberSet, MemberSet, VotingScheme>;

impl TracksInfo<TrackName, U256, RuntimeCall, u64> for TestTracks {
    type Id = u8;
    type ProposerSet = MemberSet;
    type VotingScheme = VotingScheme;
    type VoterSet = MemberSet;

    fn tracks() -> impl Iterator<
        Item = Track<
            Self::Id,
            TrackName,
            u64,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
    > {
        let overridden = current_track_override();
        if !overridden.is_empty() {
            return overridden.into_iter();
        }

        vec![
            Track {
                id: 0,
                info: TrackInfo {
                    name: pad_name(b"triumvirate"),
                    proposer_set: Some(MemberSet::Single(CollectiveId::Proposers)),
                    voter_set: MemberSet::Single(CollectiveId::Triumvirate),
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: 20,
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                        on_approval: ApprovalAction::Execute,
                    },
                },
            },
            Track {
                id: 1,
                info: TrackInfo {
                    name: pad_name(b"review"),
                    proposer_set: Some(MemberSet::Single(CollectiveId::Proposers)),
                    voter_set: MemberSet::Single(CollectiveId::Triumvirate),
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::Adjustable {
                        initial_delay: 100,
                        fast_track_threshold: Perbill::from_percent(75),
                        cancel_threshold: Perbill::from_percent(51),
                    },
                },
            },
            Track {
                id: 2,
                info: TrackInfo {
                    name: pad_name(b"delegating"),
                    proposer_set: Some(MemberSet::Single(CollectiveId::Proposers)),
                    voter_set: MemberSet::Single(CollectiveId::Triumvirate),
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: 20,
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                        on_approval: ApprovalAction::Review { track: 1 },
                    },
                },
            },
            Track {
                id: 3,
                info: TrackInfo {
                    name: pad_name(b"closed"),
                    proposer_set: None,
                    voter_set: MemberSet::Single(CollectiveId::Triumvirate),
                    voting_scheme: VotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: 20,
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                        on_approval: ApprovalAction::Execute,
                    },
                },
            },
        ]
        .into_iter()
        .filter(|t| !(t.id == 1 && review_track_hidden()))
        .map(|mut t| {
            if t.id == 1 && review_voter_set_empty() {
                t.info.voter_set = MemberSet::Union(alloc::vec![]);
            }
            if t.id == 0 && track0_swapped_to_adjustable() {
                t.info.decision_strategy = DecisionStrategy::Adjustable {
                    initial_delay: 100,
                    fast_track_threshold: Perbill::from_percent(75),
                    cancel_threshold: Perbill::from_percent(51),
                };
            }
            t
        })
        .collect::<Vec<_>>()
        .into_iter()
    }

    fn authorize_proposal(
        _track_info: &TrackInfo<
            Self::Id,
            TrackName,
            u64,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
        _call: &RuntimeCall,
    ) -> bool {
        AUTHORIZE_PROPOSAL_RESULT.with(|r| *r.borrow())
    }
}

thread_local! {
    static AUTHORIZE_PROPOSAL_RESULT: RefCell<bool> = const { RefCell::new(true) };
}

pub fn set_authorize_proposal(result: bool) {
    AUTHORIZE_PROPOSAL_RESULT.with(|r| *r.borrow_mut() = result);
}

/// Define a thread-local whose value can be temporarily replaced via an
/// RAII guard. The previous value is restored when the guard drops.
/// Used to simulate runtime-state mutations from tests without leaking
/// across cases.
macro_rules! define_scoped_state {
    ($flag:ident, $guard:ident, $reader:ident, $ty:ty, $default:expr) => {
        thread_local! {
            static $flag: RefCell<$ty> = const { RefCell::new($default) };
        }

        #[must_use = "the guard restores the prior value on drop; bind it to a local"]
        pub struct $guard {
            previous: Option<$ty>,
        }

        impl $guard {
            pub fn new(value: $ty) -> Self {
                let previous =
                    Some($flag.with(|r| core::mem::replace(&mut *r.borrow_mut(), value)));
                Self { previous }
            }
        }

        impl Drop for $guard {
            fn drop(&mut self) {
                if let Some(prev) = self.previous.take() {
                    $flag.with(|r| *r.borrow_mut() = prev);
                }
            }
        }

        fn $reader() -> $ty {
            $flag.with(|r| r.borrow().clone())
        }
    };
}

define_scoped_state!(
    HIDE_REVIEW_TRACK,
    HideReviewTrackGuard,
    review_track_hidden,
    bool,
    false
);
define_scoped_state!(
    EMPTY_REVIEW_VOTER_SET,
    EmptyReviewVoterSetGuard,
    review_voter_set_empty,
    bool,
    false
);
define_scoped_state!(
    SWAP_PASS_OR_FAIL_TRACK_TO_ADJUSTABLE,
    SwapTrack0ToAdjustableGuard,
    track0_swapped_to_adjustable,
    bool,
    false
);
define_scoped_state!(
    TRACKS_OVERRIDE,
    OverrideTracksGuard,
    current_track_override,
    Vec<MockTrack>,
    Vec::new()
);

pub struct TestCollectives;

impl CollectivesInfo<u64, [u8; 32]> for TestCollectives {
    type Id = CollectiveId;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, u64, [u8; 32]>> {
        vec![
            Collective {
                id: CollectiveId::Proposers,
                info: CollectiveInfo {
                    name: pad_name(b"proposers"),
                    min_members: 1,
                    max_members: Some(5),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Triumvirate,
                info: CollectiveInfo {
                    name: pad_name(b"triumvirate"),
                    min_members: 1,
                    max_members: Some(3),
                    term_duration: None,
                },
            },
        ]
        .into_iter()
    }
}

parameter_types! {
    pub const MaxMembers: u32 = 32;
}

impl pallet_multi_collective::Config for Test {
    type CollectiveId = CollectiveId;
    type Collectives = TestCollectives;
    type AddOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type RemoveOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type SwapOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type SetOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type RotateOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type OnMembersChanged = ();
    type OnNewTerm = ();
    type MaxMembers = MaxMembers;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ReferendaMockMcBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct ReferendaMockMcBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_multi_collective::BenchmarkHelper<CollectiveId> for ReferendaMockMcBenchmarkHelper {
    fn collective() -> CollectiveId {
        CollectiveId::Alpha
    }
    fn rotatable_collective() -> CollectiveId {
        CollectiveId::Alpha
    }
}

parameter_types! {
    pub const SignedScheme: VotingScheme = VotingScheme::Signed;
    pub const VoterSetSize: u32 = 32;
    pub const MaxPendingCleanup: u32 = 32;
    pub const CleanupChunkSize: u32 = 4;
    pub const CleanupCursorMaxLen: u32 = 128;
}

impl pallet_signed_voting::Config for Test {
    type Scheme = SignedScheme;
    type Polls = Referenda;
    type MaxVoterSetSize = VoterSetSize;
    type MaxPendingCleanup = MaxPendingCleanup;
    type CleanupChunkSize = CleanupChunkSize;
    type CleanupCursorMaxLen = CleanupCursorMaxLen;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxQueued: u32 = 10;
    pub const MaxActivePerProposer: u32 = 3;
}

impl pallet_referenda::Config for Test {
    type RuntimeCall = RuntimeCall;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type MaxQueued = MaxQueued;
    type MaxActivePerProposer = MaxActivePerProposer;
    type KillOrigin = EnsureRoot<U256>;
    type Tracks = TestTracks;
    type BlockNumberProvider = System;
    type OnPollCreated = SignedVoting;
    type OnPollCompleted = SignedVoting;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = TestBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct TestBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_referenda::BenchmarkHelper<u8, U256, RuntimeCall> for TestBenchmarkHelper {
    /// Track 2: `PassOrFail` with `Review { track: 1 }`. Worst case for
    /// the approve benchmark (creates a child referendum).
    fn track_passorfail() -> u8 {
        2
    }
    fn track_adjustable() -> u8 {
        1
    }
    fn proposer() -> U256 {
        U256::from(1)
    }
    fn call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
    }
}

pub struct TestState {
    pub proposers: Vec<U256>,
    pub triumvirate: Vec<U256>,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            proposers: vec![U256::from(1), U256::from(2)],
            triumvirate: vec![U256::from(101), U256::from(102), U256::from(103)],
        }
    }
}

impl TestState {
    pub fn build_and_execute(self, test: impl FnOnce()) {
        let mut ext = self.into_test_ext();
        ext.execute_with(test);
    }

    /// Build the externalities object pre-populated with collectives.
    /// Exposed for `impl_benchmark_test_suite!`, which expects a builder
    /// that returns `sp_io::TestExternalities` rather than a `FnOnce`.
    pub fn into_test_ext(self) -> sp_io::TestExternalities {
        let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
            system: frame_system::GenesisConfig::default(),
            balances: pallet_balances::GenesisConfig::default(),
        }
        .build_storage()
        .unwrap()
        .into();

        ext.execute_with(|| {
            System::set_block_number(1);
            set_authorize_proposal(true);

            for p in &self.proposers {
                pallet_multi_collective::Pallet::<Test>::add_member(
                    RuntimeOrigin::root(),
                    CollectiveId::Proposers,
                    *p,
                )
                .unwrap();
            }
            for t in &self.triumvirate {
                pallet_multi_collective::Pallet::<Test>::add_member(
                    RuntimeOrigin::root(),
                    CollectiveId::Triumvirate,
                    *t,
                )
                .unwrap();
            }
        });

        ext
    }
}

/// Externalities builder for `impl_benchmark_test_suite!`.
#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext() -> sp_io::TestExternalities {
    TestState::default().into_test_ext()
}

pub fn run_to_block(n: u64) {
    System::run_to_block::<AllPalletsWithSystem>(n);
}

/// Events emitted by `pallet_referenda` in insertion order.
pub fn referenda_events() -> Vec<crate::Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| match r.event {
            RuntimeEvent::Referenda(e) => Some(e),
            _ => None,
        })
        .collect()
}
