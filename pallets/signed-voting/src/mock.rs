#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::expect_used
)]

use core::cell::RefCell;
use std::collections::BTreeMap;

use frame_support::{
    derive_impl,
    pallet_prelude::*,
    parameter_types,
    sp_runtime::{BuildStorage, traits::IdentityLookup},
};
use sp_core::U256;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

use crate::{self as pallet_signed_voting};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system = 1,
        SignedVoting: pallet_signed_voting = 2,
    }
);

// --- VotingScheme enum ---

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
    /// Used to exercise the scheme-mismatch rejection in `vote` / `remove_vote`.
    Anonymous,
}

// --- SimpleVoterSet ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleVoterSet(pub Vec<U256>);

impl SetLike<U256> for SimpleVoterSet {
    fn contains(&self, who: &U256) -> bool {
        self.0.contains(who)
    }
    fn len(&self) -> u32 {
        self.0.len() as u32
    }
}

// --- Mock `Polls` backed by thread-local state ---

#[derive(Clone)]
pub struct PollState {
    pub is_ongoing: bool,
    pub scheme: VotingScheme,
    pub voter_set: Vec<U256>,
}

thread_local! {
    static POLLS_STATE: RefCell<BTreeMap<u32, PollState>> =
        const { RefCell::new(BTreeMap::new()) };
    static TALLY_UPDATES: RefCell<Vec<(u32, VoteTally)>> =
        const { RefCell::new(Vec::new()) };
}

pub struct MockPolls;

impl Polls<U256> for MockPolls {
    type Index = u32;
    type VotingScheme = VotingScheme;
    type VoterSet = SimpleVoterSet;

    fn is_ongoing(index: Self::Index) -> bool {
        POLLS_STATE.with(|p| {
            p.borrow()
                .get(&index)
                .map(|s| s.is_ongoing)
                .unwrap_or(false)
        })
    }

    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme> {
        POLLS_STATE.with(|p| p.borrow().get(&index).map(|s| s.scheme))
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        POLLS_STATE.with(|p| {
            p.borrow()
                .get(&index)
                .map(|s| SimpleVoterSet(s.voter_set.clone()))
        })
    }

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        TALLY_UPDATES.with(|t| t.borrow_mut().push((index, tally.clone())));
    }
}

// --- Helpers ---

/// Register a poll and fire `on_poll_created` so `TallyOf` / `ActivePolls`
/// are populated. After this returns, the pallet sees the poll as ongoing.
pub fn start_poll(index: u32, scheme: VotingScheme, voter_set: Vec<U256>) {
    POLLS_STATE.with(|p| {
        p.borrow_mut().insert(
            index,
            PollState {
                is_ongoing: true,
                scheme,
                voter_set,
            },
        );
    });
    <SignedVoting as PollHooks<u32>>::on_poll_created(index);
}

/// Mark the poll inactive and fire `on_poll_completed` to clean up storage.
pub fn complete_poll(index: u32) {
    POLLS_STATE.with(|p| {
        if let Some(s) = p.borrow_mut().get_mut(&index) {
            s.is_ongoing = false;
        }
    });
    <SignedVoting as PollHooks<u32>>::on_poll_completed(index);
}

/// Simulate membership rotation by removing `who` from a poll's voter set
/// *without* invoking `Pallet::remove_votes_for`. Tests that want the cleanup
/// call it explicitly.
pub fn remove_voter(index: u32, who: U256) {
    POLLS_STATE.with(|p| {
        if let Some(s) = p.borrow_mut().get_mut(&index) {
            s.voter_set.retain(|v| *v != who);
        }
    });
}

pub fn take_tally_updates() -> Vec<(u32, VoteTally)> {
    TALLY_UPDATES.with(|t| t.borrow_mut().drain(..).collect())
}

pub fn signed_voting_events() -> Vec<crate::Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| match r.event {
            RuntimeEvent::SignedVoting(e) => Some(e),
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

// --- pallet_signed_voting ---

parameter_types! {
    pub const TestScheme: VotingScheme = VotingScheme::Signed;
}

impl pallet_signed_voting::Config for Test {
    type Scheme = TestScheme;
    type Polls = MockPolls;
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
            System::set_block_number(1);
            POLLS_STATE.with(|p| p.borrow_mut().clear());
            let _ = take_tally_updates();
            test();
        });
    }
}
