#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::cell::RefCell;

use crate as pallet_signed_voting;
use crate::SignedVoteTally;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    construct_runtime, derive_impl, parameter_types,
    sp_runtime::Perbill,
    traits::{ConstU32, ConstU64},
};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{BuildStorage, traits::IdentityLookup};
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        SignedVoting: pallet_signed_voting,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub enum TestScheme {
    Signed,
    Other,
}

thread_local! {
    static ONGOING: RefCell<bool> = const { RefCell::new(true) };
    static LAST_TALLY: RefCell<Option<VoteTally>> = const { RefCell::new(None) };
    static VOTERS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn set_voters(v: Vec<u64>) {
    VOTERS.with(|cell| *cell.borrow_mut() = v);
}

fn last_tally() -> Option<VoteTally> {
    LAST_TALLY.with(|t| t.borrow().clone())
}

pub struct MockVoters;

impl SetLike<u64> for MockVoters {
    fn contains(&self, who: &u64) -> bool {
        VOTERS.with(|v| v.borrow().contains(who))
    }
    fn len(&self) -> u32 {
        VOTERS.with(|v| v.borrow().len() as u32)
    }
    fn members(&self) -> sp_runtime::Vec<u64> {
        VOTERS.with(|v| v.borrow().clone())
    }
}

pub struct MockPolls;

impl Polls<u64> for MockPolls {
    type Index = u32;
    type VotingScheme = TestScheme;
    type VoterSet = MockVoters;

    fn is_ongoing(_: u32) -> bool {
        ONGOING.with(|v| *v.borrow())
    }
    fn voting_scheme_of(_: u32) -> Option<TestScheme> {
        Some(TestScheme::Signed)
    }
    fn voter_set_of(_: u32) -> Option<MockVoters> {
        Some(MockVoters)
    }
    fn on_tally_updated(_: u32, tally: &VoteTally) {
        LAST_TALLY.with(|t| *t.borrow_mut() = Some(tally.clone()));
    }
}

parameter_types! {
    pub SignedScheme: TestScheme = TestScheme::Signed;
}

impl pallet_signed_voting::Config for Test {
    type Scheme = SignedScheme;
    type Polls = MockPolls;
    type MaxVotesToClear = ConstU32<100>;
    type MaxSnapshotMembers = ConstU32<32>;
}

fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

#[test]
fn vote_and_switch_updates_tally() {
    new_test_ext().execute_with(|| {
        set_voters(vec![1, 2, 3]);
        <SignedVoting as PollHooks<u32>>::on_poll_created(0);

        SignedVoting::vote(RuntimeOrigin::signed(1), 0, true).unwrap();
        let t = last_tally().unwrap();
        assert_eq!(t.approval, Perbill::from_rational(1u32, 3u32));
        assert_eq!(t.rejection, Perbill::zero());

        // Switch from aye to nay.
        SignedVoting::vote(RuntimeOrigin::signed(1), 0, false).unwrap();
        let t = last_tally().unwrap();
        assert_eq!(t.approval, Perbill::zero());
        assert_eq!(t.rejection, Perbill::from_rational(1u32, 3u32));

        // Remove the vote entirely.
        SignedVoting::remove_vote(RuntimeOrigin::signed(1), 0).unwrap();
        let t = last_tally().unwrap();
        assert_eq!(t.approval, Perbill::zero());
        assert_eq!(t.rejection, Perbill::zero());
    });
}

#[test]
fn non_voter_is_rejected() {
    new_test_ext().execute_with(|| {
        set_voters(vec![1, 2, 3]);
        <SignedVoting as PollHooks<u32>>::on_poll_created(0);

        assert!(SignedVoting::vote(RuntimeOrigin::signed(42), 0, true).is_err());
    });
}

#[test]
fn pollhooks_initialize_tally_with_total() {
    new_test_ext().execute_with(|| {
        set_voters(vec![1, 2, 3, 4]);
        <SignedVoting as PollHooks<u32>>::on_poll_created(7);

        let tally = pallet_signed_voting::TallyOf::<Test>::get(7).unwrap();
        assert_eq!(
            tally,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total: 4,
            }
        );
    });
}

/// Regression: adding members to the underlying collective mid-poll must NOT let the new
/// members vote on the existing poll. Eligibility freezes at `on_poll_created` via
/// `VoterSnapshot`, and the tally `total` denominator is fixed at the same time.
#[test]
fn vote_uses_voter_snapshot_not_live_set() {
    new_test_ext().execute_with(|| {
        // Initial voter set captured at poll creation.
        set_voters(vec![1, 2, 3]);
        <SignedVoting as PollHooks<u32>>::on_poll_created(0);

        // Membership grows mid-poll.
        set_voters(vec![1, 2, 3, 99]);

        // The newly-added member can NOT vote — they weren't in the snapshot.
        assert!(SignedVoting::vote(RuntimeOrigin::signed(99), 0, true).is_err());

        // Original members can still vote.
        SignedVoting::vote(RuntimeOrigin::signed(1), 0, true).unwrap();

        // Tally `total` is the snapshot size (3), not the live size (4) — so 1 aye = 1/3,
        // not 1/4. Membership growth cannot dilute thresholds either.
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0).unwrap();
        assert_eq!(tally.total, 3);
        assert_eq!(tally.ayes, 1);
    });
}

/// Regression: a 2-of-3 SignedVoteTally must clear a 2/3 rational threshold (the value the
/// triumvirate track uses), but it does NOT clear a `Perbill::from_percent(67)` threshold.
///
/// This pins the choice in `runtime/src/governance_v2.rs` to use `from_rational(2, 3)` rather
/// than `from_percent(67)`, since the latter rounds up to 670_000_000 parts while the tally
/// produces 666_666_666 parts.
#[test]
fn triumvirate_2_of_3_meets_rational_threshold_but_not_percent_67() {
    let tally = SignedVoteTally {
        ayes: 2,
        nays: 0,
        total: 3,
    };
    let v: VoteTally = tally.into();

    let two_thirds = Perbill::from_rational(2u32, 3u32);
    let percent_67 = Perbill::from_percent(67);

    assert_eq!(v.approval, two_thirds);
    assert!(v.approval >= two_thirds, "2/3 must clear a 2/3 threshold");
    assert!(
        v.approval < percent_67,
        "2/3 must NOT clear a 67% threshold (this would silently force 3/3)"
    );
}
