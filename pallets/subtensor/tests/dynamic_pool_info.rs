mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
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
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
        ));

        // Check initial dynamic pool info after registration
        let initial_pool_info = SubtensorModule::get_dynamic_pool_info(netuid).unwrap();

        assert_eq!(
            initial_pool_info.alpha_issuance.0, 0,
            "Alpha issuance should be initialized to 0"
        );
        assert_eq!(
            initial_pool_info.alpha_outstanding.0, lock_cost,
            "Alpha outstanding should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.alpha_reserve.0, lock_cost,
            "Alpha reserve should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.tao_reserve.0, lock_cost,
            "Tao reserve should be initialized to lock_cost"
        );
        assert_eq!(
            initial_pool_info.k.0,
            lock_cost as u128 * lock_cost as u128,
            "K value should be initialized to lock_cost^2"
        ); // Alpha Reserve x Tao Reserve
        assert_eq!(
            initial_pool_info.price.0, 1,
            "Price should be initialized to 1"
        ); //  Tao reserve / Alpha reserve
        assert_eq!(
            initial_pool_info.netuid.0, netuid,
            "NetUID should match the one used for registration"
        );

        let all_pool_infos = SubtensorModule::get_all_dynamic_pool_infos();
        assert_eq!(all_pool_infos.len(), 1); // Assuming only one network is added
        assert_eq!(all_pool_infos[0], Some(initial_pool_info));
    });
}
