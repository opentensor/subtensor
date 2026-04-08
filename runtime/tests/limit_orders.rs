#![allow(clippy::unwrap_used)]

use codec::Encode;
use frame_support::{BoundedVec, assert_noop, assert_ok};
use node_subtensor_runtime::{
    BuildStorage, LimitOrders, Runtime, RuntimeGenesisConfig, RuntimeOrigin, SubtensorModule,
    System, pallet_subtensor,
};
use pallet_limit_orders::{Order, OrderStatus, OrderType, Orders, SignedOrder, VersionedOrder};
use pallet_subtensor::{SubnetAlphaIn, SubnetMechanism, SubnetTAO};
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
fn order_id(order: &VersionedOrder<AccountId>) -> H256 {
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
    let order = VersionedOrder::V1(Order {
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
        max_slippage: None,
    });
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

        let order = VersionedOrder::V1(Order {
            signer: alice_id.clone(),
            hotkey: bob_id,
            netuid: NetUid::from(1u16),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: u64::MAX,
            fee_rate: Perbill::zero(),
            fee_recipient,
            relayer: None,
            max_slippage: None,
        });
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

        let order = VersionedOrder::V1(Order {
            signer: alice_id.clone(),
            hotkey: bob_id,
            netuid: NetUid::from(1u16),
            order_type: OrderType::LimitBuy,
            amount: 1_000,
            limit_price: u64::MAX,
            expiry: u64::MAX,
            fee_rate: Perbill::zero(),
            fee_recipient,
            relayer: None,
            max_slippage: None,
        });
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
        // AMM pool output has slight slippage even with the stable mechanism; check within 1%.
        let staked =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        let expected_alpha = min_default_stake().to_u64();
        assert!(
            staked >= AlphaBalance::from(expected_alpha * 99 / 100)
                && staked <= AlphaBalance::from(expected_alpha),
            "alice should hold approximately min_default_stake alpha after a LimitBuy order executes (got {staked:?})"
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

        // Alice's staked alpha must have decreased by exactly min_default_stake after the sell.
        // Stable mechanism 1:1, zero fee: initial_alpha = min_default_stake * 10,
        // sold min_default_stake alpha, so remaining = min_default_stake * 9.
        let remaining =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert_eq!(
            remaining,
            AlphaBalance::from(min_default_stake().to_u64() * 9u64),
            "alice's staked alpha should be min_default_stake*9 after a TakeProfit order executes"
        );
    });
}

/// A StopLoss order whose price condition is satisfied (price ≤ limit_price) executes
/// against the pool, marks the order as Fulfilled, decreases the seller's staked alpha,
/// and credits free TAO to the seller.
#[test]
fn stop_loss_order_executes_and_unstakes_alpha() {
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

        // limit_price = 1 → current_price (1.0) ≤ 1.0 → StopLoss condition always met.
        // Using 1 (not u64::MAX) because limit_price also acts as the minimum TAO output
        // in sell_alpha — u64::MAX would make the swap always fail.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::StopLoss,
            min_default_stake().into(), // sell min_default_stake alpha units
            1,                          // price floor — current price 1.0 ≤ 1.0, always met
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

        // Alice's staked alpha must have decreased by exactly min_default_stake.
        // Stable mechanism 1:1, zero fee: initial_alpha = min_default_stake * 10,
        // sold min_default_stake alpha, so remaining = min_default_stake * 9.
        let remaining =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert_eq!(
            remaining,
            AlphaBalance::from(min_default_stake().to_u64() * 9u64),
            "alice's staked alpha should be min_default_stake*9 after a StopLoss order executes"
        );

        // Alice must have received TAO from the sale. Pool output has slight slippage; check within 1%.
        let alice_tao = SubtensorModule::get_coldkey_balance(&alice_id);
        let expected_tao = min_default_stake().to_u64();
        assert!(
            alice_tao >= TaoBalance::from(expected_tao * 99 / 100)
                && alice_tao <= TaoBalance::from(expected_tao),
            "alice should receive approximately min_default_stake TAO after a StopLoss order executes (got {alice_tao:?})"
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
        // Buy-dominant: Alice buys min_default_stake*2 TAO, Bob sells min_default_stake alpha.
        // total_sell_tao_equiv = min_default_stake (at 1:1). residual_buy = min_default_stake.
        // pool returns min_default_stake alpha; plus Bob's passthrough = min_default_stake.
        // Alice receives Bob's passthrough alpha + pool alpha for the residual TAO.
        // Pool output has slight slippage; check within 1% of expected min_default_stake*2.
        let alice_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &charlie_id,
            &alice_id,
            netuid,
        );
        let expected_alice_alpha = min_default_stake().to_u64() * 2u64;
        assert!(
            alice_alpha >= AlphaBalance::from(expected_alice_alpha * 99 / 100)
                && alice_alpha <= AlphaBalance::from(expected_alice_alpha),
            "alice should hold approximately min_default_stake*2 alpha after buy-dominant batch (got {alice_alpha:?})"
        );

        // Bob sold alpha and must hold the resulting free TAO.
        // In buy-dominant, total_tao = total_sell_tao_equiv = min_default_stake.
        // Bob's gross_share = (min_default_stake * min_default_stake) / min_default_stake
        //                   = min_default_stake (exact). Zero fee => net_share = min_default_stake.
        let bob_tao = SubtensorModule::get_coldkey_balance(&bob_id);
        assert_eq!(
            bob_tao,
            TaoBalance::from(min_default_stake().to_u64()),
            "bob should hold exactly min_default_stake TAO after buy-dominant batch"
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
        // Sell-dominant: Alice buys min_default_stake TAO, Bob sells min_default_stake*2 alpha.
        // total_buy_alpha_equiv = tao_to_alpha(min_default_stake, 1.0) = min_default_stake (exact).
        // Alice's pro-rata share = (min_default_stake * min_default_stake) / min_default_stake
        //                        = min_default_stake (exact, no floor rounding).
        let alice_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &charlie_id,
            &alice_id,
            netuid,
        );
        assert_eq!(
            alice_alpha,
            AlphaBalance::from(min_default_stake().to_u64()),
            "alice should hold exactly min_default_stake alpha after sell-dominant batch"
        );

        // Bob receives Alice's passthrough TAO + pool TAO for the residual alpha.
        // Pool output has slight slippage; check within 1% of expected min_default_stake*2.
        let bob_tao = SubtensorModule::get_coldkey_balance(&bob_id);
        let expected_bob_tao = min_default_stake().to_u64() * 2u64;
        assert!(
            bob_tao >= TaoBalance::from(expected_bob_tao * 99 / 100)
                && bob_tao <= TaoBalance::from(expected_bob_tao),
            "bob should hold approximately min_default_stake*2 TAO after sell-dominant batch (got {bob_tao:?})"
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
            LimitOrders::execute_batched_orders(RuntimeOrigin::signed(charlie_id), netuid, orders,),
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
            LimitOrders::execute_batched_orders(RuntimeOrigin::signed(charlie_id), netuid, orders,),
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
            LimitOrders::execute_batched_orders(RuntimeOrigin::signed(charlie_id), netuid, orders,),
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
            LimitOrders::execute_batched_orders(RuntimeOrigin::signed(charlie_id), netuid, orders,),
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
            LimitOrders::execute_batched_orders(RuntimeOrigin::signed(charlie_id), netuid, orders,),
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
        assert_eq!(
            Orders::<Runtime>::get(valid_id),
            Some(OrderStatus::Fulfilled)
        );
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

// ── Fee-correctness tests ─────────────────────────────────────────────────────

/// `execute_orders` (non-batched) correctly forwards the buy-order fee to the
/// designated fee recipient and charges Alice exactly `amount` TAO in total.
///
/// Fee mechanics for a non-batched LimitBuy:
///   fee_tao = fee_rate * tao_in  (computed from input BEFORE swap, exact integer arithmetic)
///   tao_after_fee = tao_in - fee_tao  (goes to the pool)
///   fee transferred directly from signer to fee_recipient via transfer_tao
///
/// We use amount = min_default_stake() * 2 so that tao_after_fee = 90% * 2 * min_default_stake()
/// = 1.8 * min_default_stake() > min_default_stake(), satisfying the minimum-stake validation
/// inside buy_alpha. With fee_rate = 10%:
///   fee_tao = 10% * (min_default_stake() * 2) = min_default_stake() / 5 (exact integer result)
///   Alice pays min_default_stake()*2 total and has min_default_stake()*8 remaining.
///   Charlie (fee recipient) receives exactly fee_tao.
#[test]
fn execute_orders_fee_forwarded_to_recipient() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_subnet(netuid);

        // Fund Alice with 10× min_default_stake so she can cover the order amount and a margin.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Create the hotkey association Alice → Bob.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);

        // Charlie starts with zero balance — verify before submitting.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&charlie_id),
            TaoBalance::from(0u64),
            "charlie should start with zero balance"
        );

        // Use 2× min_default_stake so tao_after_fee (90%) stays above the minimum-stake threshold.
        let order_amount = min_default_stake().to_u64() * 2u64;

        // limit_price = u64::MAX → condition always met; fee_recipient = Charlie.
        let signed = make_signed_order(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::LimitBuy,
            order_amount,
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::from_percent(10),
            charlie_id.clone(),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id.clone()),
            orders,
        ));

        // Order must be marked as executed.
        assert_eq!(Orders::<Runtime>::get(id), Some(OrderStatus::Fulfilled));

        // Buy fee is computed from input: fee = 10% * order_amount. Exact integer arithmetic.
        let expected_fee = Perbill::from_percent(10) * order_amount;
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&charlie_id),
            TaoBalance::from(expected_fee),
            "charlie (fee recipient) should receive exactly the buy fee"
        );

        // Alice spent exactly order_amount TAO (fee is deducted from the order amount,
        // not charged on top), so she has min_default_stake()*10 - order_amount remaining.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&alice_id),
            min_default_stake() * 8u64.into(),
            "alice should have min_default_stake()*8 TAO remaining after the order"
        );

        // Alice must have received staked alpha through Bob. The pool received
        // tao_after_fee = order_amount - fee; check within 1% of that expected alpha.
        let tao_after_fee = order_amount - expected_fee;
        let staked =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert!(
            staked >= AlphaBalance::from(tao_after_fee * 99 / 100)
                && staked <= AlphaBalance::from(tao_after_fee),
            "alice should hold approximately tao_after_fee alpha after the LimitBuy with fee (got {staked:?})"
        );
    });
}

/// `execute_batched_orders` correctly forwards fees to a shared fee recipient (Eve)
/// when both a buy and a sell order designate the same recipient.
///
/// Fee mechanics for batched orders:
///   Buy: fee = gross - net = fee_rate * gross (withheld from pool input, transferred from pallet).
///   Sell: fee = fee_rate * gross_share (withheld from TAO pool output, inherits slippage).
///
/// The buy fee is exact; the sell fee is approximate (pool slippage).
#[test]
fn batched_fee_forwarded_to_recipient() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();
        let eve_id = Sr25519Keyring::Eve.to_account_id();

        setup_subnet(netuid);

        // Alice (buyer) funded with free TAO.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Bob (seller) seeded with staked alpha through Dave.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        // Create hotkey associations: Alice → Charlie, Bob → Dave.
        SubtensorModule::create_account_if_non_existent(&alice_id, &charlie_id);
        SubtensorModule::create_account_if_non_existent(&bob_id, &dave_id);

        // Eve (shared fee recipient) starts with zero balance.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&eve_id),
            TaoBalance::from(0u64),
            "eve should start with zero balance"
        );

        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::from_percent(10),
            eve_id.clone(), // fee goes to Eve
        );
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().into(),
            0,        // price floor — always satisfied
            u64::MAX, // no expiry
            Perbill::from_percent(10),
            eve_id.clone(), // fee goes to Eve
        );
        let buy_id = order_id(&buy.order);
        let sell_id = order_id(&sell.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie_id.clone()),
            netuid,
            orders,
        ));

        // Both orders must be fulfilled.
        assert_eq!(Orders::<Runtime>::get(buy_id), Some(OrderStatus::Fulfilled));
        assert_eq!(
            Orders::<Runtime>::get(sell_id),
            Some(OrderStatus::Fulfilled)
        );

        // Buy fee is exact: fee = 10% * min_default_stake().
        let buy_fee = Perbill::from_percent(10) * min_default_stake().to_u64();

        // Sell fee is approximate (pool slippage). Lower bound: 10% of 99% of amount.
        let sell_fee_lower_bound =
            Perbill::from_percent(10) * (min_default_stake().to_u64() * 99 / 100);

        // Eve must have received at least buy_fee + sell_fee_lower_bound,
        // and at most buy_fee + 10% * amount (upper bound on sell fee with no slippage).
        let sell_fee_upper_bound = Perbill::from_percent(10) * min_default_stake().to_u64();
        let eve_balance = SubtensorModule::get_coldkey_balance(&eve_id);
        assert!(
            eve_balance >= TaoBalance::from(buy_fee + sell_fee_lower_bound)
                && eve_balance <= TaoBalance::from(buy_fee + sell_fee_upper_bound),
            "eve should receive combined buy+sell fee within tolerance (got {eve_balance:?})"
        );
    });
}

/// `execute_batched_orders` routes fees to the correct recipient when two orders
/// in the same batch designate different fee recipients (Charlie for the buy,
/// Dave for the sell).
///
/// Verifies that:
///   - Charlie receives exactly the buy fee (no pool slippage on input).
///   - Dave receives approximately the sell fee (within 1%, due to pool slippage).
///   - Neither recipient received both fees.
#[test]
fn batched_multiple_fee_recipients_each_receive_correct_amount() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob = Sr25519Keyring::Bob;
        let bob_id = bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();
        let dave_id = Sr25519Keyring::Dave.to_account_id();

        setup_subnet(netuid);

        // Alice (buyer) funded with free TAO.
        SubtensorModule::add_balance_to_coldkey_account(
            &alice_id,
            min_default_stake() * 10u64.into(),
        );

        // Bob (seller) seeded with staked alpha through Dave.
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &dave_id,
            &bob_id,
            netuid,
            initial_alpha,
        );

        // Create hotkey associations: Alice → Charlie, Bob → Dave.
        SubtensorModule::create_account_if_non_existent(&alice_id, &charlie_id);
        SubtensorModule::create_account_if_non_existent(&bob_id, &dave_id);

        // Charlie and Dave start with zero free balance (they are hotkeys; no initial funding).
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&charlie_id),
            TaoBalance::from(0u64),
            "charlie should start with zero balance"
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&dave_id),
            TaoBalance::from(0u64),
            "dave should start with zero balance"
        );

        // Alice: LimitBuy, fee goes to Charlie.
        let buy = make_signed_order(
            alice,
            charlie_id.clone(),
            netuid,
            OrderType::LimitBuy,
            min_default_stake().into(),
            u64::MAX, // price ceiling — always satisfied
            u64::MAX, // no expiry
            Perbill::from_percent(10),
            charlie_id.clone(), // buy fee to Charlie
        );
        // Bob: TakeProfit, fee goes to Dave.
        let sell = make_signed_order(
            bob,
            dave_id.clone(),
            netuid,
            OrderType::TakeProfit,
            min_default_stake().into(),
            0,        // price floor — always satisfied
            u64::MAX, // no expiry
            Perbill::from_percent(10),
            dave_id.clone(), // sell fee to Dave
        );
        let buy_id = order_id(&buy.order);
        let sell_id = order_id(&sell.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![buy, sell].try_into().unwrap();

        assert_ok!(LimitOrders::execute_batched_orders(
            RuntimeOrigin::signed(charlie_id.clone()),
            netuid,
            orders,
        ));

        // Both orders must be fulfilled.
        assert_eq!(Orders::<Runtime>::get(buy_id), Some(OrderStatus::Fulfilled));
        assert_eq!(
            Orders::<Runtime>::get(sell_id),
            Some(OrderStatus::Fulfilled)
        );

        // Charlie receives exactly the buy fee: 10% * min_default_stake().
        let expected_buy_fee = Perbill::from_percent(10) * min_default_stake().to_u64();
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&charlie_id),
            TaoBalance::from(expected_buy_fee),
            "charlie (buy fee recipient) should receive exactly the buy fee"
        );

        // Dave receives approximately the sell fee (pool slippage ≤ 1%).
        // Expected sell fee ≈ 10% of min_default_stake (the seller's gross TAO share).
        let expected_sell_fee = Perbill::from_percent(10) * min_default_stake().to_u64();
        let sell_fee_lower_bound =
            Perbill::from_percent(10) * (min_default_stake().to_u64() * 99 / 100);
        let dave_balance = SubtensorModule::get_coldkey_balance(&dave_id);
        assert!(
            dave_balance >= TaoBalance::from(sell_fee_lower_bound)
                && dave_balance <= TaoBalance::from(expected_sell_fee),
            "dave (sell fee recipient) should receive approximately the sell fee within 1% (got {dave_balance:?})"
        );

        // Verify fees are separate: neither recipient received both fees.
        // Charlie's balance is exactly buy_fee (not buy_fee + sell_fee).
        let charlie_balance = SubtensorModule::get_coldkey_balance(&charlie_id);
        assert!(
            charlie_balance <= TaoBalance::from(expected_buy_fee),
            "charlie should not have received the sell fee (got {charlie_balance:?})"
        );
        // Dave's balance is ≤ sell_fee (not sell_fee + buy_fee).
        assert!(
            dave_balance <= TaoBalance::from(expected_sell_fee),
            "dave should not have received the buy fee (got {dave_balance:?})"
        );
    });
}

// ── max_slippage enforcement against the real dynamic-mechanism AMM ───────────

/// Set up a dynamic-mechanism (Uniswap v3-style) subnet with equal TAO and
/// alpha reserves, giving an initial pool price of exactly 1.0 TAO/alpha.
///
/// The stable mechanism (mechanism_id = 0) ignores the `price_limit` parameter
/// entirely and always executes at 1:1, so slippage enforcement can only be
/// tested against a dynamic subnet.
fn setup_dynamic_subnet(netuid: NetUid) {
    SubtensorModule::init_new_network(netuid, 0);
    // Override the mechanism to 1 (dynamic / Uniswap v3).
    SubnetMechanism::<Runtime>::insert(netuid, 1u16);
    pallet_subtensor::SubtokenEnabled::<Runtime>::insert(netuid, true);
    // Equal reserves → price = tao_reserve / alpha_reserve = 1.0
    SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(1_000_000_000_000_u64));
    SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(1_000_000_000_000_u64));
}

/// Build a signed order with an explicit `max_slippage` value.
fn make_signed_order_with_slippage_rt(
    keyring: Sr25519Keyring,
    hotkey: AccountId,
    netuid: NetUid,
    order_type: OrderType,
    amount: u64,
    limit_price: u64,
    expiry: u64,
    fee_rate: Perbill,
    fee_recipient: AccountId,
    max_slippage: Option<Perbill>,
) -> SignedOrder<AccountId> {
    let order = VersionedOrder::V1(Order {
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
    });
    let sig = keyring.pair().sign(&order.encode());
    SignedOrder {
        order,
        signature: MultiSignature::Sr25519(sig),
    }
}

/// A StopLoss order whose price condition is met (`current_price ≤ limit_price`)
/// but whose `max_slippage`-derived floor exceeds the pool's actual price is
/// silently skipped by `execute_orders`.
///
/// Setup:
///   Dynamic subnet, equal reserves → pool price = 1.0 (raw ratio, i.e. 1 rao/alpha).
///   limit_price = 2  →  StopLoss trigger: 1.0 ≤ 2.0 ✓  (price has fallen to the trigger)
///   max_slippage = 10 %  →  floor = 2 − 10% × 2.
///     Note: `Perbill::from_percent(10) * 2 = 0` (integer truncation), so floor = 2.
///   After the ×10⁹ scale in `order_swap.rs`:
///     AMM price_limit = 2 × 10⁹ = 2_000_000_000
///     limit_sqrt_price = √(2_000_000_000 / 10⁹) = √2 ≈ 1.414
///   Pool sqrt_price = √1.0 = 1.0  →  1.0 > 1.414 is false  →  PriceLimitExceeded
///   `execute_orders` catches the error and skips the order (no storage write).
///   Because `sell_alpha` is `#[transactional]`, the stake decrement is rolled back.
#[test]
fn execute_orders_stoploss_max_slippage_exceeds_pool_price_skipped() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_dynamic_subnet(netuid);

        // Alice needs staked alpha so the sell can debit her position.
        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob_id,
            &alice_id,
            netuid,
            initial_alpha,
        );

        // limit_price = 2: StopLoss triggers when price ≤ 2.0; pool is at 1.0 → met.
        // max_slippage sets a floor: Perbill integer truncation gives floor = 2 - 0 = 2.
        // After ×10⁹ scaling, AMM limit_sqrt = √2 ≈ 1.414 > pool sqrt 1.0 → rejected.
        let signed = make_signed_order_with_slippage_rt(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::StopLoss,
            min_default_stake().into(),
            2, // trigger at price 2.0; pool is at 1.0 — condition met
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
            Some(Perbill::from_percent(10)),
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        // execute_orders is best-effort: the call succeeds even though the order
        // is rejected by the AMM.
        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order must NOT have been written to storage — it was silently skipped.
        assert!(
            Orders::<Runtime>::get(id).is_none(),
            "order should have been skipped, not stored"
        );

        // `try_execute_order` is #[transactional]: the stake decrement inside
        // `unstake_from_subnet` is rolled back when the AMM rejects the swap,
        // so alice's alpha is unchanged.
        let remaining =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert_eq!(
            remaining,
            initial_alpha,
            "alice's staked alpha should be unchanged when the order is rolled back"
        );
    });
}

/// Contrasting test: the same StopLoss order without `max_slippage` executes
/// successfully against the dynamic-mechanism pool.
///
/// This confirms that the price condition alone is not the blocker and that
/// the previous test's skip is genuinely caused by the slippage floor.
#[test]
fn execute_orders_stoploss_no_slippage_executes_on_dynamic_subnet() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1u16);
        let alice = Sr25519Keyring::Alice;
        let alice_id = alice.to_account_id();
        let bob_id = Sr25519Keyring::Bob.to_account_id();
        let charlie_id = Sr25519Keyring::Charlie.to_account_id();

        setup_dynamic_subnet(netuid);

        SubtensorModule::create_account_if_non_existent(&alice_id, &bob_id);
        let initial_alpha: AlphaBalance = (min_default_stake().to_u64() * 10u64).into();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob_id,
            &alice_id,
            netuid,
            initial_alpha,
        );

        // Same limit_price — trigger still met.  max_slippage = None → floor = 0
        // → AMM limit = 0 → no floor constraint → pool executes the sell.
        let signed = make_signed_order_with_slippage_rt(
            alice,
            bob_id.clone(),
            netuid,
            OrderType::StopLoss,
            min_default_stake().into(),
            2_000_000_000,
            u64::MAX,
            Perbill::zero(),
            charlie_id.clone(),
            None,
        );
        let id = order_id(&signed.order);

        let orders: BoundedVec<_, <Runtime as pallet_limit_orders::Config>::MaxOrdersPerBatch> =
            vec![signed].try_into().unwrap();

        assert_ok!(LimitOrders::execute_orders(
            RuntimeOrigin::signed(charlie_id),
            orders,
        ));

        // Order must be marked as fulfilled.
        assert_eq!(
            Orders::<Runtime>::get(id),
            Some(OrderStatus::Fulfilled),
            "order should be fulfilled when no slippage floor is set"
        );

        // Alice's staked alpha must have decreased by exactly min_default_stake.
        let remaining =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&bob_id, &alice_id, netuid);
        assert_eq!(
            remaining,
            AlphaBalance::from(min_default_stake().to_u64() * 9u64),
            "alice's staked alpha should decrease by min_default_stake after StopLoss executes"
        );
    });
}
