

use crate::mock::*;
mod mock;
use sp_core::U256;

#[test]
#[cfg(not(tarpaulin))]
fn test_coinbase_emission_distribution() {
    new_test_ext(1).execute_with(|| {
        // Call the coinbase function
        Pallet::<Test>::coinbase();

        // Check if the emissions are distributed correctly
        let current_block = Pallet::<Test>::get_current_block_as_u64();
        let subnets = Pallet::<Test>::get_all_netuids();

        for netuid in subnets {
            // Check if the subnet emissions are accumulated correctly
            let subnet_emission = PendingEmission::<Test>::get(netuid);
            assert!(subnet_emission > 0, "Subnet emission should be greater than 0");

            // Check if the epoch should run and emissions are distributed to hotkeys
            if Pallet::<Test>::should_run_epoch(netuid, current_block) {
                let hotkey_emission = PendingdHotkeyEmission::<Test>::iter().next().unwrap().1;
                assert!(hotkey_emission > 0, "Hotkey emission should be greater than 0");
            }
        }
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_accumulate_hotkey_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey: u64 = 1;
        let netuid: u16 = 1;
        let emission: u64 = 1000;

        // Call the accumulate_hotkey_emission function
        Pallet::<Test>::accumulate_hotkey_emission(hotkey, netuid, emission);

        // Check if the hotkey emission is accumulated correctly
        let accumulated_emission = PendingdHotkeyEmission::<Test>::get(hotkey);
        assert_eq!(accumulated_emission, emission, "Accumulated emission should match the input emission");
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_drain_hotkey_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey: u64 = 1;
        let emission: u64 = 1000;
        let block_number: u64 = 1;

        // Set initial emission
        PendingdHotkeyEmission::<Test>::insert(hotkey, emission);

        // Call the drain_hotkey_emission function
        Pallet::<Test>::drain_hotkey_emission(hotkey, emission, block_number);

        // Check if the hotkey emission is drained correctly
        let remaining_emission = PendingdHotkeyEmission::<Test>::get(hotkey);
        assert_eq!(remaining_emission, 0, "Remaining emission should be 0 after draining");
    });
}