#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use alloc::collections::BTreeMap;
use core::cell::RefCell;

use frame_support::{
    derive_impl,
    pallet_prelude::*,
    parameter_types,
    sp_runtime::{BuildStorage, traits::IdentityLookup},
    traits::AsEnsureOriginWithArg,
};
use frame_system::EnsureRoot;
use sp_core::U256;

use crate::{
    self as pallet_multi_collective, AdmissionPolicy, Collective, CollectiveInfo, CollectivesInfo,
    OnMembersChanged, OnNewTerm,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system = 1,
        MultiCollective: pallet_multi_collective = 2,
    }
);

// --- CollectiveId enum ---

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
    Alpha,
    Beta,
    Gamma,
    Delta,
    /// Intentionally NOT returned by `TestCollectives::collectives()`; used
    /// to exercise the `CollectiveNotFound` error path in extrinsics.
    Unknown,
}

// --- CollectivesInfo impl ---

pub fn name_bytes(s: &[u8]) -> [u8; 32] {
    let mut n = [0u8; 32];
    let len = s.len().min(32);
    n[..len].copy_from_slice(&s[..len]);
    n
}

pub struct TestCollectives;

// Optional override used by the integrity-test panic tests. When set,
// `TestCollectives::collectives()` returns the override's output instead of
// the default config. A function pointer is used (not a Vec) so the type
// stays `Copy`.
thread_local! {
    static COLLECTIVES_OVERRIDE: RefCell<
        Option<fn() -> Vec<Collective<CollectiveId, u64, [u8; 32]>>>,
    > = const { RefCell::new(None) };
}

fn default_collectives() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    vec![
        Collective {
            id: CollectiveId::Alpha,
            info: CollectiveInfo {
                name: name_bytes(b"alpha"),
                min_members: 0,
                max_members: Some(5),
                term_duration: None,
            },
        },
        Collective {
            id: CollectiveId::Beta,
            info: CollectiveInfo {
                name: name_bytes(b"beta"),
                min_members: 2,
                max_members: Some(3),
                term_duration: Some(100),
            },
        },
        Collective {
            id: CollectiveId::Gamma,
            info: CollectiveInfo {
                name: name_bytes(b"gamma"),
                min_members: 0,
                max_members: None,
                term_duration: None,
            },
        },
        Collective {
            id: CollectiveId::Delta,
            info: CollectiveInfo {
                name: name_bytes(b"delta"),
                min_members: 1,
                max_members: Some(32),
                term_duration: Some(50),
            },
        },
    ]
}

fn effective_collectives() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    let override_fn = COLLECTIVES_OVERRIDE.with(|o| *o.borrow());
    match override_fn {
        Some(f) => f(),
        None => default_collectives(),
    }
}

/// Run `f` with `TestCollectives` temporarily returning the output of
/// `override_fn`. An RAII guard clears the override when `f` returns *or
/// panics*, so a `#[should_panic]` integrity test cannot leak state onto
/// other tests running on the same thread.
pub fn with_collectives_override<R>(
    override_fn: fn() -> Vec<Collective<CollectiveId, u64, [u8; 32]>>,
    f: impl FnOnce() -> R,
) -> R {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            COLLECTIVES_OVERRIDE.with(|o| *o.borrow_mut() = None);
        }
    }

    COLLECTIVES_OVERRIDE.with(|o| *o.borrow_mut() = Some(override_fn));
    let _guard = Guard;
    f()
}

impl CollectivesInfo<u64, [u8; 32]> for TestCollectives {
    type Id = CollectiveId;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, u64, [u8; 32]>> {
        effective_collectives().into_iter()
    }
}

// --- Recording stubs for the pallet's two hooks ---
//
// `OnNewTerm` has no event counterpart; the rotation tests need the log to
// observe firings. `OnMembersChanged` is observable indirectly through the
// pallet's events, but the events do not show what was passed to the hook,
// so the recorder lets the hook-payload tests pin the exact arguments.

thread_local! {
    static NEW_TERM_LOG: RefCell<Vec<CollectiveId>> = const { RefCell::new(Vec::new()) };
    static NEW_TERM_WEIGHT: RefCell<Weight> = const { RefCell::new(Weight::zero()) };
    static MEMBERS_CHANGED_LOG: RefCell<Vec<MembersChangedCall>> =
        const { RefCell::new(Vec::new()) };
}

pub struct TestOnNewTerm;

impl OnNewTerm<CollectiveId> for TestOnNewTerm {
    fn on_new_term(id: CollectiveId) -> Weight {
        NEW_TERM_LOG.with(|log| log.borrow_mut().push(id));
        NEW_TERM_WEIGHT.with(|w| *w.borrow())
    }

    fn weight() -> Weight {
        NEW_TERM_WEIGHT.with(|w| *w.borrow())
    }
}

/// Drain and return the recorded `OnNewTerm` calls since the last drain.
pub fn take_new_term_log() -> Vec<CollectiveId> {
    NEW_TERM_LOG.with(|log| log.borrow_mut().drain(..).collect())
}

/// Set the weight that `TestOnNewTerm::on_new_term` reports back. Used by
/// `force_rotate` to assert that the post-info weight is the static
/// `WeightInfo::force_rotate()` plus the actual hook weight.
pub fn set_new_term_weight(weight: Weight) {
    NEW_TERM_WEIGHT.with(|w| *w.borrow_mut() = weight);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MembersChangedCall {
    pub collective_id: CollectiveId,
    pub incoming: Vec<U256>,
    pub outgoing: Vec<U256>,
}

pub struct TestOnMembersChanged;

impl OnMembersChanged<CollectiveId, U256> for TestOnMembersChanged {
    fn on_members_changed(collective_id: CollectiveId, incoming: &[U256], outgoing: &[U256]) {
        MEMBERS_CHANGED_LOG.with(|log| {
            log.borrow_mut().push(MembersChangedCall {
                collective_id,
                incoming: incoming.to_vec(),
                outgoing: outgoing.to_vec(),
            })
        });
    }

    fn weight() -> Weight {
        Weight::zero()
    }
}

/// Drain and return the recorded `OnMembersChanged` calls since the last drain.
pub fn take_members_changed_log() -> Vec<MembersChangedCall> {
    MEMBERS_CHANGED_LOG.with(|log| log.borrow_mut().drain(..).collect())
}

// --- Configurable admission policy ---
//
// Thread-local state lets each `try_join` test wire up exactly the
// eligibility verdict and rank it needs. Defaults: every account is
// ineligible (which forces tests to be explicit about who can join)
// and every account ranks at `0`.

thread_local! {
    static ELIGIBILITY: RefCell<BTreeMap<(CollectiveId, U256), bool>> =
        const { RefCell::new(BTreeMap::new()) };
    static RANKS: RefCell<BTreeMap<(CollectiveId, U256), u128>> =
        const { RefCell::new(BTreeMap::new()) };
}

pub fn set_eligible(collective_id: CollectiveId, who: U256, eligible: bool) {
    ELIGIBILITY.with(|e| {
        e.borrow_mut().insert((collective_id, who), eligible);
    });
}

pub fn set_rank(collective_id: CollectiveId, who: U256, rank: u128) {
    RANKS.with(|r| {
        r.borrow_mut().insert((collective_id, who), rank);
    });
}

pub fn clear_admission_policy() {
    ELIGIBILITY.with(|e| e.borrow_mut().clear());
    RANKS.with(|r| r.borrow_mut().clear());
}

pub struct TestAdmissionPolicy;

impl AdmissionPolicy<U256, CollectiveId> for TestAdmissionPolicy {
    type Rank = u128;

    fn is_eligible(collective_id: CollectiveId, who: &U256) -> (bool, Weight) {
        let eligible = ELIGIBILITY.with(|e| {
            e.borrow()
                .get(&(collective_id, *who))
                .copied()
                .unwrap_or(false)
        });
        (eligible, Weight::zero())
    }

    fn rank(collective_id: CollectiveId, who: &U256) -> (Self::Rank, Weight) {
        let rank = RANKS.with(|r| r.borrow().get(&(collective_id, *who)).copied().unwrap_or(0));
        (rank, Weight::zero())
    }

    fn is_eligible_weight(_: u32) -> Weight {
        Weight::zero()
    }

    fn rank_weight(_: u32) -> Weight {
        Weight::zero()
    }
}

/// Returns the `pallet_multi_collective::Event<Test>` values recorded in
/// `System::events()` so far, in insertion order.
pub fn multi_collective_events() -> Vec<crate::Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| match r.event {
            RuntimeEvent::MultiCollective(e) => Some(e),
            _ => None,
        })
        .collect()
}

// --- frame_system ---

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = U256;
    type Lookup = IdentityLookup<Self::AccountId>;
}

// --- pallet_multi_collective ---

parameter_types! {
    pub const MaxMembers: u32 = 32;
}

impl pallet_multi_collective::Config for Test {
    type CollectiveId = CollectiveId;
    type Collectives = TestCollectives;
    type AddOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type SwapOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type SetOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type RotateOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type OnMembersChanged = TestOnMembersChanged;
    type OnNewTerm = TestOnNewTerm;
    type AdmissionPolicy = TestAdmissionPolicy;
    type MaxMembers = MaxMembers;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = TestBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct TestBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_multi_collective::BenchmarkHelper<Test> for TestBenchmarkHelper {
    fn collective() -> CollectiveId {
        // Gamma: max_members = None, min_members = 0 → can fill to MaxMembers
        // and drain to empty without tripping the per-collective bounds.
        CollectiveId::Gamma
    }

    fn rotatable_collective() -> CollectiveId {
        // Beta has term_duration = Some(100).
        CollectiveId::Beta
    }

    fn try_join_collective() -> CollectiveId {
        // Delta: min=1, max=32 — bounded so the `try_join` benchmark
        // exercises the ranking / eviction path.
        CollectiveId::Delta
    }

    fn prime_admission(collective_id: CollectiveId, who: &U256, rank: u32) {
        set_eligible(collective_id, *who, true);
        set_rank(collective_id, *who, rank as u128);
    }
}

// --- Test externality builder ---

/// Build a fresh `TestExternalities` for the mock runtime. Used directly
/// by `impl_benchmark_test_suite!`; `TestState::build_and_execute` wraps
/// this with the per-test bootstrap unit tests rely on.
pub fn new_test_ext() -> sp_io::TestExternalities {
    RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into()
}

pub struct TestState;

impl TestState {
    pub fn build_and_execute(test: impl FnOnce()) {
        let mut ext = new_test_ext();

        ext.execute_with(|| {
            // System::events() only records events from block >= 1, so
            // setting the block first means each test starts with an empty
            // events buffer.
            System::set_block_number(1);
            let _ = take_new_term_log();
            let _ = take_members_changed_log();
            set_new_term_weight(Weight::zero());
            clear_admission_policy();
            test();
        });
    }
}

/// Advance to block `n`, invoking `on_finalize(k-1)` + `on_initialize(k)` for
/// each block `k` from the current block+1 up to and including `n`.
pub fn run_to_block(n: u64) {
    System::run_to_block::<AllPalletsWithSystem>(n);
}

pub fn seed_members(collective_id: CollectiveId, members: &[U256]) {
    let mut sorted = members.to_vec();
    sorted.sort();
    frame_support::assert_ok!(crate::Pallet::<Test>::set_members(
        RuntimeOrigin::root(),
        collective_id,
        sorted,
    ));
    let _ = take_members_changed_log();
    System::reset_events();
}
