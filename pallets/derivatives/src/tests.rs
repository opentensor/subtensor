#![cfg(test)]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use crate::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::{assert_noop, assert_ok};
use subtensor_runtime_common::{BalanceOps, Currency, CurrencyReserve, NetUid, TaoCurrency};
use subtensor_swap_interface::{Order, SwapHandler};

// Run all tests here:
// cargo test --package pallet-derivatives --lib -- tests --nocapture

// Test plan:
//   - Open
//     - Open normally
//     - Open with insufficient balance
//     - Open with amount below min threshold
//     - Open when a position is already open (adds)
//   - Close
//     - Close normally
//     - Open - buy - liquidate before we run out of alpha
//     - Close when there's no open position

#[test]
fn test_open_short_ok() {
    new_test_ext().execute_with(|| {
        // Setup network and balances (both ema price and price are 0.001)
        let netuid = NetUid::from(1);
        let balance_before = TaoCurrency::from(10_000_000_000);
        let position_tao = TaoCurrency::from(1_000_000_000);
        TaoReserve::set_mock_reserve(netuid, 1_000_000_000.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000.into());
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);
        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let ema_price = MockSwap::get_alpha_ema_price(netuid).to_num::<f64>();
        let price_before = MockSwap::get_current_price(netuid).to_num::<f64>();

        // Expected alpha to mint
        let collateral_ratio = Derivatives::get_collateral_ratio().to_num::<f64>();
        let expected_minted_alpha =
            (u64::from(position_tao) as f64 / (collateral_ratio * ema_price)) as u64;

        // Simulate swap to estimate tao proceeds
        let order = GetTaoForAlpha::with_amount(expected_minted_alpha);
        let expected_tao_proceeds =
            <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::sim_swap(netuid, order)
                .map(|r| r.amount_paid_out.to_u64())
                .unwrap_or_default();

        // Open short
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            position_tao
        ));

        // Check that coldkey balance decreased
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_after, balance_before - position_tao);

        // Check that correct amount of alpha was minted (AlphaOut increased in mock)
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        assert_eq!(
            alpha_out_after,
            alpha_out_before + expected_minted_alpha.into()
        );
        assert!(alpha_out_after > alpha_out_before);

        // Check that minted alpha was sold (drives price down)
        let price_after = MockSwap::get_current_price(netuid).to_num::<f64>();
        assert!(price_before > price_after);

        // Position was created
        let position = Positions::<Test>::get((COLDKEY1, netuid)).unwrap();
        assert_eq!(position.hotkey, HOTKEY1);
        assert_eq!(position.pos_type, PositionType::Short);
        // assert_eq!(position.liquidation_price, ??);
        assert_eq!(position.tao_collateral, position_tao);
        assert_abs_diff_eq!(
            position.tao_proceeds,
            expected_tao_proceeds.into(),
            epsilon = 200000.into(),
        );
        assert_eq!(position.size, expected_minted_alpha.into());

        // Make sure open event gets emitted
        assert!(System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Opened { .. })
            )
        }));
    });
}

#[test]
fn test_open_short_fails_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // Setup network and reserves
        let netuid = NetUid::from(1);
        let balance_before = TaoCurrency::from(500_000_000);
        let position_tao = TaoCurrency::from(1_000_000_000);

        TaoReserve::set_mock_reserve(netuid, 1_000_000_000.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000.into());

        // Give the caller less balance than required
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);

        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let price_before = MockSwap::get_current_price(netuid);

        // Should fail because collateral exceeds available balance
        assert_noop!(
            Pallet::<Test>::open_short(
                RuntimeOrigin::signed(COLDKEY1),
                HOTKEY1,
                netuid,
                position_tao
            ),
            Error::<Test>::InsufficientBalance
        );

        // Balance unchanged
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_after, balance_before);

        // No position created
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // No swap side effects happened
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        let price_after = MockSwap::get_current_price(netuid);
        assert_eq!(alpha_out_after, alpha_out_before);
        assert_eq!(price_after, price_before);

        // No "Opened" event emitted
        assert!(!System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Opened { .. })
            )
        }));
    });
}

#[test]
fn test_open_short_fails_with_amount_too_low() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Setup reserves so pricing logic is valid and failure is specifically min size
        TaoReserve::set_mock_reserve(netuid, 1_000_000_000.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000.into());

        // Give enough balance so we do not fail with InsufficientBalance
        let balance_before = TaoCurrency::from(10_000_000_000);
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);

        let min_position_size = <Test as crate::Config>::MinPositionSize::get();
        let tao_amount = min_position_size - TaoCurrency::from(1);

        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let price_before = MockSwap::get_current_price(netuid);

        assert_noop!(
            Pallet::<Test>::open_short(
                RuntimeOrigin::signed(COLDKEY1),
                HOTKEY1,
                netuid,
                tao_amount
            ),
            Error::<Test>::AmountTooLow
        );

        // Balance unchanged
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_after, balance_before);

        // No position created
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // No swap side effects
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        let price_after = MockSwap::get_current_price(netuid);
        assert_eq!(alpha_out_after, alpha_out_before);
        assert_eq!(price_after, price_before);

        // No Opened event emitted
        assert!(!System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Opened { .. })
            )
        }));
    });
}

#[test]
fn test_open_short_twice_increases_existing_position() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Setup network and balances
        TaoReserve::set_mock_reserve(netuid, 1_000_000_000.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000.into());

        let balance_before = TaoCurrency::from(20_000_000_000);
        let first_tao = TaoCurrency::from(1_000_000_000);
        let second_tao = TaoCurrency::from(2_000_000_000);

        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);

        let alpha_out_before = MockSwap::get_alpha_out(netuid);

        // First open
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            first_tao
        ));

        let position_after_first = Positions::<Test>::get((COLDKEY1, netuid)).unwrap();
        let balance_after_first = MockBalanceOps::tao_balance(&COLDKEY1);
        let alpha_out_after_first = MockSwap::get_alpha_out(netuid);

        assert_eq!(position_after_first.hotkey, HOTKEY1);
        assert_eq!(position_after_first.pos_type, PositionType::Short);
        assert_eq!(position_after_first.tao_collateral, first_tao);
        assert!(position_after_first.size > 0.into());
        assert!(position_after_first.tao_proceeds > 0.into());
        assert_eq!(balance_after_first, balance_before - first_tao);
        assert!(alpha_out_after_first > alpha_out_before);

        // Second open on the same position
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            second_tao
        ));

        let position_after_second = Positions::<Test>::get((COLDKEY1, netuid)).unwrap();
        let balance_after_second = MockBalanceOps::tao_balance(&COLDKEY1);
        let alpha_out_after_second = MockSwap::get_alpha_out(netuid);

        // Same position key still exists, but values increased
        assert_eq!(position_after_second.hotkey, HOTKEY1);
        assert_eq!(position_after_second.pos_type, PositionType::Short);

        // Collateral should increase by the second deposit
        assert_eq!(
            position_after_second.tao_collateral,
            position_after_first.tao_collateral + second_tao
        );

        // Position size should increase
        assert!(position_after_second.size > position_after_first.size);

        // Proceeds should increase
        assert!(position_after_second.tao_proceeds > position_after_first.tao_proceeds);

        // User balance should decrease by both opens
        assert_eq!(
            balance_after_second,
            balance_before - first_tao - second_tao
        );

        // More alpha should have been minted/sold on second open
        assert!(alpha_out_after_second > alpha_out_after_first);

        // Optional sanity check: position was updated, not reset
        assert!(position_after_second.tao_collateral > position_after_first.tao_collateral);
        assert!(position_after_second.size > position_after_first.size);
        assert!(position_after_second.tao_proceeds > position_after_first.tao_proceeds);

        // Two Opened events should be emitted
        let opened_events = System::events()
            .iter()
            .filter(|event_record| {
                matches!(
                    &event_record.event,
                    RuntimeEvent::Derivatives(Event::<Test>::Opened { .. })
                )
            })
            .count();

        assert_eq!(opened_events, 2);
    });
}

#[test]
fn test_close_short_ok() {
    new_test_ext().execute_with(|| {
        // Setup network and balances (both ema price and price are 0.001)
        let netuid = NetUid::from(1);
        let balance_before = TaoCurrency::from(10_000_000_000_u64);
        let position_tao = TaoCurrency::from(1_000_000_000_u64);
        TaoReserve::set_mock_reserve(netuid, 1_000_000_000_u64.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000_u64.into());
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);
        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let price_before = MockSwap::get_current_price(netuid).to_num::<f64>();

        // Open short
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            position_tao
        ));

        // Close the position
        assert_ok!(Pallet::<Test>::close_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
        ));

        // Coldkey balance is back to initial (because there was no price change)
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_before, balance_after);

        // All minted alpha got burned
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        assert_eq!(alpha_out_before, alpha_out_after);

        // Final price is back to where it was
        let price_after = MockSwap::get_current_price(netuid).to_num::<f64>();
        assert_eq!(price_before, price_after);        

        // Position is removed
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // Make sure close event gets emitted
        assert!(System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Closed { .. })
            )
        }));
    });
}

#[test]
fn test_close_short_profit() {
    new_test_ext().execute_with(|| {
        // Setup network and balances (both ema price and price are 0.001)
        let netuid = NetUid::from(1);
        let balance_before = TaoCurrency::from(10_000_000_000_u64);
        let position_tao = TaoCurrency::from(1_000_000_000_u64);
        TaoReserve::set_mock_reserve(netuid, 10_000_000_000_u64.into());
        AlphaReserve::set_mock_reserve(netuid, 10_000_000_000_000_u64.into());
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);
        let alpha_out_before = MockSwap::get_alpha_out(netuid);

        // Open short
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            position_tao
        ));

        // Mock-sell (move price down)
        let _ = MockSwap::sell(netuid, AlphaCurrency::from(1_000_000_000_000));

        // Close the position
        assert_ok!(Pallet::<Test>::close_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
        ));

        // Coldkey balance increased
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert!(balance_before < balance_after);

        // All minted alpha got burned
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        assert_eq!(alpha_out_before, alpha_out_after);

        // Position is removed
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // Make sure close event gets emitted
        assert!(System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Closed { .. })
            )
        }));
    });
}

#[test]
fn test_close_short_loss_alpha() {
    new_test_ext().execute_with(|| {
        // Setup network and balances (both ema price and price are 0.001)
        let netuid = NetUid::from(1);
        let balance_initial = TaoCurrency::from(10_000_000_000_u64);
        let position_tao = TaoCurrency::from(1_000_000_000_u64);
        TaoReserve::set_mock_reserve(netuid, 10_000_000_000_u64.into());
        AlphaReserve::set_mock_reserve(netuid, 10_000_000_000_000_u64.into());
        MockBalanceOps::increase_balance(&COLDKEY1, balance_initial);
        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let alpha_in_before = AlphaReserve::reserve(netuid);

        // Open short
        assert_ok!(Pallet::<Test>::open_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            position_tao
        ));

        // Mock-buy (move price up so that position loses all tao and gets some alpha from pool to close)
        let buy_swap_result = MockSwap::buy(netuid, TaoCurrency::from(100_000_000_000)).unwrap();

        // Close the position
        let balance_before = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_ok!(Pallet::<Test>::close_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
        ));

        // Coldkey balance did not increase (total loss of collateral)
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_before, balance_after);

        // All minted alpha got burned
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        assert_eq!(alpha_out_before, alpha_out_after);

        // Alpha reserve is decreased by exactly buy_swap_result 
        // (no extra alpha remains in the pool and no missing alpha)
        let alpha_in_after = AlphaReserve::reserve(netuid);
        assert_eq!(alpha_in_after + buy_swap_result, alpha_in_before);

        // Position is removed
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // Close event gets emitted
        assert!(System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Closed { .. })
            )
        }));
    });
}

#[test]
fn test_close_short_fails_with_no_open_position() {
    new_test_ext().execute_with(|| {
        // Setup network and balances
        let netuid = NetUid::from(1);
        let balance_before = TaoCurrency::from(10_000_000_000_u64);

        TaoReserve::set_mock_reserve(netuid, 1_000_000_000_u64.into());
        AlphaReserve::set_mock_reserve(netuid, 1_000_000_000_000_u64.into());
        MockBalanceOps::increase_balance(&COLDKEY1, balance_before);

        let alpha_out_before = MockSwap::get_alpha_out(netuid);
        let price_before = MockSwap::get_current_price(netuid).to_num::<f64>();

        // No position was opened, so close should fail
        assert_noop!(
            Pallet::<Test>::close_short(
                RuntimeOrigin::signed(COLDKEY1),
                HOTKEY1,
                netuid,
            ),
            Error::<Test>::NoOpenPosition
        );

        // Balance unchanged
        let balance_after = MockBalanceOps::tao_balance(&COLDKEY1);
        assert_eq!(balance_before, balance_after);

        // No alpha burned / no swap side effects
        let alpha_out_after = MockSwap::get_alpha_out(netuid);
        assert_eq!(alpha_out_before, alpha_out_after);

        // Price unchanged
        let price_after = MockSwap::get_current_price(netuid).to_num::<f64>();
        assert_eq!(price_before, price_after);

        // Still no position
        assert!(Positions::<Test>::get((COLDKEY1, netuid)).is_none());

        // No Closed event emitted
        assert!(!System::events().iter().any(|event_record| {
            matches!(
                &event_record.event,
                RuntimeEvent::Derivatives(Event::<Test>::Closed { .. })
            )
        }));
    });
}
