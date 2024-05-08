mod mock;
use codec::Compact;
use codec::Encode;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use pallet_subtensor::types::TensorBytes;
use sp_core::U256;

#[test]
fn test_get_stake_info_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let coldkey = U256::from(0);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            netuid,
            10000
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_info_for_coldkey(
                TensorBytes::from(coldkey.encode()),
                netuid
            )
            .iter()
            .map(|info| info.stake.0)
            .sum::<u64>(),
            10000
        );
    });
}

#[test]
fn test_get_stake_info_for_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            netuid,
            10000
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_info_for_coldkey(
                TensorBytes::from(coldkey.encode()),
                netuid
            )
            .iter()
            .map(|info| info.stake.0)
            .sum::<u64>(),
            10000
        );
    });
}

#[test]
fn test_get_stake_info_for_multiple_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;

        // Create multiple coldkeys and hotkeys
        let coldkey1 = U256::from(1);
        let hotkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey2 = U256::from(2);

        add_network(netuid, tempo, 0);

        // Register neurons and add balance for each coldkey
        register_ok_neuron(netuid, hotkey1, coldkey1, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            5000
        ));

        register_ok_neuron(netuid, hotkey2, coldkey2, 39420843);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            3000
        ));

        // Assert individual stakes
        assert_eq!(
            SubtensorModule::get_subnet_stake_info_for_coldkey(
                TensorBytes::from(coldkey1.encode()),
                netuid
            )
            .iter()
            .map(|info| info.stake.0)
            .sum::<u64>(),
            5000
        );

        assert_eq!(
            SubtensorModule::get_subnet_stake_info_for_coldkey(
                TensorBytes::from(coldkey2.encode()),
                netuid
            )
            .iter()
            .map(|info| info.stake.0)
            .sum::<u64>(),
            3000
        );
    });
}

#[test]
fn test_get_total_subnet_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            netuid,
            10000
        ));
        assert_eq!(
            SubtensorModule::get_total_subnet_stake(Compact(netuid).into()),
            Compact(10000)
        );
    });
}

#[test]
fn test_get_all_stake_info_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let tempo: u16 = 13;
        // Create coldkey and multiple hotkeys
        let coldkey = U256::from(0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);

        add_network(netuid1, tempo, 0);
        add_network(netuid2, tempo, 0);

        // Register neurons and add balance for the coldkey in different subnets
        register_ok_neuron(netuid1, hotkey1, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 20000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            netuid1,
            10000
        ));

        register_ok_neuron(netuid2, hotkey2, coldkey, 39420843);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            netuid2,
            5000
        ));

        // Retrieve all stake info for the coldkey and assert the results
        let all_stake_info =
            SubtensorModule::get_all_stake_info_for_coldkey(TensorBytes::from(coldkey.encode()));
        log::info!("all_stake_info: {:?}", all_stake_info);
        // Assuming the function returns a Vec<(AccountId, u16, Compact<u64>)>
        assert_eq!(all_stake_info.len(), 2); // Ensure we have two entries

        let total_stake: u64 = all_stake_info.iter().map(|info| info.2 .0).sum();
        assert_eq!(total_stake, 15000); // Total stake should be the sum of stakes in both subnets
    });
}

#[test]
fn test_get_all_stake_info_for_coldkey_2() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let tempo: u16 = 13;

        // Create coldkey and multiple hotkeys
        let coldkey = U256::from(0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);

        add_network(netuid1, tempo, 0);
        add_network(netuid2, tempo, 0);

        // Assert that stake info is 0 before adding stake
        let initial_stake_info =
            SubtensorModule::get_all_stake_info_for_coldkey(TensorBytes::from(coldkey.encode()));
        log::info!("initial_stake_info: {:?}", initial_stake_info);
        let initial_total_stake: u64 = initial_stake_info.iter().map(|info| info.2 .0).sum();
        assert_eq!(initial_total_stake, 0, "Initial total stake should be 0");

        // Register neurons and add balance for the coldkey in different subnets
        register_ok_neuron(netuid1, hotkey1, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 20000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            netuid1,
            10000
        ));

        register_ok_neuron(netuid2, hotkey2, coldkey, 39420843);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            netuid2,
            5000
        ));

        // Retrieve all stake info for the coldkey and assert the results
        let all_stake_info =
            SubtensorModule::get_all_stake_info_for_coldkey(TensorBytes::from(coldkey.encode()));
        log::info!("all_stake_info: {:?}", all_stake_info);
        assert_eq!(all_stake_info.len(), 2); // Ensure we have two entries

        let total_stake: u64 = all_stake_info.iter().map(|info| info.2 .0).sum();
        assert_eq!(total_stake, 15000);
    });
}

#[test]
fn test_get_all_subnet_stake_info_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let tempo: u16 = 13;

        // Create coldkey and multiple hotkeys
        let coldkey = U256::from(0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);

        add_network(netuid1, tempo, 0);
        add_network(netuid2, tempo, 0);

        // Register neurons and add balance for the coldkey in different subnets
        register_ok_neuron(netuid1, hotkey1, coldkey, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 20000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            netuid1,
            10000
        ));

        register_ok_neuron(netuid2, hotkey2, coldkey, 39420843);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            netuid2,
            5000
        ));

        // Retrieve all stake info for the coldkey and assert the results
        let all_stake_info = SubtensorModule::get_all_subnet_stake_info_for_coldkey(
            TensorBytes::from(coldkey.encode()),
        );
        assert_eq!(all_stake_info.len(), 2); // Ensure we have two entries

        let total_stake: u64 = all_stake_info.iter().map(|info| info.stake.0).sum();
        assert_eq!(total_stake, 15000); // Total stake should be the sum of stakes in both subnets
    });
}

#[test]
fn test_get_all_subnet_stake_info_for_coldkey_32_subnets() {
    new_test_ext(1).execute_with(|| {
        let tempo: u16 = 13;

        // Create coldkey and hotkeys
        let coldkey = U256::from(0);
        let mut hotkeys = Vec::new();

        // Create 32 subnets and register neurons
        for i in 1..=32 {
            let netuid = i;
            let hotkey = U256::from(i);
            hotkeys.push(hotkey);

            add_network(netuid, tempo, 0);
            register_ok_neuron(netuid, hotkey, coldkey, 39420840 + i as u64);
        }

        // Add balance to the coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 320000);

        // Add subnet stake for each subnet
        for (i, hotkey) in hotkeys.iter().enumerate() {
            let netuid = (i + 1) as u16;
            let stake_amount = 1000;

            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
                *hotkey,
                netuid,
                stake_amount
            ));
        }

        // Retrieve all stake info for the coldkey and assert the results
        let all_stake_info = SubtensorModule::get_all_subnet_stake_info_for_coldkey(
            TensorBytes::from(coldkey.encode()),
        );
        assert_eq!(all_stake_info.len(), 32); // Ensure we have 32 entries

        let total_stake: u64 = all_stake_info.iter().map(|info| info.stake.0).sum();
        let expected_total_stake = 32 * 1000; // Total stake should be the sum of stakes in all 32 subnets
        assert_eq!(total_stake, expected_total_stake);
    });
}

#[test]
fn test_get_total_stake_for_each_subnet_single_stake() {
    new_test_ext(1).execute_with(|| {
        let tempo: u16 = 13;

        // Create coldkey and hotkeys
        let coldkey = U256::from(0);
        let mut hotkeys = Vec::new();

        // Create 32 subnets and register neurons
        for i in 1..=32 {
            let netuid = i;
            let hotkey = U256::from(i);
            hotkeys.push(hotkey);

            add_network(netuid, tempo, 0);
            register_ok_neuron(netuid, hotkey, coldkey, 39420840 + i as u64);
        }

        // Add balance to the coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 320000);

        // Add subnet stake for each subnet
        for (i, hotkey) in hotkeys.iter().enumerate() {
            let netuid = (i + 1) as u16;
            let stake_amount = (1000 + netuid) as u64;

            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
                *hotkey,
                netuid,
                stake_amount
            ));
        }

        // Retrieve total stake info for each subnet
        let total_stake = SubtensorModule::get_total_stake_for_each_subnet();
        assert_eq!(total_stake.len(), 32); // Ensure we have 32 entries

        total_stake.iter().for_each(|&s| {
            assert_eq!(s.1, Compact((1000 + s.0) as u64));
        });
    });
}

#[test]
fn test_get_total_stake_for_each_subnet_double_stake() {
    new_test_ext(1).execute_with(|| {
        let tempo: u16 = 13;

        // Create coldkey and hotkeys
        let coldkey = U256::from(0);
        let mut hotkeys = Vec::new();

        // Create 32 subnets and register neurons
        for i in 1..=32 {
            let netuid = i;
            let hotkey = U256::from(i);
            hotkeys.push(hotkey);

            add_network(netuid, tempo, 0);
            register_ok_neuron(netuid, hotkey, coldkey, 39420840 + i as u64);
        }

        // Add balance to the coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 320000);

        // Add subnet stake for each subnet
        for (i, hotkey) in hotkeys.iter().enumerate() {
            let netuid = (i + 1) as u16;
            let stake_amount = 1000;

            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
                *hotkey,
                netuid,
                stake_amount
            ));

            // Add stake to another subnet
            let netuid = ((i + 1) % 32 + 1) as u16;
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(coldkey),
                *hotkey,
                netuid,
                stake_amount
            ));
        }

        // Retrieve total stake info for each subnet
        let total_stake = SubtensorModule::get_total_stake_for_each_subnet();
        assert_eq!(total_stake.len(), 32); // Ensure we have 32 entries

        total_stake.iter().for_each(|&s| {
            assert_eq!(s.1, Compact(2000u64));
        });
    });
}
