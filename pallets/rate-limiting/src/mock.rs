#![allow(dead_code)]

use frame_support::{
    derive_impl,
    sp_runtime::{
        BuildStorage,
        traits::{BlakeTwo256, IdentityLookup},
    },
    traits::{ConstU16, ConstU32, ConstU64, Everything},
};
use sp_core::H256;
use sp_io::TestExternalities;

use crate as pallet_rate_limiting;
use crate::TransactionIdentifier;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        RateLimiting: pallet_rate_limiting,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type Block = Block;
}

pub type LimitContext = u16;

pub struct TestContextResolver;

impl pallet_rate_limiting::RateLimitContextResolver<RuntimeCall, LimitContext>
    for TestContextResolver
{
    fn context(_call: &RuntimeCall) -> Option<LimitContext> {
        None
    }
}

impl pallet_rate_limiting::Config for Test {
    type RuntimeCall = RuntimeCall;
    type LimitContext = LimitContext;
    type ContextResolver = TestContextResolver;
}

pub type RateLimitingCall = crate::Call<Test>;

pub fn new_test_ext() -> TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("genesis build succeeds");

    let mut ext = TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub(crate) fn identifier_for(call: &RuntimeCall) -> TransactionIdentifier {
    TransactionIdentifier::from_call::<Test>(call).expect("identifier for call")
}

pub(crate) fn pop_last_event() -> RuntimeEvent {
    System::events().pop().expect("event expected").event
}
