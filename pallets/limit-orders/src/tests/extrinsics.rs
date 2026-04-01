//! Integration tests for `pallet-limit-orders` extrinsics.
//!
//! Tests go through the full dispatch path: origin enforcement, storage changes,
//! and event emission are all verified. SwapInterface calls are handled by
//! `MockSwap`, which records calls and maintains in-memory balance ledgers.

use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{DispatchError, Perbill};
use subtensor_runtime_common::NetUid;

use crate::{Error, Order, OrderSide, OrderStatus, OrderType, Orders, pallet::Event};

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
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
        };
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
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
        };
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
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
        };
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
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
        };
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
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: fee_recipient(),
        };
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
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert!(Orders::<Test>::get(id).is_none());
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
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        // Skipped — storage untouched.
        assert!(Orders::<Test>::get(id).is_none());
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
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![signed])
        ));

        assert!(Orders::<Test>::get(id).is_none());
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
        );
        let valid_id = order_id(&valid.order);

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie()),
            bounded(vec![valid, expired]),
        ));

        assert_eq!(Orders::<Test>::get(valid_id), Some(OrderStatus::Fulfilled));
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
            );
            let id = order_id(&signed.order);

            assert_ok!(LimitOrders::execute_orders(
                RuntimeOrigin::signed(charlie()),
                bounded(vec![signed])
            ));

            // Skipped — storage untouched.
            assert!(Orders::<Test>::get(id).is_none());
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
            );
            let id = order_id(&signed.order);

            assert_ok!(LimitOrders::execute_orders(
                RuntimeOrigin::signed(charlie()),
                bounded(vec![signed])
            ));

            // Skipped — storage untouched.
            assert!(Orders::<Test>::get(id).is_none());
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
        );
        let id = order_id(&signed.order);
        Orders::<Test>::insert(id, OrderStatus::Cancelled);

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie()),
                netuid(),
                bounded(vec![signed]),
            ),
            Error::<Test>::OrderAlreadyProcessed
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
