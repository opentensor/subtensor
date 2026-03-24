//! Integration tests for `pallet-limit-orders` extrinsics.
//!
//! Tests go through the full dispatch path: origin enforcement, storage changes,
//! and event emission are all verified. SwapInterface calls are handled by
//! `MockSwap`, which records calls and maintains in-memory balance ledgers.

use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_core::{H256, Pair};
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{DispatchError, MultiSignature};
use subtensor_runtime_common::NetUid;

use crate::{
    Admin, Error, Order, OrderSide, OrderStatus, Orders,
    pallet::{Event, ProtocolFee},
};

type LimitOrders = crate::pallet::Pallet<Test>;

use super::mock::*;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn alice() -> AccountId {
    AccountKeyring::Alice.to_account_id()
}
fn bob() -> AccountId {
    AccountKeyring::Bob.to_account_id()
}
fn charlie() -> AccountId {
    AccountKeyring::Charlie.to_account_id()
}
fn dave() -> AccountId {
    AccountKeyring::Dave.to_account_id()
}

fn netuid() -> NetUid {
    NetUid::from(1u16)
}

fn make_signed_order(
    keyring: AccountKeyring,
    hotkey: AccountId,
    netuid: NetUid,
    side: OrderSide,
    amount: u64,
    limit_price: u64,
    expiry: u64,
) -> crate::SignedOrder<AccountId, MultiSignature> {
    use codec::Encode;
    let signer = keyring.to_account_id();
    let order = Order { signer, hotkey, netuid, side, amount, limit_price, expiry };
    let sig = keyring.pair().sign(&order.encode());
    crate::SignedOrder { order, signature: MultiSignature::Sr25519(sig) }
}

fn bounded(
    v: Vec<crate::SignedOrder<AccountId, MultiSignature>>,
) -> BoundedVec<crate::SignedOrder<AccountId, MultiSignature>, frame_support::traits::ConstU32<64>>
{
    BoundedVec::try_from(v).unwrap()
}

/// Check that a specific pallet event was emitted.
fn assert_event(event: Event<Test>) {
    assert!(
        System::events()
            .iter()
            .any(|r| r.event == RuntimeEvent::LimitOrders(event.clone())),
        "expected event not found: {event:?}",
    );
}

fn order_id(order: &Order<AccountId>) -> H256 {
    use codec::Encode;
    H256(sp_core::hashing::blake2_256(&order.encode()))
}

const FAR_FUTURE: u64 = u64::MAX;

// ─────────────────────────────────────────────────────────────────────────────
// set_admin
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn set_admin_root_can_set_admin() {
    new_test_ext().execute_with(|| {
        assert_ok!(LimitOrders::set_admin(RuntimeOrigin::root(), Some(alice())));
        assert_eq!(Admin::<Test>::get(), Some(alice()));
        assert_event(Event::AdminSet { new_admin: Some(alice()) });
    });
}

#[test]
fn set_admin_root_can_clear_admin() {
    new_test_ext().execute_with(|| {
        Admin::<Test>::put(alice());
        assert_ok!(LimitOrders::set_admin(RuntimeOrigin::root(), None));
        assert!(Admin::<Test>::get().is_none());
        assert_event(Event::AdminSet { new_admin: None });
    });
}

#[test]
fn set_admin_signed_origin_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LimitOrders::set_admin(RuntimeOrigin::signed(alice()), Some(bob())),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_admin_unsigned_origin_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LimitOrders::set_admin(RuntimeOrigin::none(), Some(alice())),
            DispatchError::BadOrigin
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// set_protocol_fee
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn set_protocol_fee_root_can_set() {
    new_test_ext().execute_with(|| {
        assert_ok!(LimitOrders::set_protocol_fee(RuntimeOrigin::root(), 1_000_000));
        assert_eq!(ProtocolFee::<Test>::get(), 1_000_000);
        assert_event(Event::ProtocolFeeSet { fee: 1_000_000 });
    });
}

#[test]
fn set_protocol_fee_admin_can_set() {
    new_test_ext().execute_with(|| {
        Admin::<Test>::put(alice());
        assert_ok!(LimitOrders::set_protocol_fee(RuntimeOrigin::signed(alice()), 500_000));
        assert_eq!(ProtocolFee::<Test>::get(), 500_000);
        assert_event(Event::ProtocolFeeSet { fee: 500_000 });
    });
}

#[test]
fn set_protocol_fee_non_admin_rejected() {
    new_test_ext().execute_with(|| {
        Admin::<Test>::put(alice());
        // Bob is not the admin.
        assert_noop!(
            LimitOrders::set_protocol_fee(RuntimeOrigin::signed(bob()), 999),
            Error::<Test>::NotAdmin
        );
    });
}

#[test]
fn set_protocol_fee_no_admin_signed_rejected() {
    new_test_ext().execute_with(|| {
        // No admin set at all; signed origin that is not root must be rejected.
        assert_noop!(
            LimitOrders::set_protocol_fee(RuntimeOrigin::signed(alice()), 999),
            Error::<Test>::NotAdmin
        );
    });
}

#[test]
fn set_protocol_fee_unsigned_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LimitOrders::set_protocol_fee(RuntimeOrigin::none(), 1),
            DispatchError::BadOrigin
        );
    });
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
            side: OrderSide::Buy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
        };
        let id = order_id(&order);

        assert_ok!(LimitOrders::cancel_order(RuntimeOrigin::signed(alice()), order));
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Cancelled));
        assert_event(Event::OrderCancelled { order_id: id, signer: alice() });
    });
}

#[test]
fn cancel_order_non_signer_rejected() {
    new_test_ext().execute_with(|| {
        let order = Order {
            signer: alice(),
            hotkey: bob(),
            netuid: netuid(),
            side: OrderSide::Buy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
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
            side: OrderSide::Buy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
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
            side: OrderSide::Buy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
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
            side: OrderSide::Buy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: FAR_FUTURE,
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, 2_000_000_000, FAR_FUTURE,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            side: OrderSide::Buy,
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Sell, 500, 1, FAR_FUTURE,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Fulfilled));
        assert_event(Event::OrderExecuted {
            order_id: id,
            signer: alice(),
            netuid: netuid(),
            side: OrderSide::Sell,
        });
    });
}

#[test]
fn execute_orders_expired_order_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(2_000_001); // now > expiry
        MockSwap::set_price(1.0);
        let signed = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, 2_000_000, // expiry in the past
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, 2, FAR_FUTURE,
        );
        let id = order_id(&signed.order);

        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

        assert!(Orders::<Test>::get(id).is_none());
    });
}

#[test]
fn execute_orders_already_processed_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        let signed = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        let id = order_id(&signed.order);
        Orders::<Test>::insert(id, OrderStatus::Fulfilled);

        // Should succeed (batch-level) but skip this order silently.
        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        let expired = make_signed_order(
            AccountKeyring::Bob, alice(), netuid(),
            OrderSide::Buy, 500, u64::MAX, 500_000, // already expired
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
        ProtocolFee::<Test>::put(10_000_000u32); // 1%

        let signed = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        MockSwap::set_tao_balance(alice(), 1_000);
        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

        // One buy_alpha call for the net amount (990 TAO after 1% fee).
        let buys: Vec<_> = MockSwap::log().into_iter()
            .filter_map(|c| if let super::mock::SwapCall::BuyAlpha { tao, .. } = c { Some(tao) } else { None })
            .collect();
        assert_eq!(buys, vec![990], "main swap must use 990 TAO after 1% fee");

        // Fee (10 TAO) forwarded directly to FeeCollector via transfer_tao.
        assert_eq!(MockSwap::tao_balance(&FeeCollectorAccount::get()), 10);
    });
}

#[test]
fn execute_orders_sell_with_fee_charges_fee() {
    new_test_ext().execute_with(|| {
        // fee = 1% (10_000_000 ppb).
        // Alice sells 1_000 alpha; pool returns 800 TAO.
        // fee_tao = 1% of 800 = 8 TAO, forwarded to FeeCollector via transfer_tao.
        // Alice keeps 792 TAO.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_sell_tao_return(800);
        MockSwap::set_alpha_balance(alice(), bob(), netuid(), 1_000);
        ProtocolFee::<Test>::put(10_000_000u32); // 1%

        let signed = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Sell, 1_000, 0, FAR_FUTURE,
        );
        assert_ok!(LimitOrders::execute_orders(RuntimeOrigin::signed(charlie()), bounded(vec![signed])));

        // Full 1_000 alpha sold (no alpha deducted for fee).
        let sells: Vec<_> = MockSwap::log().into_iter()
            .filter_map(|c| if let super::mock::SwapCall::SellAlpha { alpha, .. } = c { Some(alpha) } else { None })
            .collect();
        assert_eq!(sells, vec![1_000], "full alpha amount must be sold");

        // FeeCollector received 8 TAO (1% of 800).
        assert_eq!(MockSwap::tao_balance(&FeeCollectorAccount::get()), 8);
        // Alice kept the remaining 792 TAO.
        assert_eq!(MockSwap::tao_balance(&alice()), 792);
    });
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
fn execute_batched_orders_all_invalid_returns_ok() {
    new_test_ext().execute_with(|| {
        MockTime::set(2_000_001); // all expired
        let expired = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, 1_000_000,
        );
        // Returns Ok even when nothing executes.
        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![expired]),
        ));
        // No summary event — early return when executed_count == 0.
        let has_summary = System::events().iter().any(|r| {
            matches!(&r.event, RuntimeEvent::LimitOrders(Event::GroupExecutionSummary { .. }))
        });
        assert!(!has_summary);
    });
}

#[test]
fn execute_batched_orders_skips_wrong_netuid() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(100);

        let wrong_net = make_signed_order(
            AccountKeyring::Alice, bob(), NetUid::from(99u16), // wrong netuid
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        let id = order_id(&wrong_net.order);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(), // batch targets netuid 1
            bounded(vec![wrong_net]),
        ));

        assert!(Orders::<Test>::get(id).is_none(), "wrong-netuid order must not be fulfilled");
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
            AccountKeyring::Alice, dave(), netuid(),
            OrderSide::Buy, 600, u64::MAX, FAR_FUTURE,
        );
        let bob_order = make_signed_order(
            AccountKeyring::Bob, dave(), netuid(),
            OrderSide::Buy, 400, u64::MAX, FAR_FUTURE,
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
            AccountKeyring::Alice, dave(), netuid(),
            OrderSide::Sell, 300, 0, FAR_FUTURE, // limit=0 → accept any price
        );
        let bob_order = make_signed_order(
            AccountKeyring::Bob, dave(), netuid(),
            OrderSide::Sell, 200, 0, FAR_FUTURE,
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
            AccountKeyring::Alice, dave(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        let bob_buy = make_signed_order(
            AccountKeyring::Bob, dave(), netuid(),
            OrderSide::Buy, 600, u64::MAX, FAR_FUTURE,
        );
        let charlie_sell = make_signed_order(
            AccountKeyring::Charlie, dave(), netuid(),
            OrderSide::Sell, 200, 0, FAR_FUTURE,
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
            AccountKeyring::Alice, dave(), netuid(),
            OrderSide::Buy, 200, u64::MAX, FAR_FUTURE,
        );
        let bob_sell = make_signed_order(
            AccountKeyring::Bob, dave(), netuid(),
            OrderSide::Sell, 300, 0, FAR_FUTURE,
        );
        let charlie_sell = make_signed_order(
            AccountKeyring::Charlie, dave(), netuid(),
            OrderSide::Sell, 200, 0, FAR_FUTURE,
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
        // collect_fees transfers 10 TAO (buy fee) to FeeCollector.
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(500);
        ProtocolFee::<Test>::put(10_000_000u32);

        let alice_buy = make_signed_order(
            AccountKeyring::Alice, dave(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![alice_buy]),
        ));

        // Fee collector received the buy-side fee.
        assert_eq!(MockSwap::tao_balance(&FeeCollectorAccount::get()), 10);
    });
}

#[test]
fn execute_batched_orders_cancelled_order_skipped() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        MockSwap::set_buy_alpha_return(100);

        let signed = make_signed_order(
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
        );
        let id = order_id(&signed.order);
        Orders::<Test>::insert(id, OrderStatus::Cancelled);

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie()),
            netuid(),
            bounded(vec![signed]),
        ));

        // Still cancelled, not changed to Fulfilled.
        assert_eq!(Orders::<Test>::get(id), Some(OrderStatus::Cancelled));
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Buy, 1_000, u64::MAX, FAR_FUTURE,
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Sell, 1_000, 0, FAR_FUTURE,
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
            AccountKeyring::Alice, bob(), netuid(),
            OrderSide::Sell, 1_000, 0, FAR_FUTURE,
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
