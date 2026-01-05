#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use approx::assert_abs_diff_eq;
use frame_support::{assert_noop, assert_ok};
use sp_arithmetic::Perquintill;
use sp_runtime::DispatchError;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::NetUid;
use subtensor_swap_interface::Order as OrderT;

use super::*;
use crate::mock::*;
use crate::pallet::swap_step::*;

// Run all tests:
// cargo test --package pallet-subtensor-swap --lib -- pallet::tests --nocapture

#[allow(dead_code)]
fn get_min_price() -> U64F64 {
    U64F64::from_num(Pallet::<Test>::min_price_inner::<TaoCurrency>())
        / U64F64::from_num(1_000_000_000)
}

#[allow(dead_code)]
fn get_max_price() -> U64F64 {
    U64F64::from_num(Pallet::<Test>::max_price_inner::<TaoCurrency>())
        / U64F64::from_num(1_000_000_000)
}

mod dispatchables {
    use super::*;

    #[test]
    fn test_set_fee_rate() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(1);
            let fee_rate = 500; // 0.76% fee

            assert_noop!(
                Swap::set_fee_rate(RuntimeOrigin::signed(666), netuid, fee_rate),
                DispatchError::BadOrigin
            );

            assert_ok!(Swap::set_fee_rate(RuntimeOrigin::root(), netuid, fee_rate));

            // Check that fee rate was set correctly
            assert_eq!(FeeRate::<Test>::get(netuid), fee_rate);

            let fee_rate = fee_rate * 2;
            assert_ok!(Swap::set_fee_rate(
                RuntimeOrigin::signed(1),
                netuid,
                fee_rate
            ));
            assert_eq!(FeeRate::<Test>::get(netuid), fee_rate);

            // Verify fee rate validation - should fail if too high
            let too_high_fee = MaxFeeRate::get() + 1;
            assert_noop!(
                Swap::set_fee_rate(RuntimeOrigin::root(), netuid, too_high_fee),
                Error::<Test>::FeeRateTooHigh
            );
        });
    }

    fn perquintill_to_f64(p: Perquintill) -> f64 {
        let parts = p.deconstruct() as f64;
        parts / 1_000_000_000_000_000_000_f64
    }

    /// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::dispatchables::test_adjust_protocol_liquidity_happy --exact --nocapture
    #[test]
    fn test_adjust_protocol_liquidity_happy() {
        // test case: tao_delta, alpha_delta
        [
            (0_u64, 0_u64),
            (0_u64, 1_u64),
            (1_u64, 0_u64),
            (1_u64, 1_u64),
            (0_u64, 10_u64),
            (10_u64, 0_u64),
            (10_u64, 10_u64),
            (0_u64, 100_u64),
            (100_u64, 0_u64),
            (100_u64, 100_u64),
            (0_u64, 1_000_u64),
            (1_000_u64, 0_u64),
            (1_000_u64, 1_000_u64),
            (1_000_000_u64, 0_u64),
            (0_u64, 1_000_000_u64),
            (1_000_000_u64, 1_000_000_u64),
            (1_000_000_000_u64, 0_u64),
            (0_u64, 1_000_000_000_u64),
            (1_000_000_000_u64, 1_000_000_000_u64),
            (1_000_000_000_000_u64, 0_u64),
            (0_u64, 1_000_000_000_000_u64),
            (1_000_000_000_000_u64, 1_000_000_000_000_u64),
            (1_u64, 2_u64),
            (2_u64, 1_u64),
            (10_u64, 20_u64),
            (20_u64, 10_u64),
            (100_u64, 200_u64),
            (200_u64, 100_u64),
            (1_000_u64, 2_000_u64),
            (2_000_u64, 1_000_u64),
            (1_000_000_u64, 2_000_000_u64),
            (2_000_000_u64, 1_000_000_u64),
            (1_000_000_000_u64, 2_000_000_000_u64),
            (2_000_000_000_u64, 1_000_000_000_u64),
            (1_000_000_000_000_u64, 2_000_000_000_000_u64),
            (2_000_000_000_000_u64, 1_000_000_000_000_u64),
            (1_234_567_u64, 2_432_765_u64),
            (1_234_567_u64, 2_432_765_890_u64),
        ]
        .into_iter()
        .for_each(|(tao_delta, alpha_delta)| {
            new_test_ext().execute_with(|| {
                let netuid = NetUid::from(1);

                let tao_delta = TaoCurrency::from(tao_delta);
                let alpha_delta = AlphaCurrency::from(alpha_delta);

                // Initialize reserves and price
                let tao = TaoCurrency::from(1_000_000_000_000_u64);
                let alpha = AlphaCurrency::from(4_000_000_000_000_u64);
                TaoReserve::set_mock_reserve(netuid, tao);
                AlphaReserve::set_mock_reserve(netuid, alpha);
                let price_before = Swap::current_price(netuid);

                // Adjust reserves
                Swap::adjust_protocol_liquidity(netuid, tao_delta, alpha_delta);
                TaoReserve::set_mock_reserve(netuid, tao + tao_delta);
                AlphaReserve::set_mock_reserve(netuid, alpha + alpha_delta);

                // Check that price didn't change
                let price_after = Swap::current_price(netuid);
                assert_abs_diff_eq!(
                    price_before.to_num::<f64>(),
                    price_after.to_num::<f64>(),
                    epsilon = price_before.to_num::<f64>() / 1_000_000_000_000.
                );

                // Check that reserve weight was properly updated
                let new_tao = u64::from(tao + tao_delta) as f64;
                let new_alpha = u64::from(alpha + alpha_delta) as f64;
                let expected_quote_weight =
                    new_tao / (new_alpha * price_before.to_num::<f64>() + new_tao);
                let expected_quote_weight_delta = expected_quote_weight - 0.5;
                let res_weights = SwapBalancer::<Test>::get(netuid);
                let actual_quote_weight_delta =
                    perquintill_to_f64(res_weights.get_quote_weight()) - 0.5;
                let eps = expected_quote_weight / 1_000_000_000_000.;
                assert_abs_diff_eq!(
                    expected_quote_weight_delta,
                    actual_quote_weight_delta,
                    epsilon = eps
                );
            });
        });
    }

    /// This test case verifies that small gradual injections (like emissions in every block)
    /// in the worst case
    ///   - Do not cause price to change
    ///   - Result in the same weight change as one large injection
    ///
    /// This is a long test that only tests validity of weights math. Run again if changing
    /// Balancer::update_weights_for_added_liquidity
    ///
    /// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::dispatchables::test_adjust_protocol_liquidity_deltas --exact --nocapture
    #[ignore]
    #[test]
    fn test_adjust_protocol_liquidity_deltas() {
        // The number of times (blocks) over which gradual injections will be made
        // One year price drift due to precision is under 1e-6
        const ITERATIONS: u64 = 2_700_000;
        const PRICE_PRECISION: f64 = 0.000_001;
        const PREC_LARGE_DELTA: f64 = 0.001;
        const WEIGHT_PRECISION: f64 = 0.000_000_000_000_000_001;

        let initial_tao_reserve = TaoCurrency::from(1_000_000_000_000_000_u64);
        let initial_alpha_reserve = AlphaCurrency::from(10_000_000_000_000_000_u64);

        // test case: tao_delta, alpha_delta, price_precision
        [
            (0_u64, 0_u64, PRICE_PRECISION),
            (0_u64, 1_u64, PRICE_PRECISION),
            (1_u64, 0_u64, PRICE_PRECISION),
            (1_u64, 1_u64, PRICE_PRECISION),
            (0_u64, 10_u64, PRICE_PRECISION),
            (10_u64, 0_u64, PRICE_PRECISION),
            (10_u64, 10_u64, PRICE_PRECISION),
            (0_u64, 100_u64, PRICE_PRECISION),
            (100_u64, 0_u64, PRICE_PRECISION),
            (100_u64, 100_u64, PRICE_PRECISION),
            (0_u64, 987_u64, PRICE_PRECISION),
            (987_u64, 0_u64, PRICE_PRECISION),
            (876_u64, 987_u64, PRICE_PRECISION),
            (0_u64, 1_000_u64, PRICE_PRECISION),
            (1_000_u64, 0_u64, PRICE_PRECISION),
            (1_000_u64, 1_000_u64, PRICE_PRECISION),
            (0_u64, 1_234_u64, PRICE_PRECISION),
            (1_234_u64, 0_u64, PRICE_PRECISION),
            (1_234_u64, 4_321_u64, PRICE_PRECISION),
            (1_234_000_u64, 4_321_000_u64, PREC_LARGE_DELTA),
            (1_234_u64, 4_321_000_u64, PREC_LARGE_DELTA),
        ]
        .into_iter()
        .for_each(|(tao_delta, alpha_delta, price_precision)| {
            new_test_ext().execute_with(|| {
                let netuid1 = NetUid::from(1);

                let tao_delta = TaoCurrency::from(tao_delta);
                let alpha_delta = AlphaCurrency::from(alpha_delta);

                // Initialize realistically large reserves
                let mut tao = initial_tao_reserve;
                let mut alpha = initial_alpha_reserve;
                TaoReserve::set_mock_reserve(netuid1, tao);
                AlphaReserve::set_mock_reserve(netuid1, alpha);
                let price_before = Swap::current_price(netuid1);

                // Adjust reserves gradually
                for _ in 0..ITERATIONS {
                    Swap::adjust_protocol_liquidity(netuid1, tao_delta, alpha_delta);
                    tao += tao_delta;
                    alpha += alpha_delta;
                    TaoReserve::set_mock_reserve(netuid1, tao);
                    AlphaReserve::set_mock_reserve(netuid1, alpha);
                }

                // Check that price didn't change
                let price_after = Swap::current_price(netuid1);
                assert_abs_diff_eq!(
                    price_before.to_num::<f64>(),
                    price_after.to_num::<f64>(),
                    epsilon = price_precision
                );

                /////////////////////////

                // Now do one-time big injection with another netuid and compare weights

                let netuid2 = NetUid::from(2);

                // Initialize same large reserves
                TaoReserve::set_mock_reserve(netuid2, initial_tao_reserve);
                AlphaReserve::set_mock_reserve(netuid2, initial_alpha_reserve);

                // Adjust reserves by one large amount at once
                let tao_delta_once = TaoCurrency::from(ITERATIONS * u64::from(tao_delta));
                let alpha_delta_once = AlphaCurrency::from(ITERATIONS * u64::from(alpha_delta));
                Swap::adjust_protocol_liquidity(netuid2, tao_delta_once, alpha_delta_once);
                TaoReserve::set_mock_reserve(netuid2, initial_tao_reserve + tao_delta_once);
                AlphaReserve::set_mock_reserve(netuid2, initial_alpha_reserve + alpha_delta_once);

                // Compare reserve weights for netuid 1 and 2
                let res_weights1 = SwapBalancer::<Test>::get(netuid1);
                let res_weights2 = SwapBalancer::<Test>::get(netuid2);
                let actual_quote_weight1 = perquintill_to_f64(res_weights1.get_quote_weight());
                let actual_quote_weight2 = perquintill_to_f64(res_weights2.get_quote_weight());
                assert_abs_diff_eq!(
                    actual_quote_weight1,
                    actual_quote_weight2,
                    epsilon = WEIGHT_PRECISION
                );
            });
        });
    }

    /// Should work ok when initial alpha is zero
    /// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::dispatchables::test_adjust_protocol_liquidity_zero_alpha --exact --nocapture
    #[test]
    fn test_adjust_protocol_liquidity_zero_alpha() {
        // test case: tao_delta, alpha_delta
        [
            (0_u64, 0_u64),
            (0_u64, 1_u64),
            (1_u64, 0_u64),
            (1_u64, 1_u64),
            (0_u64, 10_u64),
            (10_u64, 0_u64),
            (10_u64, 10_u64),
            (0_u64, 100_u64),
            (100_u64, 0_u64),
            (100_u64, 100_u64),
            (0_u64, 1_000_u64),
            (1_000_u64, 0_u64),
            (1_000_u64, 1_000_u64),
            (1_000_000_u64, 0_u64),
            (0_u64, 1_000_000_u64),
            (1_000_000_u64, 1_000_000_u64),
            (1_000_000_000_u64, 0_u64),
            (0_u64, 1_000_000_000_u64),
            (1_000_000_000_u64, 1_000_000_000_u64),
            (1_000_000_000_000_u64, 0_u64),
            (0_u64, 1_000_000_000_000_u64),
            (1_000_000_000_000_u64, 1_000_000_000_000_u64),
            (1_u64, 2_u64),
            (2_u64, 1_u64),
            (10_u64, 20_u64),
            (20_u64, 10_u64),
            (100_u64, 200_u64),
            (200_u64, 100_u64),
            (1_000_u64, 2_000_u64),
            (2_000_u64, 1_000_u64),
            (1_000_000_u64, 2_000_000_u64),
            (2_000_000_u64, 1_000_000_u64),
            (1_000_000_000_u64, 2_000_000_000_u64),
            (2_000_000_000_u64, 1_000_000_000_u64),
            (1_000_000_000_000_u64, 2_000_000_000_000_u64),
            (2_000_000_000_000_u64, 1_000_000_000_000_u64),
            (1_234_567_u64, 2_432_765_u64),
            (1_234_567_u64, 2_432_765_890_u64),
        ]
        .into_iter()
        .for_each(|(tao_delta, alpha_delta)| {
            new_test_ext().execute_with(|| {
                let netuid = NetUid::from(1);

                let tao_delta = TaoCurrency::from(tao_delta);
                let alpha_delta = AlphaCurrency::from(alpha_delta);

                // Initialize reserves and price
                // broken state: Zero price because of zero alpha reserve
                let tao = TaoCurrency::from(1_000_000_000_000_u64);
                let alpha = AlphaCurrency::from(0_u64);
                TaoReserve::set_mock_reserve(netuid, tao);
                AlphaReserve::set_mock_reserve(netuid, alpha);
                let price_before = Swap::current_price(netuid);
                assert_eq!(price_before, U64F64::from_num(0));
                let new_tao = u64::from(tao + tao_delta) as f64;
                let new_alpha = u64::from(alpha + alpha_delta) as f64;

                // Adjust reserves
                Swap::adjust_protocol_liquidity(netuid, tao_delta, alpha_delta);
                TaoReserve::set_mock_reserve(netuid, tao + tao_delta);
                AlphaReserve::set_mock_reserve(netuid, alpha + alpha_delta);

                let res_weights = SwapBalancer::<Test>::get(netuid);
                let actual_quote_weight = perquintill_to_f64(res_weights.get_quote_weight());

                // Check that price didn't change
                let price_after = Swap::current_price(netuid);
                if new_alpha == 0. {
                    // If the pool state is still broken (∆x = 0), no change
                    assert_eq!(actual_quote_weight, 0.5);
                    assert_eq!(price_after, U64F64::from_num(0));
                } else {
                    // Price got fixed
                    let expected_price = new_tao / new_alpha;
                    assert_abs_diff_eq!(
                        expected_price,
                        price_after.to_num::<f64>(),
                        epsilon = price_before.to_num::<f64>() / 1_000_000_000_000.
                    );
                    assert_eq!(actual_quote_weight, 0.5);
                }
            });
        });
    }

    // #[test]
    // fn test_toggle_user_liquidity() {
    //     new_test_ext().execute_with(|| {
    //         let netuid = NetUid::from(101);

    //         assert!(!EnabledUserLiquidity::<Test>::get(netuid));

    //         assert_ok!(Swap::toggle_user_liquidity(
    //             RuntimeOrigin::root(),
    //             netuid.into(),
    //             true
    //         ));

    //         assert!(EnabledUserLiquidity::<Test>::get(netuid));

    //         assert_noop!(
    //             Swap::toggle_user_liquidity(RuntimeOrigin::signed(666), netuid.into(), true),
    //             DispatchError::BadOrigin
    //         );

    //         assert_ok!(Swap::toggle_user_liquidity(
    //             RuntimeOrigin::signed(1),
    //             netuid.into(),
    //             true
    //         ));

    //         assert_noop!(
    //             Swap::toggle_user_liquidity(
    //                 RuntimeOrigin::root(),
    //                 NON_EXISTENT_NETUID.into(),
    //                 true
    //             ),
    //             Error::<Test>::MechanismDoesNotExist
    //         );
    //     });
    // }
}

#[test]
fn test_swap_initialization() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Get reserves from the mock provider
        let tao = TaoCurrency::from(1_000_000_000u64);
        let alpha = AlphaCurrency::from(4_000_000_000u64);
        TaoReserve::set_mock_reserve(netuid, tao);
        AlphaReserve::set_mock_reserve(netuid, alpha);

        assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
        assert!(PalSwapInitialized::<Test>::get(netuid));

        // Verify current price is set
        let price = Pallet::<Test>::current_price(netuid);
        let expected_price = U64F64::from_num(0.25_f64);
        assert_abs_diff_eq!(
            price.to_num::<f64>(),
            expected_price.to_num::<f64>(),
            epsilon = 0.000000001
        );

        // Verify that swap reserve weight is initialized
        let reserve_weight = SwapBalancer::<Test>::get(netuid);
        assert_eq!(
            reserve_weight.get_quote_weight(),
            Perquintill::from_rational(1_u64, 2_u64),
        );

        // TODO: Revise when user liquidity is available
        // // Calculate expected liquidity
        // let expected_liquidity =
        //     helpers_128bit::sqrt((tao.to_u64() as u128).saturating_mul(alpha.to_u64() as u128))
        //         as u64;

        // // Get the protocol account
        // let protocol_account_id = Pallet::<Test>::protocol_account_id();

        // // Verify position created for protocol account
        // let positions = Positions::<Test>::iter_prefix_values((netuid, protocol_account_id))
        //     .collect::<Vec<_>>();
        // assert_eq!(positions.len(), 1);

        // let position = &positions[0];
        // assert_eq!(position.liquidity, expected_liquidity);
        // assert_eq!(position.fees_tao, 0);
        // assert_eq!(position.fees_alpha, 0);
    });
}

// TODO: Revise when user liquidity is available
// Test adding liquidity on top of the existing protocol liquidity
// #[test]
// fn test_add_liquidity_basic() {
//     new_test_ext().execute_with(|| {
//         let min_price = tick_to_price(TickIndex::MIN);
//         let max_price = tick_to_price(TickIndex::MAX);
//         let max_tick = price_to_tick(max_price);
//         assert_eq!(max_tick, TickIndex::MAX);

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(NetUid::from(1)));
//         let current_price = Pallet::<Test>::current_price(NetUid::from(1)).to_num::<f64>();
//         let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

//         // As a user add liquidity with all possible corner cases
//         //   - Initial price is 0.25
//         //   - liquidity is expressed in RAO units
//         // Test case is (price_low, price_high, liquidity, tao, alpha)
//         [
//             // Repeat the protocol liquidity at maximum range: Expect all the same values
//             (
//                 min_price,
//                 max_price,
//                 2_000_000_000_u64,
//                 1_000_000_000_u64,
//                 4_000_000_000_u64,
//             ),
//             // Repeat the protocol liquidity at current to max range: Expect the same alpha
//             (
//                 current_price_high,
//                 max_price,
//                 2_000_000_000_u64,
//                 0,
//                 4_000_000_000,
//             ),
//             // Repeat the protocol liquidity at min to current range: Expect all the same tao
//             (
//                 min_price,
//                 current_price_low,
//                 2_000_000_000_u64,
//                 1_000_000_000,
//                 0,
//             ),
//             // Half to double price - just some sane wothdraw amounts
//             (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
//             // Both below price - tao is non-zero, alpha is zero
//             (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
//             // Both above price - tao is zero, alpha is non-zero
//             (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
//         ]
//         .into_iter()
//         .enumerate()
//         .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3, v.4))
//         .for_each(
//             |(netuid, price_low, price_high, liquidity, expected_tao, expected_alpha)| {
//                 assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//                 // Calculate ticks (assuming tick math is tested separately)
//                 let tick_low = price_to_tick(price_low);
//                 let tick_high = price_to_tick(price_high);

//                 // Get tick infos and liquidity before adding (to account for protocol liquidity)
//                 let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
//                 let tick_high_info_before =
//                     Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
//                 let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

//                 // Add liquidity
//                 let (position_id, tao, alpha) = Pallet::<Test>::do_add_liquidity(
//                     netuid,
//                     &OK_COLDKEY_ACCOUNT_ID,
//                     &OK_HOTKEY_ACCOUNT_ID,
//                     tick_low,
//                     tick_high,
//                     liquidity,
//                 )
//                 .unwrap();

//                 assert_abs_diff_eq!(tao, expected_tao, epsilon = tao / 1000);
//                 assert_abs_diff_eq!(alpha, expected_alpha, epsilon = alpha / 1000);

//                 // Check that low and high ticks appear in the state and are properly updated
//                 let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
//                 let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
//                 let expected_liquidity_net_low = liquidity as i128;
//                 let expected_liquidity_gross_low = liquidity;
//                 let expected_liquidity_net_high = -(liquidity as i128);
//                 let expected_liquidity_gross_high = liquidity;

//                 assert_eq!(
//                     tick_low_info.liquidity_net - tick_low_info_before.liquidity_net,
//                     expected_liquidity_net_low,
//                 );
//                 assert_eq!(
//                     tick_low_info.liquidity_gross - tick_low_info_before.liquidity_gross,
//                     expected_liquidity_gross_low,
//                 );
//                 assert_eq!(
//                     tick_high_info.liquidity_net - tick_high_info_before.liquidity_net,
//                     expected_liquidity_net_high,
//                 );
//                 assert_eq!(
//                     tick_high_info.liquidity_gross - tick_high_info_before.liquidity_gross,
//                     expected_liquidity_gross_high,
//                 );

//                 // Liquidity position at correct ticks
//                 assert_eq!(
//                     Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
//                     1
//                 );

//                 let position =
//                     Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
//                 assert_eq!(position.liquidity, liquidity);
//                 assert_eq!(position.tick_low, tick_low);
//                 assert_eq!(position.tick_high, tick_high);
//                 assert_eq!(position.fees_alpha, 0);
//                 assert_eq!(position.fees_tao, 0);

//                 // Current liquidity is updated only when price range includes the current price
//                 let expected_liquidity =
//                     if (price_high > current_price) && (price_low <= current_price) {
//                         liquidity_before + liquidity
//                     } else {
//                         liquidity_before
//                     };

//                 assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity)
//             },
//         );
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn test_add_liquidity_max_limit_enforced() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(1);
//         let liquidity = 2_000_000_000_u64;
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         let limit = MaxPositions::get() as usize;

//         for _ in 0..limit {
//             Pallet::<Test>::do_add_liquidity(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 TickIndex::MIN,
//                 TickIndex::MAX,
//                 liquidity,
//             )
//             .unwrap();
//         }

//         let test_result = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             TickIndex::MIN,
//             TickIndex::MAX,
//             liquidity,
//         );

//         assert_err!(test_result, Error::<Test>::MaxPositionsExceeded);
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn test_add_liquidity_out_of_bounds() {
//     new_test_ext().execute_with(|| {
//         [
//             // For our tests, we'll construct TickIndex values that are intentionally
//             // outside the valid range for testing purposes only
//             (
//                 TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
//                 TickIndex::MAX,
//                 1_000_000_000_u64,
//             ),
//             (
//                 TickIndex::MIN,
//                 TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
//                 1_000_000_000_u64,
//             ),
//             (
//                 TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
//                 TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
//                 1_000_000_000_u64,
//             ),
//             (
//                 TickIndex::new_unchecked(TickIndex::MIN.get() - 100),
//                 TickIndex::new_unchecked(TickIndex::MAX.get() + 100),
//                 1_000_000_000_u64,
//             ),
//             // Inverted ticks: high < low
//             (
//                 TickIndex::new_unchecked(-900),
//                 TickIndex::new_unchecked(-1000),
//                 1_000_000_000_u64,
//             ),
//             // Equal ticks: high == low
//             (
//                 TickIndex::new_unchecked(-10_000),
//                 TickIndex::new_unchecked(-10_000),
//                 1_000_000_000_u64,
//             ),
//         ]
//         .into_iter()
//         .enumerate()
//         .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2))
//         .for_each(|(netuid, tick_low, tick_high, liquidity)| {
//             assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//             // Add liquidity
//             assert_err!(
//                 Swap::do_add_liquidity(
//                     netuid,
//                     &OK_COLDKEY_ACCOUNT_ID,
//                     &OK_HOTKEY_ACCOUNT_ID,
//                     tick_low,
//                     tick_high,
//                     liquidity
//                 ),
//                 Error::<Test>::InvalidTickRange,
//             );
//         });
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn test_add_liquidity_over_balance() {
//     new_test_ext().execute_with(|| {
//         let coldkey_account_id = 3;
//         let hotkey_account_id = 1002;

//         [
//             // Lower than price (not enough tao)
//             (0.1, 0.2, 100_000_000_000_u64),
//             // Higher than price (not enough alpha)
//             (0.3, 0.4, 100_000_000_000_u64),
//             // Around the price (not enough both)
//             (0.1, 0.4, 100_000_000_000_u64),
//         ]
//         .into_iter()
//         .enumerate()
//         .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2))
//         .for_each(|(netuid, price_low, price_high, liquidity)| {
//             // Calculate ticks
//             let tick_low = price_to_tick(price_low);
//             let tick_high = price_to_tick(price_high);

//             assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//             // Add liquidity
//             assert_err!(
//                 Pallet::<Test>::do_add_liquidity(
//                     netuid,
//                     &coldkey_account_id,
//                     &hotkey_account_id,
//                     tick_low,
//                     tick_high,
//                     liquidity
//                 ),
//                 Error::<Test>::InsufficientBalance,
//             );
//         });
//     });
// }

// TODO: Revise when user liquidity is available
// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_remove_liquidity_basic --exact --show-output
// #[test]
// fn test_remove_liquidity_basic() {
//     new_test_ext().execute_with(|| {
//         let min_price = tick_to_price(TickIndex::MIN);
//         let max_price = tick_to_price(TickIndex::MAX);
//         let max_tick = price_to_tick(max_price);
//         assert_eq!(max_tick, TickIndex::MAX);

//         let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

//         // As a user add liquidity with all possible corner cases
//         //   - Initial price is 0.25
//         //   - liquidity is expressed in RAO units
//         // Test case is (price_low, price_high, liquidity, tao, alpha)
//         [
//             // Repeat the protocol liquidity at maximum range: Expect all the same values
//             (
//                 min_price,
//                 max_price,
//                 2_000_000_000_u64,
//                 1_000_000_000_u64,
//                 4_000_000_000_u64,
//             ),
//             // Repeat the protocol liquidity at current to max range: Expect the same alpha
//             (
//                 current_price_high,
//                 max_price,
//                 2_000_000_000_u64,
//                 0,
//                 4_000_000_000,
//             ),
//             // Repeat the protocol liquidity at min to current range: Expect all the same tao
//             (
//                 min_price,
//                 current_price_low,
//                 2_000_000_000_u64,
//                 1_000_000_000,
//                 0,
//             ),
//             // Half to double price - just some sane wothdraw amounts
//             (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
//             // Both below price - tao is non-zero, alpha is zero
//             (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
//             // Both above price - tao is zero, alpha is non-zero
//             (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
//         ]
//         .into_iter()
//         .enumerate()
//         .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3, v.4))
//         .for_each(|(netuid, price_low, price_high, liquidity, tao, alpha)| {
//             // Calculate ticks (assuming tick math is tested separately)
//             let tick_low = price_to_tick(price_low);
//             let tick_high = price_to_tick(price_high);

//             assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
//             let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

//             // Add liquidity
//             let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 tick_low,
//                 tick_high,
//                 liquidity,
//             )
//             .unwrap();

//             // Remove liquidity
//             let remove_result =
//                 Pallet::<Test>::do_remove_liquidity(netuid, &OK_COLDKEY_ACCOUNT_ID, position_id)
//                     .unwrap();
//             assert_abs_diff_eq!(remove_result.tao.to_u64(), tao, epsilon = tao / 1000);
//             assert_abs_diff_eq!(
//                 u64::from(remove_result.alpha),
//                 alpha,
//                 epsilon = alpha / 1000
//             );
//             assert_eq!(remove_result.fee_tao, TaoCurrency::ZERO);
//             assert_eq!(remove_result.fee_alpha, AlphaCurrency::ZERO);

//             // Liquidity position is removed
//             assert_eq!(
//                 Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
//                 0
//             );
//             assert!(Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).is_none());

//             // Current liquidity is updated (back where it was)
//             assert_eq!(CurrentLiquidity::<Test>::get(netuid), liquidity_before);
//         });
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn test_remove_liquidity_nonexisting_position() {
//     new_test_ext().execute_with(|| {
//         let min_price = tick_to_price(TickIndex::MIN);
//         let max_price = tick_to_price(TickIndex::MAX);
//         let max_tick = price_to_tick(max_price);
//         assert_eq!(max_tick.get(), TickIndex::MAX.get());

//         let liquidity = 2_000_000_000_u64;
//         let netuid = NetUid::from(1);

//         // Calculate ticks (assuming tick math is tested separately)
//         let tick_low = price_to_tick(min_price);
//         let tick_high = price_to_tick(max_price);

//         // Setup swap
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Add liquidity
//         assert_ok!(Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             liquidity,
//         ));

//         assert!(Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID) > 0);

//         // Remove liquidity
//         assert_err!(
//             Pallet::<Test>::do_remove_liquidity(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 PositionId::new::<Test>()
//             ),
//             Error::<Test>::LiquidityNotFound,
//         );
//     });
// }

// TODO: Revise when user liquidity is available
// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_modify_position_basic --exact --show-output
// #[test]
// fn test_modify_position_basic() {
//     new_test_ext().execute_with(|| {
//         let max_price = tick_to_price(TickIndex::MAX);
//         let max_tick = price_to_tick(max_price);
//         let limit_price = 1000.0_f64;
//         assert_eq!(max_tick, TickIndex::MAX);
//         let (current_price_low, _current_price_high) = get_ticked_prices_around_current_price();

//         // As a user add liquidity with all possible corner cases
//         //   - Initial price is 0.25
//         //   - liquidity is expressed in RAO units
//         // Test case is (price_low, price_high, liquidity, tao, alpha)
//         [
//             // Repeat the protocol liquidity at current to max range: Expect the same alpha
//             (
//                 current_price_low,
//                 max_price,
//                 2_000_000_000_u64,
//                 4_000_000_000,
//             ),
//         ]
//         .into_iter()
//         .enumerate()
//         .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3))
//         .for_each(|(netuid, price_low, price_high, liquidity, alpha)| {
//             // Calculate ticks (assuming tick math is tested separately)
//             let tick_low = price_to_tick(price_low);
//             let tick_high = price_to_tick(price_high);

//             assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//             // Add liquidity
//             let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 tick_low,
//                 tick_high,
//                 liquidity,
//             )
//             .unwrap();

//             // Get tick infos before the swap/update
//             let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap();
//             let tick_high_info_before = Ticks::<Test>::get(netuid, tick_high).unwrap();

//             // Swap to create fees on the position
//             let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
//             let order = GetAlphaForTao::with_amount(liquidity / 10);
//             Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

//             // Modify liquidity (also causes claiming of fees)
//             let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
//             let modify_result = Pallet::<Test>::do_modify_position(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 position_id,
//                 -((liquidity / 10) as i64),
//             )
//             .unwrap();
//             assert_abs_diff_eq!(
//                 u64::from(modify_result.alpha),
//                 alpha / 10,
//                 epsilon = alpha / 1000
//             );
//             assert!(modify_result.fee_tao > TaoCurrency::ZERO);
//             assert_eq!(modify_result.fee_alpha, AlphaCurrency::ZERO);

//             // Liquidity position is reduced
//             assert_eq!(
//                 Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
//                 1
//             );

//             // Current liquidity is reduced with modify_position
//             assert!(CurrentLiquidity::<Test>::get(netuid) < liquidity_before);

//             // Position liquidity is reduced
//             let position =
//                 Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
//             assert_eq!(position.liquidity, liquidity * 9 / 10);
//             assert_eq!(position.tick_low, tick_low);
//             assert_eq!(position.tick_high, tick_high);

//             // Tick liquidity is updated properly for low and high position ticks
//             let tick_low_info_after = Ticks::<Test>::get(netuid, tick_low).unwrap();
//             let tick_high_info_after = Ticks::<Test>::get(netuid, tick_high).unwrap();

//             assert_eq!(
//                 tick_low_info_before.liquidity_net - (liquidity / 10) as i128,
//                 tick_low_info_after.liquidity_net,
//             );
//             assert_eq!(
//                 tick_low_info_before.liquidity_gross - (liquidity / 10),
//                 tick_low_info_after.liquidity_gross,
//             );
//             assert_eq!(
//                 tick_high_info_before.liquidity_net + (liquidity / 10) as i128,
//                 tick_high_info_after.liquidity_net,
//             );
//             assert_eq!(
//                 tick_high_info_before.liquidity_gross - (liquidity / 10),
//                 tick_high_info_after.liquidity_gross,
//             );

//             // Modify liquidity again (ensure fees aren't double-collected)
//             let modify_result = Pallet::<Test>::do_modify_position(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 position_id,
//                 -((liquidity / 100) as i64),
//             )
//             .unwrap();

//             assert_abs_diff_eq!(
//                 u64::from(modify_result.alpha),
//                 alpha / 100,
//                 epsilon = alpha / 1000
//             );
//             assert_eq!(modify_result.fee_tao, TaoCurrency::ZERO);
//             assert_eq!(modify_result.fee_alpha, AlphaCurrency::ZERO);
//         });
//     });
// }

// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_swap_basic --exact --nocapture
#[test]
fn test_swap_basic() {
    new_test_ext().execute_with(|| {
        fn perform_test<Order>(
            netuid: NetUid,
            order: Order,
            limit_price: f64,
            price_should_grow: bool,
        ) where
            Order: OrderT,
            BasicSwapStep<Test, Order::PaidIn, Order::PaidOut>:
                SwapStep<Test, Order::PaidIn, Order::PaidOut>,
        {
            let swap_amount = order.amount().to_u64();

            // Setup swap
            assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

            // Price is 0.25
            let initial_tao_reserve = TaoCurrency::from(1_000_000_000_u64);
            let initial_alpha_reserve = AlphaCurrency::from(4_000_000_000_u64);
            TaoReserve::set_mock_reserve(netuid, initial_tao_reserve);
            AlphaReserve::set_mock_reserve(netuid, initial_alpha_reserve);

            // Get current price
            let current_price_before = Pallet::<Test>::current_price(netuid);

            // Get reserves
            let tao_reserve = TaoReserve::reserve(netuid.into()).to_u64();
            let alpha_reserve = AlphaReserve::reserve(netuid.into()).to_u64();

            // Expected fee amount
            let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;
            let expected_fee = (swap_amount as f64 * fee_rate) as u64;

            // Calculate expected output amount using f64 math
            // This is a simple case when w1 = w2 = 0.5, so there's no
            // exponentiation needed
            let x = alpha_reserve as f64;
            let y = tao_reserve as f64;
            let expected_output_amount = if price_should_grow {
                x * (1.0 - y / (y + (swap_amount - expected_fee) as f64))
            } else {
                y * (1.0 - x / (x + (swap_amount - expected_fee) as f64))
            };

            // Swap
            let limit_price_fixed = U64F64::from_num(limit_price);
            let swap_result =
                Pallet::<Test>::do_swap(netuid, order.clone(), limit_price_fixed, false, false)
                    .unwrap();
            assert_abs_diff_eq!(
                swap_result.amount_paid_out.to_u64() as u64,
                expected_output_amount as u64,
                epsilon = 1
            );

            assert_abs_diff_eq!(
                swap_result.paid_in_reserve_delta() as u64,
                (swap_amount - expected_fee) as u64,
                epsilon = 1
            );
            assert_abs_diff_eq!(
                swap_result.paid_out_reserve_delta() as i64,
                -(expected_output_amount as i64),
                epsilon = 1
            );

            // Update reserves (because it happens outside of do_swap in stake_utils)
            if price_should_grow {
                TaoReserve::set_mock_reserve(
                    netuid,
                    TaoCurrency::from(
                        (u64::from(initial_tao_reserve) as i128
                            + swap_result.paid_in_reserve_delta()) as u64,
                    ),
                );
                AlphaReserve::set_mock_reserve(
                    netuid,
                    AlphaCurrency::from(
                        (u64::from(initial_alpha_reserve) as i128
                            + swap_result.paid_out_reserve_delta()) as u64,
                    ),
                );
            } else {
                TaoReserve::set_mock_reserve(
                    netuid,
                    TaoCurrency::from(
                        (u64::from(initial_tao_reserve) as i128
                            + swap_result.paid_out_reserve_delta()) as u64,
                    ),
                );
                AlphaReserve::set_mock_reserve(
                    netuid,
                    AlphaCurrency::from(
                        (u64::from(initial_alpha_reserve) as i128
                            + swap_result.paid_in_reserve_delta()) as u64,
                    ),
                );
            }

            // Liquidity position should not be updated
            // TODO: Revise when user liquidity is in place
            // let protocol_id = Pallet::<Test>::protocol_account_id();
            // let positions =
            //     PositionsV2::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
            // let position = positions.first().unwrap();

            // assert_eq!(
            //     position.liquidity,
            //     helpers_128bit::sqrt(
            //         TaoReserve::reserve(netuid.into()).to_u64() as u128
            //             * AlphaReserve::reserve(netuid.into()).to_u64() as u128
            //     ) as u64
            // );
            // assert_eq!(position.fees_alpha, 0);
            // assert_eq!(position.fees_tao, 0);

            // Assert that price movement is in correct direction
            let current_price_after = Pallet::<Test>::current_price(netuid);
            assert_eq!(
                current_price_after >= current_price_before,
                price_should_grow
            );
        }

        // Current price is 0.25
        // Test case is (order_type, liquidity, limit_price, output_amount)
        perform_test(1.into(), GetAlphaForTao::with_amount(1_000), 1000.0, true);
        perform_test(1.into(), GetAlphaForTao::with_amount(2_000), 1000.0, true);
        perform_test(1.into(), GetAlphaForTao::with_amount(123_456), 1000.0, true);
        perform_test(2.into(), GetTaoForAlpha::with_amount(1_000), 0.0001, false);
        perform_test(2.into(), GetTaoForAlpha::with_amount(2_000), 0.0001, false);
        perform_test(
            2.into(),
            GetTaoForAlpha::with_amount(123_456),
            0.0001,
            false,
        );
        perform_test(
            3.into(),
            GetAlphaForTao::with_amount(1_000_000_000),
            1000.0,
            true,
        );
        perform_test(
            3.into(),
            GetAlphaForTao::with_amount(10_000_000_000),
            1000.0,
            true,
        );
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_precision_edge_case --exact --show-output
#[test]
fn test_swap_precision_edge_case() {
    // Test case: tao_reserve, alpha_reserve, swap_amount
    [
        (1_000_u64, 1_000_u64, 999_500_u64),
        (1_000_000_u64, 1_000_000_u64, 999_500_000_u64),
    ]
    .into_iter()
    .for_each(|(tao_reserve, alpha_reserve, swap_amount)| {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(1);
            let order = GetTaoForAlpha::with_amount(swap_amount);

            // Very low reserves
            TaoReserve::set_mock_reserve(netuid, TaoCurrency::from(tao_reserve));
            AlphaReserve::set_mock_reserve(netuid, AlphaCurrency::from(alpha_reserve));

            // Minimum possible limit price
            let limit_price: U64F64 = get_min_price();
            println!("limit_price = {:?}", limit_price);

            // Swap
            let swap_result =
                Pallet::<Test>::do_swap(netuid, order, limit_price, false, true).unwrap();

            assert!(swap_result.amount_paid_out > TaoCurrency::ZERO);
        });
    });
}

#[test]
fn test_convert_deltas() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

        // TODO: Add more test cases with different weights and edge cases for reserves
        for (tao, alpha, w_quote, delta_in) in [
            (1500, 1000, 0.5, 1),
            (1500, 1000, 0.5, 10000),
            (1500, 1000, 0.5, 1000000),
            (1500, 1000, 0.5, u64::MAX),
            (1, 1000000, 0.5, 1),
            (1, 1000000, 0.5, 10000),
            (1, 1000000, 0.5, 1000000),
            (1, 1000000, 0.5, u64::MAX),
            (1000000, 1, 0.5, 1),
            (1000000, 1, 0.5, 10000),
            (1000000, 1, 0.5, 1000000),
            (1000000, 1, 0.5, u64::MAX),
            // Low quote weight
            (1500, 1000, 0.1, 1),
            (1500, 1000, 0.1, 10000),
            (1500, 1000, 0.1, 1000000),
            (1500, 1000, 0.1, u64::MAX),
            (1, 1000000, 0.1, 1),
            (1, 1000000, 0.1, 10000),
            (1, 1000000, 0.1, 1000000),
            (1, 1000000, 0.1, u64::MAX),
            (1000000, 1, 0.1, 1),
            (1000000, 1, 0.1, 10000),
            (1000000, 1, 0.1, 1000000),
            (1000000, 1, 0.1, u64::MAX),
            // High quote weight
            (1500, 1000, 0.9, 1),
            (1500, 1000, 0.9, 10000),
            (1500, 1000, 0.9, 1000000),
            (1500, 1000, 0.9, u64::MAX),
            (1, 1000000, 0.9, 1),
            (1, 1000000, 0.9, 10000),
            (1, 1000000, 0.9, 1000000),
            (1, 1000000, 0.9, u64::MAX),
            (1000000, 1, 0.9, 1),
            (1000000, 1, 0.9, 10000),
            (1000000, 1, 0.9, 1000000),
            (1000000, 1, 0.9, u64::MAX),
        ] {
            // Initialize reserves and weights
            TaoReserve::set_mock_reserve(netuid, TaoCurrency::from(tao));
            AlphaReserve::set_mock_reserve(netuid, AlphaCurrency::from(alpha));
            let w_accuracy = 1_000_000_000_f64;
            let w_quote_pt = Perquintill::from_rational(
                (w_quote as f64 * w_accuracy) as u128,
                w_accuracy as u128,
            );
            let bal = Balancer::new(w_quote_pt).unwrap();
            SwapBalancer::<Test>::insert(netuid, bal);

            // Calculate expected swap results (buy and sell) using f64 math
            let y = tao as f64;
            let x = alpha as f64;
            let d = delta_in as f64;
            let w1_div_w2 = (1. - w_quote) / w_quote;
            let w2_div_w1 = w_quote / (1. - w_quote);
            let expected_sell = y * (1. - (x / (x + d)).powf(w1_div_w2));
            let expected_buy = x * (1. - (y / (y + d)).powf(w2_div_w1));

            assert_abs_diff_eq!(
                u64::from(
                    BasicSwapStep::<Test, AlphaCurrency, TaoCurrency>::convert_deltas(
                        netuid,
                        delta_in.into()
                    )
                ),
                expected_sell as u64,
                epsilon = 2u64
            );
            assert_abs_diff_eq!(
                u64::from(
                    BasicSwapStep::<Test, TaoCurrency, AlphaCurrency>::convert_deltas(
                        netuid,
                        delta_in.into()
                    )
                ),
                expected_buy as u64,
                epsilon = 2u64
            );
        }
    });
}

// #[test]
// fn test_user_liquidity_disabled() {
//     new_test_ext().execute_with(|| {
//         // Use a netuid above 100 since our mock enables liquidity for 0-100
//         let netuid = NetUid::from(101);
//         let tick_low = TickIndex::new_unchecked(-1000);
//         let tick_high = TickIndex::new_unchecked(1000);
//         let position_id = PositionId::from(1);
//         let liquidity = 1_000_000_000;
//         let liquidity_delta = 500_000_000;

//         assert!(!EnabledUserLiquidity::<Test>::get(netuid));

//         assert_noop!(
//             Swap::do_add_liquidity(
//                 netuid,
//                 &OK_COLDKEY_ACCOUNT_ID,
//                 &OK_HOTKEY_ACCOUNT_ID,
//                 tick_low,
//                 tick_high,
//                 liquidity
//             ),
//             Error::<Test>::UserLiquidityDisabled
//         );

//         assert_noop!(
//             Swap::do_remove_liquidity(netuid, &OK_COLDKEY_ACCOUNT_ID, position_id),
//             Error::<Test>::LiquidityNotFound
//         );

//         assert_noop!(
//             Swap::modify_position(
//                 RuntimeOrigin::signed(OK_COLDKEY_ACCOUNT_ID),
//                 OK_HOTKEY_ACCOUNT_ID,
//                 netuid,
//                 position_id,
//                 liquidity_delta
//             ),
//             Error::<Test>::UserLiquidityDisabled
//         );

//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid,
//             true
//         ));

//         let position_id = Swap::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             liquidity,
//         )
//         .unwrap()
//         .0;

//         assert_ok!(Swap::do_modify_position(
//             netuid.into(),
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             position_id,
//             liquidity_delta,
//         ));

//         assert_ok!(Swap::do_remove_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             position_id,
//         ));
//     });
// }

// TODO: revise when user liquidity is available
// Test correctness of swap fees:
//   - Fees are distribued to (concentrated) liquidity providers
//
// #[test]
// fn test_swap_fee_correctness() {
//     new_test_ext().execute_with(|| {
//         let min_price = get_min_price();
//         let max_price = get_max_price();
//         let netuid = NetUid::from(1);

//         // Provide very spread liquidity at the range from min to max that matches protocol liquidity
//         let liquidity = 2_000_000_000_000_u64; // 1x of protocol liquidity

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Add user liquidity
//         let (position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             liquidity,
//         )
//         .unwrap();

//         // Swap buy and swap sell
//         Pallet::<Test>::do_swap(
//             netuid,
//             GetAlphaForTao::with_amount(liquidity / 10),
//             u64::MAX.into(),
//             false,
//             false,
//         )
//         .unwrap();
//         Pallet::<Test>::do_swap(
//             netuid,
//             GetTaoForAlpha::with_amount(liquidity / 10),
//             0_u64.into(),
//             false,
//             false,
//         )
//         .unwrap();

//         // Get user position
//         let mut position =
//             Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
//         assert_eq!(position.liquidity, liquidity);
//         assert_eq!(position.tick_low, tick_low);
//         assert_eq!(position.tick_high, tick_high);

//         // Check that 50% of fees were credited to the position
//         let fee_rate = FeeRate::<Test>::get(NetUid::from(netuid)) as f64 / u16::MAX as f64;
//         let (actual_fee_tao, actual_fee_alpha) = position.collect_fees();
//         let expected_fee = (fee_rate * (liquidity / 10) as f64 * 0.5) as u64;

//         assert_abs_diff_eq!(actual_fee_tao, expected_fee, epsilon = 1,);
//         assert_abs_diff_eq!(actual_fee_alpha, expected_fee, epsilon = 1,);
//     });
// }

#[test]
fn test_rollback_works() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        assert_eq!(
            Pallet::<Test>::do_swap(
                netuid,
                GetAlphaForTao::with_amount(1_000_000),
                u64::MAX.into(),
                false,
                true
            )
            .unwrap(),
            Pallet::<Test>::do_swap(
                netuid,
                GetAlphaForTao::with_amount(1_000_000),
                u64::MAX.into(),
                false,
                false
            )
            .unwrap()
        );
    })
}

// TODO: Revise when user liquidity is available
// Test correctness of swap fees:
//   - New LP is not eligible to previously accrued fees
//
// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_new_lp_doesnt_get_old_fees --exact --show-output
// #[test]
// fn test_new_lp_doesnt_get_old_fees() {
//     new_test_ext().execute_with(|| {
//         let min_price = tick_to_price(TickIndex::MIN);
//         let max_price = tick_to_price(TickIndex::MAX);
//         let netuid = NetUid::from(1);

//         // Provide very spread liquidity at the range from min to max that matches protocol liquidity
//         let liquidity = 2_000_000_000_000_u64; // 1x of protocol liquidity

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Calculate ticks
//         let tick_low = price_to_tick(min_price);
//         let tick_high = price_to_tick(max_price);

//         // Add user liquidity
//         Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             liquidity,
//         )
//         .unwrap();

//         // Swap buy and swap sell
//         Pallet::<Test>::do_swap(
//             netuid,
//             GetAlphaForTao::with_amount(liquidity / 10),
//             u64::MAX.into(),
//             false,
//             false,
//         )
//         .unwrap();
//         Pallet::<Test>::do_swap(
//             netuid,
//             GetTaoForAlpha::with_amount(liquidity / 10),
//             0_u64.into(),
//             false,
//             false,
//         )
//         .unwrap();

//         // Add liquidity from a different user to a new tick
//         let (position_id_2, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID_2,
//             &OK_HOTKEY_ACCOUNT_ID_2,
//             tick_low.next().unwrap(),
//             tick_high.prev().unwrap(),
//             liquidity,
//         )
//         .unwrap();

//         // Get user position
//         let mut position =
//             Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID_2, position_id_2)).unwrap();
//         assert_eq!(position.liquidity, liquidity);
//         assert_eq!(position.tick_low, tick_low.next().unwrap());
//         assert_eq!(position.tick_high, tick_high.prev().unwrap());

//         // Check that collected fees are 0
//         let (actual_fee_tao, actual_fee_alpha) = position.collect_fees();
//         assert_abs_diff_eq!(actual_fee_tao, 0, epsilon = 1);
//         assert_abs_diff_eq!(actual_fee_alpha, 0, epsilon = 1);
//     });
// }

#[allow(dead_code)]
fn bbox(t: U64F64, a: U64F64, b: U64F64) -> U64F64 {
    if t < a {
        a
    } else if t > b {
        b
    } else {
        t
    }
}

#[allow(dead_code)]
fn print_current_price(netuid: NetUid) {
    let current_price = Pallet::<Test>::current_price(netuid);
    log::trace!("Current price: {current_price:.6}");
}

// TODO: Revise when user liquidity is available
// RUST_LOG=pallet_subtensor_swap=trace cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_wrapping_fees --exact --show-output --nocapture
// #[test]
// fn test_wrapping_fees() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(WRAPPING_FEES_NETUID);
//         let position_1_low_price = 0.20;
//         let position_1_high_price = 0.255;
//         let position_2_low_price = 0.255;
//         let position_2_high_price = 0.257;
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID_RICH,
//             &OK_COLDKEY_ACCOUNT_ID_RICH,
//             price_to_tick(position_1_low_price),
//             price_to_tick(position_1_high_price),
//             1_000_000_000_u64,
//         )
//         .unwrap();

//         print_current_price(netuid);

//         let order = GetTaoForAlpha::with_amount(800_000_000);
//         let sqrt_limit_price = SqrtPrice::from_num(0.000001);
//         Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

//         let order = GetAlphaForTao::with_amount(1_850_000_000);
//         let sqrt_limit_price = SqrtPrice::from_num(1_000_000.0);

//         print_current_price(netuid);

//         Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

//         print_current_price(netuid);

//         let add_liquidity_result = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID_RICH,
//             &OK_COLDKEY_ACCOUNT_ID_RICH,
//             price_to_tick(position_2_low_price),
//             price_to_tick(position_2_high_price),
//             1_000_000_000_u64,
//         )
//         .unwrap();

//         let order = GetTaoForAlpha::with_amount(1_800_000_000);
//         let sqrt_limit_price = SqrtPrice::from_num(0.000001);

//         let initial_sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);
//         Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();
//         let final_sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);

//         print_current_price(netuid);

//         let mut position =
//             Positions::<Test>::get((netuid, &OK_COLDKEY_ACCOUNT_ID_RICH, add_liquidity_result.0))
//                 .unwrap();

//         let initial_box_price = bbox(
//             initial_sqrt_price,
//             position.tick_low.try_to_sqrt_price().unwrap(),
//             position.tick_high.try_to_sqrt_price().unwrap(),
//         );

//         let final_box_price = bbox(
//             final_sqrt_price,
//             position.tick_low.try_to_sqrt_price().unwrap(),
//             position.tick_high.try_to_sqrt_price().unwrap(),
//         );

//         let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;

//         log::trace!("fee_rate: {fee_rate:.6}");
//         log::trace!("position.liquidity: {}", position.liquidity);
//         log::trace!(
//             "initial_box_price: {:.6}",
//             initial_box_price.to_num::<f64>()
//         );
//         log::trace!("final_box_price: {:.6}", final_box_price.to_num::<f64>());

//         let expected_fee_tao = ((fee_rate / (1.0 - fee_rate))
//             * (position.liquidity as f64)
//             * (final_box_price.to_num::<f64>() - initial_box_price.to_num::<f64>()))
//             as u64;

//         let expected_fee_alpha = ((fee_rate / (1.0 - fee_rate))
//             * (position.liquidity as f64)
//             * ((1.0 / final_box_price.to_num::<f64>()) - (1.0 / initial_box_price.to_num::<f64>())))
//             as u64;

//         log::trace!("Expected ALPHA fee: {:.6}", expected_fee_alpha as f64);

//         let (fee_tao, fee_alpha) = position.collect_fees();

//         log::trace!("Collected fees: TAO: {fee_tao}, ALPHA: {fee_alpha}");

//         assert_abs_diff_eq!(fee_tao, expected_fee_tao, epsilon = 1);
//         assert_abs_diff_eq!(fee_alpha, expected_fee_alpha, epsilon = 1);
//     });
// }

// TODO: Revise when user liquidity is available
// Test that price moves less with more liquidity
// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_less_price_movement --exact --show-output
// #[test]
// fn test_less_price_movement() {
//     let netuid = NetUid::from(1);
//     let mut last_end_price = U64F64::from_num(0);
//     let initial_stake_liquidity = 1_000_000_000;
//     let swapped_liquidity = 1_000_000;

//     // Test case is (order_type, provided_liquidity)
//     // Testing algorithm:
//     //   - Stake initial_stake_liquidity
//     //   - Provide liquidity if iteration provides lq
//     //   - Buy or sell
//     //   - Save end price if iteration doesn't provide lq
//     macro_rules! perform_test {
//         ($order_t:ident, $provided_liquidity:expr, $limit_price:expr, $should_price_shrink:expr) => {
//             let provided_liquidity = $provided_liquidity;
//             let should_price_shrink = $should_price_shrink;
//             let limit_price = $limit_price;
//             new_test_ext().execute_with(|| {
//                 // Setup swap
//                 assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//                 // Buy Alpha
//                 assert_ok!(Pallet::<Test>::do_swap(
//                     netuid,
//                     GetAlphaForTao::with_amount(initial_stake_liquidity),
//                     SqrtPrice::from_num(10_000_000_000_u64),
//                     false,
//                     false
//                 ));

//                 // Get current price
//                 let start_price = Pallet::<Test>::current_price(netuid);

//                 // Add liquidity if this test iteration provides
//                 if provided_liquidity > 0 {
//                     let tick_low = price_to_tick(start_price.to_num::<f64>() * 0.5);
//                     let tick_high = price_to_tick(start_price.to_num::<f64>() * 1.5);
//                     assert_ok!(Pallet::<Test>::do_add_liquidity(
//                         netuid,
//                         &OK_COLDKEY_ACCOUNT_ID,
//                         &OK_HOTKEY_ACCOUNT_ID,
//                         tick_low,
//                         tick_high,
//                         provided_liquidity,
//                     ));
//                 }

//                 // Swap
//                 let sqrt_limit_price = SqrtPrice::from_num(limit_price);
//                 assert_ok!(Pallet::<Test>::do_swap(
//                     netuid,
//                     $order_t::with_amount(swapped_liquidity),
//                     sqrt_limit_price,
//                     false,
//                     false
//                 ));

//                 let end_price = Pallet::<Test>::current_price(netuid);

//                 // Save end price if iteration doesn't provide or compare with previous end price if
//                 // it does
//                 if provided_liquidity > 0 {
//                     assert_eq!(should_price_shrink, end_price < last_end_price);
//                 } else {
//                     last_end_price = end_price;
//                 }
//             });
//         };
//     }

//     for provided_liquidity in [0, 1_000_000_000_000_u64] {
//         perform_test!(GetAlphaForTao, provided_liquidity, 1000.0_f64, true);
//     }
//     for provided_liquidity in [0, 1_000_000_000_000_u64] {
//         perform_test!(GetTaoForAlpha, provided_liquidity, 0.001_f64, false);
//     }
// }

#[test]
fn test_swap_subtoken_disabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(SUBTOKEN_DISABLED_NETUID); // Use a netuid not used elsewhere
        let liquidity = 1_000_000_u64;

        assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

        assert_noop!(
            Pallet::<Test>::add_liquidity(
                RuntimeOrigin::signed(OK_COLDKEY_ACCOUNT_ID),
                OK_HOTKEY_ACCOUNT_ID,
                netuid,
                liquidity,
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            Pallet::<Test>::modify_position(
                RuntimeOrigin::signed(OK_COLDKEY_ACCOUNT_ID),
                OK_HOTKEY_ACCOUNT_ID,
                netuid,
                PositionId::from(0),
                liquidity as i64,
            ),
            Error::<Test>::SubtokenDisabled
        );
    });
}

// TODO: Revise when user liquidity is available
// #[test]
// fn test_liquidate_v3_removes_positions_ticks_and_state() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(1);

//         // Initialize V3 (creates protocol position, ticks, price, liquidity)
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
//         assert!(PalSwapInitialized::<Test>::get(netuid));

//         // Enable user LP
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid.into(),
//             true
//         ));

//         // Add a user position across the full range to ensure ticks/bitmap are populated.
//         let min_price = get_min_price();
//         let max_price = get_max_price();
//         let liquidity = 2_000_000_000_u64;

//         let (_pos_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             liquidity,
//         )
//         .expect("add liquidity");

//         // Accrue some global fees so we can verify fee storage is cleared later.
//         let sqrt_limit_price = SqrtPrice::from_num(1_000_000.0);
//         assert_ok!(Pallet::<Test>::do_swap(
//             netuid,
//             GetAlphaForTao::with_amount(1_000_000),
//             sqrt_limit_price,
//             false,
//             false
//         ));

//         // Sanity: protocol & user positions exist, ticks exist, liquidity > 0
//         let protocol_id = Pallet::<Test>::protocol_account_id();
//         let prot_positions =
//             Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
//         assert!(!prot_positions.is_empty());

//         let user_positions = Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//             .collect::<Vec<_>>();
//         assert_eq!(user_positions.len(), 1);

//         assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_some());
//         assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_some());
//         assert!(CurrentLiquidity::<Test>::get(netuid) > 0);

//         let had_bitmap_words = TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//             .next()
//             .is_some();
//         assert!(had_bitmap_words);

//         // ACT: users-only liquidation then protocol clear
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         // ASSERT: positions cleared (both user and protocol)
//         assert_eq!(
//             Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
//             0
//         );
//         let prot_positions_after =
//             Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
//         assert!(prot_positions_after.is_empty());
//         let user_positions_after =
//             Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .collect::<Vec<_>>();
//         assert!(user_positions_after.is_empty());

//         // ASSERT: ticks cleared
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_none());
//         assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_none());

//         // ASSERT: fee globals cleared
//         assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

//         // ASSERT: price/tick/liquidity flags cleared
//         assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
//         assert!(!CurrentTick::<Test>::contains_key(netuid));
//         assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));

//         // ASSERT: active tick bitmap cleared
//         assert!(
//             TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//                 .next()
//                 .is_none()
//         );

//         // ASSERT: knobs removed on dereg
//         assert!(!FeeRate::<Test>::contains_key(netuid));
//         assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
// V3 path with user liquidity disabled at teardown:
// must still remove positions and clear state (after protocol clear).
// #[test]
// fn test_liquidate_v3_with_user_liquidity_disabled() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(101);

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
//         assert!(PalSwapInitialized::<Test>::get(netuid));

//         // Enable temporarily to add a user position
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid.into(),
//             true
//         ));

//         let min_price = tick_to_price(TickIndex::MIN);
//         let max_price = tick_to_price(TickIndex::MAX);
//         let tick_low = price_to_tick(min_price);
//         let tick_high = price_to_tick(max_price);
//         let liquidity = 1_000_000_000_u64;

//         let (_pos_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             liquidity,
//         )
//         .expect("add liquidity");

//         // Disable user LP *before* liquidation; removal must ignore this flag.
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid.into(),
//             false
//         ));

//         // Users-only dissolve, then clear protocol liquidity/state.
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         // ASSERT: positions & ticks gone, state reset
//         assert_eq!(
//             Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
//             0
//         );
//         assert!(
//             Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .next()
//                 .is_none()
//         );
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(
//             TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//                 .next()
//                 .is_none()
//         );
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//         assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
//         assert!(!CurrentTick::<Test>::contains_key(netuid));
//         assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

//         // `EnabledUserLiquidity` is removed by protocol clear stage.
//         assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
// Non‑palswap path: PalSwap not initialized (no positions, no map values); function
// must still clear any residual storages and succeed.
// #[test]
// fn test_liquidate_pal_uninitialized_ok_and_clears() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(202);

//         // Insert map values
//         PalSwapInitialized::<Test>::insert(netuid, false);

//         // Sanity: PalSwap is not initialized
//         assert!(!PalSwapInitialized::<Test>::get(netuid));
//         assert!(
//             PositionsV2::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .next()
//                 .is_none()
//         );

//         // ACT
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // ASSERT: Defensive clears leave no residues and do not panic
//         assert!(
//             PositionsV2::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .next()
//                 .is_none()
//         );

//         // All single-key maps should not have the key after liquidation
//         assert!(!FeeRate::<Test>::contains_key(netuid));
//         assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
//         assert!(!FeesTao::<Test>::contains_key(netuid));
//         assert!(!FeesAlpha::<Test>::contains_key(netuid));
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//         assert!(!SwapBalancer::<Test>::contains_key(netuid));
//     });
// }

/// Simple palswap path: PalSwap is initialized, but no positions, only protocol; function
/// must still clear any residual storages and succeed.
/// TODO: Revise when user liquidity is available
#[test]
fn test_liquidate_pal_simple_ok_and_clears() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(202);

        // Insert map values
        FeeRate::<Test>::insert(netuid, 1_000);
        EnabledUserLiquidity::<Test>::insert(netuid, false);
        FeesTao::<Test>::insert(netuid, TaoCurrency::from(1_000));
        FeesAlpha::<Test>::insert(netuid, AlphaCurrency::from(1_000));
        PalSwapInitialized::<Test>::insert(netuid, true);
        let w_quote_pt = Perquintill::from_rational(1u128, 2u128);
        let bal = Balancer::new(w_quote_pt).unwrap();
        SwapBalancer::<Test>::insert(netuid, bal);

        // Sanity: PalSwap is not initialized
        assert!(PalSwapInitialized::<Test>::get(netuid));
        assert!(
            PositionsV2::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );

        // ACT
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // ASSERT: Defensive clears leave no residues and do not panic
        assert!(
            PositionsV2::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );

        // All single-key maps should not have the key after liquidation
        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
        assert!(!FeesTao::<Test>::contains_key(netuid));
        assert!(!FeesAlpha::<Test>::contains_key(netuid));
        assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
        assert!(!SwapBalancer::<Test>::contains_key(netuid));
    });
}

// TODO: Revise when user liquidity is available
// #[test]
// fn test_liquidate_idempotent() {
//     // V3 flavor
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(7);
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Add a small user position
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid.into(),
//             true
//         ));
//         let tick_low = price_to_tick(0.2);
//         let tick_high = price_to_tick(0.3);
//         assert_ok!(Pallet::<Test>::do_add_liquidity(
//             netuid,
//             &OK_COLDKEY_ACCOUNT_ID,
//             &OK_HOTKEY_ACCOUNT_ID,
//             tick_low,
//             tick_high,
//             123_456_789
//         ));

//         // Users-only liquidations are idempotent.
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // Now clear protocol liquidity/state—also idempotent.
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         // State remains empty
//         assert!(
//             Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .next()
//                 .is_none()
//         );
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(
//             TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//                 .next()
//                 .is_none()
//         );
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//     });

//     // Non‑V3 flavor
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(8);

//         // Never initialize V3; both calls no-op and succeed.
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         assert!(
//             Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
//                 .next()
//                 .is_none()
//         );
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(
//             TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//                 .next()
//                 .is_none()
//         );
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn liquidate_v3_refunds_user_funds_and_clears_state() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(1);

//         // Enable V3 path & initialize price/ticks (also creates a protocol position).
//         assert_ok!(Pallet::<Test>::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid,
//             true
//         ));
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Use distinct cold/hot to demonstrate alpha refund/stake accounting.
//         let cold = OK_COLDKEY_ACCOUNT_ID;
//         let hot = OK_HOTKEY_ACCOUNT_ID;

//         // Tight in‑range band around current tick.
//         let ct = CurrentTick::<Test>::get(netuid);
//         let tick_low = ct.saturating_sub(10);
//         let tick_high = ct.saturating_add(10);
//         let liquidity: u64 = 1_000_000;

//         // Snapshot balances BEFORE.
//         let tao_before = <Test as Config>::BalanceOps::tao_balance(&cold);
//         let alpha_before_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_before_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_before_total = alpha_before_hot + alpha_before_owner;

//         // Create the user position (storage & v3 state only; no balances moved yet).
//         let (_pos_id, need_tao, need_alpha) =
//             Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
//                 .expect("add liquidity");

//         // Mirror extrinsic bookkeeping: withdraw funds & bump provided‑reserve counters.
//         let tao_taken = <Test as Config>::BalanceOps::decrease_balance(&cold, need_tao.into())
//             .expect("decrease TAO");
//         let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
//             &cold,
//             &hot,
//             netuid.into(),
//             need_alpha.into(),
//         )
//         .expect("decrease ALPHA");
//         TaoReserve::increase_provided(netuid.into(), tao_taken);
//         AlphaReserve::increase_provided(netuid.into(), alpha_taken);

//         // Users‑only liquidation.
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // Expect balances restored to BEFORE snapshots (no swaps ran -> zero fees).
//         let tao_after = <Test as Config>::BalanceOps::tao_balance(&cold);
//         assert_eq!(tao_after, tao_before, "TAO principal must be refunded");

//         // ALPHA totals conserved to owner (distribution may differ).
//         let alpha_after_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_after_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_after_total = alpha_after_hot + alpha_after_owner;
//         assert_eq!(
//             alpha_after_total, alpha_before_total,
//             "ALPHA principal must be refunded/staked for the account (check totals)"
//         );

//         // Clear protocol liquidity and V3 state now.
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         // User position(s) are gone and all V3 state cleared.
//         assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn refund_alpha_single_provider_exact() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(11);
//         let cold = OK_COLDKEY_ACCOUNT_ID;
//         let hot = OK_HOTKEY_ACCOUNT_ID;

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // --- Create an alpha‑only position (range entirely above current tick → TAO = 0, ALPHA > 0).
//         let ct = CurrentTick::<Test>::get(netuid);
//         let tick_low = ct.next().expect("current tick should not be MAX in tests");
//         let tick_high = TickIndex::MAX;

//         let liquidity = 1_000_000_u64;
//         let (_pos_id, tao_needed, alpha_needed) =
//             Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
//                 .expect("add alpha-only liquidity");
//         assert_eq!(tao_needed, 0, "alpha-only position must not require TAO");
//         assert!(alpha_needed > 0, "alpha-only position must require ALPHA");

//         // --- Snapshot BEFORE we withdraw funds (baseline for conservation).
//         let alpha_before_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_before_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_before_total = alpha_before_hot + alpha_before_owner;

//         // --- Mimic extrinsic bookkeeping: withdraw α and record provided reserve.
//         let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
//             &cold,
//             &hot,
//             netuid.into(),
//             alpha_needed.into(),
//         )
//         .expect("decrease ALPHA");
//         AlphaReserve::increase_provided(netuid.into(), alpha_taken);

//         // --- Act: users‑only dissolve.
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // --- Assert: total α conserved to owner (may be staked to validator).
//         let alpha_after_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_after_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_after_total = alpha_after_hot + alpha_after_owner;
//         assert_eq!(
//             alpha_after_total, alpha_before_total,
//             "ALPHA principal must be conserved to the account"
//         );

//         // Clear protocol liquidity and V3 state now.
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         // --- State is cleared.
//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn refund_alpha_multiple_providers_proportional_to_principal() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(12);
//         let c1 = OK_COLDKEY_ACCOUNT_ID;
//         let h1 = OK_HOTKEY_ACCOUNT_ID;
//         let c2 = OK_COLDKEY_ACCOUNT_ID_2;
//         let h2 = OK_HOTKEY_ACCOUNT_ID_2;

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Use the same "above current tick" trick for alpha‑only positions.
//         let ct = CurrentTick::<Test>::get(netuid);
//         let tick_low = ct.next().expect("current tick should not be MAX in tests");
//         let tick_high = TickIndex::MAX;

//         // Provider #1 (smaller α)
//         let liq1 = 700_000_u64;
//         let (_p1, t1, a1) =
//             Pallet::<Test>::do_add_liquidity(netuid, &c1, &h1, tick_low, tick_high, liq1)
//                 .expect("add alpha-only liquidity #1");
//         assert_eq!(t1, 0);
//         assert!(a1 > 0);

//         // Provider #2 (larger α)
//         let liq2 = 2_100_000_u64;
//         let (_p2, t2, a2) =
//             Pallet::<Test>::do_add_liquidity(netuid, &c2, &h2, tick_low, tick_high, liq2)
//                 .expect("add alpha-only liquidity #2");
//         assert_eq!(t2, 0);
//         assert!(a2 > 0);

//         // Baselines BEFORE withdrawing
//         let a1_before_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &h1);
//         let a1_before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &c1);
//         let a1_before = a1_before_hot + a1_before_owner;

//         let a2_before_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &h2);
//         let a2_before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &c2);
//         let a2_before = a2_before_hot + a2_before_owner;

//         // Withdraw α and account reserves for each provider.
//         let a1_taken =
//             <Test as Config>::BalanceOps::decrease_stake(&c1, &h1, netuid.into(), a1.into())
//                 .expect("decrease α #1");
//         AlphaReserve::increase_provided(netuid.into(), a1_taken);

//         let a2_taken =
//             <Test as Config>::BalanceOps::decrease_stake(&c2, &h2, netuid.into(), a2.into())
//                 .expect("decrease α #2");
//         AlphaReserve::increase_provided(netuid.into(), a2_taken);

//         // Act
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // Each owner is restored to their exact baseline.
//         let a1_after_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &h1);
//         let a1_after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &c1);
//         let a1_after = a1_after_hot + a1_after_owner;
//         assert_eq!(
//             a1_after, a1_before,
//             "owner #1 must receive their α principal back"
//         );

//         let a2_after_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &h2);
//         let a2_after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &c2);
//         let a2_after = a2_after_hot + a2_after_owner;
//         assert_eq!(
//             a2_after, a2_before,
//             "owner #2 must receive their α principal back"
//         );
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn refund_alpha_same_cold_multiple_hotkeys_conserved_to_owner() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(13);
//         let cold = OK_COLDKEY_ACCOUNT_ID;
//         let hot1 = OK_HOTKEY_ACCOUNT_ID;
//         let hot2 = OK_HOTKEY_ACCOUNT_ID_2;

//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));

//         // Two alpha‑only positions on different hotkeys of the same owner.
//         let ct = CurrentTick::<Test>::get(netuid);
//         let tick_low = ct.next().expect("current tick should not be MAX in tests");
//         let tick_high = TickIndex::MAX;

//         let (_p1, _t1, a1) =
//             Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot1, tick_low, tick_high, 900_000)
//                 .expect("add alpha-only pos (hot1)");
//         let (_p2, _t2, a2) =
//             Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot2, tick_low, tick_high, 1_500_000)
//                 .expect("add alpha-only pos (hot2)");
//         assert!(a1 > 0 && a2 > 0);

//         // Baseline BEFORE: sum over (cold,hot1) + (cold,hot2) + (cold,cold).
//         let before_hot1 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot1);
//         let before_hot2 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot2);
//         let before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let before_total = before_hot1 + before_hot2 + before_owner;

//         // Withdraw α from both hotkeys; track provided‑reserve.
//         let t1 =
//             <Test as Config>::BalanceOps::decrease_stake(&cold, &hot1, netuid.into(), a1.into())
//                 .expect("decr α #hot1");
//         AlphaReserve::increase_provided(netuid.into(), t1);

//         let t2 =
//             <Test as Config>::BalanceOps::decrease_stake(&cold, &hot2, netuid.into(), a2.into())
//                 .expect("decr α #hot2");
//         AlphaReserve::increase_provided(netuid.into(), t2);

//         // Act
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // The total α "owned" by the coldkey is conserved (credit may land on (cold,cold)).
//         let after_hot1 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot1);
//         let after_hot2 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot2);
//         let after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let after_total = after_hot1 + after_hot2 + after_owner;

//         assert_eq!(
//             after_total, before_total,
//             "owner’s α must be conserved across hot ledgers + (owner,owner)"
//         );
//     });
// }

// TODO: Revise when user liquidity is available
// #[test]
// fn test_dissolve_v3_green_path_refund_tao_stake_alpha_and_clear_state() {
//     new_test_ext().execute_with(|| {
//         // --- Setup ---
//         let netuid = NetUid::from(42);
//         let cold = OK_COLDKEY_ACCOUNT_ID;
//         let hot = OK_HOTKEY_ACCOUNT_ID;

//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid.into(),
//             true
//         ));
//         assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
//         assert!(PalSwapInitialized::<Test>::get(netuid));

//         // Tight in‑range band so BOTH τ and α are required.
//         let ct = CurrentTick::<Test>::get(netuid);
//         let tick_low = ct.saturating_sub(10);
//         let tick_high = ct.saturating_add(10);
//         let liquidity: u64 = 1_250_000;

//         // Add liquidity and capture required τ/α.
//         let (_pos_id, tao_needed, alpha_needed) =
//             Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
//                 .expect("add in-range liquidity");
//         assert!(tao_needed > 0, "in-range pos must require TAO");
//         assert!(alpha_needed > 0, "in-range pos must require ALPHA");

//         // Determine the permitted validator with the highest trust (green path).
//         let trust = <Test as Config>::SubnetInfo::get_validator_trust(netuid.into());
//         let permit = <Test as Config>::SubnetInfo::get_validator_permit(netuid.into());
//         assert_eq!(trust.len(), permit.len(), "trust/permit must align");
//         let target_uid: u16 = trust
//             .iter()
//             .zip(permit.iter())
//             .enumerate()
//             .filter(|(_, (_t, p))| **p)
//             .max_by_key(|(_, (t, _))| *t)
//             .map(|(i, _)| i as u16)
//             .expect("at least one permitted validator");
//         let validator_hotkey: <Test as frame_system::Config>::AccountId =
//             <Test as Config>::SubnetInfo::hotkey_of_uid(netuid.into(), target_uid)
//                 .expect("uid -> hotkey mapping must exist");

//         // --- Snapshot BEFORE we withdraw τ/α to fund the position ---
//         let tao_before = <Test as Config>::BalanceOps::tao_balance(&cold);

//         let alpha_before_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_before_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_before_val =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &validator_hotkey);

//         let alpha_before_total = if validator_hotkey == hot {
//             alpha_before_hot + alpha_before_owner
//         } else {
//             alpha_before_hot + alpha_before_owner + alpha_before_val
//         };

//         // --- Mirror extrinsic bookkeeping: withdraw τ & α; bump provided reserves ---
//         let tao_taken = <Test as Config>::BalanceOps::decrease_balance(&cold, tao_needed.into())
//             .expect("decrease TAO");
//         let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
//             &cold,
//             &hot,
//             netuid.into(),
//             alpha_needed.into(),
//         )
//         .expect("decrease ALPHA");

//         TaoReserve::increase_provided(netuid.into(), tao_taken);
//         AlphaReserve::increase_provided(netuid.into(), alpha_taken);

//         // --- Act: dissolve (GREEN PATH: permitted validators exist) ---
//         assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

//         // --- Assert: τ principal refunded to user ---
//         let tao_after = <Test as Config>::BalanceOps::tao_balance(&cold);
//         assert_eq!(tao_after, tao_before, "TAO principal must be refunded");

//         // --- α ledger assertions ---
//         let alpha_after_hot =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
//         let alpha_after_owner =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
//         let alpha_after_val =
//             <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &validator_hotkey);

//         // Owner ledger must be unchanged in the green path.
//         assert_eq!(
//             alpha_after_owner, alpha_before_owner,
//             "Owner α ledger must be unchanged (staked to validator, not refunded)"
//         );

//         if validator_hotkey == hot {
//             assert_eq!(
//                 alpha_after_hot, alpha_before_hot,
//                 "When validator == hotkey, user's hot ledger must net back to its original balance"
//             );
//             let alpha_after_total = alpha_after_hot + alpha_after_owner;
//             assert_eq!(
//                 alpha_after_total, alpha_before_total,
//                 "Total α for the coldkey must be conserved (validator==hotkey)"
//             );
//         } else {
//             assert!(
//                 alpha_before_hot >= alpha_after_hot,
//                 "hot ledger should not increase"
//             );
//             assert!(
//                 alpha_after_val >= alpha_before_val,
//                 "validator ledger should not decrease"
//             );

//             let hot_loss = alpha_before_hot - alpha_after_hot;
//             let val_gain = alpha_after_val - alpha_before_val;
//             assert_eq!(
//                 val_gain, hot_loss,
//                 "α that left the user's hot ledger must equal α credited to the validator ledger"
//             );

//             let alpha_after_total = alpha_after_hot + alpha_after_owner + alpha_after_val;
//             assert_eq!(
//                 alpha_after_total, alpha_before_total,
//                 "Total α for the coldkey must be conserved"
//             );
//         }

//         // Now clear protocol liquidity & state and assert full reset.
//         assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

//         let protocol_id = Pallet::<Test>::protocol_account_id();
//         assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
//         let prot_positions_after =
//             Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
//         assert!(
//             prot_positions_after.is_empty(),
//             "protocol positions must be removed"
//         );

//         assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
//         assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_none());
//         assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_none());
//         assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
//         assert!(!CurrentTick::<Test>::contains_key(netuid));
//         assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
//         assert!(!PalSwapInitialized::<Test>::contains_key(netuid));

//         assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

//         assert!(
//             TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
//                 .next()
//                 .is_none(),
//             "active tick bitmap words must be cleared"
//         );

//         assert!(!FeeRate::<Test>::contains_key(netuid));
//         assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
//     });
// }

// TODO: Revise when user liquidity is available
#[test]
fn test_clear_protocol_liquidity_green_path() {
    new_test_ext().execute_with(|| {
        // --- Arrange ---
        let netuid = NetUid::from(1);

        // Ensure the "user liquidity enabled" flag exists so we can verify it's removed later.
        EnabledUserLiquidity::<Test>::insert(netuid, true);

        // Initialize swap state
        assert_ok!(Pallet::<Test>::maybe_initialize_palswap(netuid));
        assert!(
            PalSwapInitialized::<Test>::get(netuid),
            "Swap must be initialized"
        );

        // Sanity: protocol positions exist before clearing.
        // TODO: Revise when user liquidity is available
        // let protocol_id = Pallet::<Test>::protocol_account_id();
        // let prot_positions_before =
        //     Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        // assert!(
        //     !prot_positions_before.is_empty(),
        //     "protocol positions should exist after V3 init"
        // );

        // --- Act ---
        // Green path: just clear protocol liquidity and wipe all V3 state.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // --- Assert: all protocol positions removed ---
        // TODO: Revise when user liquidity is available
        // let prot_positions_after =
        //     Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        // assert!(
        //     prot_positions_after.is_empty(),
        //     "protocol positions must be removed by do_clear_protocol_liquidity"
        // );

        // --- Assert: Swap data wiped (idempotent even if some maps were empty) ---

        // Fee globals
        assert!(!FeesTao::<Test>::contains_key(netuid));
        assert!(!FeesAlpha::<Test>::contains_key(netuid));

        // Flags
        assert!(!PalSwapInitialized::<Test>::contains_key(netuid));

        // Knobs removed
        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));

        // --- And it's idempotent ---
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));
        // assert!(
        //     PositionsV2::<Test>::iter_prefix_values((netuid, protocol_id))
        //         .next()
        //         .is_none()
        // );
        assert!(!PalSwapInitialized::<Test>::contains_key(netuid));
    });
}

#[allow(dead_code)]
fn as_tuple(
    (t_used, a_used, t_rem, a_rem): (TaoCurrency, AlphaCurrency, TaoCurrency, AlphaCurrency),
) -> (u64, u64, u64, u64) {
    (
        u64::from(t_used),
        u64::from(a_used),
        u64::from(t_rem),
        u64::from(a_rem),
    )
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_migrate_swapv3_to_balancer --exact --nocapture
#[test]
fn test_migrate_swapv3_to_balancer() {
    use crate::migrations::migrate_swapv3_to_balancer::deprecated_swap_maps;
    use substrate_fixed::types::U64F64;

    new_test_ext().execute_with(|| {
        let migration =
            crate::migrations::migrate_swapv3_to_balancer::migrate_swapv3_to_balancer::<Test>;

        // Insert deprecated maps values
        deprecated_swap_maps::AlphaSqrtPrice::<Test>::insert(
            NetUid::from(1),
            U64F64::from_num(1.23),
        );
        deprecated_swap_maps::ScrapReservoirTao::<Test>::insert(
            NetUid::from(1),
            TaoCurrency::from(9876),
        );
        deprecated_swap_maps::ScrapReservoirAlpha::<Test>::insert(
            NetUid::from(1),
            AlphaCurrency::from(9876),
        );

        // Run migration
        migration();

        // Test that values are removed from state
        assert!(!deprecated_swap_maps::AlphaSqrtPrice::<Test>::contains_key(
            NetUid::from(1)
        ),);
        assert!(!deprecated_swap_maps::ScrapReservoirAlpha::<Test>::contains_key(NetUid::from(1)),);
    });
}
