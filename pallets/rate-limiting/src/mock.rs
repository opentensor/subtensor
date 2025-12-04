#![allow(dead_code)]

use core::convert::TryInto;

use frame_support::{
    derive_impl,
    sp_runtime::{
        BuildStorage,
        traits::{BlakeTwo256, IdentityLookup},
    },
    traits::{ConstU16, ConstU32, ConstU64, Everything},
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_std::vec::Vec;

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

pub type LimitScope = u16;
pub type UsageKey = u16;
pub type GroupId = u32;

pub struct TestScopeResolver;
pub struct TestUsageResolver;

impl pallet_rate_limiting::RateLimitScopeResolver<RuntimeOrigin, RuntimeCall, LimitScope, u64>
    for TestScopeResolver
{
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<LimitScope> {
        match call {
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span }) => {
                (*block_span).try_into().ok()
            }
            RuntimeCall::RateLimiting(_) => Some(1),
            _ => None,
        }
    }

    fn should_bypass(
        _origin: &RuntimeOrigin,
        call: &RuntimeCall,
    ) -> pallet_rate_limiting::types::BypassDecision {
        match call {
            RuntimeCall::RateLimiting(RateLimitingCall::remove_call_from_group { .. }) => {
                pallet_rate_limiting::types::BypassDecision::bypass_and_skip()
            }
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { .. }) => {
                pallet_rate_limiting::types::BypassDecision::bypass_and_record()
            }
            _ => pallet_rate_limiting::types::BypassDecision::enforce_and_record(),
        }
    }

    fn adjust_span(_origin: &RuntimeOrigin, call: &RuntimeCall, span: u64) -> u64 {
        if matches!(
            call,
            RuntimeCall::RateLimiting(RateLimitingCall::deregister_call { .. })
        ) {
            span.saturating_mul(2)
        } else {
            span
        }
    }
}

impl pallet_rate_limiting::RateLimitUsageResolver<RuntimeOrigin, RuntimeCall, UsageKey>
    for TestUsageResolver
{
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<UsageKey> {
        match call {
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span }) => {
                (*block_span).try_into().ok()
            }
            RuntimeCall::RateLimiting(_) => Some(1),
            _ => None,
        }
    }
}

impl pallet_rate_limiting::Config for Test {
    type RuntimeCall = RuntimeCall;
    type LimitScope = LimitScope;
    type LimitScopeResolver = TestScopeResolver;
    type UsageKey = UsageKey;
    type UsageResolver = TestUsageResolver;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type GroupId = GroupId;
    type MaxGroupMembers = ConstU32<32>;
    type MaxGroupNameLength = ConstU32<64>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchHelper;

#[cfg(feature = "runtime-benchmarks")]
impl crate::BenchmarkHelper<RuntimeCall> for BenchHelper {
    fn sample_call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::remark { remark: Vec::new() })
    }
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
    TransactionIdentifier::from_call::<Test, ()>(call).expect("identifier for call")
}

pub(crate) fn pop_last_event() -> RuntimeEvent {
    System::events().pop().expect("event expected").event
}
