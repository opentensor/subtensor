#![allow(dead_code)]
#![allow(clippy::expect_used)]

use core::convert::TryInto;

use frame_support::{
    derive_impl,
    dispatch::DispatchResult,
    sp_runtime::{
        BuildStorage,
        traits::{BlakeTwo256, IdentityLookup},
    },
    traits::{ConstU16, ConstU32, ConstU64, EnsureOrigin, Everything},
};
use frame_system::{EnsureRoot, ensure_signed};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

use crate as pallet_rate_limiting;
use crate::{RateLimitKind, TransactionIdentifier};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system = 0,
        RateLimiting: pallet_rate_limiting = 1,
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

#[derive(
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    codec::MaxEncodedLen,
    Debug,
)]
pub enum LimitSettingRule {
    RootOnly,
    AnySigned,
}

frame_support::parameter_types! {
    pub const DefaultLimitSettingRule: LimitSettingRule = LimitSettingRule::RootOnly;
}

pub struct LimitSettingOrigin;

impl pallet_rate_limiting::EnsureLimitSettingRule<RuntimeOrigin, LimitSettingRule, LimitScope>
    for LimitSettingOrigin
{
    fn ensure_origin(
        origin: RuntimeOrigin,
        rule: &LimitSettingRule,
        _scope: &Option<LimitScope>,
    ) -> DispatchResult {
        match rule {
            LimitSettingRule::RootOnly => EnsureRoot::<u64>::ensure_origin(origin)
                .map(|_| ())
                .map_err(Into::into),
            LimitSettingRule::AnySigned => {
                let _ = ensure_signed(origin)?;
                Ok(())
            }
        }
    }
}

pub struct TestScopeResolver;
pub struct TestUsageResolver;

impl pallet_rate_limiting::RateLimitScopeResolver<RuntimeOrigin, RuntimeCall, LimitScope, u64>
    for TestScopeResolver
{
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<BTreeSet<LimitScope>> {
        match call {
            RuntimeCall::RateLimiting(RateLimitingCall::set_rate_limit { limit, .. }) => {
                let RateLimitKind::Exact(span) = limit else {
                    let mut scopes = BTreeSet::new();
                    scopes.insert(1);
                    return Some(scopes);
                };
                let scope: LimitScope = (*span).try_into().ok()?;
                // Multi-scope path used by tests: Exact(42/43) returns two scopes.
                let mut scopes = BTreeSet::new();
                scopes.insert(scope);
                if *span == 42 || *span == 43 {
                    scopes.insert(scope.saturating_add(1));
                }
                Some(scopes)
            }
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span }) => {
                let scope: LimitScope = (*block_span).try_into().ok()?;
                let mut scopes = BTreeSet::new();
                scopes.insert(scope);
                Some(scopes)
            }
            RuntimeCall::RateLimiting(_) => {
                let mut scopes = BTreeSet::new();
                scopes.insert(1);
                Some(scopes)
            }
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
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<BTreeSet<UsageKey>> {
        match call {
            RuntimeCall::RateLimiting(RateLimitingCall::set_rate_limit { limit, .. }) => {
                let RateLimitKind::Exact(span) = limit else {
                    let mut usage = BTreeSet::new();
                    usage.insert(1);
                    return Some(usage);
                };
                let key: UsageKey = (*span).try_into().ok()?;
                // Multi-usage path used by tests: Exact(42) returns two usage keys.
                let mut usage = BTreeSet::new();
                usage.insert(key);
                if *span == 42 {
                    usage.insert(key.saturating_add(1));
                }
                Some(usage)
            }
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span }) => {
                let key: UsageKey = (*block_span).try_into().ok()?;
                let mut usage = BTreeSet::new();
                usage.insert(key);
                Some(usage)
            }
            RuntimeCall::RateLimiting(_) => {
                let mut usage = BTreeSet::new();
                usage.insert(1);
                Some(usage)
            }
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
    type LimitSettingRule = LimitSettingRule;
    type DefaultLimitSettingRule = DefaultLimitSettingRule;
    type LimitSettingOrigin = LimitSettingOrigin;
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
    TransactionIdentifier::from_call(call).expect("identifier for call")
}

pub(crate) fn pop_last_event() -> RuntimeEvent {
    System::events().pop().expect("event expected").event
}
