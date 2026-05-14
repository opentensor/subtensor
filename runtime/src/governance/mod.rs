pub mod collectives;
pub mod member_set;
pub mod tracks;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::parameter_types;
use frame_support::traits::AsEnsureOriginWithArg;
use frame_system::EnsureRoot;
use scale_info::TypeInfo;

use crate::{
    AccountId, Preimage, Referenda, Runtime, RuntimeCall, Scheduler, SignedVoting, System,
};

use self::collectives::{CollectiveId, Collectives, TermManagement};
pub use self::member_set::MemberSet;

parameter_types! {
    /// Storage cap shared by all collectives; sized for the widest one
    /// (`EconomicEligible`). Per-collective `info.max_members` are the
    /// logical caps; this is just the `BoundedVec` capacity.
    pub const MaxMembers: u32 = collectives::ECONOMIC_ELIGIBLE_SIZE;
}

impl pallet_multi_collective::Config for Runtime {
    type CollectiveId = CollectiveId;
    type Collectives = Collectives;
    type AddOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type SwapOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type SetOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type RotateOrigin = AsEnsureOriginWithArg<EnsureRoot<AccountId>>;
    type OnMembersChanged = ();
    type OnNewTerm = TermManagement;
    type MaxMembers = MaxMembers;
    type WeightInfo = pallet_multi_collective::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MultiCollectiveBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct MultiCollectiveBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_multi_collective::BenchmarkHelper<Runtime> for MultiCollectiveBenchmarkHelper {
    fn collective() -> CollectiveId {
        CollectiveId::Proposers
    }

    fn rotatable_collective() -> CollectiveId {
        CollectiveId::Economic
    }
}

/// Voting scheme for each referenda track.
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

parameter_types! {
    pub const Scheme: VotingScheme = VotingScheme::Signed;
    /// Headroom over the widest track's voter set (see guard below).
    pub const MaxVoterSetSize: u32 = 64;
    /// 2x `MaxQueued` for headroom; queue overflow leaks `VotingFor` storage.
    pub const MaxPendingCleanup: u32 = 40;
    /// `VotingFor` entries drained per `on_idle` step. A full poll drains
    /// in `MaxVoterSetSize / CleanupChunkSize` idle blocks.
    pub const CleanupChunkSize: u32 = 16;
    /// Resume cursor for chunked cleanup; 128 bytes covers any FRAME
    /// double-map partial trie key.
    pub const CleanupCursorMaxLen: u32 = 128;
}

impl pallet_signed_voting::Config for Runtime {
    type Scheme = Scheme;
    type Polls = Referenda;
    type MaxVoterSetSize = MaxVoterSetSize;
    type MaxPendingCleanup = MaxPendingCleanup;
    type CleanupChunkSize = CleanupChunkSize;
    type CleanupCursorMaxLen = CleanupCursorMaxLen;
    type WeightInfo = pallet_signed_voting::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = SignedVotingBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct SignedVotingBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_signed_voting::benchmarking::BenchmarkHelper<Runtime> for SignedVotingBenchmarkHelper {
    fn ongoing_poll() -> u32 {
        use self::ReferendaBenchmarkHelper as RBH;
        use pallet_referenda::BenchmarkHelper as BH;

        let proposer = <RBH as BH<u8, AccountId, RuntimeCall>>::proposer();
        let track = <RBH as BH<u8, AccountId, RuntimeCall>>::track_adjustable();
        let call = <RBH as BH<u8, AccountId, RuntimeCall>>::call();
        let index = pallet_referenda::ReferendumCount::<Runtime>::get();

        Referenda::submit(
            frame_system::RawOrigin::Signed(proposer).into(),
            track,
            sp_std::boxed::Box::new(call),
        )
        .expect("submit must succeed in benchmark setup");
        index
    }
}

parameter_types! {
    pub const MaxQueued: u32 = 20;
    pub const MaxActivePerProposer: u32 = 5;
}

impl pallet_referenda::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type MaxQueued = MaxQueued;
    type MaxActivePerProposer = MaxActivePerProposer;
    type KillOrigin = EnsureRoot<AccountId>;
    type Tracks = tracks::Tracks;
    type AdjustmentCurve = tracks::LinearAdjustmentCurve;
    type BlockNumberProvider = System;
    type OnPollCreated = SignedVoting;
    type OnPollCompleted = SignedVoting;
    type WeightInfo = pallet_referenda::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ReferendaBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct ReferendaBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_referenda::BenchmarkHelper<u8, AccountId, RuntimeCall> for ReferendaBenchmarkHelper {
    fn track_passorfail() -> u8 {
        0
    }

    fn track_adjustable() -> u8 {
        1
    }

    fn proposer() -> AccountId {
        let proposer: AccountId = sp_core::crypto::AccountId32::new([1u8; 32]).into();
        let _ = pallet_multi_collective::Pallet::<Runtime>::add_member(
            frame_system::RawOrigin::Root.into(),
            CollectiveId::Proposers,
            proposer.clone(),
        );
        proposer
    }

    fn call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::remark {
            remark: alloc::vec![],
        })
    }
}

// Compile-time guards on the relationships between the constants above.
// A misconfiguration here would degrade signed-voting silently (oversized
// voter set collapses to an empty snapshot, queue overflow leaks state),
// so catch the obvious foot-guns at build time.
const _: () = {
    // The widest track today is `Union(Economic, Building)`. Union members
    // can overlap (a coldkey may sit in both), so this sum is an upper
    // bound on the voter set's true cardinality before `MemberSet::Union`'s
    // dedup runs.
    let widest_union = (collectives::ECONOMIC_SIZE as u64) + (collectives::BUILDING_SIZE as u64);
    assert!(
        MaxVoterSetSize::get() as u64 >= widest_union,
        "MaxVoterSetSize must fit the widest track's voter set",
    );
    assert!(
        MaxVoterSetSize::get() >= MaxMembers::get(),
        "MaxVoterSetSize must fit any single-collective track",
    );
    assert!(
        MaxPendingCleanup::get() >= MaxQueued::get(),
        "MaxPendingCleanup must absorb at least one full simultaneous-completion event from `pallet-referenda`",
    );
};
