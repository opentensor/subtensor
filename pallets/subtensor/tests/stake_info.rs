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
