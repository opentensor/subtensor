#![cfg(test)]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use crate::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use subtensor_runtime_common::{AlphaCurrency, BalanceOps, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::{Order, SwapHandler};

// Run all tests here:
// cargo test --package pallet-derivatives --lib -- tests --nocapture

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
        let position_id = LastPositionId::<Test>::get();
        let position = Positions::<Test>::get(position_id).unwrap();
        assert_eq!(position.netuid, netuid);
        assert_eq!(position.owner_coldkey, COLDKEY1);
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
fn test_close_short_ok() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        TaoReserve::set_mock_reserve(netuid, 1_000_000_000.into());
        AlphaReserve::set_mock_reserve(netuid, 100_000_000_000.into());

        assert_ok!(Pallet::<Test>::close_short(
            RuntimeOrigin::signed(COLDKEY1),
            HOTKEY1,
            netuid,
            AlphaCurrency::from(1_000_000_000)
        ),);
    });
}
