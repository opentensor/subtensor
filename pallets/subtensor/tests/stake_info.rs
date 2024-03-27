mod mock;
use codec::Compact;
use codec::Encode;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;

#[test]
fn test_get_stake_info_for_coldkey() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        let _uid: u16 = 0;
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
            SubtensorModule::get_subnet_stake_info_for_coldkey(coldkey.encode(), netuid)
                .iter()
                .map(|info| info.stake.0)
                .sum::<u64>(),
            10000
        );
    });
}

#[test]
fn test_get_stake_info_for_coldkeys() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        let _uid: u16 = 0;
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
            SubtensorModule::get_subnet_stake_info_for_coldkey(coldkey.encode(), netuid)
                .iter()
                .map(|info| info.stake.0)
                .sum::<u64>(),
            10000
        );
    });
}

#[test]
fn test_get_stake_info_for_multiple_coldkeys() {
    new_test_ext().execute_with(|| {
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
            SubtensorModule::get_subnet_stake_info_for_coldkey(coldkey1.encode(), netuid)
                .iter()
                .map(|info| info.stake.0)
                .sum::<u64>(),
            5000
        );

        assert_eq!(
            SubtensorModule::get_subnet_stake_info_for_coldkey(coldkey2.encode(), netuid)
                .iter()
                .map(|info| info.stake.0)
                .sum::<u64>(),
            3000
        );
    });
}

#[test]
fn test_get_total_subnet_stake() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        let _uid: u16 = 0;
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
    new_test_ext().execute_with(|| {
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
        let all_stake_info = SubtensorModule::get_all_stake_info_for_coldkey(coldkey.encode());

        // Assuming the function returns a Vec<(AccountId, u16, Compact<u64>)>
        assert_eq!(all_stake_info.len(), 2); // Ensure we have two entries

        let total_stake: u64 = all_stake_info.iter().map(|info| info.2 .0).sum();
        assert_eq!(total_stake, 15000); // Total stake should be the sum of stakes in both subnets
    });
}
