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

use crate::{self as pallet_referenda, *};
use pallet_multi_collective::{
    self, Collective, CollectiveInfo, CollectiveInspect, CollectivesInfo, OnMembersChanged,
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

impl pallet_multi_collective::CanRotate for CollectiveId {
    fn can_rotate(&self) -> bool {
        matches!(self, Self::Economic | Self::Building)
    }
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
        match self {
            MemberSet::Single(id) => <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                U256,
                CollectiveId,
            >>::member_count(*id),
            MemberSet::Union(ids) => ids
                .iter()
                .map(|id| {
                    <pallet_multi_collective::Pallet<Test> as CollectiveInspect<
                        U256,
                        CollectiveId,
                    >>::member_count(*id)
                })
                .sum(),
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
        let mut triumvirate_name = [0u8; 32];
        triumvirate_name[..11].copy_from_slice(b"triumvirate");

        let mut review_name = [0u8; 32];
        review_name[..6].copy_from_slice(b"review");

        let mut delegating_name = [0u8; 32];
        delegating_name[..10].copy_from_slice(b"delegating");

        let mut closed_name = [0u8; 32];
        closed_name[..6].copy_from_slice(b"closed");

        vec![
            // Track 0: PassOrFail with Execute on approval.
            Track {
                id: 0,
                info: TrackInfo {
                    name: triumvirate_name,
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
            // Track 1: Adjustable.
            Track {
                id: 1,
                info: TrackInfo {
                    name: review_name,
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
            // Track 2: PassOrFail with Review handoff to track 1.
            Track {
                id: 2,
                info: TrackInfo {
                    name: delegating_name,
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
            // Track 3: PassOrFail with no proposer set (not submittable).
            Track {
                id: 3,
                info: TrackInfo {
                    name: closed_name,
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

/// Set the value returned by `TestTracks::authorize_proposal` for the current thread.
pub fn set_authorize_proposal(result: bool) {
    AUTHORIZE_PROPOSAL_RESULT.with(|r| *r.borrow_mut() = result);
}

pub struct TestCollectives;

impl CollectivesInfo<u64, [u8; 32]> for TestCollectives {
    type Id = CollectiveId;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, u64, [u8; 32]>> {
        vec![
            Collective {
                id: CollectiveId::Proposers,
                info: CollectiveInfo {
                    name: {
                        let mut n = [0u8; 32];
                        n[..9].copy_from_slice(b"proposers");
                        n
                    },
                    min_members: 1,
                    max_members: Some(5),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Triumvirate,
                info: CollectiveInfo {
                    name: {
                        let mut n = [0u8; 32];
                        n[..11].copy_from_slice(b"triumvirate");
                        n
                    },
                    min_members: 1,
                    max_members: Some(3),
                    term_duration: None,
                },
            },
        ]
        .into_iter()
    }
}

pub struct VoteCleanup;
impl OnMembersChanged<CollectiveId, U256> for VoteCleanup {
    fn on_members_changed(_id: CollectiveId, _incoming: &[U256], outgoing: &[U256]) {
        for who in outgoing {
            SignedVoting::remove_votes_for(who);
        }
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
    type SetMembersOrigin = frame_support::traits::AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type OnMembersChanged = VoteCleanup;
    type OnNewTerm = ();
    type MaxMembers = MaxMembers;
}

parameter_types! {
    pub const SignedScheme: VotingScheme = VotingScheme::Signed;
}

impl pallet_signed_voting::Config for Test {
    type Scheme = SignedScheme;
    type Polls = Referenda;
}

parameter_types! {
    pub const MaxQueued: u32 = 10;
}

impl pallet_referenda::Config for Test {
    type RuntimeCall = RuntimeCall;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type MaxQueued = MaxQueued;
    type KillOrigin = EnsureRoot<U256>;
    type Tracks = TestTracks;
    type BlockNumberProvider = System;
    type PollHooks = SignedVoting;
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

            // Set up collectives via root origin
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

            test();
        });
    }
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
