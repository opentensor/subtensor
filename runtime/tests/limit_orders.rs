#![allow(clippy::unwrap_used)]

use codec::Encode;
use frame_support::{BoundedVec, assert_noop, assert_ok};
use node_subtensor_runtime::{
    BuildStorage, LimitOrders, Runtime, RuntimeGenesisConfig, RuntimeOrigin, SubtensorModule,
    System, pallet_subtensor,
};
use pallet_limit_orders::{Order, OrderStatus, OrderType, Orders, SignedOrder};
use sp_core::{Get, H256, Pair};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{MultiSignature, Perbill};
use subtensor_runtime_common::{AccountId, AlphaBalance, NetUid, TaoBalance, Token};

fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Initialise a subnet so that limit-order execution has a pool to interact with.
///
/// We use the stable mechanism (mechanism_id = 0, the default), which swaps at a
/// fixed 1 TAO : 1 alpha rate without requiring pre-seeded AMM liquidity.
fn setup_subnet(netuid: NetUid) {
    SubtensorModule::init_new_network(netuid, 0);
    pallet_subtensor::SubtokenEnabled::<Runtime>::insert(netuid, true);
}

fn min_default_stake() -> TaoBalance {
    pallet_subtensor::DefaultMinStake::<Runtime>::get()
}
fn order_id(order: &Order<AccountId>) -> H256 {
    H256(sp_io::hashing::blake2_256(&order.encode()))
}

fn make_signed_order(
    keyring: Sr25519Keyring,
    hotkey: AccountId,
    netuid: NetUid,
    order_type: OrderType,
    amount: u64,
    limit_price: u64,
    expiry: u64,
    fee_rate: Perbill,
    fee_recipient: AccountId,
) -> SignedOrder<AccountId> {
    let order = Order {
        signer: keyring.to_account_id(),
        hotkey,
        netuid,
        order_type,
        amount,
        limit_price,
        expiry,
        fee_rate,
        fee_recipient,
    };
    let sig = keyring.pair().sign(&order.encode());
    SignedOrder {
        order,
        signature: MultiSignature::Sr25519(sig),
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Signing and cancelling an order writes the order id to storage as Cancelled
/// and emits OrderCancelled. No subnet or balance setup required.
#[test]
fn cancel_order_works() {
    new_test_ext().execute_with(|| {
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let fee_recipient = Sr25519Keyring::Charlie.to_account_id();

        let order = Order {
            signer: alice_id.clone(),
            hotkey: bob_id,
            netuid: NetUid::from(1u16),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: u64::MAX,
            fee_rate: Perbill::zero(),
            fee_recipient,
        };
        let id = order_id(&order);

        assert_ok!(LimitOrders::cancel_order(
            RuntimeOrigin::signed(alice_id),
            order,
        ));

        assert_eq!(Orders::<Runtime>::get(id), Some(OrderStatus::Cancelled));
    });
}

/// An order signed with an Ed25519 key is rejected at validation time even
/// though the signature itself is cryptographically valid. The order must not
/// appear in the Orders storage map after the batch runs.
#[test]
fn execute_orders_ed25519_signature_rejected() {
    new_test_ext().execute_with(|| {
        let alice_id = Sr25519Keyring::Alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let fee_recipient = Sr25519Keyring::Charlie.to_account_id();

        let order = Order {
            signer: alice_id.clone(),
            hotkey: bob_id,
            netuid: NetUid::from(1u16),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: u64::MAX,
            fee_rate: Perbill::zero(),
            fee_recipient,
        };
        let id = order_id(&order);

        // Sign with ed25519 — valid signature, wrong scheme.
        let ed_pair = sp_core::ed25519::Pair::from_legacy_string("//Alice", None);
        let ed_sig = ed_pair.sign(&order.encode());
        let signed = SignedOrder {
            order,
            signature: MultiSignature::Ed25519(ed_sig),
        };

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(alice_id),
            orders,
        ));

        // Order was silently skipped — nothing written to storage.
        assert!(Orders::<Runtime>::get(id).is_none());
    });
}

/// A LimitBuy order whose price condition is satisfied executes against the pool,
/// marks the order as Fulfilled, and credits staked alpha to the buyer.
#[test]
fn limit_buy_order_executes_and_stakes_alpha() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Fund Alice so buy_alpha can debit her balance.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Create the hot-key association.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        // limit_price = u64::MAX → current_price (1.0) ≤ MAX → condition always met.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(), // default min stake units of TAO to spend
            u64::MAX,                   // price ceiling — always satisfied
            u64::MAX,                   // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order must be marked as executed.
        assert_eq!(Orders::<Runtime>::get(id), Some(OrderStatus::Fulfilled));

        // Alice must now have staked alpha delegated through Bob on this subnet.
        let staked =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert!(
            staked > AlphaBalance::ZERO,
            "alice should hold staked alpha after a LimitBuy order executes"
        );
    });
}

/// A TakeProfit order whose price condition is satisfied executes against the pool,
/// marks the order as Fulfilled, and burns the seller's staked alpha position.
#[test]
fn take_profit_order_executes_and_unstakes_alpha() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Create the hot-key association.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        // Seed Alice with staked alpha through Bob so she has something to sell.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob_id,
            &alice_id,
            netuid,
            initial_alpha,
        );

        // limit_price = 0 → current_price (1.0) ≥ 0 → condition always met.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().into(), // sell min default alpha units
            0,                          // price floor — always satisfied
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order must be marked as executed.
        assert_eq!(Orders::<Runtime>::get(id), Some(OrderStatus::Fulfilled));

        // Alice's staked alpha must have decreased after the sell executes.
        let remaining =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert!(
            remaining < initial_alpha.into(),
            "alice's staked alpha should decrease after a TakeProfit order executes"
        );
    });
}

// ── Batched execution ─────────────────────────────────────────────────────────

/// Buy side (5 000 TAO) exceeds sell side (2 000 alpha ≈ 2 000 TAO at 1:1).
///
/// Residual 3 000 TAO goes to the pool; buyers receive pool alpha + seller passthrough
/// alpha. Sellers receive the passthrough TAO that corresponds to their alpha.
///
/// With the stable mechanism (1 TAO = 1 alpha):
///   • Alice (buyer 5 000 TAO) → 5 000 alpha staked to Dave
///   • Bob  (seller 2 000 α)   → 2 000 free TAO
#[test]
fn batched_buy_dominant_executes_correctly() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();

        setup_subnet(netuid);

        // Alice has free TAO to spend on a buy order.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Seed Bob with staked alph so he has something to sell.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();

        // Bob has staked alpha (through Dave) to sell.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        // Create the hot-key association. Alice-> Charlie, Bob -> Dave
        SubtensorModule::create_account_if_non_existent(&alice_id, &charlie_id);
        SubtensorModule::create_account_if_non_existent(&bob_id, &dave_id);

        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().to_u64() * 2u64,
            u64::MAX,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().into(),
            0,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie_id.clone()),
            netuid,
            orders,
        ));

        // Alice spent TAO and must hold the resulting staked alpha.
        let alice_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &charlie_id,
            &alice_id,
            netuid,
        );
        assert!(
            alice_alpha > AlphaBalance::ZERO,
            "alice should hold staked alpha after buy-dominant batch"
        );

        // Bob sold alpha and must hold the resulting free TAO.
        let bob_tao = SubtensorModule::get_coldkey_balance(&bob_id);
        assert!(
            bob_tao > TaoBalance::ZERO,
            "bob should hold free TAO after buy-dominant batch"
        );
    });
}

/// Sell side (min_default_stake()*2 alpha ≈ min_default_stake()*2 TAO at 1:1) exceeds buy side (min_default_stake() TAO).
///
/// Residual min_default_stake() alpha goes to the pool; sellers receive pool TAO + buyer
/// passthrough TAO. Buyers receive the passthrough alpha corresponding to their TAO.
///
/// With the stable mechanism (1 TAO = 1 alpha):
///   • Alice (buyer min_default_stake() TAO) →  alpha staked to Dave
///   • Bob  (seller min_default_stake()*2 α)   → min_default_stake()*2 free TAO
#[test]
fn batched_sell_dominant_executes_correctly() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();

        setup_subnet(netuid);

        // Create the hot-key association. Alice-> Charlie, Bob -> Dave
        SubtensorModule::create_account_if_non_existent(&alice_id, &charlie_id);
        SubtensorModule::create_account_if_non_existent(&bob_id, &dave_id);

        // Alice has free TAO to spend on a buy order.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Seed Bob with staked alph so he has something to sell.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().to_u64() * 2u64,
            0,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie_id.clone()),
            netuid,
            orders,
        ));

        // Alice spent TAO and must hold the resulting staked alpha.
        let alice_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &charlie_id,
            &alice_id,
            netuid,
        );
        assert!(
            alice_alpha > AlphaBalance::ZERO,
            "alice should hold staked alpha after sell-dominant batch"
        );

        // Bob sold alpha and must hold the resulting free TAO.
        let bob_tao = SubtensorModule::get_coldkey_balance(&bob_id);
        assert!(
            bob_tao > TaoBalance::ZERO,
            "bob should hold free TAO after sell-dominant batch"
        );
    });
}

#[test]
fn batched_fails_if_executing_below_minimum_on_sell() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();

        setup_subnet(netuid);

        // Create the hot-key association. Alice-> Charlie, Bob -> Dave
        SubtensorModule::create_account_if_non_existent(&alice_id, &charlie_id);
        SubtensorModule::create_account_if_non_existent(&bob_id, &dave_id);

        // Alice has free TAO to spend on a buy order.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Seed Bob with staked alph so he has something to sell.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            1u64,
            0,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id.clone()),
                netuid,
                orders,
            ),
            pallet_subtensor::Error::<Runtime>::AmountTooLow
        );
    });
}

#[test]
fn batched_fails_if_executing_without_hot_key_association() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();

        setup_subnet(netuid);

        // Create the hot-key association. Alice is not associating to charlie

        // Alice has free TAO to spend on a buy order.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Seed Bob with staked alph so he has something to sell.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().to_u64() * 2u64,
            0,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id.clone()),
                netuid,
                orders,
            ),
            pallet_subtensor::Error::<Runtime>::HotKeyAccountNotExists
        );
    });
}

/// `execute_batched_orders` fails when the target subnet does not exist.
/// The subnet is never initialised (no `setup_subnet`), so `buy_alpha`
/// returns `SubnetNotExists` during the pool-swap step.
#[test]
fn batched_fails_for_nonexistent_subnet() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(2u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        // Fund Alice so that `transfer_tao` succeeds; the subnet check happens
        // later inside `buy_alpha`.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        let buy = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id),
                netuid,
                orders,
            ),
            pallet_subtensor::Error::<Runtime>::SubnetNotExists
        );
    });
}

/// `execute_batched_orders` fails when the subnet exists but its subtoken is
/// not enabled. The order passes validation (price condition is met) and the
/// TAO transfer succeeds, but `buy_alpha` then returns `SubtokenDisabled`.
#[test]
fn batched_fails_if_subtoken_not_enabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        // Initialise the network but deliberately skip setting SubtokenEnabled.
        SubtensorModule::init_new_network(netuid, 0);

        // Fund Alice so that the TAO transfer in `collect_assets` succeeds.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        let buy = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id),
                netuid,
                orders,
            ),
            pallet_subtensor::Error::<Runtime>::SubtokenDisabled
        );
    });
}

/// An order whose `expiry` is in the past causes `execute_batched_orders` to
/// fail with `OrderExpired`.
#[test]
fn batched_fails_for_expired_order() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Advance the runtime timestamp so that `now_ms` exceeds the order's expiry.
        // `pallet_timestamp::Now` stores milliseconds; set it to 100_000 ms.
        pallet_timestamp::Now::<Runtime>::put(100_000u64);

        // Build an order that expired at 50_000 ms — already in the past.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            50_000, // expiry in ms — before current timestamp of 100_000
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id),
                netuid,
                orders,
            ),
            pallet_limit_orders::Error::<Runtime>::OrderExpired
        );
    });
}

/// An order whose price condition is not met causes `execute_batched_orders` to
/// fail with `PriceConditionNotMet`. A `LimitBuy` with `limit_price = 0`
/// requires `current_price <= 0`; since the stable mechanism prices alpha at
/// 1.0 TAO the condition is never met.
#[test]
fn batched_fails_if_price_condition_not_met() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // limit_price = 0 requires current_price <= 0, but current_price ~= 1.0 → fails.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            0,        // price ceiling of 0 — never satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id),
                netuid,
                orders,
            ),
            pallet_limit_orders::Error::<Runtime>::PriceConditionNotMet
        );
    });
}

/// `execute_batched_orders` fails immediately with `RootNetUidNotAllowed` when
/// called with `netuid = 0` (the root network).
#[test]
fn batched_fails_for_root_netuid() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(0u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        // Fund Alice so the call gets past any balance checks before hitting the root guard.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        let buy = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy].try_into().unwrap();

        assert_noop!(
            LimitOrders::execute_batched_orders(
                RuntimeOrigin::signed(charlie_id),
                netuid,
                orders,
            ),
            pallet_limit_orders::Error::<Runtime>::RootNetUidNotAllowed
        );
    });
}

// ── execute_orders — silent-skip behaviour ────────────────────────────────────

/// `execute_orders` silently skips an expired order: the call returns `Ok`
/// and the order is NOT written to the `Orders` storage map.
#[test]
fn execute_orders_skips_expired_order() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Advance the runtime timestamp so that `now_ms` exceeds the order's expiry.
        pallet_timestamp::Now::<Runtime>::put(100_000u64);

        // Build an order that expired at 50_000 ms — already in the past.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            50_000, // expiry in ms — before current timestamp of 100_000
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        // The call must succeed even though the order is expired.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Expired order silently skipped — nothing written to storage.
        assert!(Orders::<Runtime>::get(id).is_none());
    });
}

/// `execute_orders` processes a mixed batch: the valid order executes and is
/// stored as `Fulfilled`; the expired order is silently skipped and is NOT
/// written to storage.  The call always returns `Ok`.
#[test]
fn execute_orders_valid_and_invalid_mixed() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Fund Alice so that her LimitBuy order can execute.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Create the hotkey association for Alice so buy_alpha succeeds.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        // Timestamp at 100_000 ms — Bob's order (expiry 50_000) will be expired.
        pallet_timestamp::Now::<Runtime>::put(100_000u64);

        // Valid order: LimitBuy with price ceiling always satisfied and no expiry.
        let valid = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );
        // Invalid order: already expired.
        let expired = make_signed_order(
            bob,
            alice_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX,
            50_000, // expiry in ms — before current timestamp of 100_000
            Perbill::zero(),
            charlie_id.clone(),
        );
        let valid_id = order_id(&valid.order);
        let expired_id = order_id(&expired.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![valid, expired].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Valid order executed — stored as Fulfilled.
        assert_eq!(Orders::<Runtime>::get(valid_id), Some(OrderStatus::Fulfilled));
        // Expired order silently skipped — not written to storage.
        assert!(Orders::<Runtime>::get(expired_id).is_none());
    });
}

/// `execute_orders` silently skips an order whose signer has no hotkey
/// association: the call returns `Ok` and the order is NOT written to the
/// `Orders` storage map.
#[test]
fn execute_orders_skips_order_with_unassociated_hotkey() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Fund Alice so that any balance check is not the reason for skipping.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Deliberately do NOT call create_account_if_non_existent — Alice has no
        // hotkey association, so the order should be silently skipped.

        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        // The call must succeed even though the hotkey association is missing.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order was silently skipped — nothing written to storage.
        assert!(Orders::<Runtime>::get(id).is_none());
    });
}

/// `execute_orders` silently skips an order whose amount is below the minimum
/// stake threshold: the call returns `Ok` and the order is NOT written to the
/// `Orders` storage map.
#[test]
fn execute_orders_skips_order_below_minimum_stake() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Fund Alice so that any balance check is not the reason for skipping.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Create the hotkey association so that is not the reason for skipping.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        // amount = 1 is well below min_default_stake(), triggering AmountTooLow.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            1u64,
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        // The call must succeed even though the amount is below the minimum.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order was silently skipped — nothing written to storage.
        assert!(Orders::<Runtime>::get(id).is_none());
    });
}

/// `execute_orders` silently skips an order targeting a subnet that does not
/// exist: the call returns `Ok` and the order is NOT written to the `Orders`
/// storage map.
#[test]
fn execute_orders_skips_order_for_nonexistent_subnet() {
    new_test_ext().execute_with(|| {
        // netuid 2 is not initialised — no setup_subnet call.
        let netuid = NetUid::from(2u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        // Fund Alice so that any balance check is not the reason for skipping.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Create the hotkey association so that is not the reason for skipping.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::zero(),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        // The call must succeed even though the subnet does not exist.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order was silently skipped — nothing written to storage.
        assert!(Orders::<Runtime>::get(id).is_none());
    });
}
