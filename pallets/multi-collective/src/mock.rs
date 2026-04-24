#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

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
    self as pallet_multi_collective, Collective, CollectiveInfo, CollectivesInfo, OnNewTerm,
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
    /// Intentionally NOT returned by `TestCollectives::collectives()` — used to
    /// exercise the `CollectiveNotFound` error path in extrinsics.
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

// Optional override used by Section 8 integrity-test panic tests. When set,
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
/// panics* — so a `#[should_panic]` integrity test cannot leak state onto
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

// --- Recording stub for the `OnNewTerm` hook ---
//
// `OnMembersChanged` observations go through the pallet's `Event` enum
// (MemberAdded / MemberRemoved / MemberSwapped / MembersReset) — see
// `multi_collective_events()` below. `OnNewTerm` has no corresponding event,
// so we keep a thread_local log for the rotation tests in Section 6.

thread_local! {
    static NEW_TERM_LOG: RefCell<Vec<CollectiveId>> = const { RefCell::new(Vec::new()) };
}

pub struct TestOnNewTerm;

impl OnNewTerm<CollectiveId> for TestOnNewTerm {
    fn on_new_term(id: CollectiveId) -> Weight {
        NEW_TERM_LOG.with(|log| log.borrow_mut().push(id));
        Weight::zero()
    }
}

/// Drain and return the recorded `OnNewTerm` calls since the last drain.
pub fn take_new_term_log() -> Vec<CollectiveId> {
    NEW_TERM_LOG.with(|log| log.borrow_mut().drain(..).collect())
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
    type ResetOrigin = AsEnsureOriginWithArg<EnsureRoot<U256>>;
    type OnMembersChanged = ();
    type OnNewTerm = TestOnNewTerm;
    type MaxMembers = MaxMembers;
}

// --- Test externality builder ---

pub struct TestState;

impl TestState {
    pub fn build_and_execute(test: impl FnOnce()) {
        let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
            .build_storage()
            .unwrap()
            .into();

        ext.execute_with(|| {
            // System::events() only records events from block >= 1, so
            // setting the block first means each test starts with an empty
            // events buffer.
            System::set_block_number(1);
            let _ = take_new_term_log();
            test();
        });
    }
}

/// Advance to block `n`, invoking `on_finalize(k-1)` + `on_initialize(k)` for
/// each block `k` from the current block+1 up to and including `n`.
pub fn run_to_block(n: u64) {
    System::run_to_block::<AllPalletsWithSystem>(n);
}
