//! Unit tests for the auxiliary helper functions in `pallet-limit-orders`.
//!
//! Extrinsics are NOT tested here. Each section focuses on one helper.

use frame_support::{BoundedVec, traits::ConstU32};
use sp_core::{H256, Pair};
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{AccountId32, MultiSignature};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};

use crate::{Order, OrderSide, OrderStatus, Orders, SignedOrder, pallet::ProtocolFee};
use crate::pallet::Pallet as LimitOrders;

use super::mock::*;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn alice() -> AccountId32 {
    AccountKeyring::Alice.to_account_id()
}

fn bob() -> AccountId32 {
    AccountKeyring::Bob.to_account_id()
}

fn charlie() -> AccountId32 {
    AccountKeyring::Charlie.to_account_id()
}

fn netuid_1() -> NetUid {
    NetUid::from(1u16)
}

/// Create a `SignedOrder` signed by the given `AccountKeyring` key.
fn make_signed_order(
    keyring: AccountKeyring,
    hotkey: AccountId32,
    netuid: NetUid,
    side: OrderSide,
    amount: u64,
    limit_price: u64,
    expiry: u64,
) -> SignedOrder<AccountId32, MultiSignature> {
    let signer = keyring.to_account_id();
    let order = Order {
        signer,
        hotkey,
        netuid,
        side,
        amount,
        limit_price,
        expiry,
    };
    use codec::Encode;
    let msg = order.encode();
    let sig = keyring.pair().sign(&msg);
    SignedOrder {
        order,
        signature: MultiSignature::Sr25519(sig),
    }
}

fn bounded_orders(
    v: Vec<SignedOrder<AccountId32, MultiSignature>>,
) -> BoundedVec<SignedOrder<AccountId32, MultiSignature>, ConstU32<64>> {
    BoundedVec::try_from(v).unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// ppb_of_tao / ppb_of_alpha
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ppb_of_tao_zero_fee_returns_zero() {
    new_test_ext().execute_with(|| {
        // 0 ppb → no fee regardless of amount
        let fee = LimitOrders::<Test>::ppb_of_tao(TaoBalance::from(1_000_000u64), 0);
        assert_eq!(fee, TaoBalance::from(0u64));
    });
}

#[test]
fn ppb_of_tao_full_ppb_returns_amount() {
    new_test_ext().execute_with(|| {
        // 1_000_000_000 ppb = 100% → fee == amount
        let amount = TaoBalance::from(500_000u64);
        let fee = LimitOrders::<Test>::ppb_of_tao(amount, 1_000_000_000u32);
        assert_eq!(fee, amount);
    });
}

#[test]
fn ppb_of_tao_one_tenth_percent() {
    new_test_ext().execute_with(|| {
        // 1_000_000 ppb = 0.1%
        // 1_000_000 * 1_000_000 / 1_000_000_000 = 1_000
        let fee = LimitOrders::<Test>::ppb_of_tao(TaoBalance::from(1_000_000_000u64), 1_000_000u32);
        assert_eq!(fee, TaoBalance::from(1_000_000u64));
    });
}

#[test]
fn ppb_of_alpha_one_tenth_percent() {
    new_test_ext().execute_with(|| {
        let fee =
            LimitOrders::<Test>::ppb_of_alpha(AlphaBalance::from(1_000_000_000u64), 1_000_000u32);
        assert_eq!(fee, AlphaBalance::from(1_000_000u64));
    });
}

#[test]
fn ppb_of_tao_rounds_down() {
    new_test_ext().execute_with(|| {
        // amount=1, ppb=999_999_999 (just under 100%) → floor(0.999…) = 0
        let fee = LimitOrders::<Test>::ppb_of_tao(TaoBalance::from(1u64), 999_999_999u32);
        assert_eq!(fee, TaoBalance::from(0u64));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// net_amount_for_event
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn net_amount_for_event_buy_dominant() {
    new_test_ext().execute_with(|| {
        // Buys = 1000 TAO net, sells TAO-equiv = 300 TAO → net 700 TAO buy-side
        let price = U96F32::from_num(2u32); // 2 TAO/alpha
        let net = LimitOrders::<Test>::net_amount_for_event(
            &OrderSide::Buy,
            1_000u128, // total_buy_net (TAO)
            150u128,   // total_sell_net (alpha)  ← not used in Buy branch
            300u128,   // total_sell_tao_equiv
            price,
        );
        assert_eq!(net, 700u64);
    });
}

#[test]
fn net_amount_for_event_sell_dominant() {
    new_test_ext().execute_with(|| {
        // Sells = 500 alpha net, buys TAO = 200 TAO at price 2 → buy_alpha_equiv = 100
        // net sell = 500 - 100 = 400 alpha
        let price = U96F32::from_num(2u32); // 2 TAO/alpha → 1 alpha = 2 TAO
        let net = LimitOrders::<Test>::net_amount_for_event(
            &OrderSide::Sell,
            200u128, // total_buy_net (TAO)
            500u128, // total_sell_net (alpha)
            400u128, // total_sell_tao_equiv (not used in Sell branch directly)
            price,
        );
        // buy_alpha_equiv = 200 / 2 = 100; net = 500 - 100 = 400
        assert_eq!(net, 400u64);
    });
}

#[test]
fn net_amount_for_event_perfectly_offset() {
    new_test_ext().execute_with(|| {
        // Buys = 200 TAO, sells TAO-equiv = 200 → net = 0 (buy-side result = 0)
        let price = U96F32::from_num(2u32);
        let net = LimitOrders::<Test>::net_amount_for_event(
            &OrderSide::Buy,
            200u128,
            100u128,
            200u128,
            price,
        );
        assert_eq!(net, 0u64);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_and_classify
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn validate_and_classify_separates_buys_and_sells() {
    new_test_ext().execute_with(|| {
        // Current time = 1_000_000 ms; expiry = 2_000_000 ms (well in the future).
        MockTime::set(1_000_000);
        // Price = 1.0 TAO/alpha.
        MockSwap::set_price(1.0);

        // Fee = 0 ppb for simplicity.
        ProtocolFee::<Test>::put(0u32);

        let buy_order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid_1(),
            OrderSide::Buy,
            1_000u64,     // amount in TAO
            2_000_000u64, // limit_price: willing to pay up to 2 TAO/alpha (price=1 < 2 ✓)
            2_000_000u64, // expiry ms
        );
        let sell_order = make_signed_order(
            AccountKeyring::Bob,
            alice(),
            netuid_1(),
            OrderSide::Sell,
            500u64,   // amount in alpha
            1u64,     // limit_price: sell if price >= 1 TAO/alpha (price=1 >= 1 ✓)
            2_000_000u64,
        );

        let orders = bounded_orders(vec![buy_order, sell_order]);
        let (buys, sells) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(),
            &orders,
            1_000_000u64,
            0u32,
            U96F32::from_num(1u32),
        );

        assert_eq!(buys.len(), 1, "expected 1 valid buy");
        assert_eq!(sells.len(), 1, "expected 1 valid sell");

        // Buy entry: gross=1000, net=1000 (0% fee), fee=0
        let (_, signer, _, gross, net, fee) = &buys[0];
        assert_eq!(signer, &alice());
        assert_eq!(*gross, 1_000u64);
        assert_eq!(*net, 1_000u64);
        assert_eq!(*fee, 0u64);

        // Sell entry: gross=500, net=500, fee=0 (fee deferred to distribution)
        let (_, signer, _, gross, net, fee) = &sells[0];
        assert_eq!(signer, &bob());
        assert_eq!(*gross, 500u64);
        assert_eq!(*net, 500u64);
        assert_eq!(*fee, 0u64, "sell fee is always 0 here — applied on TAO output");
    });
}

#[test]
fn validate_and_classify_skips_wrong_netuid() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        MockSwap::set_price(1.0);
        ProtocolFee::<Test>::put(0u32);

        let wrong_netuid_order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            NetUid::from(99u16), // different netuid
            OrderSide::Buy,
            1_000u64,
            2_000_000u64,
            2_000_000u64,
        );

        let orders = bounded_orders(vec![wrong_netuid_order]);
        let (buys, sells) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(), // batch is for netuid 1
            &orders,
            1_000_000u64,
            0u32,
            U96F32::from_num(1u32),
        );

        assert_eq!(buys.len(), 0);
        assert_eq!(sells.len(), 0);
    });
}

#[test]
fn validate_and_classify_skips_expired_order() {
    new_test_ext().execute_with(|| {
        // now_ms = 2_000_001, expiry = 2_000_000 → expired
        MockTime::set(2_000_001);
        MockSwap::set_price(1.0);
        ProtocolFee::<Test>::put(0u32);

        let expired = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid_1(),
            OrderSide::Buy,
            1_000u64,
            2_000_000u64,
            2_000_000u64, // expiry already past
        );

        let orders = bounded_orders(vec![expired]);
        let (buys, sells) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(),
            &orders,
            2_000_001u64,
            0u32,
            U96F32::from_num(1u32),
        );

        assert_eq!(buys.len(), 0);
        assert_eq!(sells.len(), 0);
    });
}

#[test]
fn validate_and_classify_skips_price_condition_not_met_for_buy() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        // Price = 3.0 TAO/alpha, buyer's limit = 2.0 → price > limit → skip
        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid_1(),
            OrderSide::Buy,
            1_000u64,
            2u64, // limit_price = 2 TAO/alpha
            2_000_000u64,
        );

        let orders = bounded_orders(vec![order]);
        let (buys, _) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(),
            &orders,
            1_000_000u64,
            0u32,
            U96F32::from_num(3u32), // current price = 3 > limit 2 → skip
        );

        assert_eq!(buys.len(), 0);
    });
}

#[test]
fn validate_and_classify_skips_already_processed_order() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid_1(),
            OrderSide::Buy,
            1_000u64,
            2_000_000u64,
            2_000_000u64,
        );

        // Pre-mark as fulfilled on-chain.
        use codec::Encode;
        let order_id = H256(sp_core::hashing::blake2_256(&order.order.encode()));
        Orders::<Test>::insert(order_id, OrderStatus::Fulfilled);

        let orders = bounded_orders(vec![order]);
        let (buys, _) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(),
            &orders,
            1_000_000u64,
            0u32,
            U96F32::from_num(1u32),
        );

        assert_eq!(buys.len(), 0);
    });
}

#[test]
fn validate_and_classify_applies_buy_fee_to_net() {
    new_test_ext().execute_with(|| {
        MockTime::set(1_000_000);
        // 1_000_000 ppb = 0.1%
        // amount = 1_000_000_000, fee = 1_000_000, net = 999_000_000
        ProtocolFee::<Test>::put(1_000_000u32);

        let order = make_signed_order(
            AccountKeyring::Alice,
            bob(),
            netuid_1(),
            OrderSide::Buy,
            1_000_000_000u64,
            u64::MAX,     // limit price: accept any price
            2_000_000u64,
        );

        let orders = bounded_orders(vec![order]);
        let (buys, _) = LimitOrders::<Test>::validate_and_classify(
            netuid_1(),
            &orders,
            1_000_000u64,
            1_000_000u32,
            U96F32::from_num(1u32),
        );

        assert_eq!(buys.len(), 1);
        let (_, _, _, gross, net, fee) = &buys[0];
        assert_eq!(*gross, 1_000_000_000u64);
        assert_eq!(*fee, 1_000_000u64);
        assert_eq!(*net, 999_000_000u64);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// distribute_alpha_pro_rata
// ─────────────────────────────────────────────────────────────────────────────
//
// Scenario A – buy-dominant
// ──────────────────────────
// 3 buyers: Alice 300 TAO net, Bob 200 TAO net, Charlie 500 TAO net (total 1000)
// Pool returns 800 alpha; seller alpha passed-through = 200.
// Total alpha pool = 800 + 200 = 1000 alpha.
//
// Pro-rata shares (proportional to each buyer's net TAO):
//   Alice:   1000 * 300 / 1000 = 300 alpha
//   Bob:     1000 * 200 / 1000 = 200 alpha
//   Charlie: 1000 * 500 / 1000 = 500 alpha
//
// Scenario B – sell-dominant
// ───────────────────────────
// 2 buyers: Alice 400 TAO net, Bob 600 TAO net (total 1000)
// Price = 2.0 TAO/alpha → total alpha for buyers = 1000 / 2 = 500 alpha.
//
// Pro-rata shares:
//   Alice:  500 * 400 / 1000 = 200 alpha
//   Bob:    500 * 600 / 1000 = 300 alpha

fn make_buy_entry(
    order_id: H256,
    signer: AccountId32,
    hotkey: AccountId32,
    gross: u64,
    net: u64,
    fee: u64,
) -> (H256, AccountId32, AccountId32, u64, u64, u64) {
    (order_id, signer, hotkey, gross, net, fee)
}

fn bounded_buy_entries(
    v: Vec<(H256, AccountId32, AccountId32, u64, u64, u64)>,
) -> BoundedVec<(H256, AccountId32, AccountId32, u64, u64, u64), ConstU32<64>> {
    BoundedVec::try_from(v).unwrap()
}

fn bounded_sell_entries(
    v: Vec<(H256, AccountId32, AccountId32, u64, u64, u64)>,
) -> BoundedVec<(H256, AccountId32, AccountId32, u64, u64, u64), ConstU32<64>> {
    BoundedVec::try_from(v).unwrap()
}

#[test]
fn distribute_alpha_pro_rata_buy_dominant() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();
        // Pool returned 800 alpha; sell-side passthrough = 200 alpha.
        // Total = 1000 alpha distributed across 3 buyers (300, 200, 500 TAO net).
        // Expected shares: Alice 300, Bob 200, Charlie 500.

        let hotkey = AccountKeyring::Dave.to_account_id();
        let entries = bounded_buy_entries(vec![
            make_buy_entry(H256::repeat_byte(1), alice(), hotkey.clone(), 300, 300, 0),
            make_buy_entry(H256::repeat_byte(2), bob(),   hotkey.clone(), 200, 200, 0),
            make_buy_entry(H256::repeat_byte(3), charlie(), hotkey.clone(), 500, 500, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get(); // reuse as coldkey for brevity
        let pallet_hk = PalletHotkeyAccount::get();

        LimitOrders::<Test>::distribute_alpha_pro_rata(
            &entries,
            800u128,   // actual_out from pool (alpha)
            1_000u128, // total_buy_net (TAO)
            200u128,   // total_sell_net (alpha passthrough)
            &OrderSide::Buy,
            U96F32::from_num(1u32),
            &pallet_acct,
            &pallet_hk,
            netuid_1(),
        )
        .unwrap();

        let transfers = MockSwap::alpha_transfers();
        // 3 transfers expected (one per buyer)
        assert_eq!(transfers.len(), 3);

        // Check each recipient's amount (signer is to_coldkey).
        let alice_amt = transfers.iter().find(|(_, _, to_ck, _, _, _)| to_ck == &alice()).unwrap().5;
        let bob_amt   = transfers.iter().find(|(_, _, to_ck, _, _, _)| to_ck == &bob()).unwrap().5;
        let charlie_amt = transfers.iter().find(|(_, _, to_ck, _, _, _)| to_ck == &charlie()).unwrap().5;

        assert_eq!(alice_amt, 300u64, "Alice should receive 300 alpha");
        assert_eq!(bob_amt, 200u64, "Bob should receive 200 alpha");
        assert_eq!(charlie_amt, 500u64, "Charlie should receive 500 alpha");
    });
}

#[test]
fn distribute_alpha_pro_rata_sell_dominant() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();
        // Price = 2.0 TAO/alpha; buyers have 400 + 600 = 1000 TAO net.
        // Total alpha = 1000 / 2 = 500.
        // Expected: Alice 200 alpha, Bob 300 alpha.

        let hotkey = AccountKeyring::Dave.to_account_id();
        let entries = bounded_buy_entries(vec![
            make_buy_entry(H256::repeat_byte(4), alice(), hotkey.clone(), 400, 400, 0),
            make_buy_entry(H256::repeat_byte(5), bob(),   hotkey.clone(), 600, 600, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();
        let pallet_hk = PalletHotkeyAccount::get();

        LimitOrders::<Test>::distribute_alpha_pro_rata(
            &entries,
            0u128,     // actual_out unused in sell-dominant branch
            1_000u128, // total_buy_net (TAO)
            999u128,   // total_sell_net — doesn't matter for sell-dominant logic
            &OrderSide::Sell,
            U96F32::from_num(2u32), // price = 2 TAO/alpha
            &pallet_acct,
            &pallet_hk,
            netuid_1(),
        )
        .unwrap();

        let transfers = MockSwap::alpha_transfers();
        assert_eq!(transfers.len(), 2);

        let alice_amt = transfers.iter().find(|(_, _, to_ck, _, _, _)| to_ck == &alice()).unwrap().5;
        let bob_amt   = transfers.iter().find(|(_, _, to_ck, _, _, _)| to_ck == &bob()).unwrap().5;

        assert_eq!(alice_amt, 200u64, "Alice should receive 200 alpha");
        assert_eq!(bob_amt, 300u64, "Bob should receive 300 alpha");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// distribute_tao_pro_rata
// ─────────────────────────────────────────────────────────────────────────────
//
// Scenario A – sell-dominant, fee = 0
// ──────────────────────────────────
// 2 sellers: Alice 400 alpha, Bob 600 alpha (total 1000 alpha)
// Price = 2.0 TAO/alpha → sell_tao_equiv: Alice 800, Bob 1200, total 2000
// Pool returned 1200 TAO; buy-side passthrough = 800 TAO. Total = 2000 TAO.
//
// Pro-rata shares (proportional to each seller's TAO-equiv):
//   Alice:  2000 * 800 / 2000 = 800 TAO
//   Bob:    2000 * 1200 / 2000 = 1200 TAO
//
// Scenario B – sell-dominant, fee = 1% (10_000_000 ppb)
// ─────────────────────────────────────────────────────
// Same setup. Fee on gross TAO payout:
//   Alice:  gross 800, fee 8 (1% of 800), net 792 TAO
//   Bob:    gross 1200, fee 12, net 1188 TAO
//
// Scenario C – buy-dominant
// ──────────────────────────
// 2 sellers: Alice 300 alpha, Bob 200 alpha (total 500 alpha)
// Price = 2.0 TAO/alpha → sell_tao_equiv: Alice 600, Bob 400, total 1000.
// (buy-dominant branch) total_tao = total_sell_tao_equiv = 1000.
//
// Shares:
//   Alice:  1000 * 600 / 1000 = 600 TAO
//   Bob:    1000 * 400 / 1000 = 400 TAO

#[test]
fn distribute_tao_pro_rata_sell_dominant_no_fee() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();
        // Price = 2, total_tao = 1200 (pool) + 800 (buy passthrough) = 2000
        // Alice alpha=400 → tao_equiv=800; Bob alpha=600 → tao_equiv=1200.
        // total_sell_tao_equiv = 2000.
        // Shares: Alice 800, Bob 1200.

        let hotkey = AccountKeyring::Dave.to_account_id();
        let entries = bounded_sell_entries(vec![
            make_buy_entry(H256::repeat_byte(6), alice(), hotkey.clone(), 400, 400, 0),
            make_buy_entry(H256::repeat_byte(7), bob(),   hotkey.clone(), 600, 600, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();

        let sell_fee = LimitOrders::<Test>::distribute_tao_pro_rata(
            &entries,
            1_200u128, // actual_out (pool TAO)
            800u128,   // total_buy_net (buy passthrough TAO)
            2_000u128, // total_sell_tao_equiv (Alice 800 + Bob 1200)
            &OrderSide::Sell,
            U96F32::from_num(2u32),
            0u32, // fee_ppb = 0
            &pallet_acct,
            netuid_1(),
        )
        .unwrap();

        let transfers = MockSwap::tao_transfers();
        assert_eq!(transfers.len(), 2);
        let alice_tao = transfers.iter().find(|(_, to, _)| to == &alice()).unwrap().2;
        let bob_tao   = transfers.iter().find(|(_, to, _)| to == &bob()).unwrap().2;

        assert_eq!(alice_tao, 800u64, "Alice should receive 800 TAO");
        assert_eq!(bob_tao, 1_200u64, "Bob should receive 1200 TAO");
        assert_eq!(sell_fee, 0u64, "No fees at 0 ppb");
    });
}

#[test]
fn distribute_tao_pro_rata_sell_dominant_with_fee() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();
        // Same setup as above but fee = 10_000_000 ppb = 1%.
        // Alice gross=800, fee=8, net=792; Bob gross=1200, fee=12, net=1188.
        // Total sell fee = 20.

        let hotkey = AccountKeyring::Dave.to_account_id();
        let entries = bounded_sell_entries(vec![
            make_buy_entry(H256::repeat_byte(8), alice(), hotkey.clone(), 400, 400, 0),
            make_buy_entry(H256::repeat_byte(9), bob(),   hotkey.clone(), 600, 600, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();

        let sell_fee = LimitOrders::<Test>::distribute_tao_pro_rata(
            &entries,
            1_200u128,
            800u128,
            2_000u128,
            &OrderSide::Sell,
            U96F32::from_num(2u32),
            10_000_000u32, // 1% fee
            &pallet_acct,
            netuid_1(),
        )
        .unwrap();

        let transfers = MockSwap::tao_transfers();
        assert_eq!(transfers.len(), 2);
        let alice_tao = transfers.iter().find(|(_, to, _)| to == &alice()).unwrap().2;
        let bob_tao   = transfers.iter().find(|(_, to, _)| to == &bob()).unwrap().2;

        assert_eq!(alice_tao, 792u64, "Alice net after 1% fee on 800");
        assert_eq!(bob_tao, 1_188u64, "Bob net after 1% fee on 1200");
        assert_eq!(sell_fee, 20u64, "total sell fee = 8 + 12");
    });
}

#[test]
fn distribute_tao_pro_rata_buy_dominant() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();
        // Buy-dominant: total_tao = total_sell_tao_equiv = 1000.
        // Alice alpha=300 → tao_equiv=600; Bob alpha=200 → tao_equiv=400.
        // Shares: Alice 600, Bob 400.

        let hotkey = AccountKeyring::Dave.to_account_id();
        let entries = bounded_sell_entries(vec![
            make_buy_entry(H256::repeat_byte(10), alice(), hotkey.clone(), 300, 300, 0),
            make_buy_entry(H256::repeat_byte(11), bob(),   hotkey.clone(), 200, 200, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();

        let sell_fee = LimitOrders::<Test>::distribute_tao_pro_rata(
            &entries,
            0u128,     // actual_out unused in Buy-dominant branch
            0u128,     // total_buy_net unused in Buy-dominant branch
            1_000u128, // total_sell_tao_equiv (total_tao = this in Buy branch)
            &OrderSide::Buy,
            U96F32::from_num(2u32),
            0u32,
            &pallet_acct,
            netuid_1(),
        )
        .unwrap();

        let transfers = MockSwap::tao_transfers();
        assert_eq!(transfers.len(), 2);
        let alice_tao = transfers.iter().find(|(_, to, _)| to == &alice()).unwrap().2;
        let bob_tao   = transfers.iter().find(|(_, to, _)| to == &bob()).unwrap().2;

        assert_eq!(alice_tao, 600u64, "Alice should receive 600 TAO");
        assert_eq!(bob_tao, 400u64, "Bob should receive 400 TAO");
        assert_eq!(sell_fee, 0u64);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// collect_fees
// ─────────────────────────────────────────────────────────────────────────────
//
// Scenario:
// 2 buy orders with fees 50 and 150 TAO → total_buy_fee = 200 TAO.
// sell_fee_tao passed in = 80 TAO.
// Total fee = 280 TAO forwarded to FeeCollector in one transfer.

#[test]
fn collect_fees_forwards_combined_fees_to_collector() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();

        let hotkey = AccountKeyring::Dave.to_account_id();
        // Buy entries carry fee in field index 5.
        let buys = bounded_buy_entries(vec![
            make_buy_entry(H256::repeat_byte(20), alice(), hotkey.clone(), 1_000, 950, 50),
            make_buy_entry(H256::repeat_byte(21), bob(),   hotkey.clone(), 1_500, 1_350, 150),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();

        LimitOrders::<Test>::collect_fees(&buys, 80u64, &pallet_acct);

        let tao_transfers = MockSwap::tao_transfers();
        assert_eq!(tao_transfers.len(), 1, "single transfer to FeeCollector");
        let (from, to, amount) = &tao_transfers[0];
        assert_eq!(from, &pallet_acct, "fee comes from pallet account");
        assert_eq!(to, &FeeCollectorAccount::get(), "fee goes to FeeCollector");
        assert_eq!(*amount, 280u64, "total fee = 200 (buy) + 80 (sell)");
    });
}

#[test]
fn collect_fees_no_transfer_when_zero_fees() {
    new_test_ext().execute_with(|| {
        MockSwap::clear_log();

        // No buy fees, no sell fee.
        let hotkey = AccountKeyring::Dave.to_account_id();
        let buys = bounded_buy_entries(vec![
            make_buy_entry(H256::repeat_byte(22), alice(), hotkey, 1_000, 1_000, 0),
        ]);
        let pallet_acct = PalletHotkeyAccount::get();

        LimitOrders::<Test>::collect_fees(&buys, 0u64, &pallet_acct);

        let tao_transfers = MockSwap::tao_transfers();
        assert_eq!(tao_transfers.len(), 0, "no transfer when total fee is zero");
    });
}
