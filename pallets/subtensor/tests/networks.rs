use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use pallet_subtensor::{DissolveNetworkScheduleDuration, Event};
use sp_core::U256;

mod mock;

#[test]
fn test_registration_ok() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 2;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id,
            coldkey_account_id
        ));

        assert_ok!(SubtensorModule::user_remove_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));

        assert!(!SubtensorModule::if_subnet_exist(netuid))
    })
}

#[test]
fn test_schedule_dissolve_network_execution() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 2;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id,
            coldkey_account_id
        ));

        assert!(SubtensorModule::if_subnet_exist(netuid));

        assert_ok!(SubtensorModule::schedule_dissolve_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));

        let current_block = System::block_number();
        let execution_block = current_block + DissolveNetworkScheduleDuration::<Test>::get();

        System::assert_last_event(
            Event::DissolveNetworkScheduled {
                account: coldkey_account_id,
                netuid,
                execution_block,
            }
            .into(),
        );

        run_to_block(execution_block);
        assert!(!SubtensorModule::if_subnet_exist(netuid));
    })
}
