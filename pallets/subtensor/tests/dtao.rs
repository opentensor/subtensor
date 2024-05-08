use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use sp_core::U256;
use substrate_fixed::types::I64F64;
mod mock;

#[macro_use]
mod helpers;

// To run just the tests in this file, use the following command:
// Use the following command to run the tests in this file with verbose logging:
// RUST_LOG=debug cargo test -p pallet-subtensor --test dtao

#[test]
fn test_add_subnet_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);
        let lock_cost = 100_000_000_000; // 100 TAO

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, lock_cost);
        // Check
        // -- that the lock cost is 100 TAO.
        // -- that the balance is 100 TAO.
        // -- that the root pool is empty.
        // -- that the root alpha pool is empty.
        // -- that the root price is 1.0.
        // -- that the root has zero k value.
        assert_eq!(SubtensorModule::get_network_lock_cost(), lock_cost);
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey), lock_cost);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 0),
            0
        ); // 1 subnets * 100 TAO lock cost.
        assert_eq!(SubtensorModule::get_total_stake_for_subnet(0), 0);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(0), 1.0);
        assert_eq!(SubtensorModule::get_tao_reserve(0), 0);
        assert_eq!(SubtensorModule::get_alpha_reserve(0), 0);
        assert_eq!(SubtensorModule::get_pool_k(0), 0);
        assert_eq!(SubtensorModule::is_subnet_dynamic(0), false);

        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(0)
        );
        // Register a network with this coldkey + hotkey for a lock cost of 100 TAO.
        step_block(1);
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));

        // Check:
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network has someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet is 100 TAO.
        // -- that the new alpha/tao price is 1.0.
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 100 ALPHA
        // -- that the k factor is 100 TAO * 100 ALPHA.
        // -- that the new network is dynamic
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // 200 TAO.
                                                                               // TODO:(sam)Decide how to deal with ED , as this account can only stake 199
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            ExistentialDeposit::get()
        ); // 0 TAO.
        assert_eq!(SubtensorModule::get_subnet_owner(1), coldkey);
        assert_eq!(SubtensorModule::get_subnetwork_n(1), 1);
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(1, 0).unwrap(),
            hotkey
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 1),
            100_000_000_000
        ); // 1 subnets * 100 TAO lock cost.
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(1),
            100_000_000_000
        );
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 1.0);
        assert_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 100_000_000_000);
        assert_eq!(
            SubtensorModule::get_pool_k(1),
            100_000_000_000 * 100_000_000_000
        );
        assert_eq!(SubtensorModule::is_subnet_dynamic(1), true);
        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(1)
        );

        // Register a new network
        assert_eq!(SubtensorModule::get_network_lock_cost(), lock_cost * 2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, lock_cost * 2);
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));

        // Check:
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network as someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet 2 is 400 TAO.
        // -- that the new alpha/tao price is 0.5.
        // -- that the tao reserve is 200 TAO.
        // -- that the alpha reserve is 400 ALPHA
        // -- that the k factor is 200 TAO * 400 ALPHA.
        // -- that the new network is dynamic
        assert_eq!(SubtensorModule::get_network_lock_cost(), 400_000_000_000); // 4 TAO.
                                                                               // TODO:(sam)Decide how to deal with ED , as this account can only stake 199
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            ExistentialDeposit::get()
        ); // 0 TAO.
        assert_eq!(SubtensorModule::get_subnet_owner(2), coldkey);
        assert_eq!(SubtensorModule::get_subnetwork_n(2), 1);
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(2, 0).unwrap(),
            hotkey
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey, 2),
            400_000_000_000
        ); // 2 subnets * 2 TAO lock cost.
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(2),
            400_000_000_000
        );
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.5);
        assert_eq!(SubtensorModule::get_tao_reserve(2), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(2), 400_000_000_000);
        assert_eq!(
            SubtensorModule::get_pool_k(2),
            200_000_000_000 * 400_000_000_000
        );
        assert_eq!(SubtensorModule::is_subnet_dynamic(2), true);
        log::info!(
            "Alpha Outstanding is {:?}",
            SubtensorModule::get_alpha_outstanding(2)
        );

        // Let's remove all of our stake from subnet 2.
        // Check:
        // -- that the balance is initially 0
        // -- that the unstake event is ok.
        // -- that the balance is 100 TAO. Given the slippage.
        // -- that the price per alpha has changed to 0.125
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 800 ALPHA
        // -- that the k factor is 100 TAO * 400 ALPHA. (unchanged)
        // TODO:(sam)Decide how to deal with ED , free balance will always be 1
        assert_eq!(Balances::free_balance(coldkey), ExistentialDeposit::get());
        // We set this to zero , otherwise the alpha calculation is off due to the fact that many tempos will be run
        // over the default lock period (3 months)
        SubtensorModule::set_subnet_owner_lock_period(0);
        assert_eq!(
            SubtensorModule::get_pool_k(2),
            200_000_000_000 * 400_000_000_000
        );

        run_to_block(3);
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            2,
            400_000_000_000
        ));
        // assert_eq!( Balances::free_balance(coldkey), 100_000_000_000);
        // Also use more rigour calculation for slippage via K
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.125);
        assert_eq!(
            round_to_significant_figures(SubtensorModule::get_tao_reserve(2), 3),
            100_000_000_000
        );
        // Yet another ugly approximation
        assert_eq!(
            round_to_significant_figures(SubtensorModule::get_alpha_reserve(2), 2),
            800_000_000_000
        );

        log::info!(
            "Alpha Reserve is {:?}",
            SubtensorModule::get_alpha_reserve(2)
        );
        log::info!("Tao Reserve is {:?}", SubtensorModule::get_tao_reserve(2));

        // Let's run a block step.
        // Alpha pending emission is not zero at start because we already ran to block 3
        // and had emissions
        // Check
        // -- that the pending emission for the 2 subnets is correct
        // -- that the pending alpha emission of the 2 subnets is correct.
        let tao = 1_000_000_000;

        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(1), 0.9901); // diluted because of emissions in run_to_block
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(2), 0.125);
        step_block(1);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000u64);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 101);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 102);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 802);
        run_to_block(10);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1).div_ceil(tao), 100);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 101);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 108);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 808);
        run_to_block(30);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(1).div_ceil(tao), 107);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_reserve(2).div_ceil(tao), 101);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(1).div_ceil(tao), 121);
        assert_i64f64_approx_eq!(SubtensorModule::get_alpha_reserve(2).div_ceil(tao), 821);

        for _ in 0..100 {
            step_block(1);
            log::info!(
                "S1: {}, S2: {}",
                SubtensorModule::get_tao_per_alpha_price(1),
                SubtensorModule::get_tao_per_alpha_price(2)
            );
        }
    });
}

#[test]
fn test_stake_unstake() {
    new_test_ext(1).execute_with(|| {
        // init params.
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);

        // Register subnet.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000); // 100 TAO.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));
        assert_eq!(SubtensorModule::get_tao_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 1.0);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000); // 100 TAO.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            1,
            100_000_000_000
        ));
        assert_eq!(SubtensorModule::get_tao_reserve(1), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(1), 50_000_000_000);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(1), 4); // Price is increased from the stake operation.
    })
}

// To run this test, use the following command:
// cargo test -p pallet-subtensor --test dtao test_calculate_tempos
fn round_to_significant_figures(num: u64, significant_figures: u32) -> u64 {
    if num == 0 {
        return 0;
    }
    let digits = (num as f64).log10().floor() as u32 + 1; // Calculate the number of digits in the number
    let scale = 10u64.pow(digits - significant_figures); // Determine the scaling factor

    // Scale down, round, and scale up
    ((num as f64 / scale as f64).round() as u64) * scale
}
#[test]
fn test_calculate_tempos() {
    new_test_ext(1).execute_with(|| {
        let netuids = vec![1, 2, 3];
        let k = I64F64::from_num(10); // Example constant K
        let prices = vec![
            I64F64::from_num(100.0),
            I64F64::from_num(200.0),
            I64F64::from_num(300.0),
        ];

        let expected_tempos = vec![
            (1, 60), // Calculated tempo for netuid 1
            (2, 30), // Calculated tempo for netuid 2
            (3, 20), // Calculated tempo for netuid 3
        ];

        let tempos = SubtensorModule::calculate_tempos(&netuids, k, &prices).unwrap();
        assert_eq!(tempos, expected_tempos, "Tempos calculated incorrectly");

        // Edge case: Empty netuids and prices
        let empty_netuids = vec![];
        let empty_prices = vec![];
        let empty_tempos =
            SubtensorModule::calculate_tempos(&empty_netuids, k, &empty_prices).unwrap();
        assert!(
            empty_tempos.is_empty(),
            "Empty tempos should be an empty vector"
        );

        // Edge case: Zero prices
        let zero_prices = vec![
            I64F64::from_num(0.0),
            I64F64::from_num(0.0),
            I64F64::from_num(0.0),
        ];
        let zero_tempos = SubtensorModule::calculate_tempos(&netuids, k, &zero_prices).unwrap();
        assert_eq!(
            zero_tempos,
            vec![(1, 0), (2, 0), (3, 0)],
            "Zero prices should lead to zero tempos"
        );

        // Edge case: Negative prices
        let negative_prices = vec![
            I64F64::from_num(-100.0),
            I64F64::from_num(-200.0),
            I64F64::from_num(-300.0),
        ];
        let negative_tempos =
            SubtensorModule::calculate_tempos(&netuids, k, &negative_prices).unwrap();
        assert_eq!(
            negative_tempos, expected_tempos,
            "Negative prices should be treated as positive for tempo calculation"
        );

        // Edge case: Very large prices
        let large_prices = vec![
            I64F64::from_num(1e12),
            I64F64::from_num(2e12),
            I64F64::from_num(3e12),
        ];
        let large_tempos = SubtensorModule::calculate_tempos(&netuids, k, &large_prices).unwrap();
        assert_eq!(
            large_tempos, expected_tempos,
            "Large prices should scale similarly in tempo calculation"
        );

        // Edge case: Mismatched vector sizes
        let mismatched_prices = vec![I64F64::from_num(100.0), I64F64::from_num(200.0)]; // Missing price for netuid 3
        assert!(
            SubtensorModule::calculate_tempos(&netuids, k, &mismatched_prices).is_err(),
            "Mismatched vector sizes should result in an error"
        );

        // Edge case: Extremely small non-zero prices
        let small_prices = vec![
            I64F64::from_num(1e-12),
            I64F64::from_num(1e-12),
            I64F64::from_num(1e-12),
        ];
        let small_tempos = SubtensorModule::calculate_tempos(&netuids, k, &small_prices).unwrap();
        assert_eq!(
            small_tempos,
            vec![(1, 30), (2, 30), (3, 30)],
            "Extremely small prices should return same tempos"
        );

        // Edge case: Prices with high precision
        let high_precision_prices = vec![
            I64F64::from_num(100.123456789),
            I64F64::from_num(200.123456789),
            I64F64::from_num(300.123456789),
        ];
        let high_precision_tempos =
            SubtensorModule::calculate_tempos(&netuids, k, &high_precision_prices).unwrap();
        assert_eq!(
            high_precision_tempos,
            vec![(1, 59), (2, 30), (3, 20)],
            "High precision prices should affect tempo calculations"
        );
    });
}
