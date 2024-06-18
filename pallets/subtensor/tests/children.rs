use crate::mock::*;
use frame_support::{assert_ok};
mod mock;
use sp_core::U256;
use frame_system::Config;
use pallet_subtensor::Error;

/// The line `use pallet_subtensor::{Pallet, PendingEmission, PendingdHotkeyEmission};` is importing
/// specific items (`Pallet`, `PendingEmission`, `PendingdHotkeyEmission`) from the `pallet_subtensor`
/// module into the current scope. This allows the code in the current module to directly reference and
/// use these items without needing to fully qualify their paths each time they are used.
// use pallet_subtensor::{Pallet, PendingEmission, PendingdHotkeyEmission};

// To run this test specifically, use the following command:
// cargo test --test children test_add_singular_child -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_add_singular_child() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let child = U256::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        assert_eq!(
            SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey, child, netuid, u64::MAX),
            Err(Error::<Test>::SubNetworkDoesNotExist.into())
        );
        add_network(netuid, 0, 0);
        assert_eq!(
            SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey, child, 0, u64::MAX),
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
        assert_eq!(
            SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(child), hotkey, child, netuid, u64::MAX),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        assert_eq!(
            SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey, child, netuid, u64::MAX),
            Err(Error::<Test>::InvalidChild.into())
        );
        let child = U256::from(3);
        assert_ok!( SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey, child, netuid, u64::MAX) );
    })
}


// To run this test specifically, use the following command:
// cargo test --test children test_get_stake_with_children_and_parents -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_get_stake_with_children_and_parents() {
    new_test_ext(1).execute_with(|| {
        // Define network ID
        let netuid: u16 = 1;
        // Define hotkeys and coldkeys
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        // Add network with netuid
        add_network(netuid, 0, 0);
        // Create accounts if they do not exist
        SubtensorModule::create_account_if_non_existent(&coldkey0, &hotkey0);
        SubtensorModule::create_account_if_non_existent(&coldkey1, &hotkey1);
        // Increase stake on coldkey-hotkey accounts
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey1, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000);
        // Assert total stake for hotkeys
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 2000);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2000);
        // Assert stake with children and parents for hotkeys
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid), 2000);
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid), 2000);
        // Create a child relationship of 100% from hotkey0 to hotkey1
        assert_ok!(SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, hotkey1, netuid, u64::MAX));
        // Assert stake with children and parents after relationship
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid), 0);
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid), 4000);
        // Recreate a child relationship of 50% from hotkey0 to hotkey1
        assert_ok!(SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, hotkey1, netuid, u64::MAX / 2));
        // Assert stake with children and parents after 50% relationship
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid), 1001);
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid), 2999);
        // Create a new inverse child relationship of 100% from hotkey1 to hotkey0
        assert_ok!(SubtensorModule::do_set_child_singular(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, hotkey0, netuid, u64::MAX));
        // Assert stake with children and parents after inverse relationship
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid), 3001);
        assert_eq!(SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid), 999);
    });
}
