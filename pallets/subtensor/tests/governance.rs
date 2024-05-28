mod mock;
use mock::*;

use frame_support::assert_ok;
use sp_core::{bounded_vec, U256};
use sp_runtime::BuildStorage;

use frame_system::Config;
use pallet_subtensor::migration;

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();

    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        senate_members: pallet_membership::GenesisConfig::<Test, pallet_membership::Instance2> {
            members: bounded_vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()],
            phantom: Default::default(),
        },
        governance: pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
            members: vec![1.into()],
            phantom: Default::default(),
        },
        ..Default::default()
    }
    .build_storage()
    .unwrap()
    .into();

    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn test_subnet_owners_join_works() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        let coldkey_account_id = U256::from(667);

        // Get lock cost
        let lock_cost = SubtensorModule::get_network_lock_cost();

        // Give enough balance to the coldkey for the lock cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, lock_cost + 10_000);

        // Verify the coldkey is NOT in governance (yet)
        assert_ne!(SubnetOwners::is_member(&coldkey_account_id), true);

        // Register new network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
        ));
        assert_eq!(SubtensorModule::get_subnet_owner(1), coldkey_account_id);

        // Check that the owner is in governance
        assert!(SubnetOwners::is_member(&coldkey_account_id));
    });
}

#[test]
fn test_subnet_owners_leave_works() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        let coldkey_account_id = U256::from(667);

        // Get lock cost
        let lock_cost = SubtensorModule::get_network_lock_cost();

        // Give enough balance to the coldkey for the lock cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, lock_cost + 10_000);
        // Register new network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
        ));
        assert_eq!(SubtensorModule::get_subnet_owner(1), coldkey_account_id);

        // Check that the owner is in governance
        assert!(SubnetOwners::is_member(&coldkey_account_id));

        // Unregister network
        SubtensorModule::remove_network(1);

        // Check that the owner is no longer in governance
        assert_ne!(SubnetOwners::is_member(&coldkey_account_id), true);
    });
}

#[test]
fn test_subnet_owners_swap_works() {
    // Should swap when a new owner is added/removed for the same netuid
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        let coldkey_account_id = U256::from(667);
        let coldkey_account_id2 = U256::from(668);

        // Get lock cost
        let lock_cost = SubtensorModule::get_network_lock_cost();

        // Give enough balance to the coldkey for the lock cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, lock_cost + 10_000);
        // Register new network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
        ));
        assert_eq!(SubtensorModule::get_subnet_owner(1), coldkey_account_id);

        // Check that the owner is in governance
        assert!(SubnetOwners::is_member(&coldkey_account_id));

        // Set the maximum number of subnets to 1
        SubtensorModule::set_max_subnets(1); // Doesn't count the root network

        // ---- Register new network under other Owner
        // Set immunity period to 0; Allow immediate swap
        SubtensorModule::set_network_immunity_period(0);

        // Get lock cost
        let lock_cost_2 = SubtensorModule::get_network_lock_cost();

        // Give enough balance to the coldkey for the lock cost
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id2, lock_cost_2 + 10_000);
        // Register new network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id2),
        ));
        assert_eq!(
            // Should now own the same netuid
            SubtensorModule::get_subnet_owner(1),
            coldkey_account_id2
        );

        // Check that the new owner is in governance
        assert!(SubnetOwners::is_member(&coldkey_account_id2));

        // Check that the old owner is no longer in governance
        assert_ne!(SubnetOwners::is_member(&coldkey_account_id), true);
    });
}
