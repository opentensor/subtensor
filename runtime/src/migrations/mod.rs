use crate::{Runtime, RuntimeHoldReason};
use alloc::string::String;
use deprecated::RegistryHoldReason as OldRegistryHoldReason;
use deprecated::RuntimeHoldReason as OldRuntimeHoldReason;
use frame_support::{
    BoundedVec,
    pallet_prelude::Zero,
    traits::{OnRuntimeUpgrade, StoredMap, tokens::IdAmount},
    weights::Weight,
};
use sp_runtime::Saturating;

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;
type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
type AccountStoreOf<T> = <T as pallet_balances::Config>::AccountStore;

const MIGRATION_NAME: &[u8] = b"remove_registry_balance_holds";

mod deprecated {
    use super::BalanceOf;
    use crate::Runtime;
    use codec::Decode;
    use frame_support::{
        BoundedVec,
        traits::{ConstU32, tokens::IdAmount},
    };

    #[cfg_attr(test, derive(codec::Encode))]
    #[derive(Decode, Copy, Clone, Eq, PartialEq, Debug)]
    pub(super) enum RegistryHoldReason {
        #[codec(index = 0)]
        RegistryIdentity,
    }

    #[cfg_attr(test, derive(codec::Encode))]
    #[derive(Decode, Copy, Clone, Eq, PartialEq, Debug)]
    pub(super) enum RuntimeHoldReason {
        #[codec(index = 14)]
        Preimage(pallet_preimage::HoldReason),
        #[codec(index = 17)]
        Registry(RegistryHoldReason),
        #[codec(index = 20)]
        SafeMode(pallet_safe_mode::HoldReason),
        #[codec(index = 29)]
        Contracts(pallet_contracts::HoldReason),
    }

    // Aggregated variant count across all pallets defining a
    // composite HoldReason when the pallet was removed.
    pub(super) const VARIANT_COUNT: u32 = 5;

    pub(super) type Holds =
        BoundedVec<IdAmount<RuntimeHoldReason, BalanceOf<Runtime>>, ConstU32<VARIANT_COUNT>>;
}

pub struct PalletRegistryCleanupMigration;

impl OnRuntimeUpgrade for PalletRegistryCleanupMigration {
    fn on_runtime_upgrade() -> Weight {
        let migration_name = MIGRATION_NAME.to_vec();
        let mut weight = Weight::zero();

        log::info!(
            "Running migration '{}'",
            String::from_utf8_lossy(&migration_name)
        );

        pallet_balances::Holds::<Runtime>::translate::<deprecated::Holds, _>(
            |account_id, old_holds| {
                weight.saturating_accrue(DbWeightOf::<Runtime>::get().reads_writes(1, 1));
                let mut current_holds = BoundedVec::new();
                let mut unlocked_amount = BalanceOf::<Runtime>::zero();

                // Translate old holds to new holds and keep track of cleaned up amount.
                for hold in old_holds {
                    match map_reason(hold.id) {
                        Some(id) => {
                            if current_holds
                                .try_push(IdAmount {
                                    id,
                                    amount: hold.amount,
                                })
                                .is_err()
                            {
                                log::error!(
                                    "too many balance holds after migration for account {:?}",
                                    account_id
                                );
                            }
                        }
                        None => {
                            unlocked_amount = unlocked_amount.saturating_add(hold.amount);
                        }
                    }
                }

                // Unlock the balance if there is any.
                if !unlocked_amount.is_zero() {
                    weight.saturating_accrue(DbWeightOf::<Runtime>::get().reads_writes(1, 1));
                    if let Err(error) = AccountStoreOf::<Runtime>::mutate(&account_id, |account| {
                        account.reserved = account.reserved.saturating_sub(unlocked_amount);
                        account.free = account.free.saturating_add(unlocked_amount);
                    }) {
                        log::error!(
                            "failed to unlock balance during holds migration: {:?}",
                            error
                        );
                    }
                }

                (!current_holds.is_empty()).then(|| current_holds)
            },
        );

        weight
    }
}

fn map_reason(reason: OldRuntimeHoldReason) -> Option<RuntimeHoldReason> {
    match reason {
        OldRuntimeHoldReason::Preimage(reason) => Some(RuntimeHoldReason::Preimage(reason)),
        OldRuntimeHoldReason::SafeMode(reason) => Some(RuntimeHoldReason::SafeMode(reason)),
        OldRuntimeHoldReason::Contracts(reason) => Some(RuntimeHoldReason::Contracts(reason)),
        OldRuntimeHoldReason::Registry(OldRegistryHoldReason::RegistryIdentity) => None,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use alloc::vec;
    use codec::Encode;
    use frame_support::{
        assert_ok,
        storage::unhashed,
        traits::{Currency, ReservableCurrency},
    };
    use sp_runtime::{AccountId32, BuildStorage};

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut ext: sp_io::TestExternalities = crate::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("runtime genesis storage should build")
            .into();
        ext.execute_with(|| crate::System::set_block_number(1));
        ext
    }

    fn account(seed: u8) -> AccountId32 {
        AccountId32::new([seed; 32])
    }

    fn balance(amount: u64) -> BalanceOf<Runtime> {
        amount.into()
    }

    fn old_hold(
        id: OldRuntimeHoldReason,
        amount: u64,
    ) -> IdAmount<OldRuntimeHoldReason, BalanceOf<Runtime>> {
        IdAmount {
            id,
            amount: balance(amount),
        }
    }

    fn old_holds(
        holds: alloc::vec::Vec<IdAmount<OldRuntimeHoldReason, BalanceOf<Runtime>>>,
    ) -> deprecated::Holds {
        holds
            .try_into()
            .expect("test old holds should fit the deprecated bound")
    }

    fn holds_key(account_id: &AccountId32) -> alloc::vec::Vec<u8> {
        pallet_balances::Holds::<Runtime>::hashed_key_for(account_id)
    }

    fn insert_old_holds(account_id: &AccountId32, holds: deprecated::Holds) {
        unhashed::put_raw(&holds_key(account_id), &holds.encode());
    }

    #[test]
    fn drops_registry_holds_and_unlocks_their_balance() {
        new_test_ext().execute_with(|| {
            let account_id = account(1);

            let _ = crate::Balances::make_free_balance_be(&account_id, balance(10_000));
            assert_ok!(crate::Balances::reserve(&account_id, balance(225)));

            insert_old_holds(
                &account_id,
                old_holds(vec![
                    old_hold(
                        OldRuntimeHoldReason::Registry(OldRegistryHoldReason::RegistryIdentity),
                        125,
                    ),
                    old_hold(
                        OldRuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage),
                        75,
                    ),
                    old_hold(
                        OldRuntimeHoldReason::SafeMode(pallet_safe_mode::HoldReason::EnterOrExtend),
                        25,
                    ),
                ]),
            );

            let issuance_before = crate::Balances::total_issuance();

            let weight = PalletRegistryCleanupMigration::on_runtime_upgrade();

            let account = crate::System::account(&account_id).data;
            assert!(!weight.is_zero());
            assert_eq!(account.free, balance(9_900));
            assert_eq!(account.reserved, balance(100));
            assert_eq!(crate::Balances::total_issuance(), issuance_before);

            let current_holds = pallet_balances::Holds::<Runtime>::get(&account_id);
            assert_eq!(current_holds.len(), 2);
            assert!(current_holds.contains(&IdAmount {
                id: RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage),
                amount: balance(75),
            }));
            assert!(current_holds.contains(&IdAmount {
                id: RuntimeHoldReason::SafeMode(pallet_safe_mode::HoldReason::EnterOrExtend),
                amount: balance(25),
            }));
        });
    }

    #[test]
    fn removes_holds_storage_when_only_registry_holds_remain() {
        new_test_ext().execute_with(|| {
            let account_id = account(2);

            let _ = crate::Balances::make_free_balance_be(&account_id, balance(10_000));
            assert_ok!(crate::Balances::reserve(&account_id, balance(125)));

            insert_old_holds(
                &account_id,
                old_holds(vec![old_hold(
                    OldRuntimeHoldReason::Registry(OldRegistryHoldReason::RegistryIdentity),
                    125,
                )]),
            );

            let storage_key = holds_key(&account_id);
            let issuance_before = crate::Balances::total_issuance();

            PalletRegistryCleanupMigration::on_runtime_upgrade();

            let account = crate::System::account(&account_id).data;
            assert_eq!(account.free, balance(10_000));
            assert_eq!(account.reserved, balance(0));
            assert_eq!(crate::Balances::total_issuance(), issuance_before);
            assert!(pallet_balances::Holds::<Runtime>::get(&account_id).is_empty());
            assert!(unhashed::get_raw(&storage_key).is_none());
        });
    }

    #[test]
    fn preserves_non_registry_holds_without_changing_balances() {
        new_test_ext().execute_with(|| {
            let account_id = account(3);

            let _ = crate::Balances::make_free_balance_be(&account_id, balance(10_000));
            assert_ok!(crate::Balances::reserve(&account_id, balance(100)));

            insert_old_holds(
                &account_id,
                old_holds(vec![
                    old_hold(
                        OldRuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage),
                        70,
                    ),
                    old_hold(
                        OldRuntimeHoldReason::Contracts(
                            pallet_contracts::HoldReason::StorageDepositReserve,
                        ),
                        30,
                    ),
                ]),
            );

            let issuance_before = crate::Balances::total_issuance();

            PalletRegistryCleanupMigration::on_runtime_upgrade();

            let account = crate::System::account(&account_id).data;
            assert_eq!(account.free, balance(9_900));
            assert_eq!(account.reserved, balance(100));
            assert_eq!(crate::Balances::total_issuance(), issuance_before);

            let current_holds = pallet_balances::Holds::<Runtime>::get(&account_id);
            assert_eq!(current_holds.len(), 2);
            assert!(current_holds.contains(&IdAmount {
                id: RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage),
                amount: balance(70),
            }));
            assert!(current_holds.contains(&IdAmount {
                id: RuntimeHoldReason::Contracts(
                    pallet_contracts::HoldReason::StorageDepositReserve,
                ),
                amount: balance(30),
            }));
        });
    }
}
