use super::mock::*;
use crate::*;
use sp_core::U256;
use substrate_fixed::types::I96F32;

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --workspace --test staking2 -- test_swap_tao_for_alpha_dynamic_mechanism --exact --nocapture
#[test]
fn test_stake_base_case() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let tao_to_swap = 1_000_000_000; // 1 TAO

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, 1);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial total stake
        let initial_total_stake = TotalStake::<Test>::get();

        // Perform swap
        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, tao_to_swap);

        // Verify correct alpha calculation using constant product formula
        let k: I96F32 =
            I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_alpha: I96F32 = I96F32::from_num(initial_subnet_alpha)
            - (k / (I96F32::from_num(initial_subnet_tao + tao_to_swap)));
        let expected_alpha_u64 = expected_alpha.to_num::<u64>();

        assert_eq!(
            alpha_received, expected_alpha_u64,
            "Alpha received calculation is incorrect"
        );

        // Check subnet updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + tao_to_swap,
            "Subnet TAO not updated correctly"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha - alpha_received,
            "Subnet Alpha In not updated correctly"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha + alpha_received,
            "Subnet Alpha Out not updated correctly"
        );

        // Check total stake update
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake + tao_to_swap,
            "Total stake not updated correctly"
        );
    });
}

// Test: Share-based Staking System
// This test verifies the functionality of the share-based staking system where:
// 1. Stakes are represented as shares in a pool
// 2. Multiple coldkeys can stake to a single hotkey
// 3. Direct hotkey stakes are distributed proportionally among existing coldkey stakes
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_share_based_staking --exact --show-output
#[test]
fn test_share_based_staking() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let primary_hotkey = U256::from(1);
        let primary_coldkey = U256::from(2);
        let stake_amount = 1_000_000_000; // 1 TAO

        // Test Case 1: Initial Stake
        // The first stake should create shares 1:1 with the staked amount
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
            stake_amount,
        );
        let initial_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        log::info!(
            "Initial stake: {} = {} + {} = {}",
            initial_stake,
            0,
            stake_amount,
            stake_amount
        );
        assert_eq!(
            initial_stake, stake_amount,
            "Initial stake should match the staked amount exactly"
        );

        // Test Case 2: Additional Stake to Same Account
        // Adding more stake to the same account should increase shares proportionally
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
            stake_amount,
        );
        let stake_after_second = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        log::info!(
            "Stake after second deposit: {} = {} + {} = {}",
            stake_after_second,
            initial_stake,
            stake_amount,
            initial_stake + stake_amount
        );
        assert!(
            (stake_after_second as i64 - (initial_stake + stake_amount) as i64).abs() <= 1,
            "Total stake should double after second deposit (within rounding error)"
        );

        // Test Case 3: Direct Hotkey Stake
        // When staking directly to hotkey, the stake should be distributed proportionally
        SubtensorModule::increase_stake_for_hotkey_on_subnet(&primary_hotkey, netuid, stake_amount);
        let stake_after_direct = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        log::info!(
            "Stake after direct hotkey deposit: {} = {} + {} = {}",
            stake_after_direct,
            stake_after_second,
            stake_amount,
            stake_after_second + stake_amount
        );
        assert!(
            (stake_after_direct as i64 - (stake_after_second + stake_amount) as i64).abs() <= 1,
            "Direct hotkey stake should be added to existing stake (within rounding error)"
        );

        // Test Case 4: Multiple Coldkey Support
        // System should support multiple coldkeys staking to the same hotkey
        let secondary_coldkey = U256::from(3);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &secondary_coldkey,
            netuid,
            stake_amount,
        );
        let secondary_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &secondary_coldkey,
            netuid,
        );
        log::info!(
            "Secondary coldkey stake: {} = {} + {} = {}",
            secondary_stake,
            0,
            stake_amount,
            stake_amount
        );
        assert!(
            (secondary_stake as i64 - (stake_amount) as i64).abs() <= 1,
            "Secondary coldkey should receive full stake amount (within rounding error)"
        );

        // Test Case 5: Total Stake Verification
        // Verify the total stake across all coldkeys matches expected amount
        let total_hotkey_stake =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&primary_hotkey, netuid);
        log::info!(
            "Total hotkey stake: {} = {}",
            total_hotkey_stake,
            stake_after_direct + stake_amount
        );
        assert!(
            (total_hotkey_stake as i64 - (stake_after_direct + stake_amount) as i64).abs() <= 1,
            "Total hotkey stake should match sum of all coldkey stakes"
        );

        // Test Case 6: Proportional Distribution
        // When adding stake directly to hotkey, it should be distributed proportionally
        SubtensorModule::increase_stake_for_hotkey_on_subnet(&primary_hotkey, netuid, stake_amount);
        let primary_final_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        let secondary_final_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &secondary_coldkey,
            netuid,
        );

        // Calculate expected proportional distribution
        let primary_expected = stake_after_direct as f64
            + stake_amount as f64 * (stake_after_direct as f64 / total_hotkey_stake as f64);
        let secondary_expected = secondary_stake as f64
            + stake_amount as f64 * (secondary_stake as f64 / total_hotkey_stake as f64);

        log::info!(
            "Primary final stake: {} (expected: {})",
            primary_final_stake,
            primary_expected
        );
        log::info!(
            "Secondary final stake: {} (expected: {})",
            secondary_final_stake,
            secondary_expected
        );

        assert!(
            (primary_final_stake as f64 - primary_expected).abs() <= 1.0,
            "Primary stake should increase proportionally"
        );
        assert!(
            (secondary_final_stake as f64 - secondary_expected).abs() <= 1.0,
            "Secondary stake should increase proportionally"
        );

        // Test Case 7: Stake Removal
        // Verify correct stake removal from both accounts
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
            stake_amount,
        );
        let primary_after_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        log::info!(
            "Primary stake after removal: {} = {} - {} = {}",
            primary_after_removal,
            primary_final_stake,
            stake_amount,
            primary_final_stake - stake_amount
        );
        assert!(
            (primary_after_removal as i64 - (primary_final_stake - stake_amount) as i64).abs() <= 1,
            "Stake removal should decrease balance by exact amount"
        );

        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &secondary_coldkey,
            netuid,
            stake_amount,
        );
        let secondary_after_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &secondary_coldkey,
            netuid,
        );
        log::info!(
            "Secondary stake after removal: {} = {} - {} = {}",
            secondary_after_removal,
            secondary_final_stake,
            stake_amount,
            secondary_final_stake - stake_amount
        );
        assert!(
            (secondary_after_removal as i64 - (secondary_final_stake - stake_amount) as i64).abs()
                <= 1,
            "Stake removal should decrease balance by exact amount"
        );

        // Test Case 8: Final Total Verification
        // Verify final total matches sum of remaining stakes
        let final_total = SubtensorModule::get_stake_for_hotkey_on_subnet(&primary_hotkey, netuid);
        log::info!(
            "Final total stake: {} = {} + {} = {}",
            final_total,
            primary_after_removal,
            secondary_after_removal,
            primary_after_removal + secondary_after_removal
        );
        assert!(
            (final_total as i64 - (primary_after_removal + secondary_after_removal) as i64).abs()
                <= 1,
            "Final total should match sum of remaining stakes"
        );

        // Additional Edge Cases to Test:

        // Test staking with zero amount
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
            0,
        );
        let zero_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        assert!(
            zero_stake == primary_after_removal,
            "Staking with zero amount should not change the stake"
        );

        // Test removing more stake than available
        let available_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        let excessive_amount = available_stake + 1000;
        log::info!(
            "Attempting to remove excessive stake: {} + 1000 = {}",
            available_stake,
            excessive_amount
        );
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
            excessive_amount,
        );
        let after_excessive_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &primary_coldkey,
            netuid,
        );
        log::info!(
            "Stake after attempting excessive removal: {}",
            after_excessive_removal
        );
        assert!(
            after_excessive_removal == available_stake,
            "Removing more stake performs no action"
        );

        // Test staking to non-existent hotkey
        let non_existent_hotkey = U256::from(4);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &non_existent_hotkey,
            &primary_coldkey,
            netuid,
            stake_amount,
        );
        let non_existent_hotkey_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &non_existent_hotkey,
            &primary_coldkey,
            netuid,
        );
        assert!(
            non_existent_hotkey_stake == stake_amount,
            "Staking to non-existent hotkey should initialize the stake"
        );

        // Test removing stake from non-existent coldkey
        let non_existent_coldkey = U256::from(5);
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &primary_hotkey,
            &non_existent_coldkey,
            netuid,
            stake_amount,
        );
        let non_existent_coldkey_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &primary_hotkey,
                &non_existent_coldkey,
                netuid,
            );
        assert!(
            non_existent_coldkey_stake == 0,
            "Removing stake from non-existent coldkey should not change the stake"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_share_based_staking_denominator_precision --exact --show-output
#[test]
fn test_share_based_staking_denominator_precision() {
    // Test case amounts: stake, unstake, inject, tolerance
    [
        (1_000, 990),
        (1_000, 999),
        (1_000_000, 990_000),
        (1_000_000, 999_990),
        (1_000_000_000, 999_999_990),
        (1_000_000_000_000, 999_999_999_990),
    ]
    .iter()
    .for_each(|test_case| {
        new_test_ext(1).execute_with(|| {
            let netuid = 1;
            let hotkey1 = U256::from(1);
            let coldkey1 = U256::from(2);
            let stake_amount = test_case.0;
            let unstake_amount = test_case.1;

            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey1,
                netuid,
                stake_amount,
            );
            assert_eq!(
                Alpha::<Test>::get((hotkey1, coldkey1, netuid)),
                stake_amount
            );
            SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey1,
                netuid,
                unstake_amount,
            );

            let stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid,
            );
            assert_eq!(stake1, stake_amount - unstake_amount);
        });
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_share_based_staking_denominator_precision_2 --exact --show-output --nocapture
#[test]
fn test_share_based_staking_stake_unstake_inject() {
    // Test case amounts: stake, unstake, inject, tolerance
    [
        (1_000, 999, 1_000_000, 0),
        (1_000_000, 999_999, 100_000_000, 0),
        (1_000_000, 900_000, 100_000_000, 0),
        (100_000_000_000, 1_000_000_000, 1_000_000_000_000, 1),
        (100_000_000_000, 99_000_000_000, 1_000_000_000_000, 1),
        (100_000_000_000, 99_999_999_500, 1_000_000_000_000, 1),
        (100_000_000_000, 99_999_999_500, 1_234_567_890, 1),
    ]
    .iter()
    .for_each(|test_case| {
        new_test_ext(1).execute_with(|| {
            let netuid = 1;
            let hotkey1 = U256::from(1);
            let coldkey1 = U256::from(2);
            let coldkey2 = U256::from(3);
            let stake_amount = test_case.0;
            let unstake_amount = test_case.1;
            let inject_amount = test_case.2;
            let tolerance = test_case.3;

            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey1,
                netuid,
                stake_amount,
            );
            SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey1,
                netuid,
                unstake_amount,
            );
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey2,
                netuid,
                stake_amount,
            );
            SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey2,
                netuid,
                unstake_amount,
            );
            SubtensorModule::increase_stake_for_hotkey_on_subnet(&hotkey1, netuid, inject_amount);

            let stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid,
            );
            let stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid,
            );

            assert!(
                (stake1 as i64
                    - (stake_amount as i64 - unstake_amount as i64 + (inject_amount / 2) as i64))
                    .abs()
                    <= tolerance
            );
            assert!(
                (stake2 as i64
                    - (stake_amount as i64 - unstake_amount as i64 + (inject_amount / 2) as i64))
                    .abs()
                    <= tolerance
            );
        });
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_share_based_staking_stake_inject_stake_new --exact --show-output --nocapture
#[test]
fn test_share_based_staking_stake_inject_stake_new() {
    // Test case amounts: stake, inject, stake, tolerance
    [
        (1, 2_000_000_000, 500_000_000, 1),
        (1, 5_000_000_000, 50_000_000, 1),
        (500_000_000, 1_000_000_000, 1_000_000_000, 1),
    ]
    .iter()
    .for_each(|test_case| {
        new_test_ext(1).execute_with(|| {
            let netuid = 1;
            let hotkey1 = U256::from(1);
            let coldkey1 = U256::from(2);
            let coldkey2 = U256::from(3);
            let stake_amount = test_case.0;
            let inject_amount = test_case.1;
            let stake_amount_2 = test_case.2;
            let tolerance = test_case.3;

            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey1,
                netuid,
                stake_amount,
            );
            SubtensorModule::increase_stake_for_hotkey_on_subnet(&hotkey1, netuid, inject_amount);
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &coldkey2,
                netuid,
                stake_amount_2,
            );

            let stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid,
            );
            let stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid,
            );

            assert!((stake1 as i64 - (stake_amount + inject_amount) as i64).abs() <= tolerance);
            assert!((stake2 as i64 - stake_amount_2 as i64).abs() <= tolerance);
        });
    });
}

// Tests the moving average alpha price calculation
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_moving_average_alpha_price --exact --show-output --nocapture
// #[test]
// fn test_moving_average_alpha_price() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = 1;
        
//         // Set initial price to 1.0
//         SubnetMovingPrice::<Test>::insert(netuid, SubtensorModule::encode_price(I96F32::from_num(1.0)));

//         // Set alpha price to 2.0 by setting TAO and alpha reserves
//         SubnetTAO::<Test>::set(netuid, 2_000_000);
//         SubnetAlphaIn::<Test>::set(netuid, 1_000_000);

//         // Get moving average price
//         let moving_price = SubtensorModule::get_moving_average_alpha_price(netuid);
//         log::info!("Moving average price: {:?}", moving_price);

//         // With alpha = 0.99:
//         // new_price = 2.0 * 0.99 = 1.98
//         // old_price = 1.0 * 0.01 = 0.01
//         // moving_price = 1.98 + 0.01 = 1.99
//         let expected = I96F32::from_num(1.99);
//         let tolerance = I96F32::from_num(0.01);
//         log::info!("Expected price: {:?}", expected);
//         log::info!("Tolerance: {:?}", tolerance);
//         log::info!("Difference: {:?}", (moving_price - expected).abs());
        
//         assert!((moving_price - expected).abs() <= tolerance);

//         // Verify storage was updated
//         let stored_price = SubtensorModule::decode_price(SubnetPrice::<Test>::get(netuid));
//         let stored_moving = SubtensorModule::decode_price(SubnetMovingPrice::<Test>::get(netuid));
        
//         assert!((stored_price - I96F32::from_num(2.0)).abs() <= tolerance);
//         assert!((stored_moving - expected).abs() <= tolerance);
//     });
// }

// // // Tests the moving average alpha price calculation with zero alpha reserves
// // // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_moving_average_alpha_price_zero_alpha --exact --show-output --nocapture
// // #[test]
// // fn test_moving_average_alpha_price_zero_alpha() {
// //     new_test_ext(1).execute_with(|| {
// //         let netuid = 1;

// //         // Set initial prices
// //         SubnetMovingPrice::<Test>::insert(netuid, SubtensorModule::encode_price(I96F32::from_num(1.0)));

// //         // Set alpha reserves to 0
// //         SubnetTAO::<Test>::set(netuid, 1_000_000);
// //         SubnetAlphaIn::<Test>::set(netuid, 0);

// //         let moving_price = SubtensorModule::get_moving_average_alpha_price(netuid);

// //         // With zero alpha reserves, price should be 0
// //         let expected = I96F32::from_num(0.01); // 1.0 * 0.01 (old price contribution)
// //         let tolerance = I96F32::from_num(0.001);

// //         assert!((moving_price - expected).abs() <= tolerance);
// //     });
// // }

// // // Tests the moving average alpha price calculation for root subnet
// // // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::staking2::test_moving_average_alpha_price_root --exact --show-output --nocapture
// // #[test]
// // fn test_moving_average_alpha_price_root() {
// //     new_test_ext(1).execute_with(|| {
// //         let root_netuid = 0;

// //         // Set some arbitrary TAO and alpha values that should be ignored
// //         SubnetTAO::<Test>::set(root_netuid, 2_000_000);
// //         SubnetAlphaIn::<Test>::set(root_netuid, 1_000_000);

// //         let moving_price = SubtensorModule::get_moving_average_alpha_price(root_netuid);

// //         // Root subnet should always return price of 1.0
// //         let expected = I96F32::from_num(1.0);
// //         let tolerance = I96F32::from_num(0.001);

// //         assert!((moving_price - expected).abs() <= tolerance);
// //     });
// // }

