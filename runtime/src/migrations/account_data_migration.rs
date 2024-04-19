use crate::*;
use frame_support::log;
use pallet_balances::ExtraFlags;

mod prev {
    use super::*;
    use frame_support::{pallet_prelude::ValueQuery, storage_alias, Blake2_128Concat};

    #[derive(
        Clone, Eq, PartialEq, Default, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    pub struct AccountDataStruct<Balance> {
        pub free: Balance,
        pub reserved: Balance,
        pub misc_frozen: Balance,
        pub fee_frozen: Balance,
    }

    #[derive(
        Clone, Eq, PartialEq, Default, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    pub struct AccountStruct<Balance> {
        pub nonce: u32,
        pub consumers: u32,
        pub providers: u32,
        pub sufficients: u32,
        pub data: AccountDataStruct<Balance>,
    }

    #[storage_alias]
    pub type Account<T: frame_system::pallet::Config> = StorageMap<
        frame_system::pallet::Pallet<T>,
        Blake2_128Concat,
        AccountId,
        AccountStruct<Balance>,
        ValueQuery,
    >;
}

const TARGET: &'static str = "runtime::account_data_migration";
pub struct Migration;
impl OnRuntimeUpgrade for Migration {
    /// Save pre-upgrade account ids to check are decodable post-upgrade.
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        use sp_std::collections::btree_map::BTreeMap;
        log::info!(target: TARGET, "pre-upgrade");

        // Save the expected post-migration account state.
        let mut expected_account: BTreeMap<
            AccountId,
            frame_system::AccountInfo<u32, pallet_balances::AccountData<Balance>>,
        > = BTreeMap::new();

        for (acc_id, acc) in prev::Account::<Runtime>::iter() {
            let expected_data = pallet_balances::AccountData {
                free: acc.data.free,
                reserved: acc.data.reserved,
                frozen: acc.data.misc_frozen.saturating_add(acc.data.fee_frozen),
                flags: ExtraFlags::default(),
            };

            // `ensure_upgraded` bumps the consumers if there is a non zero reserved balance and no frozen balance.
            // https://github.com/paritytech/polkadot-sdk/blob/305d311d5c732fcc4629f3295768f1ed44ef434c/substrate/frame/balances/src/lib.rs#L785
            let expected_consumers = if acc.data.reserved > 0 && expected_data.frozen == 0 {
                acc.consumers + 1
            } else {
                acc.consumers
            };
            let expected_acc = frame_system::AccountInfo {
                nonce: acc.nonce,
                consumers: expected_consumers,
                providers: acc.providers,
                sufficients: acc.sufficients,
                data: expected_data,
            };
            expected_account.insert(acc_id, expected_acc);
        }

        Ok(expected_account.encode())
    }

    /// Migrates Account storage to the new format, and calls `ensure_upgraded` for them.
    fn on_runtime_upgrade() -> Weight {
        // Pull the storage in the previous format into memory
        let accounts = prev::Account::<Runtime>::iter().collect::<Vec<_>>();
        log::info!(target: TARGET, "Migrating {} accounts...", accounts.len());

        for (acc_id, acc_info) in accounts.clone().into_iter() {
            let prev_data = acc_info.clone().data;

            // Move account to new data format
            let new_data = pallet_balances::AccountData {
                free: prev_data.free,
                reserved: prev_data.reserved,
                frozen: prev_data.misc_frozen.saturating_add(prev_data.fee_frozen),
                flags: ExtraFlags::old_logic(),
            };
            let new_account = frame_system::AccountInfo {
                nonce: acc_info.nonce,
                consumers: acc_info.consumers,
                providers: acc_info.providers,
                sufficients: acc_info.sufficients,
                data: new_data,
            };
            frame_system::pallet::Account::<Runtime>::insert(acc_id.clone(), new_account);

            // Ensure upgraded
            pallet_balances::Pallet::<Runtime, ()>::ensure_upgraded(&acc_id);
        }

        log::info!(target: TARGET, "Migrated {} accounts ✅", accounts.len());

        // R/W not important for solo chain.
        <Runtime as frame_system::Config>::DbWeight::get().reads_writes(0u64, 0u64)
    }

    /// Ensures post-upgrade that every Account entry matches what is expected.
    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use frame_support::ensure;
        use sp_std::collections::btree_map::BTreeMap;

        log::info!(target: TARGET, "Running post-upgrade...");

        let expected_accounts: BTreeMap<
            AccountId,
            frame_system::AccountInfo<u32, pallet_balances::AccountData<Balance>>,
        > = Decode::decode(&mut &state[..]).expect("decoding state failed");

        // Ensure the actual post-migration state matches the expected
        for (acc_id, acc) in frame_system::pallet::Account::<Runtime>::iter() {
            let expected = expected_accounts.get(&acc_id).expect("account not found");

            // New system logic nukes the account if no providers or sufficients.
            if acc.providers > 0 || acc.sufficients > 0 {
                ensure!(acc.nonce == expected.nonce, "nonce mismatch");
                ensure!(acc.consumers == expected.consumers, "consumers mismatch");
                ensure!(acc.providers == expected.providers, "providers mismatch");
                ensure!(
                    acc.sufficients == expected.sufficients,
                    "sufficients mismatch"
                );
                ensure!(acc.data.free == expected.data.free, "data.free mismatch");
                ensure!(
                    acc.data.reserved == expected.data.reserved,
                    "data.reserved mismatch"
                );
                ensure!(
                    acc.data.frozen == expected.data.frozen,
                    "data.frozen mismatch"
                );
                ensure!(acc.data.flags == expected.data.flags, "data.flags mismatch");
            }
        }

        log::info!(target: TARGET, "post-upgrade success ✅");
        Ok(())
    }
}
