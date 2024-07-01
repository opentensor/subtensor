mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use pallet_subtensor::types::SubnetType;
use sp_core::U256;

#[test]
fn test_dynamic_pool_info() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);
        let lock_cost = SubtensorModule::get_network_lock_cost();

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 500_000_000_000_000); // 500 TAO.
        log::info!("Network lock cost is {:?}", lock_cost);

        // Register a network
        assert_ok!(SubtensorModule::user_add_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            SubnetType::DTAO
        ));

        // Check initial dynamic pool info after registration
        let initial_pool_info = SubtensorModule::get_dynamic_pool_info_v2(netuid).unwrap();

        assert_eq!(
            initial_pool_info.alpha_issuance, 0,
            "Alpha issuance should be initialized to 0"
        );
        assert_eq!(
            initial_pool_info.alpha_outstanding, lock_cost,
            "Alpha outstanding should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.alpha_reserve, lock_cost,
            "Alpha reserve should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.tao_reserve, lock_cost,
            "Tao reserve should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.netuid, netuid,
            "NetUID should match the one used for registration"
        );

        let all_pool_infos = SubtensorModule::get_all_dynamic_pool_infos_v2();
        assert_eq!(all_pool_infos.len(), 1); // Assuming only one network is added
        assert_eq!(all_pool_infos[0], initial_pool_info);
    });
}
