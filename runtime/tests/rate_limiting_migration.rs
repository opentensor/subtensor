#![allow(clippy::unwrap_used)]

use frame_support::traits::OnRuntimeUpgrade;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_rate_limiting::{RateLimit, RateLimitKind, RateLimitTarget, TransactionIdentifier};
use pallet_subtensor::{HasMigrationRun, LastRateLimitedBlock, RateLimitKey};
use sp_runtime::traits::SaturatedConversion;
use subtensor_runtime_common::{NetUid, rate_limiting::GROUP_REGISTER_NETWORK};

use node_subtensor_runtime::{
    BuildStorage, Runtime, RuntimeGenesisConfig, SubtensorModule, System, rate_limiting,
    rate_limiting::migration::{MIGRATION_NAME, Migration},
};

type AccountId = <Runtime as frame_system::Config>::AccountId;
type UsageKey = rate_limiting::RateLimitUsageKey<AccountId>;

fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn migrates_global_register_network_last_seen() {
    new_test_ext().execute_with(|| {
        HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);

        // Seed legacy global register rate-limit state.
        LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::NetworkLastRegistered, 10u64);
        System::set_block_number(12);

        // Run migration.
        Migration::<Runtime>::on_runtime_upgrade();

        let target = RateLimitTarget::Group(GROUP_REGISTER_NETWORK);

        // LastSeen preserved globally (usage = None).
        let stored = pallet_rate_limiting::LastSeen::<Runtime>::get(target, None::<UsageKey>)
            .expect("last seen entry");
        assert_eq!(stored, 10u64.saturated_into::<BlockNumberFor<Runtime>>());
    });
}

#[test]
fn sn_owner_hotkey_limit_not_tempo_scaled_and_last_seen_preserved() {
    new_test_ext().execute_with(|| {
        HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);

        let netuid = NetUid::from(1);
        // Give the subnet a non-1 tempo to catch accidental scaling.
        SubtensorModule::set_tempo(netuid, 5);
        LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::SetSNOwnerHotkey(netuid), 100u64);

        Migration::<Runtime>::on_runtime_upgrade();

        let target = RateLimitTarget::Transaction(TransactionIdentifier::new(19, 67));

        // Limit should remain the fixed default (50400 blocks), not tempo-scaled.
        let limit = pallet_rate_limiting::Limits::<Runtime>::get(target).expect("limit stored");
        assert!(matches!(limit, RateLimit::Global(kind) if kind == RateLimitKind::Exact(50_400)));

        // LastSeen preserved per subnet.
        let usage: Option<<Runtime as pallet_rate_limiting::Config>::UsageKey> =
            Some(UsageKey::Subnet(netuid).into());
        let stored =
            pallet_rate_limiting::LastSeen::<Runtime>::get(target, usage).expect("last seen entry");
        assert_eq!(stored, 100u64.saturated_into::<BlockNumberFor<Runtime>>());
    });
}
