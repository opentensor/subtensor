#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use approx::assert_abs_diff_eq;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_arithmetic::helpers_128bit;
use sp_runtime::DispatchError;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::NetUid;
use subtensor_swap_interface::Order as OrderT;

use super::*;
use crate::pallet::swap_step::*;
use crate::{SqrtPrice, mock::*};

// this function is used to convert price (NON-SQRT price!) to TickIndex. it's only utility for
// testing, all the implementation logic is based on sqrt prices
fn price_to_tick(price: f64) -> TickIndex {
    let price_sqrt: SqrtPrice = SqrtPrice::from_num(price.sqrt());
    // Handle potential errors in the conversion
    match TickIndex::try_from_sqrt_price(price_sqrt) {
        Ok(mut tick) => {
            // Ensure the tick is within bounds
            if tick > TickIndex::MAX {
                tick = TickIndex::MAX;
            } else if tick < TickIndex::MIN {
                tick = TickIndex::MIN;
            }
            tick
        }
        // Default to a reasonable value when conversion fails
        Err(_) => {
            if price > 1.0 {
                TickIndex::MAX
            } else {
                TickIndex::MIN
            }
        }
    }
}

fn get_ticked_prices_around_current_price() -> (f64, f64) {
    // Get current price, ticks around it, and prices on the tick edges for test cases
    let netuid = NetUid::from(1);
    assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
    let current_tick = CurrentTick::<Test>::get(netuid);

    // Low and high prices that match to a lower and higher tick that doesn't contain the current price
    let current_price_low_sqrt = current_tick.as_sqrt_price_bounded();
    let current_price_high_sqrt = current_tick.next().unwrap().as_sqrt_price_bounded();
    let current_price_low = U96F32::from_num(current_price_low_sqrt * current_price_low_sqrt);
    let current_price_high = U96F32::from_num(current_price_high_sqrt * current_price_high_sqrt);

    (
        current_price_low.to_num::<f64>(),
        current_price_high.to_num::<f64>() + 0.000000001,
    )
}

// this function is used to convert tick index NON-SQRT (!) price. it's only utility for
// testing, all the implementation logic is based on sqrt prices
fn tick_to_price(tick: TickIndex) -> f64 {
    // Handle errors gracefully
    match tick.try_to_sqrt_price() {
        Ok(price_sqrt) => (price_sqrt * price_sqrt).to_num::<f64>(),
        Err(_) => {
            // Return a sensible default based on whether the tick is above or below the valid range
            if tick > TickIndex::MAX {
                tick_to_price(TickIndex::MAX) // Use the max valid tick price
            } else {
                tick_to_price(TickIndex::MIN) // Use the min valid tick price
            }
        }
    }
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

            // Verify fee rate validation - should fail if too high
            let too_high_fee = MaxFeeRate::get() + 1;
            assert_noop!(
                Swap::set_fee_rate(RuntimeOrigin::root(), netuid, too_high_fee),
                Error::<Test>::FeeRateTooHigh
            );
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
        let tao = TaoReserve::reserve(netuid.into());
        let alpha = AlphaReserve::reserve(netuid.into());

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        assert!(SwapV3Initialized::<Test>::get(netuid));

        // Verify current price is set
        let sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);
        let expected_sqrt_price = U64F64::from_num(0.5_f64);
        assert_abs_diff_eq!(
            sqrt_price.to_num::<f64>(),
            expected_sqrt_price.to_num::<f64>(),
            epsilon = 0.000000001
        );

        // Verify that current tick is set
        let current_tick = CurrentTick::<Test>::get(netuid);
        let expected_current_tick = TickIndex::from_sqrt_price_bounded(expected_sqrt_price);
        assert_eq!(current_tick, expected_current_tick);

        // Calculate expected liquidity
        let expected_liquidity =
            helpers_128bit::sqrt((tao.to_u64() as u128).saturating_mul(alpha.to_u64() as u128))
                as u64;

        // Get the protocol account
        let protocol_account_id = Pallet::<Test>::protocol_account_id();

        // Verify position created for protocol account
        let positions = Positions::<Test>::iter_prefix_values((netuid, protocol_account_id))
            .collect::<Vec<_>>();
        assert_eq!(positions.len(), 1);

        let position = &positions[0];
        assert_eq!(position.liquidity, expected_liquidity);
        assert_eq!(position.tick_low, TickIndex::MIN);
        assert_eq!(position.tick_high, TickIndex::MAX);
        assert_eq!(position.fees_tao, 0);
        assert_eq!(position.fees_alpha, 0);

        // Verify ticks were created
        let tick_low = Ticks::<Test>::get(netuid, TickIndex::MIN).unwrap();
        let tick_high = Ticks::<Test>::get(netuid, TickIndex::MAX).unwrap();

        // Check liquidity values
        assert_eq!(tick_low.liquidity_net, expected_liquidity as i128);
        assert_eq!(tick_low.liquidity_gross, expected_liquidity);
        assert_eq!(tick_high.liquidity_net, -(expected_liquidity as i128));
        assert_eq!(tick_high.liquidity_gross, expected_liquidity);

        // Verify current liquidity is set
        assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity);
    });
}

// Test adding liquidity on top of the existing protocol liquidity
#[test]
fn test_add_liquidity_basic() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick, TickIndex::MAX);

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(NetUid::from(1)));
        let current_price = Pallet::<Test>::current_price(NetUid::from(1)).to_num::<f64>();
        let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

        // As a user add liquidity with all possible corner cases
        //   - Initial price is 0.25
        //   - liquidity is expressed in RAO units
        // Test case is (price_low, price_high, liquidity, tao, alpha)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (
                min_price,
                max_price,
                2_000_000_000_u64,
                1_000_000_000_u64,
                4_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range: Expect the same alpha
            (
                current_price_high,
                max_price,
                2_000_000_000_u64,
                0,
                4_000_000_000,
            ),
            // Repeat the protocol liquidity at min to current range: Expect all the same tao
            (
                min_price,
                current_price_low,
                2_000_000_000_u64,
                1_000_000_000,
                0,
            ),
            // Half to double price - just some sane wothdraw amounts
            (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
            // Both below price - tao is non-zero, alpha is zero
            (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
            // Both above price - tao is zero, alpha is non-zero
            (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
        ]
        .into_iter()
        .enumerate()
        .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3, v.4))
        .for_each(
            |(netuid, price_low, price_high, liquidity, expected_tao, expected_alpha)| {
                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                // Calculate ticks (assuming tick math is tested separately)
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);

                // Get tick infos and liquidity before adding (to account for protocol liquidity)
                let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
                let tick_high_info_before =
                    Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

                // Add liquidity
                let (position_id, tao, alpha) = Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity,
                )
                .unwrap();

                assert_abs_diff_eq!(tao, expected_tao, epsilon = tao / 1000);
                assert_abs_diff_eq!(alpha, expected_alpha, epsilon = alpha / 1000);

                // Check that low and high ticks appear in the state and are properly updated
                let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
                let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
                let expected_liquidity_net_low = liquidity as i128;
                let expected_liquidity_gross_low = liquidity;
                let expected_liquidity_net_high = -(liquidity as i128);
                let expected_liquidity_gross_high = liquidity;

                assert_eq!(
                    tick_low_info.liquidity_net - tick_low_info_before.liquidity_net,
                    expected_liquidity_net_low,
                );
                assert_eq!(
                    tick_low_info.liquidity_gross - tick_low_info_before.liquidity_gross,
                    expected_liquidity_gross_low,
                );
                assert_eq!(
                    tick_high_info.liquidity_net - tick_high_info_before.liquidity_net,
                    expected_liquidity_net_high,
                );
                assert_eq!(
                    tick_high_info.liquidity_gross - tick_high_info_before.liquidity_gross,
                    expected_liquidity_gross_high,
                );

                // Liquidity position at correct ticks
                assert_eq!(
                    Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                    1
                );

                let position =
                    Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
                assert_eq!(position.liquidity, liquidity);
                assert_eq!(position.tick_low, tick_low);
                assert_eq!(position.tick_high, tick_high);
                assert_eq!(position.fees_alpha, 0);
                assert_eq!(position.fees_tao, 0);

                // Current liquidity is updated only when price range includes the current price
                let expected_liquidity =
                    if (price_high > current_price) && (price_low <= current_price) {
                        liquidity_before + liquidity
                    } else {
                        liquidity_before
                    };

                assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity)
            },
        );
    });
}

#[test]
fn test_add_liquidity_max_limit_enforced() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let liquidity = 2_000_000_000_u64;
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        let limit = MaxPositions::get() as usize;

        for _ in 0..limit {
            Pallet::<Test>::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                TickIndex::MIN,
                TickIndex::MAX,
                liquidity,
            )
            .unwrap();
        }

        let test_result = Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            TickIndex::MIN,
            TickIndex::MAX,
            liquidity,
        );

        assert_err!(test_result, Error::<Test>::MaxPositionsExceeded);
    });
}

#[test]
fn test_add_liquidity_out_of_bounds() {
    new_test_ext().execute_with(|| {
        [
            // For our tests, we'll construct TickIndex values that are intentionally
            // outside the valid range for testing purposes only
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                TickIndex::MAX,
                1_000_000_000_u64,
            ),
            (
                TickIndex::MIN,
                TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                1_000_000_000_u64,
            ),
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                1_000_000_000_u64,
            ),
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 100),
                TickIndex::new_unchecked(TickIndex::MAX.get() + 100),
                1_000_000_000_u64,
            ),
            // Inverted ticks: high < low
            (
                TickIndex::new_unchecked(-900),
                TickIndex::new_unchecked(-1000),
                1_000_000_000_u64,
            ),
            // Equal ticks: high == low
            (
                TickIndex::new_unchecked(-10_000),
                TickIndex::new_unchecked(-10_000),
                1_000_000_000_u64,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2))
        .for_each(|(netuid, tick_low, tick_high, liquidity)| {
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Add liquidity
            assert_err!(
                Swap::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity
                ),
                Error::<Test>::InvalidTickRange,
            );
        });
    });
}

#[test]
fn test_add_liquidity_over_balance() {
    new_test_ext().execute_with(|| {
        let coldkey_account_id = 3;
        let hotkey_account_id = 1002;

        [
            // Lower than price (not enough tao)
            (0.1, 0.2, 100_000_000_000_u64),
            // Higher than price (not enough alpha)
            (0.3, 0.4, 100_000_000_000_u64),
            // Around the price (not enough both)
            (0.1, 0.4, 100_000_000_000_u64),
        ]
        .into_iter()
        .enumerate()
        .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2))
        .for_each(|(netuid, price_low, price_high, liquidity)| {
            // Calculate ticks
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);

            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Add liquidity
            assert_err!(
                Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &coldkey_account_id,
                    &hotkey_account_id,
                    tick_low,
                    tick_high,
                    liquidity
                ),
                Error::<Test>::InsufficientBalance,
            );
        });
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_remove_liquidity_basic --exact --show-output
#[test]
fn test_remove_liquidity_basic() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick, TickIndex::MAX);

        let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

        // As a user add liquidity with all possible corner cases
        //   - Initial price is 0.25
        //   - liquidity is expressed in RAO units
        // Test case is (price_low, price_high, liquidity, tao, alpha)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (
                min_price,
                max_price,
                2_000_000_000_u64,
                1_000_000_000_u64,
                4_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range: Expect the same alpha
            (
                current_price_high,
                max_price,
                2_000_000_000_u64,
                0,
                4_000_000_000,
            ),
            // Repeat the protocol liquidity at min to current range: Expect all the same tao
            (
                min_price,
                current_price_low,
                2_000_000_000_u64,
                1_000_000_000,
                0,
            ),
            // Half to double price - just some sane wothdraw amounts
            (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
            // Both below price - tao is non-zero, alpha is zero
            (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
            // Both above price - tao is zero, alpha is non-zero
            (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
        ]
        .into_iter()
        .enumerate()
        .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3, v.4))
        .for_each(|(netuid, price_low, price_high, liquidity, tao, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);

            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
            let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

            // Add liquidity
            let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            )
            .unwrap();

            // Remove liquidity
            let remove_result =
                Pallet::<Test>::do_remove_liquidity(netuid, &OK_COLDKEY_ACCOUNT_ID, position_id)
                    .unwrap();
            assert_abs_diff_eq!(remove_result.tao.to_u64(), tao, epsilon = tao / 1000);
            assert_abs_diff_eq!(
                u64::from(remove_result.alpha),
                alpha,
                epsilon = alpha / 1000
            );
            assert_eq!(remove_result.fee_tao, TaoCurrency::ZERO);
            assert_eq!(remove_result.fee_alpha, AlphaCurrency::ZERO);

            // Liquidity position is removed
            assert_eq!(
                Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                0
            );
            assert!(Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).is_none());

            // Current liquidity is updated (back where it was)
            assert_eq!(CurrentLiquidity::<Test>::get(netuid), liquidity_before);
        });
    });
}

#[test]
fn test_remove_liquidity_nonexisting_position() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick.get(), TickIndex::MAX.get());

        let liquidity = 2_000_000_000_u64;
        let netuid = NetUid::from(1);

        // Calculate ticks (assuming tick math is tested separately)
        let tick_low = price_to_tick(min_price);
        let tick_high = price_to_tick(max_price);

        // Setup swap
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Add liquidity
        assert_ok!(Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            tick_low,
            tick_high,
            liquidity,
        ));

        assert!(Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID) > 0);

        // Remove liquidity
        assert_err!(
            Pallet::<Test>::do_remove_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                PositionId::new::<Test>()
            ),
            Error::<Test>::LiquidityNotFound,
        );
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_modify_position_basic --exact --show-output
#[test]
fn test_modify_position_basic() {
    new_test_ext().execute_with(|| {
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        let limit_price = 1000.0_f64;
        assert_eq!(max_tick, TickIndex::MAX);
        let (current_price_low, _current_price_high) = get_ticked_prices_around_current_price();

        // As a user add liquidity with all possible corner cases
        //   - Initial price is 0.25
        //   - liquidity is expressed in RAO units
        // Test case is (price_low, price_high, liquidity, tao, alpha)
        [
            // Repeat the protocol liquidity at current to max range: Expect the same alpha
            (
                current_price_low,
                max_price,
                2_000_000_000_u64,
                4_000_000_000,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3))
        .for_each(|(netuid, price_low, price_high, liquidity, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);

            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Add liquidity
            let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            )
            .unwrap();

            // Get tick infos before the swap/update
            let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap();
            let tick_high_info_before = Ticks::<Test>::get(netuid, tick_high).unwrap();

            // Swap to create fees on the position
            let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
            let order = GetAlphaForTao::with_amount(liquidity / 10);
            Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

            // Modify liquidity (also causes claiming of fees)
            let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
            let modify_result = Pallet::<Test>::do_modify_position(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                position_id,
                -((liquidity / 10) as i64),
            )
            .unwrap();
            assert_abs_diff_eq!(
                u64::from(modify_result.alpha),
                alpha / 10,
                epsilon = alpha / 1000
            );
            assert!(modify_result.fee_tao > TaoCurrency::ZERO);
            assert_eq!(modify_result.fee_alpha, AlphaCurrency::ZERO);

            // Liquidity position is reduced
            assert_eq!(
                Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                1
            );

            // Current liquidity is reduced with modify_position
            assert!(CurrentLiquidity::<Test>::get(netuid) < liquidity_before);

            // Position liquidity is reduced
            let position =
                Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
            assert_eq!(position.liquidity, liquidity * 9 / 10);
            assert_eq!(position.tick_low, tick_low);
            assert_eq!(position.tick_high, tick_high);

            // Tick liquidity is updated properly for low and high position ticks
            let tick_low_info_after = Ticks::<Test>::get(netuid, tick_low).unwrap();
            let tick_high_info_after = Ticks::<Test>::get(netuid, tick_high).unwrap();

            assert_eq!(
                tick_low_info_before.liquidity_net - (liquidity / 10) as i128,
                tick_low_info_after.liquidity_net,
            );
            assert_eq!(
                tick_low_info_before.liquidity_gross - (liquidity / 10),
                tick_low_info_after.liquidity_gross,
            );
            assert_eq!(
                tick_high_info_before.liquidity_net + (liquidity / 10) as i128,
                tick_high_info_after.liquidity_net,
            );
            assert_eq!(
                tick_high_info_before.liquidity_gross - (liquidity / 10),
                tick_high_info_after.liquidity_gross,
            );

            // Modify liquidity again (ensure fees aren't double-collected)
            let modify_result = Pallet::<Test>::do_modify_position(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                position_id,
                -((liquidity / 100) as i64),
            )
            .unwrap();

            assert_abs_diff_eq!(
                u64::from(modify_result.alpha),
                alpha / 100,
                epsilon = alpha / 1000
            );
            assert_eq!(modify_result.fee_tao, TaoCurrency::ZERO);
            assert_eq!(modify_result.fee_alpha, AlphaCurrency::ZERO);
        });
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_basic --exact --show-output
#[test]
fn test_swap_basic() {
    new_test_ext().execute_with(|| {
        fn perform_test<Order>(
            netuid: NetUid,
            order: Order,
            limit_price: f64,
            output_amount: u64,
            price_should_grow: bool,
        ) where
            Order: OrderT,
            Order::PaidIn: GlobalFeeInfo,
            BasicSwapStep<Test, Order::PaidIn, Order::PaidOut>:
                SwapStep<Test, Order::PaidIn, Order::PaidOut>,
        {
            // Consumed liquidity ticks
            let tick_low = TickIndex::MIN;
            let tick_high = TickIndex::MAX;
            let liquidity = order.amount().to_u64();

            // Setup swap
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Get tick infos before the swap
            let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
            let tick_high_info_before = Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
            let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

            // Get current price
            let current_price = Pallet::<Test>::current_price(netuid);

            // Swap
            let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
            let swap_result =
                Pallet::<Test>::do_swap(netuid, order.clone(), sqrt_limit_price, false, false)
                    .unwrap();
            assert_abs_diff_eq!(
                swap_result.amount_paid_out.to_u64(),
                output_amount,
                epsilon = output_amount / 100
            );

            assert_abs_diff_eq!(
                swap_result.paid_in_reserve_delta() as u64,
                liquidity,
                epsilon = liquidity / 10
            );
            assert_abs_diff_eq!(
                swap_result.paid_out_reserve_delta() as i64,
                -(output_amount as i64),
                epsilon = output_amount as i64 / 10
            );

            // Check that low and high ticks' fees were updated properly, and liquidity values were not updated
            let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
            let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
            let expected_liquidity_net_low = tick_low_info_before.liquidity_net;
            let expected_liquidity_gross_low = tick_low_info_before.liquidity_gross;
            let expected_liquidity_net_high = tick_high_info_before.liquidity_net;
            let expected_liquidity_gross_high = tick_high_info_before.liquidity_gross;
            assert_eq!(tick_low_info.liquidity_net, expected_liquidity_net_low,);
            assert_eq!(tick_low_info.liquidity_gross, expected_liquidity_gross_low,);
            assert_eq!(tick_high_info.liquidity_net, expected_liquidity_net_high,);
            assert_eq!(
                tick_high_info.liquidity_gross,
                expected_liquidity_gross_high,
            );

            // Expected fee amount
            let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;
            let expected_fee = (liquidity as f64 * fee_rate) as u64;

            // Global fees should be updated
            let actual_global_fee = (order.amount().global_fee(netuid).to_num::<f64>()
                * (liquidity_before as f64)) as u64;

            assert!((swap_result.fee_paid.to_u64() as i64 - expected_fee as i64).abs() <= 1);
            assert!((actual_global_fee as i64 - expected_fee as i64).abs() <= 1);

            // Tick fees should be updated

            // Liquidity position should not be updated
            let protocol_id = Pallet::<Test>::protocol_account_id();
            let positions =
                Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
            let position = positions.first().unwrap();

            assert_eq!(
                position.liquidity,
                helpers_128bit::sqrt(
                    TaoReserve::reserve(netuid.into()).to_u64() as u128
                        * AlphaReserve::reserve(netuid.into()).to_u64() as u128
                ) as u64
            );
            assert_eq!(position.tick_low, tick_low);
            assert_eq!(position.tick_high, tick_high);
            assert_eq!(position.fees_alpha, 0);
            assert_eq!(position.fees_tao, 0);

            // Current liquidity is not updated
            assert_eq!(CurrentLiquidity::<Test>::get(netuid), liquidity_before);

            // Assert that price movement is in correct direction
            let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
            let current_price_after = Pallet::<Test>::current_price(netuid);
            assert_eq!(current_price_after >= current_price, price_should_grow);

            // Assert that current tick is updated
            let current_tick = CurrentTick::<Test>::get(netuid);
            let expected_current_tick =
                TickIndex::from_sqrt_price_bounded(sqrt_current_price_after);
            assert_eq!(current_tick, expected_current_tick);
        }

        // Current price is 0.25
        // Test case is (order_type, liquidity, limit_price, output_amount)
        perform_test(
            1.into(),
            GetAlphaForTao::with_amount(1_000),
            1000.0,
            3990,
            true,
        );
        perform_test(
            2.into(),
            GetTaoForAlpha::with_amount(1_000),
            0.0001,
            250,
            false,
        );
        perform_test(
            3.into(),
            GetAlphaForTao::with_amount(500_000_000),
            1000.0,
            2_000_000_000,
            true,
        );
    });
}

// In this test the swap starts and ends within one (large liquidity) position
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_swap_single_position --exact --show-output
#[test]
fn test_swap_single_position() {
    let min_price = tick_to_price(TickIndex::MIN);
    let max_price = tick_to_price(TickIndex::MAX);
    let max_tick = price_to_tick(max_price);
    let netuid = NetUid::from(1);
    assert_eq!(max_tick, TickIndex::MAX);

    let mut current_price_low = 0_f64;
    let mut current_price_high = 0_f64;
    let mut current_price = 0_f64;
    new_test_ext().execute_with(|| {
        let (low, high) = get_ticked_prices_around_current_price();
        current_price_low = low;
        current_price_high = high;
        current_price = Pallet::<Test>::current_price(netuid).to_num::<f64>();
    });

    macro_rules! perform_test {
        ($order_t:ident,
		 $price_low_offset:expr,
		 $price_high_offset:expr,
		 $position_liquidity:expr,
		 $liquidity_fraction:expr,
		 $limit_price:expr,
		 $price_should_grow:expr
		 ) => {
            new_test_ext().execute_with(|| {
                let price_low_offset = $price_low_offset;
                let price_high_offset = $price_high_offset;
                let position_liquidity = $position_liquidity;
                let order_liquidity_fraction = $liquidity_fraction;
                let limit_price = $limit_price;
                let price_should_grow = $price_should_grow;

                //////////////////////////////////////////////
                // Initialize pool and add the user position
                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
                let tao_reserve = TaoReserve::reserve(netuid.into()).to_u64();
                let alpha_reserve = AlphaReserve::reserve(netuid.into()).to_u64();
                let protocol_liquidity = (tao_reserve as f64 * alpha_reserve as f64).sqrt();

                // Add liquidity
                let current_price = Pallet::<Test>::current_price(netuid).to_num::<f64>();
                let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid).to_num::<f64>();

                let price_low = price_low_offset + current_price;
                let price_high = price_high_offset + current_price;
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);
                let (_position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    position_liquidity,
                )
                .unwrap();

                // Liquidity position at correct ticks
                assert_eq!(
                    Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                    1
                );

                // Get tick infos before the swap
                let tick_low_info_before = Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
                let tick_high_info_before =
                    Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
                assert_abs_diff_eq!(
                    liquidity_before as f64,
                    protocol_liquidity + position_liquidity as f64,
                    epsilon = liquidity_before as f64 / 1000.
                );

                //////////////////////////////////////////////
                // Swap

                // Calculate the expected output amount for the cornercase of one step
                let order_liquidity = order_liquidity_fraction * position_liquidity as f64;

                let output_amount = <Test as TestExt<$order_t>>::approx_expected_swap_output(
                    sqrt_current_price,
                    liquidity_before as f64,
                    order_liquidity,
                );

                // Do the swap
                let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                let order = $order_t::with_amount(order_liquidity as u64);
                let swap_result =
                    Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();
                assert_abs_diff_eq!(
                    swap_result.amount_paid_out.to_u64() as f64,
                    output_amount,
                    epsilon = output_amount / 10.
                );

                if order_liquidity_fraction <= 0.001 {
                    assert_abs_diff_eq!(
                        swap_result.paid_in_reserve_delta() as i64,
                        order_liquidity as i64,
                        epsilon = order_liquidity as i64 / 10
                    );
                    assert_abs_diff_eq!(
                        swap_result.paid_out_reserve_delta() as i64,
                        -(output_amount as i64),
                        epsilon = output_amount as i64 / 10
                    );
                }

                // Assert that price movement is in correct direction
                let current_price_after = Pallet::<Test>::current_price(netuid);
                assert_eq!(price_should_grow, current_price_after > current_price);

                // Assert that for small amounts price stays within the user position
                if (order_liquidity_fraction <= 0.001)
                    && (price_low_offset > 0.0001)
                    && (price_high_offset > 0.0001)
                {
                    assert!(current_price_after <= price_high);
                    assert!(current_price_after >= price_low);
                }

                // Check that low and high ticks' fees were updated properly
                let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
                let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
                let expected_liquidity_net_low = tick_low_info_before.liquidity_net;
                let expected_liquidity_gross_low = tick_low_info_before.liquidity_gross;
                let expected_liquidity_net_high = tick_high_info_before.liquidity_net;
                let expected_liquidity_gross_high = tick_high_info_before.liquidity_gross;
                assert_eq!(tick_low_info.liquidity_net, expected_liquidity_net_low,);
                assert_eq!(tick_low_info.liquidity_gross, expected_liquidity_gross_low,);
                assert_eq!(tick_high_info.liquidity_net, expected_liquidity_net_high,);
                assert_eq!(
                    tick_high_info.liquidity_gross,
                    expected_liquidity_gross_high,
                );

                // Expected fee amount
                let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;
                let expected_fee = (order_liquidity - order_liquidity / (1.0 + fee_rate)) as u64;

                // // Global fees should be updated
                let actual_global_fee = ($order_t::with_amount(0)
                    .amount()
                    .global_fee(netuid)
                    .to_num::<f64>()
                    * (liquidity_before as f64)) as u64;

                assert_abs_diff_eq!(
                    swap_result.fee_paid.to_u64(),
                    expected_fee,
                    epsilon = expected_fee / 10
                );
                assert_abs_diff_eq!(actual_global_fee, expected_fee, epsilon = expected_fee / 10);

                // Tick fees should be updated

                // Liquidity position should not be updated
                let positions =
                    Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                        .collect::<Vec<_>>();
                let position = positions.first().unwrap();

                assert_eq!(position.liquidity, position_liquidity,);
                assert_eq!(position.tick_low, tick_low);
                assert_eq!(position.tick_high, tick_high);
                assert_eq!(position.fees_alpha, 0);
                assert_eq!(position.fees_tao, 0);
            });
        };
    }

    // Current price is 0.25
    // The test case is based on the current price and position prices are defined as a price
    // offset from the current price
    // Outer part of test case is Position: (price_low_offset, price_high_offset, liquidity)
    [
        // Very localized position at the current price
        (-0.1, 0.1, 500_000_000_000_u64),
        // Repeat the protocol liquidity at maximum range
        (
            min_price - current_price,
            max_price - current_price,
            2_000_000_000_u64,
        ),
        // Repeat the protocol liquidity at current to max range
        (
            current_price_high - current_price,
            max_price - current_price,
            2_000_000_000_u64,
        ),
        // Repeat the protocol liquidity at min to current range
        (
            min_price - current_price,
            current_price_low - current_price,
            2_000_000_000_u64,
        ),
        // Half to double price
        (-0.125, 0.25, 2_000_000_000_u64),
        // A few other price ranges and liquidity volumes
        (-0.1, 0.1, 2_000_000_000_u64),
        (-0.1, 0.1, 10_000_000_000_u64),
        (-0.1, 0.1, 100_000_000_000_u64),
        (-0.01, 0.01, 100_000_000_000_u64),
        (-0.001, 0.001, 100_000_000_000_u64),
    ]
    .into_iter()
    .for_each(
        |(price_low_offset, price_high_offset, position_liquidity)| {
            // Inner part of test case is Order: (order_type, order_liquidity, limit_price)
            // order_liquidity is represented as a fraction of position_liquidity
            for liquidity_fraction in [0.0001, 0.001, 0.01, 0.1, 0.2, 0.5] {
                perform_test!(
                    GetAlphaForTao,
                    price_low_offset,
                    price_high_offset,
                    position_liquidity,
                    liquidity_fraction,
                    1000.0_f64,
                    true
                );
                perform_test!(
                    GetTaoForAlpha,
                    price_low_offset,
                    price_high_offset,
                    position_liquidity,
                    liquidity_fraction,
                    0.0001_f64,
                    false
                );
            }
        },
    );
}

// This test is a sanity check for swap and multiple positions
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_multiple_positions --exact --show-output --nocapture
#[test]
fn test_swap_multiple_positions() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        let netuid = NetUid::from(1);
        assert_eq!(max_tick, TickIndex::MAX);

        //////////////////////////////////////////////
        // Initialize pool and add the user position
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Add liquidity
        let current_price = Pallet::<Test>::current_price(netuid).to_num::<f64>();

        // Current price is 0.25
        // All positions below are placed at once
        [
            // Very localized position at the current price
            (-0.1, 0.1, 500_000_000_000_u64),
            // Repeat the protocol liquidity at maximum range
            (
                min_price - current_price,
                max_price - current_price,
                2_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range
            (0.0, max_price - current_price, 2_000_000_000_u64),
            // Repeat the protocol liquidity at min to current range
            (min_price - current_price, 0.0, 2_000_000_000_u64),
            // Half to double price
            (-0.125, 0.25, 2_000_000_000_u64),
            // A few other price ranges and liquidity volumes
            (-0.1, 0.1, 2_000_000_000_u64),
            (-0.1, 0.1, 10_000_000_000_u64),
            (-0.1, 0.1, 100_000_000_000_u64),
            (-0.01, 0.01, 100_000_000_000_u64),
            (-0.001, 0.001, 100_000_000_000_u64),
            // A few (overlapping) positions up the range
            (0.01, 0.02, 100_000_000_000_u64),
            (0.02, 0.03, 100_000_000_000_u64),
            (0.03, 0.04, 100_000_000_000_u64),
            (0.03, 0.05, 100_000_000_000_u64),
            // A few (overlapping) positions down the range
            (-0.02, -0.01, 100_000_000_000_u64),
            (-0.03, -0.02, 100_000_000_000_u64),
            (-0.04, -0.03, 100_000_000_000_u64),
            (-0.05, -0.03, 100_000_000_000_u64),
        ]
        .into_iter()
        .for_each(
            |(price_low_offset, price_high_offset, position_liquidity)| {
                let price_low = price_low_offset + current_price;
                let price_high = price_high_offset + current_price;
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);
                let (_position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    position_liquidity,
                )
                .unwrap();
            },
        );

        macro_rules! perform_test {
            ($order_t:ident, $order_liquidity:expr, $limit_price:expr, $should_price_grow:expr) => {
                //////////////////////////////////////////////
                // Swap
                let order_liquidity = $order_liquidity;
                let limit_price = $limit_price;
                let should_price_grow = $should_price_grow;

                let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
                let current_price = (sqrt_current_price * sqrt_current_price).to_num::<f64>();
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
                let output_amount = <Test as TestExt<$order_t>>::approx_expected_swap_output(
                    sqrt_current_price.to_num(),
                    liquidity_before as f64,
                    order_liquidity as f64,
                );

                // Do the swap
                let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                let order = $order_t::with_amount(order_liquidity);
                let swap_result =
                    Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();
                assert_abs_diff_eq!(
                    swap_result.amount_paid_out.to_u64() as f64,
                    output_amount,
                    epsilon = output_amount / 10.
                );

                let tao_reserve = TaoReserve::reserve(netuid.into()).to_u64();
                let alpha_reserve = AlphaReserve::reserve(netuid.into()).to_u64();
                let output_amount = output_amount as u64;

                assert!(output_amount > 0);

                if alpha_reserve > order_liquidity && tao_reserve > order_liquidity {
                    assert_abs_diff_eq!(
                        swap_result.paid_in_reserve_delta() as i64,
                        order_liquidity as i64,
                        epsilon = order_liquidity as i64 / 100
                    );
                    assert_abs_diff_eq!(
                        swap_result.paid_out_reserve_delta() as i64,
                        -(output_amount as i64),
                        epsilon = output_amount as i64 / 100
                    );
                }

                // Assert that price movement is in correct direction
                let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
                let current_price_after =
                    (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
                assert_eq!(should_price_grow, current_price_after > current_price);
            };
        }

        // All these orders are executed without swap reset
        for order_liquidity in [
            (100_000_u64),
            (1_000_000),
            (10_000_000),
            (100_000_000),
            (200_000_000),
            (500_000_000),
            (1_000_000_000),
            (10_000_000_000),
        ] {
            perform_test!(GetAlphaForTao, order_liquidity, 1000.0_f64, true);
            perform_test!(GetTaoForAlpha, order_liquidity, 0.0001_f64, false);
        }

        // Current price shouldn't be much different from the original
        let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
        let current_price_after =
            (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
        assert_abs_diff_eq!(
            current_price,
            current_price_after,
            epsilon = current_price / 10.
        )
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_precision_edge_case --exact --show-output
#[test]
fn test_swap_precision_edge_case() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(123); // 123 is netuid with low edge case liquidity
        let order = GetTaoForAlpha::with_amount(1_000_000_000_000_000_000);
        let tick_low = TickIndex::MIN;

        let sqrt_limit_price: SqrtPrice = tick_low.try_to_sqrt_price().unwrap();

        // Setup swap
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Swap
        let swap_result =
            Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, true).unwrap();

        assert!(swap_result.amount_paid_out > TaoCurrency::ZERO);
    });
}

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_price_tick_price_roundtrip --exact --show-output
#[test]
fn test_price_tick_price_roundtrip() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Setup swap
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        let current_price = SqrtPrice::from_num(0.500_000_512_192_122_7);
        let tick = TickIndex::try_from_sqrt_price(current_price).unwrap();

        let round_trip_price = TickIndex::try_to_sqrt_price(&tick).unwrap();
        assert!(round_trip_price <= current_price);

        let roundtrip_tick = TickIndex::try_from_sqrt_price(round_trip_price).unwrap();
        assert!(tick == roundtrip_tick);
    });
}

#[test]
fn test_convert_deltas() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        for (sqrt_price, delta_in, expected_buy, expected_sell) in [
            (SqrtPrice::from_num(1.5), 1, 0, 2),
            (SqrtPrice::from_num(1.5), 10000, 4444, 22500),
            (SqrtPrice::from_num(1.5), 1000000, 444444, 2250000),
            (
                SqrtPrice::from_num(1.5),
                u64::MAX,
                2000000000000,
                3000000000000,
            ),
            (
                TickIndex::MIN.as_sqrt_price_bounded(),
                1,
                18406523739291577836,
                465,
            ),
            (TickIndex::MIN.as_sqrt_price_bounded(), 10000, u64::MAX, 465),
            (
                TickIndex::MIN.as_sqrt_price_bounded(),
                1000000,
                u64::MAX,
                465,
            ),
            (
                TickIndex::MIN.as_sqrt_price_bounded(),
                u64::MAX,
                u64::MAX,
                464,
            ),
            (
                TickIndex::MAX.as_sqrt_price_bounded(),
                1,
                0,
                18406523745214495085,
            ),
            (TickIndex::MAX.as_sqrt_price_bounded(), 10000, 0, u64::MAX),
            (TickIndex::MAX.as_sqrt_price_bounded(), 1000000, 0, u64::MAX),
            (
                TickIndex::MAX.as_sqrt_price_bounded(),
                u64::MAX,
                2000000000000,
                u64::MAX,
            ),
        ] {
            {
                AlphaSqrtPrice::<Test>::insert(netuid, sqrt_price);

                assert_abs_diff_eq!(
                    BasicSwapStep::<Test, AlphaCurrency, TaoCurrency>::convert_deltas(
                        netuid,
                        delta_in.into()
                    ),
                    expected_sell.into(),
                    epsilon = 2.into()
                );
                assert_abs_diff_eq!(
                    BasicSwapStep::<Test, TaoCurrency, AlphaCurrency>::convert_deltas(
                        netuid,
                        delta_in.into()
                    ),
                    expected_buy.into(),
                    epsilon = 2.into()
                );
            }
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

/// Test correctness of swap fees:
///   - Fees are distribued to (concentrated) liquidity providers
///
#[test]
fn test_swap_fee_correctness() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let netuid = NetUid::from(1);

        // Provide very spread liquidity at the range from min to max that matches protocol liquidity
        let liquidity = 2_000_000_000_000_u64; // 1x of protocol liquidity

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Calculate ticks
        let tick_low = price_to_tick(min_price);
        let tick_high = price_to_tick(max_price);

        // Add user liquidity
        let (position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            tick_low,
            tick_high,
            liquidity,
        )
        .unwrap();

        // Swap buy and swap sell
        Pallet::<Test>::do_swap(
            netuid,
            GetAlphaForTao::with_amount(liquidity / 10),
            u64::MAX.into(),
            false,
            false,
        )
        .unwrap();
        Pallet::<Test>::do_swap(
            netuid,
            GetTaoForAlpha::with_amount(liquidity / 10),
            0_u64.into(),
            false,
            false,
        )
        .unwrap();

        // Get user position
        let mut position =
            Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
        assert_eq!(position.liquidity, liquidity);
        assert_eq!(position.tick_low, tick_low);
        assert_eq!(position.tick_high, tick_high);

        // Check that 50% of fees were credited to the position
        let fee_rate = FeeRate::<Test>::get(NetUid::from(netuid)) as f64 / u16::MAX as f64;
        let (actual_fee_tao, actual_fee_alpha) = position.collect_fees();
        let expected_fee = (fee_rate * (liquidity / 10) as f64 * 0.5) as u64;

        assert_abs_diff_eq!(actual_fee_tao, expected_fee, epsilon = 1,);
        assert_abs_diff_eq!(actual_fee_alpha, expected_fee, epsilon = 1,);
    });
}

#[test]
fn test_current_liquidity_updates() {
    let netuid = NetUid::from(1);
    let liquidity = 1_000_000_000;

    // Get current price
    let (current_price, current_price_low, current_price_high) =
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
            let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
            let current_price = (sqrt_current_price * sqrt_current_price).to_num::<f64>();
            let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();
            (current_price, current_price_low, current_price_high)
        });

    // Test case: (price_low, price_high, expect_to_update)
    [
        // Current price is out of position range (lower), no current lq update
        (current_price * 2., current_price * 3., false),
        // Current price is out of position range (higher), no current lq update
        (current_price / 3., current_price / 2., false),
        // Current price is just below position range, no current lq update
        (current_price_high, current_price * 3., false),
        // Position lower edge is just below the current price, current lq updates
        (current_price_low, current_price * 3., true),
        // Current price is exactly at lower edge of position range, current lq updates
        (current_price, current_price * 3., true),
        // Current price is exactly at higher edge of position range, no current lq update
        (current_price / 2., current_price, false),
    ]
    .into_iter()
    .for_each(|(price_low, price_high, expect_to_update)| {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);
            let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

            // Add liquidity
            assert_ok!(Pallet::<Test>::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            ));

            // Current liquidity is updated only when price range includes the current price
            let expected_liquidity = if (price_high > current_price) && (price_low <= current_price)
            {
                assert!(expect_to_update);
                liquidity_before + liquidity
            } else {
                assert!(!expect_to_update);
                liquidity_before
            };

            assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity)
        });
    });
}

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

/// Test correctness of swap fees:
///   - New LP is not eligible to previously accrued fees
///
/// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_new_lp_doesnt_get_old_fees --exact --show-output
#[test]
fn test_new_lp_doesnt_get_old_fees() {
    new_test_ext().execute_with(|| {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let netuid = NetUid::from(1);

        // Provide very spread liquidity at the range from min to max that matches protocol liquidity
        let liquidity = 2_000_000_000_000_u64; // 1x of protocol liquidity

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Calculate ticks
        let tick_low = price_to_tick(min_price);
        let tick_high = price_to_tick(max_price);

        // Add user liquidity
        Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            tick_low,
            tick_high,
            liquidity,
        )
        .unwrap();

        // Swap buy and swap sell
        Pallet::<Test>::do_swap(
            netuid,
            GetAlphaForTao::with_amount(liquidity / 10),
            u64::MAX.into(),
            false,
            false,
        )
        .unwrap();
        Pallet::<Test>::do_swap(
            netuid,
            GetTaoForAlpha::with_amount(liquidity / 10),
            0_u64.into(),
            false,
            false,
        )
        .unwrap();

        // Add liquidity from a different user to a new tick
        let (position_id_2, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID_2,
            &OK_HOTKEY_ACCOUNT_ID_2,
            tick_low.next().unwrap(),
            tick_high.prev().unwrap(),
            liquidity,
        )
        .unwrap();

        // Get user position
        let mut position =
            Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID_2, position_id_2)).unwrap();
        assert_eq!(position.liquidity, liquidity);
        assert_eq!(position.tick_low, tick_low.next().unwrap());
        assert_eq!(position.tick_high, tick_high.prev().unwrap());

        // Check that collected fees are 0
        let (actual_fee_tao, actual_fee_alpha) = position.collect_fees();
        assert_abs_diff_eq!(actual_fee_tao, 0, epsilon = 1);
        assert_abs_diff_eq!(actual_fee_alpha, 0, epsilon = 1);
    });
}

fn bbox(t: U64F64, a: U64F64, b: U64F64) -> U64F64 {
    if t < a {
        a
    } else if t > b {
        b
    } else {
        t
    }
}

fn print_current_price(netuid: NetUid) {
    let current_sqrt_price = AlphaSqrtPrice::<Test>::get(netuid).to_num::<f64>();
    let current_price = current_sqrt_price * current_sqrt_price;
    log::trace!("Current price: {current_price:.6}");
}

/// RUST_LOG=pallet_subtensor_swap=trace cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_wrapping_fees --exact --show-output --nocapture
#[test]
fn test_wrapping_fees() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(WRAPPING_FEES_NETUID);
        let position_1_low_price = 0.20;
        let position_1_high_price = 0.255;
        let position_2_low_price = 0.255;
        let position_2_high_price = 0.257;
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID_RICH,
            &OK_COLDKEY_ACCOUNT_ID_RICH,
            price_to_tick(position_1_low_price),
            price_to_tick(position_1_high_price),
            1_000_000_000_u64,
        )
        .unwrap();

        print_current_price(netuid);

        let order = GetTaoForAlpha::with_amount(800_000_000);
        let sqrt_limit_price = SqrtPrice::from_num(0.000001);
        Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

        let order = GetAlphaForTao::with_amount(1_850_000_000);
        let sqrt_limit_price = SqrtPrice::from_num(1_000_000.0);

        print_current_price(netuid);

        Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();

        print_current_price(netuid);

        let add_liquidity_result = Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID_RICH,
            &OK_COLDKEY_ACCOUNT_ID_RICH,
            price_to_tick(position_2_low_price),
            price_to_tick(position_2_high_price),
            1_000_000_000_u64,
        )
        .unwrap();

        let order = GetTaoForAlpha::with_amount(1_800_000_000);
        let sqrt_limit_price = SqrtPrice::from_num(0.000001);

        let initial_sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);
        Pallet::<Test>::do_swap(netuid, order, sqrt_limit_price, false, false).unwrap();
        let final_sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);

        print_current_price(netuid);

        let mut position =
            Positions::<Test>::get((netuid, &OK_COLDKEY_ACCOUNT_ID_RICH, add_liquidity_result.0))
                .unwrap();

        let initial_box_price = bbox(
            initial_sqrt_price,
            position.tick_low.try_to_sqrt_price().unwrap(),
            position.tick_high.try_to_sqrt_price().unwrap(),
        );

        let final_box_price = bbox(
            final_sqrt_price,
            position.tick_low.try_to_sqrt_price().unwrap(),
            position.tick_high.try_to_sqrt_price().unwrap(),
        );

        let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;

        log::trace!("fee_rate: {fee_rate:.6}");
        log::trace!("position.liquidity: {}", position.liquidity);
        log::trace!(
            "initial_box_price: {:.6}",
            initial_box_price.to_num::<f64>()
        );
        log::trace!("final_box_price: {:.6}", final_box_price.to_num::<f64>());

        let expected_fee_tao = ((fee_rate / (1.0 - fee_rate))
            * (position.liquidity as f64)
            * (final_box_price.to_num::<f64>() - initial_box_price.to_num::<f64>()))
            as u64;

        let expected_fee_alpha = ((fee_rate / (1.0 - fee_rate))
            * (position.liquidity as f64)
            * ((1.0 / final_box_price.to_num::<f64>()) - (1.0 / initial_box_price.to_num::<f64>())))
            as u64;

        log::trace!("Expected ALPHA fee: {:.6}", expected_fee_alpha as f64);

        let (fee_tao, fee_alpha) = position.collect_fees();

        log::trace!("Collected fees: TAO: {fee_tao}, ALPHA: {fee_alpha}");

        assert_abs_diff_eq!(fee_tao, expected_fee_tao, epsilon = 1);
        assert_abs_diff_eq!(fee_alpha, expected_fee_alpha, epsilon = 1);
    });
}

/// Test that price moves less with provided liquidity
/// cargo test --package pallet-subtensor-swap --lib -- pallet::tests::test_less_price_movement --exact --show-output
#[test]
fn test_less_price_movement() {
    let netuid = NetUid::from(1);
    let mut last_end_price = U96F32::from_num(0);
    let initial_stake_liquidity = 1_000_000_000;
    let swapped_liquidity = 1_000_000;

    // Test case is (order_type, provided_liquidity)
    // Testing algorithm:
    //   - Stake initial_stake_liquidity
    //   - Provide liquidity if iteration provides lq
    //   - Buy or sell
    //   - Save end price if iteration doesn't provide lq
    macro_rules! perform_test {
        ($order_t:ident, $provided_liquidity:expr, $limit_price:expr, $should_price_shrink:expr) => {
            let provided_liquidity = $provided_liquidity;
            let should_price_shrink = $should_price_shrink;
            let limit_price = $limit_price;
            new_test_ext().execute_with(|| {
                // Setup swap
                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                // Buy Alpha
                assert_ok!(Pallet::<Test>::do_swap(
                    netuid,
                    GetAlphaForTao::with_amount(initial_stake_liquidity),
                    SqrtPrice::from_num(10_000_000_000_u64),
                    false,
                    false
                ));

                // Get current price
                let start_price = Pallet::<Test>::current_price(netuid);

                // Add liquidity if this test iteration provides
                if provided_liquidity > 0 {
                    let tick_low = price_to_tick(start_price.to_num::<f64>() * 0.5);
                    let tick_high = price_to_tick(start_price.to_num::<f64>() * 1.5);
                    assert_ok!(Pallet::<Test>::do_add_liquidity(
                        netuid,
                        &OK_COLDKEY_ACCOUNT_ID,
                        &OK_HOTKEY_ACCOUNT_ID,
                        tick_low,
                        tick_high,
                        provided_liquidity,
                    ));
                }

                // Swap
                let sqrt_limit_price = SqrtPrice::from_num(limit_price);
                assert_ok!(Pallet::<Test>::do_swap(
                    netuid,
                    $order_t::with_amount(swapped_liquidity),
                    sqrt_limit_price,
                    false,
                    false
                ));

                let end_price = Pallet::<Test>::current_price(netuid);

                // Save end price if iteration doesn't provide or compare with previous end price if
                // it does
                if provided_liquidity > 0 {
                    assert_eq!(should_price_shrink, end_price < last_end_price);
                } else {
                    last_end_price = end_price;
                }
            });
        };
    }

    for provided_liquidity in [0, 1_000_000_000_000_u64] {
        perform_test!(GetAlphaForTao, provided_liquidity, 1000.0_f64, true);
    }
    for provided_liquidity in [0, 1_000_000_000_000_u64] {
        perform_test!(GetTaoForAlpha, provided_liquidity, 0.001_f64, false);
    }
}

#[test]
fn test_swap_subtoken_disabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(SUBTOKEN_DISABLED_NETUID); // Use a netuid not used elsewhere
        let price_low = 0.1;
        let price_high = 0.2;
        let tick_low = price_to_tick(price_low);
        let tick_high = price_to_tick(price_high);
        let liquidity = 1_000_000_u64;

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        assert_noop!(
            Pallet::<Test>::add_liquidity(
                RuntimeOrigin::signed(OK_COLDKEY_ACCOUNT_ID),
                OK_HOTKEY_ACCOUNT_ID,
                netuid,
                tick_low,
                tick_high,
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

#[test]
fn test_liquidate_v3_removes_positions_ticks_and_state() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Initialize V3 (creates protocol position, ticks, price, liquidity)
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
        assert!(SwapV3Initialized::<Test>::get(netuid));

        // Enable user LP
        assert_ok!(Swap::toggle_user_liquidity(
            RuntimeOrigin::root(),
            netuid.into(),
            true
        ));

        // Add a user position across the full range to ensure ticks/bitmap are populated.
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let tick_low = price_to_tick(min_price);
        let tick_high = price_to_tick(max_price);
        let liquidity = 2_000_000_000_u64;

        let (_pos_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            tick_low,
            tick_high,
            liquidity,
        )
        .expect("add liquidity");

        // Accrue some global fees so we can verify fee storage is cleared later.
        let sqrt_limit_price = SqrtPrice::from_num(1_000_000.0);
        assert_ok!(Pallet::<Test>::do_swap(
            netuid,
            GetAlphaForTao::with_amount(1_000_000),
            sqrt_limit_price,
            false,
            false
        ));

        // Sanity: protocol & user positions exist, ticks exist, liquidity > 0
        let protocol_id = Pallet::<Test>::protocol_account_id();
        let prot_positions =
            Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        assert!(!prot_positions.is_empty());

        let user_positions = Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
            .collect::<Vec<_>>();
        assert_eq!(user_positions.len(), 1);

        assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_some());
        assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_some());
        assert!(CurrentLiquidity::<Test>::get(netuid) > 0);

        let had_bitmap_words = TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
            .next()
            .is_some();
        assert!(had_bitmap_words);

        // ACT: users-only liquidation then protocol clear
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // ASSERT: positions cleared (both user and protocol)
        assert_eq!(
            Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
            0
        );
        let prot_positions_after =
            Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        assert!(prot_positions_after.is_empty());
        let user_positions_after =
            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .collect::<Vec<_>>();
        assert!(user_positions_after.is_empty());

        // ASSERT: ticks cleared
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_none());
        assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_none());

        // ASSERT: fee globals cleared
        assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
        assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

        // ASSERT: price/tick/liquidity flags cleared
        assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
        assert!(!CurrentTick::<Test>::contains_key(netuid));
        assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));

        // ASSERT: active tick bitmap cleared
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none()
        );

        // ASSERT: knobs removed on dereg
        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
    });
}

// V3 path with user liquidity disabled at teardown:
// must still remove positions and clear state (after protocol clear).
// #[test]
// fn test_liquidate_v3_with_user_liquidity_disabled() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(101);

//         assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
//         assert!(SwapV3Initialized::<Test>::get(netuid));

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
//         assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
//         assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
//         assert!(!CurrentTick::<Test>::contains_key(netuid));
//         assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
//         assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

//         // `EnabledUserLiquidity` is removed by protocol clear stage.
//         assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
//     });
// }

/// Non‑V3 path: V3 not initialized (no positions); function must still clear any residual storages and succeed.
#[test]
fn test_liquidate_non_v3_uninitialized_ok_and_clears() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(202);

        // Sanity: V3 is not initialized
        assert!(!SwapV3Initialized::<Test>::get(netuid));
        assert!(
            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );

        // ACT
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // ASSERT: Defensive clears leave no residues and do not panic
        assert!(
            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none()
        );

        // All single-key maps should not have the key after liquidation
        assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
        assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));
        assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
        assert!(!CurrentTick::<Test>::contains_key(netuid));
        assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
    });
}

#[test]
fn test_liquidate_idempotent() {
    // V3 flavor
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(7);
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Add a small user position
        assert_ok!(Swap::toggle_user_liquidity(
            RuntimeOrigin::root(),
            netuid.into(),
            true
        ));
        let tick_low = price_to_tick(0.2);
        let tick_high = price_to_tick(0.3);
        assert_ok!(Pallet::<Test>::do_add_liquidity(
            netuid,
            &OK_COLDKEY_ACCOUNT_ID,
            &OK_HOTKEY_ACCOUNT_ID,
            tick_low,
            tick_high,
            123_456_789
        ));

        // Users-only liquidations are idempotent.
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // Now clear protocol liquidity/state—also idempotent.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // State remains empty
        assert!(
            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none()
        );
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
    });

    // Non‑V3 flavor
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(8);

        // Never initialize V3; both calls no-op and succeed.
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        assert!(
            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                .next()
                .is_none()
        );
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none()
        );
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
    });
}

#[test]
fn liquidate_v3_refunds_user_funds_and_clears_state() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Enable V3 path & initialize price/ticks (also creates a protocol position).
        assert_ok!(Pallet::<Test>::toggle_user_liquidity(
            RuntimeOrigin::root(),
            netuid,
            true
        ));
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Use distinct cold/hot to demonstrate alpha refund/stake accounting.
        let cold = OK_COLDKEY_ACCOUNT_ID;
        let hot = OK_HOTKEY_ACCOUNT_ID;

        // Tight in‑range band around current tick.
        let ct = CurrentTick::<Test>::get(netuid);
        let tick_low = ct.saturating_sub(10);
        let tick_high = ct.saturating_add(10);
        let liquidity: u64 = 1_000_000;

        // Snapshot balances BEFORE.
        let tao_before = <Test as Config>::BalanceOps::tao_balance(&cold);
        let alpha_before_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_before_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_before_total = alpha_before_hot + alpha_before_owner;

        // Create the user position (storage & v3 state only; no balances moved yet).
        let (_pos_id, need_tao, need_alpha) =
            Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
                .expect("add liquidity");

        // Mirror extrinsic bookkeeping: withdraw funds & bump provided‑reserve counters.
        let tao_taken = <Test as Config>::BalanceOps::decrease_balance(&cold, need_tao.into())
            .expect("decrease TAO");
        let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
            &cold,
            &hot,
            netuid.into(),
            need_alpha.into(),
        )
        .expect("decrease ALPHA");
        TaoReserve::increase_provided(netuid.into(), tao_taken);
        AlphaReserve::increase_provided(netuid.into(), alpha_taken);

        // Users‑only liquidation.
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // Expect balances restored to BEFORE snapshots (no swaps ran -> zero fees).
        let tao_after = <Test as Config>::BalanceOps::tao_balance(&cold);
        assert_eq!(tao_after, tao_before, "TAO principal must be refunded");

        // ALPHA totals conserved to owner (distribution may differ).
        let alpha_after_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_after_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_after_total = alpha_after_hot + alpha_after_owner;
        assert_eq!(
            alpha_after_total, alpha_before_total,
            "ALPHA principal must be refunded/staked for the account (check totals)"
        );

        // Clear protocol liquidity and V3 state now.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // User position(s) are gone and all V3 state cleared.
        assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
    });
}

#[test]
fn refund_alpha_single_provider_exact() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(11);
        let cold = OK_COLDKEY_ACCOUNT_ID;
        let hot = OK_HOTKEY_ACCOUNT_ID;

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // --- Create an alpha‑only position (range entirely above current tick → TAO = 0, ALPHA > 0).
        let ct = CurrentTick::<Test>::get(netuid);
        let tick_low = ct.next().expect("current tick should not be MAX in tests");
        let tick_high = TickIndex::MAX;

        let liquidity = 1_000_000_u64;
        let (_pos_id, tao_needed, alpha_needed) =
            Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
                .expect("add alpha-only liquidity");
        assert_eq!(tao_needed, 0, "alpha-only position must not require TAO");
        assert!(alpha_needed > 0, "alpha-only position must require ALPHA");

        // --- Snapshot BEFORE we withdraw funds (baseline for conservation).
        let alpha_before_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_before_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_before_total = alpha_before_hot + alpha_before_owner;

        // --- Mimic extrinsic bookkeeping: withdraw α and record provided reserve.
        let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
            &cold,
            &hot,
            netuid.into(),
            alpha_needed.into(),
        )
        .expect("decrease ALPHA");
        AlphaReserve::increase_provided(netuid.into(), alpha_taken);

        // --- Act: users‑only dissolve.
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // --- Assert: total α conserved to owner (may be staked to validator).
        let alpha_after_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_after_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_after_total = alpha_after_hot + alpha_after_owner;
        assert_eq!(
            alpha_after_total, alpha_before_total,
            "ALPHA principal must be conserved to the account"
        );

        // Clear protocol liquidity and V3 state now.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // --- State is cleared.
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
    });
}

#[test]
fn refund_alpha_multiple_providers_proportional_to_principal() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(12);
        let c1 = OK_COLDKEY_ACCOUNT_ID;
        let h1 = OK_HOTKEY_ACCOUNT_ID;
        let c2 = OK_COLDKEY_ACCOUNT_ID_2;
        let h2 = OK_HOTKEY_ACCOUNT_ID_2;

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Use the same "above current tick" trick for alpha‑only positions.
        let ct = CurrentTick::<Test>::get(netuid);
        let tick_low = ct.next().expect("current tick should not be MAX in tests");
        let tick_high = TickIndex::MAX;

        // Provider #1 (smaller α)
        let liq1 = 700_000_u64;
        let (_p1, t1, a1) =
            Pallet::<Test>::do_add_liquidity(netuid, &c1, &h1, tick_low, tick_high, liq1)
                .expect("add alpha-only liquidity #1");
        assert_eq!(t1, 0);
        assert!(a1 > 0);

        // Provider #2 (larger α)
        let liq2 = 2_100_000_u64;
        let (_p2, t2, a2) =
            Pallet::<Test>::do_add_liquidity(netuid, &c2, &h2, tick_low, tick_high, liq2)
                .expect("add alpha-only liquidity #2");
        assert_eq!(t2, 0);
        assert!(a2 > 0);

        // Baselines BEFORE withdrawing
        let a1_before_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &h1);
        let a1_before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &c1);
        let a1_before = a1_before_hot + a1_before_owner;

        let a2_before_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &h2);
        let a2_before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &c2);
        let a2_before = a2_before_hot + a2_before_owner;

        // Withdraw α and account reserves for each provider.
        let a1_taken =
            <Test as Config>::BalanceOps::decrease_stake(&c1, &h1, netuid.into(), a1.into())
                .expect("decrease α #1");
        AlphaReserve::increase_provided(netuid.into(), a1_taken);

        let a2_taken =
            <Test as Config>::BalanceOps::decrease_stake(&c2, &h2, netuid.into(), a2.into())
                .expect("decrease α #2");
        AlphaReserve::increase_provided(netuid.into(), a2_taken);

        // Act
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // Each owner is restored to their exact baseline.
        let a1_after_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &h1);
        let a1_after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c1, &c1);
        let a1_after = a1_after_hot + a1_after_owner;
        assert_eq!(
            a1_after, a1_before,
            "owner #1 must receive their α principal back"
        );

        let a2_after_hot = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &h2);
        let a2_after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &c2, &c2);
        let a2_after = a2_after_hot + a2_after_owner;
        assert_eq!(
            a2_after, a2_before,
            "owner #2 must receive their α principal back"
        );
    });
}

#[test]
fn refund_alpha_same_cold_multiple_hotkeys_conserved_to_owner() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(13);
        let cold = OK_COLDKEY_ACCOUNT_ID;
        let hot1 = OK_HOTKEY_ACCOUNT_ID;
        let hot2 = OK_HOTKEY_ACCOUNT_ID_2;

        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

        // Two alpha‑only positions on different hotkeys of the same owner.
        let ct = CurrentTick::<Test>::get(netuid);
        let tick_low = ct.next().expect("current tick should not be MAX in tests");
        let tick_high = TickIndex::MAX;

        let (_p1, _t1, a1) =
            Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot1, tick_low, tick_high, 900_000)
                .expect("add alpha-only pos (hot1)");
        let (_p2, _t2, a2) =
            Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot2, tick_low, tick_high, 1_500_000)
                .expect("add alpha-only pos (hot2)");
        assert!(a1 > 0 && a2 > 0);

        // Baseline BEFORE: sum over (cold,hot1) + (cold,hot2) + (cold,cold).
        let before_hot1 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot1);
        let before_hot2 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot2);
        let before_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let before_total = before_hot1 + before_hot2 + before_owner;

        // Withdraw α from both hotkeys; track provided‑reserve.
        let t1 =
            <Test as Config>::BalanceOps::decrease_stake(&cold, &hot1, netuid.into(), a1.into())
                .expect("decr α #hot1");
        AlphaReserve::increase_provided(netuid.into(), t1);

        let t2 =
            <Test as Config>::BalanceOps::decrease_stake(&cold, &hot2, netuid.into(), a2.into())
                .expect("decr α #hot2");
        AlphaReserve::increase_provided(netuid.into(), t2);

        // Act
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // The total α "owned" by the coldkey is conserved (credit may land on (cold,cold)).
        let after_hot1 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot1);
        let after_hot2 = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot2);
        let after_owner = <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let after_total = after_hot1 + after_hot2 + after_owner;

        assert_eq!(
            after_total, before_total,
            "owner’s α must be conserved across hot ledgers + (owner,owner)"
        );
    });
}

#[test]
fn test_dissolve_v3_green_path_refund_tao_stake_alpha_and_clear_state() {
    new_test_ext().execute_with(|| {
        // --- Setup ---
        let netuid = NetUid::from(42);
        let cold = OK_COLDKEY_ACCOUNT_ID;
        let hot = OK_HOTKEY_ACCOUNT_ID;

        assert_ok!(Swap::toggle_user_liquidity(
            RuntimeOrigin::root(),
            netuid.into(),
            true
        ));
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
        assert!(SwapV3Initialized::<Test>::get(netuid));

        // Tight in‑range band so BOTH τ and α are required.
        let ct = CurrentTick::<Test>::get(netuid);
        let tick_low = ct.saturating_sub(10);
        let tick_high = ct.saturating_add(10);
        let liquidity: u64 = 1_250_000;

        // Add liquidity and capture required τ/α.
        let (_pos_id, tao_needed, alpha_needed) =
            Pallet::<Test>::do_add_liquidity(netuid, &cold, &hot, tick_low, tick_high, liquidity)
                .expect("add in-range liquidity");
        assert!(tao_needed > 0, "in-range pos must require TAO");
        assert!(alpha_needed > 0, "in-range pos must require ALPHA");

        // Determine the permitted validator with the highest trust (green path).
        let trust = <Test as Config>::SubnetInfo::get_validator_trust(netuid.into());
        let permit = <Test as Config>::SubnetInfo::get_validator_permit(netuid.into());
        assert_eq!(trust.len(), permit.len(), "trust/permit must align");
        let target_uid: u16 = trust
            .iter()
            .zip(permit.iter())
            .enumerate()
            .filter(|(_, (_t, p))| **p)
            .max_by_key(|(_, (t, _))| *t)
            .map(|(i, _)| i as u16)
            .expect("at least one permitted validator");
        let validator_hotkey: <Test as frame_system::Config>::AccountId =
            <Test as Config>::SubnetInfo::hotkey_of_uid(netuid.into(), target_uid)
                .expect("uid -> hotkey mapping must exist");

        // --- Snapshot BEFORE we withdraw τ/α to fund the position ---
        let tao_before = <Test as Config>::BalanceOps::tao_balance(&cold);

        let alpha_before_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_before_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_before_val =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &validator_hotkey);

        let alpha_before_total = if validator_hotkey == hot {
            alpha_before_hot + alpha_before_owner
        } else {
            alpha_before_hot + alpha_before_owner + alpha_before_val
        };

        // --- Mirror extrinsic bookkeeping: withdraw τ & α; bump provided reserves ---
        let tao_taken = <Test as Config>::BalanceOps::decrease_balance(&cold, tao_needed.into())
            .expect("decrease TAO");
        let alpha_taken = <Test as Config>::BalanceOps::decrease_stake(
            &cold,
            &hot,
            netuid.into(),
            alpha_needed.into(),
        )
        .expect("decrease ALPHA");

        TaoReserve::increase_provided(netuid.into(), tao_taken);
        AlphaReserve::increase_provided(netuid.into(), alpha_taken);

        // --- Act: dissolve (GREEN PATH: permitted validators exist) ---
        assert_ok!(Pallet::<Test>::do_dissolve_all_liquidity_providers(netuid));

        // --- Assert: τ principal refunded to user ---
        let tao_after = <Test as Config>::BalanceOps::tao_balance(&cold);
        assert_eq!(tao_after, tao_before, "TAO principal must be refunded");

        // --- α ledger assertions ---
        let alpha_after_hot =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &hot);
        let alpha_after_owner =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &cold);
        let alpha_after_val =
            <Test as Config>::BalanceOps::alpha_balance(netuid.into(), &cold, &validator_hotkey);

        // Owner ledger must be unchanged in the green path.
        assert_eq!(
            alpha_after_owner, alpha_before_owner,
            "Owner α ledger must be unchanged (staked to validator, not refunded)"
        );

        if validator_hotkey == hot {
            assert_eq!(
                alpha_after_hot, alpha_before_hot,
                "When validator == hotkey, user's hot ledger must net back to its original balance"
            );
            let alpha_after_total = alpha_after_hot + alpha_after_owner;
            assert_eq!(
                alpha_after_total, alpha_before_total,
                "Total α for the coldkey must be conserved (validator==hotkey)"
            );
        } else {
            assert!(
                alpha_before_hot >= alpha_after_hot,
                "hot ledger should not increase"
            );
            assert!(
                alpha_after_val >= alpha_before_val,
                "validator ledger should not decrease"
            );

            let hot_loss = alpha_before_hot - alpha_after_hot;
            let val_gain = alpha_after_val - alpha_before_val;
            assert_eq!(
                val_gain, hot_loss,
                "α that left the user's hot ledger must equal α credited to the validator ledger"
            );

            let alpha_after_total = alpha_after_hot + alpha_after_owner + alpha_after_val;
            assert_eq!(
                alpha_after_total, alpha_before_total,
                "Total α for the coldkey must be conserved"
            );
        }

        // Now clear protocol liquidity & state and assert full reset.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        let protocol_id = Pallet::<Test>::protocol_account_id();
        assert_eq!(Pallet::<Test>::count_positions(netuid, &cold), 0);
        let prot_positions_after =
            Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        assert!(
            prot_positions_after.is_empty(),
            "protocol positions must be removed"
        );

        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(Ticks::<Test>::get(netuid, TickIndex::MIN).is_none());
        assert!(Ticks::<Test>::get(netuid, TickIndex::MAX).is_none());
        assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
        assert!(!CurrentTick::<Test>::contains_key(netuid));
        assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));

        assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
        assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none(),
            "active tick bitmap words must be cleared"
        );

        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));
    });
}

#[test]
fn test_clear_protocol_liquidity_green_path() {
    new_test_ext().execute_with(|| {
        // --- Arrange ---
        let netuid = NetUid::from(55);

        // Ensure the "user liquidity enabled" flag exists so we can verify it's removed later.
        assert_ok!(Pallet::<Test>::toggle_user_liquidity(
            RuntimeOrigin::root(),
            netuid,
            true
        ));

        // Initialize V3 state; this should set price/tick flags and create a protocol position.
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
        assert!(
            SwapV3Initialized::<Test>::get(netuid),
            "V3 must be initialized"
        );

        // Sanity: protocol positions exist before clearing.
        let protocol_id = Pallet::<Test>::protocol_account_id();
        let prot_positions_before =
            Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        assert!(
            !prot_positions_before.is_empty(),
            "protocol positions should exist after V3 init"
        );

        // --- Act ---
        // Green path: just clear protocol liquidity and wipe all V3 state.
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));

        // --- Assert: all protocol positions removed ---
        let prot_positions_after =
            Positions::<Test>::iter_prefix_values((netuid, protocol_id)).collect::<Vec<_>>();
        assert!(
            prot_positions_after.is_empty(),
            "protocol positions must be removed by do_clear_protocol_liquidity"
        );

        // --- Assert: V3 data wiped (idempotent even if some maps were empty) ---
        // Ticks / active tick bitmap
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none(),
            "active tick bitmap words must be cleared"
        );

        // Fee globals
        assert!(!FeeGlobalTao::<Test>::contains_key(netuid));
        assert!(!FeeGlobalAlpha::<Test>::contains_key(netuid));

        // Price / tick / liquidity / flags
        assert!(!AlphaSqrtPrice::<Test>::contains_key(netuid));
        assert!(!CurrentTick::<Test>::contains_key(netuid));
        assert!(!CurrentLiquidity::<Test>::contains_key(netuid));
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));

        // Knobs removed
        assert!(!FeeRate::<Test>::contains_key(netuid));
        assert!(!EnabledUserLiquidity::<Test>::contains_key(netuid));

        // --- And it's idempotent ---
        assert_ok!(Pallet::<Test>::do_clear_protocol_liquidity(netuid));
        assert!(
            Positions::<Test>::iter_prefix_values((netuid, protocol_id))
                .next()
                .is_none()
        );
        assert!(Ticks::<Test>::iter_prefix(netuid).next().is_none());
        assert!(
            TickIndexBitmapWords::<Test>::iter_prefix((netuid,))
                .next()
                .is_none()
        );
        assert!(!SwapV3Initialized::<Test>::contains_key(netuid));
    });
}

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

#[test]
fn proportional_when_price_is_one_and_tao_is_plenty() {
    // sqrt_price = 1.0  => price = 1.0
    let sqrt = U64F64::from_num(1u64);
    let amount_tao: TaoCurrency = 10u64.into();
    let amount_alpha: AlphaCurrency = 3u64.into();

    // alpha * price = 3 * 1 = 3 <= amount_tao(10)
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (3, 3, 7, 0));
}

#[test]
fn proportional_when_price_is_one_and_alpha_is_excess() {
    // sqrt_price = 1.0  => price = 1.0
    let sqrt = U64F64::from_num(1u64);
    let amount_tao: TaoCurrency = 5u64.into();
    let amount_alpha: AlphaCurrency = 10u64.into();

    // tao is limiting: alpha_equiv = floor(5 / 1) = 5
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (5, 5, 0, 5));
}

#[test]
fn proportional_with_higher_price_and_alpha_limiting() {
    // Choose sqrt_price = 2.0 => price = 4.0 (since implementation squares it)
    let sqrt = U64F64::from_num(2u64);
    let amount_tao: TaoCurrency = 85u64.into();
    let amount_alpha: AlphaCurrency = 20u64.into();

    // tao_equivalent = alpha * price = 20 * 4 = 80 < 85 => alpha limits tao
    // remainders: tao 5, alpha 0
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (80, 20, 5, 0));
}

#[test]
fn proportional_with_higher_price_and_tao_limiting() {
    // Choose sqrt_price = 2.0 => price = 4.0 (since implementation squares it)
    let sqrt = U64F64::from_num(2u64);
    let amount_tao: TaoCurrency = 50u64.into();
    let amount_alpha: AlphaCurrency = 20u64.into();

    // tao_equivalent = alpha * price = 20 * 4 = 80 > 50 => tao limits alpha
    // alpha_equivalent = floor(50 / 4) = 12
    // remainders: tao 0, alpha 20 - 12 = 8
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (50, 12, 0, 8));
}

#[test]
fn zero_price_uses_no_tao_and_all_alpha() {
    // sqrt_price = 0 => price = 0
    let sqrt = U64F64::from_num(0u64);
    let amount_tao: TaoCurrency = 42u64.into();
    let amount_alpha: AlphaCurrency = 17u64.into();

    // tao_equivalent = 17 * 0 = 0 <= 42
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (0, 17, 42, 0));
}

#[test]
fn rounding_down_behavior_when_dividing_by_price() {
    // sqrt_price = 2.0 => price = 4.0
    let sqrt = U64F64::from_num(2u64);
    let amount_tao: TaoCurrency = 13u64.into();
    let amount_alpha: AlphaCurrency = 100u64.into();

    // tao is limiting; alpha_equiv = floor(13 / 4) = 3
    // remainders: tao 0, alpha 100 - 3 = 97
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (13, 3, 0, 97));
}

#[test]
fn exact_fit_when_tao_matches_alpha_times_price() {
    // sqrt_price = 1.0 => price = 1.0
    let sqrt = U64F64::from_num(1u64);
    let amount_tao: TaoCurrency = 9u64.into();
    let amount_alpha: AlphaCurrency = 9u64.into();

    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, amount_tao, amount_alpha);
    assert_eq!(as_tuple(out), (9, 9, 0, 0));
}

#[test]
fn handles_zero_balances() {
    let sqrt = U64F64::from_num(1u64);

    // Zero TAO, some alpha
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, 0u64.into(), 7u64.into());
    // tao limits; alpha_equiv = floor(0 / 1) = 0
    assert_eq!(as_tuple(out), (0, 0, 0, 7));

    // Some TAO, zero alpha
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, 7u64.into(), 0u64.into());
    // tao_equiv = 0 * 1 = 0 <= 7
    assert_eq!(as_tuple(out), (0, 0, 7, 0));

    // Both zero
    let out =
        Pallet::<Test>::get_proportional_alpha_tao_and_remainders(sqrt, 0u64.into(), 0u64.into());
    assert_eq!(as_tuple(out), (0, 0, 0, 0));
}

#[test]
fn adjust_protocol_liquidity_uses_and_sets_scrap_reservoirs() {
    new_test_ext().execute_with(|| {
        // --- Arrange
        let netuid: NetUid = 1u16.into();
        // Price = 1.0 (since sqrt_price^2 = 1), so proportional match is 1:1
        AlphaSqrtPrice::<Test>::insert(netuid, U64F64::saturating_from_num(1u64));

        // Start with some non-zero scrap reservoirs
        ScrapReservoirTao::<Test>::insert(netuid, TaoCurrency::from(7u64));
        ScrapReservoirAlpha::<Test>::insert(netuid, AlphaCurrency::from(5u64));

        // Create a minimal protocol position so the function’s body executes.
        let protocol = Pallet::<Test>::protocol_account_id();
        let position = Position::new(
            PositionId::from(0),
            netuid,
            TickIndex::MIN,
            TickIndex::MAX,
            0,
        );
        // Ensure collect_fees() returns (0,0) via zeroed fees in `position` (default).
        Positions::<Test>::insert((netuid, protocol, position.id), position.clone());

        // --- Act
        // No external deltas or fees; only reservoirs should be considered.
        // With price=1, the exact proportional pair uses 5 alpha and 5 tao,
        // leaving tao scrap = 7 - 5 = 2, alpha scrap = 5 - 5 = 0.
        Pallet::<Test>::adjust_protocol_liquidity(netuid, 0u64.into(), 0u64.into());

        // --- Assert: reservoirs were READ (used in proportional calc) and then SET (updated)
        assert_eq!(
            ScrapReservoirTao::<Test>::get(netuid),
            TaoCurrency::from(2u64)
        );
        assert_eq!(
            ScrapReservoirAlpha::<Test>::get(netuid),
            AlphaCurrency::from(0u64)
        );
    });
}
