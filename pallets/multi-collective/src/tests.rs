#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use crate as pallet_multi_collective;
use crate::{Collective, CollectiveInfo, CollectiveInspect, CollectiveName, CollectivesInfo};
use frame_support::{
    construct_runtime, derive_impl, parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU32, ConstU64},
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{BuildStorage, traits::IdentityLookup};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        MultiCollective: pallet_multi_collective,
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
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    codec::MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum TestCollective {
    A,
    B,
}

pub struct TestCollectives;

impl CollectivesInfo<u64, CollectiveName> for TestCollectives {
    type Id = TestCollective;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, u64, CollectiveName>> {
        [
            Collective {
                id: TestCollective::A,
                info: CollectiveInfo {
                    name: name("a"),
                    min_members: 0,
                    max_members: Some(3),
                    term_duration: None,
                },
            },
            Collective {
                id: TestCollective::B,
                info: CollectiveInfo {
                    name: name("b"),
                    min_members: 1,
                    max_members: Some(5),
                    term_duration: None,
                },
            },
        ]
        .into_iter()
    }
}

fn name(s: &str) -> CollectiveName {
    let mut n = [0u8; 32];
    let bytes = s.as_bytes();
    let len = bytes.len().min(32);
    n[..len].copy_from_slice(&bytes[..len]);
    n
}

parameter_types! {
    pub const MaxMembers: u32 = 8;
}

impl pallet_multi_collective::Config for Test {
    type CollectiveId = TestCollective;
    type Collectives = TestCollectives;
    type AddOrigin = AsEnsureOriginWithArg<EnsureRoot<u64>>;
    type RemoveOrigin = AsEnsureOriginWithArg<EnsureRoot<u64>>;
    type SwapOrigin = AsEnsureOriginWithArg<EnsureRoot<u64>>;
    type ResetOrigin = AsEnsureOriginWithArg<EnsureRoot<u64>>;
    type OnMembersChanged = ();
    type OnNewTerm = ();
    type MaxMembers = MaxMembers;
}

fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

#[test]
fn add_and_remove_member_round_trip() {
    new_test_ext().execute_with(|| {
        assert_eq!(MultiCollective::member_count(TestCollective::A), 0);

        MultiCollective::add_member(RuntimeOrigin::root(), TestCollective::A, 1).unwrap();
        MultiCollective::add_member(RuntimeOrigin::root(), TestCollective::A, 2).unwrap();

        assert_eq!(MultiCollective::member_count(TestCollective::A), 2);
        assert!(MultiCollective::is_member(TestCollective::A, &1));
        assert!(MultiCollective::is_member(TestCollective::A, &2));

        MultiCollective::remove_member(RuntimeOrigin::root(), TestCollective::A, 1).unwrap();

        assert_eq!(MultiCollective::member_count(TestCollective::A), 1);
        assert!(!MultiCollective::is_member(TestCollective::A, &1));
        assert!(MultiCollective::is_member(TestCollective::A, &2));
    });
}

#[test]
fn reset_members_respects_bounds() {
    new_test_ext().execute_with(|| {
        MultiCollective::reset_members(RuntimeOrigin::root(), TestCollective::B, vec![10, 11, 12])
            .unwrap();
        assert_eq!(MultiCollective::member_count(TestCollective::B), 3);

        // Rejects dropping below min_members (1) for B.
        assert!(
            MultiCollective::reset_members(RuntimeOrigin::root(), TestCollective::B, vec![],)
                .is_err()
        );

        // Rejects duplicates.
        assert!(
            MultiCollective::reset_members(RuntimeOrigin::root(), TestCollective::B, vec![10, 10],)
                .is_err()
        );
    });
}

#[test]
fn swap_member_preserves_count() {
    new_test_ext().execute_with(|| {
        MultiCollective::add_member(RuntimeOrigin::root(), TestCollective::A, 1).unwrap();
        MultiCollective::swap_member(RuntimeOrigin::root(), TestCollective::A, 1, 9).unwrap();
        assert_eq!(MultiCollective::member_count(TestCollective::A), 1);
        assert!(!MultiCollective::is_member(TestCollective::A, &1));
        assert!(MultiCollective::is_member(TestCollective::A, &9));
    });
}
