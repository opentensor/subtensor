#![allow(clippy::indexing_slicing)]
//! Integration tests for `pallet-limit-orders` extrinsics.
//!
//! Tests go through the full dispatch path: origin enforcement, storage changes,
//! and event emission are all verified. SwapInterface calls are handled by
//! `MockSwap`, which records calls and maintains in-memory balance ledgers.

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{DispatchError, Perbill};
use subtensor_runtime_common::NetUid;

use crate::{
    Error, Order, OrderSide, OrderStatus, OrderType, Orders, VersionedOrder, pallet::Event,
};

type LimitOrders = crate::pallet::Pallet<Test>;

use super::mock::*;

/// Check that a specific pallet event was emitted.
fn assert_event(event: Event<Test>) {
    assert!(
        System::events()
            .iter()
            .any(|r| r.event == RuntimeEvent::LimitOrders(event.clone())),
        "expected event not found: {event:?}",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// cancel_order
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cancel_order_signer_can_cancel() {
    new_test_ext().execute_with(|| {
        let order = VersionedOrder::V1(Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None,
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: false,
        });
        let id = order_id(&order);

        assert_ok!(LimitOrders::cancel_order(
            RuntimeOrigin::signed(alice()),
            order
        ));
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Cancelled));
        assert_event(Event::OrderCancelled {
            order_id: id,
            signer: alice(),
        });
    });
}

#[test]
fn cancel_order_non_signer_rejected() {
    new_test_ext().execute_with(|| {
        let order = VersionedOrder::V1(Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None,
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: false,
        });
        // Bob tries to cancel Alice's order.
        assert_noop!(
            LimitOrders::cancel_order(RuntimeOrigin::signed(bob()), order),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn cancel_order_already_cancelled_rejected() {
    new_test_ext().execute_with(|| {
        let order = VersionedOrder::V1(Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None,
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: false,
        });
        let id = order_id(&order);
        Orders::<Test>::insert(id, OrderStatus::Cancelled);

        assert_noop!(
            LimitOrders::cancel_order(RuntimeOrigin::signed(alice()), order),
            Error::<Test>::OrderAlreadyProcessed
        );
    });
}

#[test]
fn cancel_order_already_fulfilled_rejected() {
    new_test_ext().execute_with(|| {
        let order = VersionedOrder::V1(Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None,
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: false,
        });
        let id = order_id(&order);
        Orders::<Test>::insert(id, OrderStatus::Fulfilled);

        assert_noop!(
            LimitOrders::cancel_order(RuntimeOrigin::signed(alice()), order),
            Error::<Test>::OrderAlreadyProcessed
        );
    });
}

#[test]
fn cancel_order_unsigned_rejected() {
    new_test_ext().execute_with(|| {
        let order = VersionedOrder::V1(Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None,
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: false,
        });
        assert_noop!(
            LimitOrders::cancel_order(RuntimeOrigin::none(), order),
            DispatchError::BadOrigin
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_orders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_orders_buy_order_fulfilled() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        // Price = 1.0 ≤ limit = 2.0 → condition met.
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            2_000_000_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount_in: 1_000,
            amount_out: 0,
        });
    });
}

#[test]
fn execute_orders_sell_order_fulfilled() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0);
        // Price = 2.0 ≥ limit = 1 → condition met.
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            500,
            1,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            order_type: OrderType::TakeProfit,
            amount_in: 500,
            amount_out: 0,
        });
    });
}

#[test]
fn execute_orders_stop_loss_order_fulfilled() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(0.5);
        // Price = 0.5 ≤ limit = 1.0 → condition met.
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::StopLoss,
            500,
            1, // raw limit_price = 1 TAO/alpha
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            order_type: OrderType::StopLoss,
            amount_in: 500,
            amount_out: 0,
        });
    });
}

#[test]
fn execute_orders_stop_loss_price_not_met_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0); // price 2.0 > limit 1.0 → stop loss condition not met
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::StopLoss,
            500,
            1, // raw limit_price = 1 TAO/alpha
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert!(Orders::<Test>::get(id).is_none());
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::PriceConditionNotMet.into(),
        });
    });
}

#[test]
fn execute_orders_expired_order_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(2_000_001); // now > expiry
        MockSwap::set_price(1.0);
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            2_000_000, // expiry in the past
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        // Skipped — storage untouched.
        assert!(Orders::<Test>::get(id).is_none());
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::OrderExpired.into(),
        });
    });
}

#[test]
fn execute_orders_price_not_met_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(5.0); // price 5.0 > limit 2 → buy condition not met
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            2,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert!(Orders::<Test>::get(id).is_none());
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::PriceConditionNotMet.into(),
        });
    });
}

#[test]
fn execute_orders_already_processed_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);
        Orders::<Test>::insert(id, OrderStatus::Fulfilled);

        // Should succeed (batch-level) but skip this order silently.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));
        // Still Fulfilled (not changed).
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::OrderAlreadyProcessed.into(),
        });
    });
}

#[test]
fn execute_orders_mixed_batch_valid_and_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        let valid = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let expired = make_signed_order(
            AccountKeyring::Bob,
            alice(),
            netuid(),
            OrderType::LimitBuy,
            500,
            u64::MAX,
            500_000, // already expired
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let valid_id = order_id(&valid.order);
        let expired_id = order_id(&expired.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![valid, expired]),
        ));

        assert_eq!(Orders::<Test>::get(valid_id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderSkipped {
            order_id: expired_id,
            reason: Error::<Test>::OrderExpired.into(),
        });
    });
}

#[test]
fn execute_orders_unsigned_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LimitOrders::execute_orders(RuntimeOrigin::none(), bounded(vec![])),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn execute_orders_buy_with_fee_charges_fee() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        // fee_rate = 1% (10_000_000 parts-per-billion), recipient = fee_recipient().
        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );
        MockSwap::set_tao_balance(alice(), 1_000);
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        // One buy_alpha call for the net amount (990 TAO after 1% fee).
        let buys: Vec<_> = MockSwap::log()
            .into_iter()
            .filter_map(|c| {
                if let super::mock::SwapCall::BuyAlpha { tao, .. } = c {
                    Some(tao)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(buys, vec![990], "main swap must use 990 TAO after 1% fee");

        // Fee (10 TAO) forwarded directly to fee_recipient via transfer_tao.
        assert_eq!(MockSwap::tao_balance(&fee_recipient()), 10);
    });
}

#[test]
fn execute_orders_sell_with_fee_charges_fee() {
    new_test_ext().execute_with(|| {
        // fee = 1% (10_000_000 ppb).
        // Alice sells 1_000 alpha; pool returns 800 TAO.
        // fee_tao = 1% of 800 = 8 TAO, forwarded to fee_recipient via transfer_tao.
        // Alice keeps 792 TAO.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_sell_tao_return(800);
        MockSwap::set_alpha_balance(alice(), bob(), netuid(), 1_000);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        // Full 1_000 alpha sold (no alpha deducted for fee).
        let sells: Vec<_> = MockSwap::log()
            .into_iter()
            .filter_map(|c| {
                if let super::mock::SwapCall::SellAlpha { alpha, .. } = c {
                    Some(alpha)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(sells, vec![1_000], "full alpha amount must be sold");

        // fee_recipient received 8 TAO (1% of 800).
        assert_eq!(MockSwap::tao_balance(&fee_recipient()), 8);
        // Alice kept the remaining 792 TAO.
        assert_eq!(MockSwap::tao_balance(&alice()), 792);
    });
}

#[test]
fn execute_orders_empty_batch_returns_ok() {
    new_test_ext().execute_with(|| {
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![])
        ));
    });
}

#[test]
fn execute_orders_fee_transfer_failure_emits_event() {
    new_test_ext().execute_with(|| {
        // Order executes successfully, but the fee transfer to the recipient fails.
        // The order should still be marked Fulfilled and FeeTransferFailed emitted.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);
        MockSwap::set_tao_balance(alice(), 10_000);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );

        FAIL_FEE_TRANSFER.with(|f| *f.borrow_mut() = true);
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed.clone()])
        ));
        FAIL_FEE_TRANSFER.with(|f| *f.borrow_mut() = false);

        // Order was executed despite the failed fee transfer.
        let id = crate::tests::mock::order_id(&signed.order);
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));

        // FeeTransferFailed was emitted with the correct recipient and error.
        assert_event(Event::FeeTransferFailed {
            recipient: fee_recipient(),
            amount: 10, // 1% of 1_000
            reason: DispatchError::CannotLookup,
        });

        // fee_recipient received nothing.
        assert_eq!(MockSwap::tao_balance(&fee_recipient()), 0);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_orders — silent-skip behaviour
// ─────────────────────────────────────────────────────────────────────────────

mod execute_orders_skip_invalid {
    use super::*;

    /// A single expired order is silently skipped: the call returns `Ok` and
    /// nothing is written to the `Orders` storage map.
    #[test]
    fn execute_orders_skips_expired_order() {
        new_test_ext().execute_with(|| {
            MockTime::set(2_000_001); // now > expiry
            MockSwap::set_price(1.0);

            let signed = make_signed_order(
                AccountKeyring::Alice,
                bob(),
                netuid(),
                OrderType::LimitBuy,
                1_000,
                u64::MAX,
                2_000_000, // expiry in the past
                Perbill::zero(),
                fee_recipient(),
                None,
            );
            let id = order_id(&signed.order);

            assert_ok!(LimitOrders::execute_orders(
                RuntimeOrigin::signed(charlie()),
                bounded(vec![signed])
            ));

            // Skipped — storage untouched.
            assert!(Orders::<Test>::get(id).is_none());
            assert_event(Event::OrderSkipped {
                order_id: id,
                reason: Error::<Test>::OrderExpired.into(),
            });
        });
    }

    /// A LimitBuy with `limit_price = 0` (price ceiling below current price)
    /// is silently skipped: the call returns `Ok` and nothing is written to
    /// the `Orders` storage map.
    #[test]
    fn execute_orders_skips_price_condition_not_met() {
        new_test_ext().execute_with(|| {
            MockTime::set(1_000_000);
            MockSwap::set_price(5.0); // price 5.0 > limit 0 → buy condition not met

            let signed = make_signed_order(
                AccountKeyring::Alice,
                bob(),
                netuid(),
                OrderType::LimitBuy,
                1_000,
                0, // price ceiling of 0 — never satisfied at price 5.0
                FAR_FUTURE,
                Perbill::zero(),
                fee_recipient(),
                None,
            );
            let id = order_id(&signed.order);

            assert_ok!(LimitOrders::execute_orders(
                RuntimeOrigin::signed(charlie()),
                bounded(vec![signed])
            ));

            // Skipped — storage untouched.
            assert!(Orders::<Test>::get(id).is_none());
            assert_event(Event::OrderSkipped {
                order_id: id,
                reason: Error::<Test>::PriceConditionNotMet.into(),
            });
        });
    }

    /// A batch containing one valid order and one expired order: the call
    /// returns `Ok`, the valid order is stored as `Fulfilled`, and the expired
    /// order is NOT written to storage.
    #[test]
    fn execute_orders_valid_and_invalid_mixed() {
        new_test_ext().execute_with(|| {
            MockTime::set(1_000_000);
            MockSwap::set_price(1.0);

            let valid = make_signed_order(
                AccountKeyring::Alice,
                bob(),
                netuid(),
                OrderType::LimitBuy,
                1_000,
                u64::MAX,
                FAR_FUTURE,
                Perbill::zero(),
                fee_recipient(),
                None,
            );
            let expired = make_signed_order(
                AccountKeyring::Bob,
                alice(),
                netuid(),
                OrderType::LimitBuy,
                500,
                u64::MAX,
                500_000, // already expired
                Perbill::zero(),
                fee_recipient(),
                None,
            );
            let valid_id = order_id(&valid.order);
            let expired_id = order_id(&expired.order);

            assert_ok!(LimitOrders::execute_orders(
                RuntimeOrigin::signed(charlie()),
                bounded(vec![valid, expired]),
            ));

            // Valid order executed successfully.
            assert_eq!(Orders::<Test>::get(valid_id), Some(OrderStatus::Fulfilled));
            // Expired order silently skipped — not written to storage.
            assert!(Orders::<Test>::get(expired_id).is_none());
            assert_event(Event::OrderSkipped {
                order_id: expired_id,
                reason: Error::<Test>::OrderExpired.into(),
            });
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_batched_orders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_batched_orders_unsigned_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LimitOrders::execute_batched_orders(RuntimeOrigin::none(), netuid(), bounded(vec![])),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn execute_batched_orders_all_invalid_fails() {
    new_test_ext().execute_with(|| {
        // An expired order causes the whole batch to fail.
        MockTime::set(2_000_001); // all expired
        let expired = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            1_000_000,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![expired]),
            ),
            Error::<Test>::OrderExpired
        );
    });
}

#[test]
fn execute_batched_orders_fails_for_wrong_netuid() {
    new_test_ext().execute_with(|| {
        // An order whose netuid does not match the batch netuid must cause the batch to fail.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(100);

        let wrong_net = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            NetUid::from(99u16), // wrong netuid
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(), // batch targets netuid 1
                bounded(vec![wrong_net]),
            ),
            Error::<Test>::OrderNetUidMismatch
        );
    });
}

#[test]
fn execute_batched_orders_price_condition_not_met_fails_entire_batch() {
    new_test_ext().execute_with(|| {
        // Price condition not met is a hard-fail in execute_batched_orders —
        // unlike execute_orders where it silently skips the order.
        MockTime::set(1_000_000);
        MockSwap::set_price(100.0); // current price = 100

        // LimitBuy requires current_price <= limit_price; with limit_price=1 this fails.
        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            1, // limit_price = 1, far below current price of 100
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![order])
            ),
            Error::<Test>::PriceConditionNotMet
        );
    });
}

#[test]
fn execute_batched_orders_buy_only_fulfills_orders_and_distributes_alpha() {
    new_test_ext().execute_with(|| {
        // Setup:
        //   Alice buys 600 TAO, Bob buys 400 TAO (total 1000 TAO net, fee=0).
        //   Pool returns 500 alpha (MOCK_BUY_ALPHA_RETURN).
        //   No sellers → total_alpha = 500.
        //   Pro-rata: Alice 500*600/1000=300, Bob 500*400/1000=200.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);
        MockSwap::set_tao_balance(alice(), 600);
        MockSwap::set_tao_balance(bob(), 400);

        let alice_order = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            600,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let bob_order = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            400,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let alice_id = order_id(&alice_order.order);
        let bob_id = order_id(&bob_order.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order]),
        ));

        // Both orders fulfilled.
        assert_eq!(Orders::<Test>::get(alice_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(bob_id), Some(OrderStatus::Fulfilled));

        // Alpha distributed pro-rata.
        assert_eq!(MockSwap::alpha_balance(&alice(), &dave(), netuid()), 300);
        assert_eq!(MockSwap::alpha_balance(&bob(), &dave(), netuid()), 200);

        // Summary event.
        assert_event(Event::GroupExecutionSummary {
            netuid: netuid(),
            net_side: OrderSide::Buy,
            net_amount: 1_000,
            actual_out: 500,
            executed_count: 2,
        });
    });
}

#[test]
fn execute_batched_orders_sell_only_fulfills_orders_and_distributes_tao() {
    new_test_ext().execute_with(|| {
        // Setup:
        //   Alice sells 300 alpha, Bob sells 200 alpha (total 500 alpha, fee=0).
        //   Price = 2.0 → sell_tao_equiv: Alice 600, Bob 400, total 1000.
        //   Pool returns 800 TAO (MOCK_SELL_TAO_RETURN) for the net 500 alpha.
        //   No buyers → total_tao = 800 + 0 = 800.
        //   Pro-rata: Alice 800*600/1000=480, Bob 800*400/1000=320.
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0);
        MockSwap::set_sell_tao_return(800);
        MockSwap::set_alpha_balance(alice(), dave(), netuid(), 300);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 200);

        let alice_order = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            300,
            0,
            FAR_FUTURE, // limit=0 → accept any price
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let bob_order = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let alice_id = order_id(&alice_order.order);
        let bob_id = order_id(&bob_order.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order]),
        ));

        assert_eq!(Orders::<Test>::get(alice_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(bob_id), Some(OrderStatus::Fulfilled));

        // TAO distributed pro-rata.
        assert_eq!(MockSwap::tao_balance(&alice()), 480);
        assert_eq!(MockSwap::tao_balance(&bob()), 320);

        assert_event(Event::GroupExecutionSummary {
            netuid: netuid(),
            net_side: OrderSide::Sell,
            net_amount: 500,
            actual_out: 800,
            executed_count: 2,
        });
    });
}

#[test]
fn execute_batched_orders_buy_dominant_mixed() {
    new_test_ext().execute_with(|| {
        // Setup (fee=0, price=2.0 TAO/alpha):
        //   Buyers: Alice 1000 TAO, Bob 600 TAO → total_buy_net = 1600.
        //   Sellers: Charlie 200 alpha → sell_tao_equiv = 400 TAO.
        //   Net (buy-dominant): 1600 - 400 = 1200 TAO goes to pool.
        //   Pool returns 300 alpha (MOCK_BUY_ALPHA_RETURN).
        //   total_alpha for buyers = 300 (pool) + 200 (seller passthrough) = 500.
        //   Pro-rata buyers (by buy_net TAO):
        //     Alice:  500 * 1000/1600 = 312 alpha
        //     Bob:    500 *  600/1600 = 187 alpha
        //     (dust = 1 alpha stays in pallet)
        //   Sellers (buy-dominant branch): total_tao = total_sell_tao_equiv = 400.
        //     Charlie: 400 * 400/400 = 400 TAO.
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0);
        MockSwap::set_buy_alpha_return(300);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_tao_balance(bob(), 600);
        MockSwap::set_alpha_balance(charlie(), dave(), netuid(), 200);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let bob_buy = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            600,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let charlie_sell = make_signed_order(
            AccountKeyring::Charlie,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(dave()),
            netuid(),
            bounded(vec![alice_buy, bob_buy, charlie_sell]),
        ));

        assert_eq!(MockSwap::alpha_balance(&alice(), &dave(), netuid()), 312);
        assert_eq!(MockSwap::alpha_balance(&bob(), &dave(), netuid()), 187);
        assert_eq!(MockSwap::tao_balance(&charlie()), 400);

        assert_event(Event::GroupExecutionSummary {
            netuid: netuid(),
            net_side: OrderSide::Buy,
            net_amount: 1_200,
            actual_out: 300,
            executed_count: 3,
        });
    });
}

#[test]
fn execute_batched_orders_sell_dominant_mixed() {
    new_test_ext().execute_with(|| {
        // Setup (fee=0, price=2.0 TAO/alpha):
        //   Buyers: Alice 200 TAO → total_buy_net = 200.
        //   Sellers: Bob 300 alpha, Charlie 200 alpha → total_sell_net = 500.
        //     sell_tao_equiv: Bob 600, Charlie 400, total 1000.
        //   Net (sell-dominant): buy_alpha_equiv = 200/2 = 100 alpha;
        //     residual sell alpha = 500 - 100 = 400 alpha → pool returns 300 TAO.
        //   total_tao for sellers = 300 (pool) + 200 (buy passthrough) = 500 TAO.
        //   Pro-rata sellers (by sell_tao_equiv):
        //     Bob:     500 * 600/1000 = 300 TAO
        //     Charlie: 500 * 400/1000 = 200 TAO
        //   total_alpha for buyers = buy_net / price = 200/2 = 100 alpha.
        //   Alice: 100 * 200/200 = 100 alpha.
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0);
        MockSwap::set_sell_tao_return(300);
        MockSwap::set_tao_balance(alice(), 200);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 300);
        MockSwap::set_alpha_balance(charlie(), dave(), netuid(), 200);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            200,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let bob_sell = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            300,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let charlie_sell = make_signed_order(
            AccountKeyring::Charlie,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(dave()),
            netuid(),
            bounded(vec![alice_buy, bob_sell, charlie_sell]),
        ));

        assert_eq!(MockSwap::alpha_balance(&alice(), &dave(), netuid()), 100);
        assert_eq!(MockSwap::tao_balance(&bob()), 300);
        assert_eq!(MockSwap::tao_balance(&charlie()), 200);

        assert_event(Event::GroupExecutionSummary {
            netuid: netuid(),
            net_side: OrderSide::Sell,
            net_amount: 400,
            actual_out: 300,
            executed_count: 3,
        });
    });
}

#[test]
fn execute_batched_orders_fee_forwarded_to_collector() {
    new_test_ext().execute_with(|| {
        // fee = 1% (10_000_000 ppb).
        // Alice buys 1000 TAO: fee = 10, net = 990.
        // Pool returns 500 alpha for 990 TAO.
        // collect_fees transfers 10 TAO (buy fee) to fee_recipient.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_buy]),
        ));

        // Fee recipient received the buy-side fee.
        assert_eq!(MockSwap::tao_balance(&fee_recipient()), 10);
    });
}

#[test]
fn execute_batched_orders_fails_for_cancelled_order() {
    new_test_ext().execute_with(|| {
        // A cancelled order is already processed; including it in the batch must cause a hard failure.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(100);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let id = order_id(&signed.order);
        Orders::<Test>::insert(id, OrderStatus::Cancelled);

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![signed]),
            ),
            Error::<Test>::OrderCancelled
        );

        // Still cancelled, not changed to Fulfilled.
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Cancelled));
    });
}

#[test]
fn execute_batched_orders_fees_charged_on_both_sides_when_matched_internally() {
    new_test_ext().execute_with(|| {
        // fee = 1% (10_000_000 ppb), price = 1.0 TAO/alpha.
        //
        // Alice buys  1_000 TAO  → buy fee = 10 TAO, net = 990 TAO.
        // Bob   sells 1_000 alpha → sell_tao_equiv = 1_000 TAO.
        //
        // sell-dominant: residual = 1_000 - 990 = 10 alpha sent to pool.
        // Pool returns 9 TAO (mocked) for that residual.
        // total_tao for sellers = 9 (pool) + 990 (buy passthrough) = 999.
        // Bob gross_share = 999 * 1_000/1_000 = 999.
        // Sell fee = 1% of 999 = 9.99 → rounds to 10 TAO; Bob nets 989 TAO.
        // fee_recipient total = buy_fee(10) + sell_fee(10) = 20 TAO.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_sell_tao_return(9);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 1_000);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );
        let bob_sell = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_buy, bob_sell]),
        ));

        // Both sides charged: fee_recipient gets buy fee (10) + sell fee (10) = 20.
        assert_eq!(MockSwap::tao_balance(&fee_recipient()), 20);
        // Bob receives 989 TAO after sell-side fee.
        assert_eq!(MockSwap::tao_balance(&bob()), 989);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// net_pool_swap – SwapReturnedZero errors
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_batched_orders_buy_zero_alpha_returns_error() {
    new_test_ext().execute_with(|| {
        // buy_alpha returns 0 alpha for a non-zero TAO input → SwapReturnedZero.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(0); // pool gives back nothing
        MockSwap::set_tao_balance(alice(), 1_000);

        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![order]),
            ),
            Error::<Test>::SwapReturnedZero
        );
    });
}

#[test]
fn execute_batched_orders_sell_zero_tao_returns_error() {
    new_test_ext().execute_with(|| {
        // sell_alpha returns 0 TAO for a non-zero alpha input → SwapReturnedZero.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_sell_tao_return(0); // pool gives back nothing
        MockSwap::set_alpha_balance(alice(), bob(), netuid(), 1_000);

        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![order]),
            ),
            Error::<Test>::SwapReturnedZero
        );
    });
}

#[test]
fn execute_batched_orders_sell_alpha_respects_swap_fail() {
    new_test_ext().execute_with(|| {
        // sell_alpha should propagate DispatchError when MOCK_SWAP_FAIL is set.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_swap_fail(true);
        MockSwap::set_alpha_balance(alice(), bob(), netuid(), 1_000);

        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![order]),
            ),
            DispatchError::Other("pool error")
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// fee routing – multiple recipients
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_batched_orders_fees_routed_to_different_recipients() {
    new_test_ext().execute_with(|| {
        // Alice and Bob both buy; Alice's fee goes to charlie(), Bob's to dave().
        // fee = 1% for both orders.
        // Alice buys 1_000 TAO: fee = 10 → charlie().
        // Bob   buys 1_000 TAO: fee = 10 → dave().
        // Pool returns 900 alpha total for 1_980 TAO net.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(900);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_tao_balance(bob(), 1_000);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            charlie(),
            None,
        );
        let bob_buy = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            dave(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_buy, bob_buy]),
        ));

        // Each recipient gets exactly their order's fee.
        assert_eq!(
            MockSwap::tao_balance(&charlie()),
            10,
            "charlie gets Alice's fee"
        );
        assert_eq!(MockSwap::tao_balance(&dave()), 10, "dave gets Bob's fee");
    });
}

#[test]
fn execute_batched_orders_fees_batched_for_shared_recipient() {
    new_test_ext().execute_with(|| {
        // Both Alice and Bob's fees go to the same recipient (charlie()).
        // Expect a single combined transfer of 20 TAO to charlie().
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(900);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_tao_balance(bob(), 1_000);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            charlie(),
            None,
        );
        let bob_buy = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            charlie(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_buy, bob_buy]),
        ));

        // One combined transfer: charlie() receives 10 + 10 = 20 TAO.
        let fee_transfers: Vec<_> = MockSwap::tao_transfers()
            .into_iter()
            .filter(|(_, to, _)| to == &charlie())
            .collect();
        assert_eq!(
            fee_transfers.len(),
            1,
            "single transfer to shared recipient"
        );
        assert_eq!(fee_transfers[0].2, 20, "combined fee = 20 TAO");
    });
}

/// 4 orders split across 2 fee recipients.
///
/// Orders:
///   Alice  LimitBuy    1_000 TAO   fee_recipient = ferdie (buy-fee collector)
///   Bob    LimitBuy    1_000 TAO   fee_recipient = ferdie (buy-fee collector)
///   Charlie TakeProfit 1_000 α    fee_recipient = fee_recipient() (sell-fee collector)
///   Eve    TakeProfit  1_000 α    fee_recipient = fee_recipient() (sell-fee collector)
///
/// Neither ferdie nor fee_recipient() are order signers, so every TAO transfer
/// to those accounts is exclusively a fee transfer — making the single-transfer
/// assertion unambiguous.
///
/// At price 1.0 (1 TAO = 1 α), fee = 1%:
///   net buy TAO  = (1_000 - 10) + (1_000 - 10) = 1_980
///   sell α equiv = 2_000 TAO  →  sell-dominant, residual = 20 α → pool
///   pool returns 18 TAO for residual
///   total TAO for sellers = 18 + 1_980 = 1_998
///   each seller gross_share = 1_998 * 1_000 / 2_000 = 999
///   sell fee = 1% * 999 = 10 TAO each
///
/// Expected:
///   ferdie          receives 10 (Alice) + 10 (Bob)     = 20 TAO (1 transfer)
///   fee_recipient() receives 10 (Charlie) + 10 (Eve)   = 20 TAO (1 transfer)
#[test]
fn execute_batched_orders_four_orders_two_fee_recipients() {
    new_test_ext().execute_with(|| {
        let ferdie = AccountKeyring::Ferdie.to_account_id();
        let eve = AccountKeyring::Eve.to_account_id();

        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_sell_tao_return(18);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_tao_balance(bob(), 1_000);
        MockSwap::set_alpha_balance(charlie(), dave(), netuid(), 1_000);
        MockSwap::set_alpha_balance(eve.clone(), dave(), netuid(), 1_000);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            ferdie.clone(),
            None,
        );
        let bob_buy = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            ferdie.clone(),
            None,
        );
        let charlie_sell = make_signed_order(
            AccountKeyring::Charlie,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );
        let eve_sell = make_signed_order(
            AccountKeyring::Eve,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            1_000,
            0,
            FAR_FUTURE,
            Perbill::from_parts(10_000_000), // 1%
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(alice()),
            netuid(),
            bounded(vec![alice_buy, bob_buy, charlie_sell, eve_sell]),
        ));

        // ferdie collects Alice's and Bob's buy fees: 10 + 10 = 20 TAO in one transfer.
        let ferdie_transfers: Vec<_> = MockSwap::tao_transfers()
            .into_iter()
            .filter(|(_, to, _)| to == &ferdie)
            .collect();
        assert_eq!(ferdie_transfers.len(), 1, "single transfer to ferdie");
        assert_eq!(
            ferdie_transfers[0].2, 20,
            "ferdie receives 20 TAO in buy fees"
        );

        // fee_recipient() collects Charlie's and Eve's sell fees: 10 + 10 = 20 TAO in one transfer.
        let fp_transfers: Vec<_> = MockSwap::tao_transfers()
            .into_iter()
            .filter(|(_, to, _)| to == &fee_recipient())
            .collect();
        assert_eq!(fp_transfers.len(), 1, "single transfer to fee_recipient");
        assert_eq!(
            fp_transfers[0].2, 20,
            "fee_recipient receives 20 TAO in sell fees"
        );
    });
}

/// A mixed batch (buy + sell) must not rate-limit the pallet intermediary
/// account during asset collection, which would otherwise block the
/// subsequent alpha distribution to buyers.
///
/// Regression test: previously `transfer_staked_alpha` with a single
/// `apply_limits: true` flag set the rate-limit on `to_coldkey` (pallet)
/// during collection, then the distribution step checked `from_coldkey`
/// (pallet) and failed with `StakingOperationRateLimitExceeded`.
#[test]
fn execute_batched_orders_mixed_batch_does_not_rate_limit_pallet_intermediary() {
    new_test_ext().execute_with(|| {
        // Alice buys 1_000 TAO; Bob sells 500 alpha.
        // Buy-dominant: residual 500 TAO goes to pool, pool returns 400 alpha.
        // Total alpha = 400 (pool) + 500 (Bob passthrough) = 900 → all to Alice.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(400);
        MockSwap::set_tao_balance(alice(), 1_000);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 500);

        let buy = make_signed_order(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );
        let sell = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            500,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        // Must succeed: collecting Bob's alpha must not rate-limit the pallet
        // intermediary, so distributing alpha to Alice is not blocked.
        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![buy, sell]),
        ));

        // Alice received staked alpha.
        assert!(
            MockSwap::alpha_balance(&alice(), &dave(), netuid()) > 0,
            "alice should hold staked alpha after the buy"
        );
        // Alice is rate-limited after receiving stake (set_receiver_limit=true).
        assert!(
            MockSwap::is_rate_limited(&dave(), &alice(), netuid()),
            "alice should be rate-limited after receiving stake"
        );
        // Bob's hotkey on the pallet side is NOT rate-limited (set_receiver_limit=false on collect).
        assert!(
            !MockSwap::is_rate_limited(&dave(), &bob(), netuid()),
            "bob's rate-limit should not be set by the collection step"
        );
    });
}

/// Root changes the pallet status, extrinsics are filtered
#[test]
fn root_disables_and_extrinsics_are_filtered() {
    new_test_ext().execute_with(|| {
        // Disable the pallet
        assert_ok!(LimitOrders::set_pallet_status(RuntimeOrigin::root(), false));

        let sell = make_signed_order(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            500,
            0,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        // Must succeed: collecting Bob's alpha must not rate-limit the pallet
        // intermediary, so distributing alpha to Alice is not blocked.
        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![sell])
            ),
            Error::<Test>::LimitOrdersDisabled
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// max_slippage — execute_orders passes effective_swap_limit to pool
// ─────────────────────────────────────────────────────────────────────────────

/// Build a signed order with a specific `max_slippage` value.
fn make_signed_order_with_slippage(
    keyring: AccountKeyring,
    hotkey: AccountId,
    netuid: subtensor_runtime_common::NetUid,
    order_type: OrderType,
    amount: u64,
    limit_price: u64,
    expiry: u64,
    fee_rate: sp_runtime::Perbill,
    fee_recipient: AccountId,
    max_slippage: Option<sp_runtime::Perbill>,
) -> crate::SignedOrder<AccountId> {
    let order = crate::VersionedOrder::V1(crate::Order {
        signer: keyring.to_account_id(),
        hotkey,
        netuid,
        order_type,
        amount,
        limit_price,
        expiry,
        fee_rate,
        fee_recipient,
        relayer: None,
        max_slippage,
        chain_id: 945,
        partial_fills_enabled: false,
    });
    let sig = keyring.pair().sign(&order.encode());
    crate::SignedOrder {
        order,
        signature: sp_runtime::MultiSignature::Sr25519(sig),
        partial_fill: None,
    }
}

#[test]
fn execute_orders_buy_no_slippage_passes_u64_max_to_pool() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        let signed = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None, // no slippage → u64::MAX ceiling
        );

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        // Pool must have been called with u64::MAX as price ceiling.
        assert_eq!(MockSwap::buy_alpha_limit_prices(), vec![u64::MAX]);
    });
}

#[test]
fn execute_orders_sell_no_slippage_passes_zero_to_pool() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(2.0);

        let signed = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            500,
            1,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None, // no slippage → 0 floor
        );

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(MockSwap::sell_alpha_limit_prices(), vec![0]);
    });
}

#[test]
fn execute_orders_buy_one_percent_slippage_passes_ceiling_to_pool() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        // limit_price=1000, 1% slippage → ceiling = 1010.
        let signed = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)),
        );

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(MockSwap::buy_alpha_limit_prices(), vec![1_010]);
    });
}

#[test]
fn execute_orders_sell_one_percent_slippage_passes_floor_to_pool() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        // Price must be >= limit_price for TakeProfit to trigger.
        MockSwap::set_price(2_000.0);

        // limit_price=1000, 1% slippage → floor = 990.
        let signed = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::TakeProfit,
            500,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)),
        );

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert_eq!(MockSwap::sell_alpha_limit_prices(), vec![990]);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// max_slippage — execute_batched_orders aggregates tightest constraint
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_batched_orders_buy_dominant_uses_min_ceiling() {
    new_test_ext().execute_with(|| {
        // 3 buy orders with different slippage constraints.
        //   Alice: limit=1000, 2% → ceiling=1020
        //   Bob:   limit=1000, 1% → ceiling=1010  ← tightest
        //   Charlie (as signer, not relayer): limit=1000, 3% → ceiling=1030
        // Expected pool price_limit = min(1020, 1010, 1030) = 1010.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);
        MockSwap::set_tao_balance(alice(), 600);
        MockSwap::set_tao_balance(bob(), 200);
        MockSwap::set_tao_balance(dave(), 200);

        let alice_order = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            600,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(2)), // ceiling = 1020
        );
        let bob_order = make_signed_order_with_slippage(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            200,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)), // ceiling = 1010 ← tightest
        );
        let dave_order = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            dave(),
            netuid(),
            OrderType::LimitBuy,
            200,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(3)), // ceiling = 1030
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order, dave_order]),
        ));

        // Net pool swap must have been called with the tightest ceiling = 1010.
        assert_eq!(MockSwap::buy_alpha_limit_prices(), vec![1_010]);
    });
}

#[test]
fn execute_batched_orders_sell_dominant_uses_max_floor() {
    new_test_ext().execute_with(|| {
        // 3 sell orders with different slippage constraints.
        //   Alice: limit=1000, 3% → floor=970
        //   Bob:   limit=1000, 1% → floor=990  ← tightest (highest floor)
        //   Dave:  limit=1000, 2% → floor=980
        // Expected pool price_limit = max(970, 990, 980) = 990.
        // Price must be >= limit_price=1000 for TakeProfit to trigger.
        MockTime::set(1_000_000);
        MockSwap::set_price(2_000.0);
        MockSwap::set_sell_tao_return(500);
        MockSwap::set_alpha_balance(alice(), dave(), netuid(), 600);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 200);
        MockSwap::set_alpha_balance(dave(), dave(), netuid(), 200);

        let alice_order = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            600,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(3)), // floor = 970
        );
        let bob_order = make_signed_order_with_slippage(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)), // floor = 990 ← tightest
        );
        let dave_order = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(2)), // floor = 980
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order, dave_order]),
        ));

        // Net pool swap must have been called with the tightest floor = 990.
        assert_eq!(MockSwap::sell_alpha_limit_prices(), vec![990]);
    });
}

#[test]
fn execute_batched_orders_no_slippage_uses_unconstrained_limits() {
    new_test_ext().execute_with(|| {
        // Orders without max_slippage should pass u64::MAX (buy) or 0 (sell).
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);
        MockSwap::set_tao_balance(alice(), 1_000);

        let order = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![order]),
        ));

        assert_eq!(MockSwap::buy_alpha_limit_prices(), vec![u64::MAX]);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// max_slippage — mixed order type coexistence
// ─────────────────────────────────────────────────────────────────────────────

/// Sell-dominant batch: TakeProfit orders (with slippage) + StopLoss (no slippage).
///
/// TakeProfit orders set meaningful floors; StopLoss contributes 0 (no constraint).
/// pool_price_limit = max(take_floors..., 0s) = max(take_floors).
/// All three orders are fulfilled.
#[test]
fn execute_batched_orders_takeprofit_and_stoploss_coexist_sell_dominant() {
    new_test_ext().execute_with(|| {
        // Price = 2000 — above all TakeProfit limits (≥1000 ✓) and below StopLoss limit (≤5000 ✓).
        MockTime::set(1_000_000);
        MockSwap::set_price(2_000.0);
        MockSwap::set_sell_tao_return(500);

        // Alice TakeProfit: limit=1000, 3% → floor=970.
        // Bob TakeProfit:   limit=1000, 1% → floor=990.  ← tightest
        // Dave StopLoss:    limit=5000, None → floor=0.
        MockSwap::set_alpha_balance(alice(), dave(), netuid(), 600);
        MockSwap::set_alpha_balance(bob(), dave(), netuid(), 200);
        MockSwap::set_alpha_balance(dave(), alice(), netuid(), 200);

        let alice_order = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            600,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(3)),
        );
        let bob_order = make_signed_order_with_slippage(
            AccountKeyring::Bob,
            dave(),
            netuid(),
            OrderType::TakeProfit,
            200,
            1_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)),
        );
        let dave_stoploss = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            alice(),
            netuid(),
            OrderType::StopLoss,
            200,
            5_000,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None, // StopLoss: no slippage → floor=0, does not constrain pool
        );

        let alice_id = order_id(&alice_order.order);
        let bob_id = order_id(&bob_order.order);
        let dave_id = order_id(&dave_stoploss.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order, dave_stoploss]),
        ));

        // All three fulfilled.
        assert_eq!(Orders::<Test>::get(alice_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(bob_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(dave_id), Some(OrderStatus::Fulfilled));

        // Pool called once with the tightest TakeProfit floor (990), not 0 from StopLoss.
        assert_eq!(MockSwap::sell_alpha_limit_prices(), vec![990]);
    });
}

/// Buy-dominant batch: LimitBuy orders (with slippage) dominant + StopLoss (no slippage) on offset side.
///
/// The offset StopLoss is settled internally at spot price; it does not contribute
/// to the pool's price ceiling (which comes only from the dominant buy side).
/// pool_price_limit = min(buy_ceilings) = 101.
#[test]
fn execute_batched_orders_limitbuy_and_stoploss_offset_coexist_buy_dominant() {
    new_test_ext().execute_with(|| {
        // Price = 1. LimitBuy triggers (1 ≤ 100 ✓). StopLoss triggers (1 ≤ 5 ✓).
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(900);

        // Alice LimitBuy: limit=100, 2% → ceiling=102.
        // Bob   LimitBuy: limit=100, 1% → ceiling=101.  ← tightest
        // Dave  StopLoss: limit=5,   None → floor=0 (offset side, not used for pool limit).
        MockSwap::set_tao_balance(alice(), 600);
        MockSwap::set_tao_balance(bob(), 400);
        MockSwap::set_alpha_balance(dave(), alice(), netuid(), 100);

        let alice_order = make_signed_order_with_slippage(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            600,
            100,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(2)),
        );
        let bob_order = make_signed_order_with_slippage(
            AccountKeyring::Bob,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            400,
            100,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)),
        );
        let dave_stoploss = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            alice(),
            netuid(),
            OrderType::StopLoss,
            100,
            5,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            None, // StopLoss: no slippage; settled at spot, never constrains pool ceiling
        );

        let alice_id = order_id(&alice_order.order);
        let bob_id = order_id(&bob_order.order);
        let dave_id = order_id(&dave_stoploss.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_order, bob_order, dave_stoploss]),
        ));

        // All three fulfilled.
        assert_eq!(Orders::<Test>::get(alice_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(bob_id), Some(OrderStatus::Fulfilled));
        assert_eq!(Orders::<Test>::get(dave_id), Some(OrderStatus::Fulfilled));

        // Pool buy called with min(102, 101) = 101. StopLoss's floor (0) is ignored on buy side.
        assert_eq!(MockSwap::buy_alpha_limit_prices(), vec![101]);
    });
}

/// StopLoss with a narrow slippage sets an effective floor above the current market price,
/// making the pool swap impossible and failing the entire batch.
///
/// This demonstrates Issue 1 from the design: relayers should not apply max_slippage to
/// StopLoss orders. StopLoss triggers when price has already fallen; a floor derived from
/// the (higher) trigger threshold will almost always exceed the actual market price.
#[test]
fn execute_batched_orders_stoploss_narrow_slippage_breaks_batch() {
    new_test_ext().execute_with(|| {
        // StopLoss: limit=100, triggers at price=50 (50 ≤ 100 ✓).
        // 1% slippage → floor=99. Market is at 50 → pool cannot deliver ≥99.
        MockTime::set(1_000_000);
        MockSwap::set_price(50.0);
        MockSwap::set_sell_tao_return(100); // non-zero so SwapReturnedZero is not the cause
        MockSwap::set_enforce_price_limit(true);
        MockSwap::set_alpha_balance(dave(), alice(), netuid(), 200);

        let stoploss = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            alice(),
            netuid(),
            OrderType::StopLoss,
            200,
            100,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)), // floor=99, but market=50 → pool rejects
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![stoploss]),
            ),
            DispatchError::Other("price limit exceeded")
        );
    });
}

/// Same StopLoss scenario through execute_orders (best-effort): the order is silently
/// skipped rather than failing the whole call.
///
/// Note: `DispatchError::Other` has `#[codec(skip)]` on its string field, so the reason
/// string is lost when stored in the event log. We verify the skip via storage absence
/// and by asserting the floor (99) was actually passed to the pool — which is what caused
/// the rejection. The `execute_batched_orders` variant below uses `assert_noop!` (checks
/// the return value directly, no storage round-trip) and can verify the string.
#[test]
fn execute_orders_stoploss_narrow_slippage_skips_order() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(50.0);
        MockSwap::set_sell_tao_return(100);
        MockSwap::set_enforce_price_limit(true);

        let stoploss = make_signed_order_with_slippage(
            AccountKeyring::Dave,
            alice(),
            netuid(),
            OrderType::StopLoss,
            200,
            100,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(Perbill::from_percent(1)), // floor=99, but market=50 → pool rejects
        );
        let id = order_id(&stoploss.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![stoploss]),
        ));

        // Order not stored — pool rejected the floor.
        assert!(Orders::<Test>::get(id).is_none());

        // An OrderSkipped event must have been emitted for this order.
        assert!(
            System::events().iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::LimitOrders(Event::OrderSkipped { order_id, .. })
                    if *order_id == id
            )),
            "expected OrderSkipped event for this order"
        );

        // The sell was attempted with the correct floor (99 = 100 - 1%).
        // This is the value that exceeded the market price and caused the rejection.
        assert_eq!(MockSwap::sell_alpha_limit_prices(), vec![99]);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// relayer enforcement
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_orders_wrong_relayer_skipped() {
    new_test_ext().execute_with(|| {
        // Order locks execution to charlie(); submitting as bob() must be silently skipped.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(charlie()), // only charlie may relay this order
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(bob()), // wrong relayer
            bounded(vec![signed])
        ));

        // Order not stored — it was skipped.
        assert!(Orders::<Test>::get(id).is_none());
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::RelayerMissMatch.into(),
        });
    });
}

#[test]
fn execute_orders_correct_relayer_executed() {
    new_test_ext().execute_with(|| {
        // Same order submitted by the designated relayer (charlie) — must succeed.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(charlie()), // charlie is the designated relayer
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()), // correct relayer
            bounded(vec![signed])
        ));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount_in: 1_000,
            amount_out: 0,
        });
    });
}

#[test]
fn execute_batched_orders_wrong_relayer_fails_entire_batch() {
    new_test_ext().execute_with(|| {
        // In execute_batched_orders a relayer mismatch is a hard failure — the
        // whole call is reverted, unlike the best-effort skip in execute_orders.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(charlie()), // only charlie may relay this order
        );

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(bob()), // wrong relayer
                netuid(),
                bounded(vec![signed])
            ),
            Error::<Test>::RelayerMissMatch
        );
    });
}

#[test]
fn execute_batched_orders_correct_relayer_succeeds() {
    new_test_ext().execute_with(|| {
        // Same order submitted by the designated relayer — must execute and
        // distribute alpha to the buyer.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(1_000);
        MockSwap::set_tao_balance(alice(), 1_000);

        let signed = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            Perbill::zero(),
            fee_recipient(),
            Some(charlie()), // charlie is the designated relayer
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()), // correct relayer
            netuid(),
            bounded(vec![signed])
        ));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Partial fills — execute_orders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_orders_partial_fill_sets_partially_filled_status() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_tao_balance(alice(), 1_000);

        // Order for 1000 TAO; relayer is charlie (required for partial fills).
        let signed = make_partial_fill_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            charlie(),
            400, // fill 400 out of 1000
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed]),
        ));

        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(400))
        );
    });
}

#[test]
fn execute_orders_second_partial_fill_completes_order() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_tao_balance(alice(), 1_000);

        let signed_first = make_partial_fill_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            charlie(),
            600,
        );
        let id = order_id(&signed_first.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed_first.clone()]),
        ));
        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(600))
        );

        // Re-submit the same signed order payload with a different partial_fill amount.
        let mut signed_second = signed_first.clone();
        signed_second.partial_fill = Some(400);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed_second]),
        ));
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
    });
}

#[test]
fn execute_orders_partial_fill_without_relayer_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_tao_balance(alice(), 1_000);

        // Build an order with partial_fills_enabled but no relayer set.
        let inner = crate::Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
            relayer: None, // <-- no relayer
            max_slippage: None,
            chain_id: 945,
            partial_fills_enabled: true,
        };
        let versioned = VersionedOrder::V1(inner);
        let sig = AccountKeyring::Alice.pair().sign(&versioned.encode());
        let signed = crate::SignedOrder {
            order: versioned,
            signature: sp_runtime::MultiSignature::Sr25519(sig),
            partial_fill: Some(400),
        };
        let id = order_id(&signed.order);

        // The order is skipped (best-effort), not reverting the batch.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed]),
        ));

        // Nothing written to storage.
        assert_eq!(Orders::<Test>::get(id), None);
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::RelayerRequiredForPartialFill.into(),
        });
    });
}

#[test]
fn execute_orders_partial_fill_exceeding_remaining_is_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_tao_balance(alice(), 1_000);

        // Pre-fill 700 of 1000.
        let signed = make_partial_fill_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            charlie(),
            700,
        );
        let id = order_id(&signed.order);
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed.clone()]),
        ));
        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(700))
        );

        // Try to fill 500 more, but only 300 remain → should be skipped.
        let mut over_fill = signed.clone();
        over_fill.partial_fill = Some(500);
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![over_fill]),
        ));

        // Status unchanged.
        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(700))
        );
        assert_event(Event::OrderSkipped {
            order_id: id,
            reason: Error::<Test>::IncorrectPartialFillAmount.into(),
        });
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Partial fills — execute_batched_orders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_batched_orders_partial_fill_sets_partially_filled_status() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(400);
        MockSwap::set_tao_balance(alice(), 1_000);

        let signed = make_partial_fill_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            charlie(),
            400,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![signed]),
        ));

        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(400))
        );
    });
}

#[test]
fn execute_batched_orders_second_partial_fill_completes_order() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(600);
        MockSwap::set_tao_balance(alice(), 1_000);

        let signed_first = make_partial_fill_order(
            AccountKeyring::Alice,
            bob(),
            netuid(),
            OrderType::LimitBuy,
            1_000,
            u64::MAX,
            FAR_FUTURE,
            charlie(),
            600,
        );
        let id = order_id(&signed_first.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![signed_first.clone()]),
        ));
        assert_eq!(
            Orders::<Test>::get(id),
            Some(OrderStatus::PartiallyFilled(600))
        );

        let mut signed_second = signed_first.clone();
        signed_second.partial_fill = Some(400);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![signed_second]),
        ));
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
    });
}

/// Non-root origin cannot disable the pallet
#[test]
fn non_root_cannot_disable_the_pallet() {
    new_test_ext().execute_with(|| {
        // Try disabling the pallet with charlie
        assert_noop!(
            LimitOrders::set_pallet_status(RuntimeOrigin::signed(charlie()), false),
            DispatchError::BadOrigin
        );
    });
}
