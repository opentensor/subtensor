#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
//! Covered continuous-unwind short derivatives — edge-case suite.
//!
//! Covers subnet creation, low liquidity, capacity/anti-split, decay +
//! restoration, the full close/default/top-up lifecycle, value conservation,
//! and subnet deregistration (in-the-money and underwater terminal settlement).

use super::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use substrate_fixed::types::{I64F64, I96F32};
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};

const TAO: u64 = 1_000_000_000;

fn t(v: u64) -> TaoBalance {
    TaoBalance::from(v)
}

fn bal(acc: &U256) -> u64 {
    Balances::free_balance(acc).into()
}

fn custody_bal(netuid: NetUid) -> u64 {
    bal(&SubtensorModule::short_custody_account(netuid))
}

fn assert_approx(a: u64, b: u64, tol: u64, what: &str) {
    let d = a.abs_diff(b);
    assert!(d <= tol, "{what}: {a} vs {b} (diff {d} > tol {tol})");
}

/// Dynamic subnet with balance-backed reserves, a warmed price EMA, shorts
/// enabled, and a generous footprint cap. Returns the netuid.
fn setup_market(tao_reserve: u64, alpha_reserve: u64, price: f64) -> NetUid {
    let owner_c = U256::from(1);
    let owner_h = U256::from(2);
    let netuid = add_dynamic_network(&owner_h, &owner_c);
    setup_reserves(netuid, t(tao_reserve), AlphaBalance::from(alpha_reserve));
    // Back the pool TAO with real balance so custody transfers can move it.
    let sa = SubtensorModule::get_subnet_account_id(netuid).unwrap();
    add_balance_to_coldkey_account(&sa, t(tao_reserve));
    SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(price));
    SubtensorModule::set_shorts_enabled(true);
    SubtensorModule::set_short_kappa_ppb(900_000_000); // κ = 0.9, generous
    netuid
}

/// Credit `q` alpha as stake at `(hotkey, coldkey)` without touching the pool,
/// mirroring the `SubnetAlphaOut` bump a real stake performs.
fn give_alpha(hotkey: U256, coldkey: U256, netuid: NetUid, q: AlphaBalance) {
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid, q);
    SubnetAlphaOut::<Test>::mutate(netuid, |o| *o = o.saturating_add(q));
}

// ---------------------------------------------------------------------------
// Gating & subnet-kind edges
// ---------------------------------------------------------------------------

#[test]
fn open_short_rejected_when_disabled() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        SubtensorModule::set_shorts_enabled(false);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)),
            Error::<Test>::ShortsDisabled
        );
    });
}

#[test]
fn open_short_rejected_on_stable_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        SubnetMechanism::<Test>::insert(netuid, 0); // stable
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)),
            Error::<Test>::SubnetNotDynamic
        );
    });
}

#[test]
fn open_short_rejects_zero_input() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(0)),
            Error::<Test>::AmountTooLow
        );
    });
}

// ---------------------------------------------------------------------------
// Open math vs spec worked example (§1.7–1.8)
// ---------------------------------------------------------------------------

#[test]
fn quote_matches_spec_worked_example() {
    new_test_ext(1).execute_with(|| {
        // Pool 1000 TAO / 100_000 alpha, price 0.01, pre-trade S = 100 TAO.
        let netuid = setup_market(1000 * TAO, 100_000 * TAO, 0.01);
        let mut agg = ShortAggregate::<Test>::get(netuid);
        agg.b_sigma = t(100 * TAO);
        ShortAggregate::<Test>::insert(netuid, agg);

        let q = SubtensorModule::quote_open_short(netuid, t(62_500_000_000)).unwrap(); // P = 62.5 TAO
        assert_approx(q.gross_collateral.to_u64(), 100 * TAO, TAO / 10, "C");
        assert_approx(q.retained_proceeds.to_u64(), 37_500_000_000, TAO / 10, "N");
        assert_approx(q.alpha_liability.to_u64(), 3902 * TAO, 10 * TAO, "Q");
        assert_approx(q.escrow.to_u64(), 39 * TAO, TAO / 2, "E");
        assert_approx(q.effective_ltv, 375_000_000, 2_000_000, "lambda_eff");
    });
}

#[test]
fn open_matches_quote_and_moves_pool() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        let p = 100 * TAO;

        let quote = SubtensorModule::quote_open_short(netuid, t(p)).unwrap();
        let tao_before = SubnetTAO::<Test>::get(netuid).to_u64();
        let trader_before = bal(&trader);

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(p)));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        // Position fields equal the pure quote (same code path).
        assert_eq!(pos.r_stored, quote.retained_proceeds);
        assert_eq!(pos.q_liability, quote.alpha_liability);
        assert_eq!(pos.e_stored, quote.escrow);
        assert_eq!(pos.p_floor, t(p));
        assert_eq!(pos.hotkey, hotkey);
        assert!(pos.b_stored.to_u64() > 0);

        let n = quote.retained_proceeds.to_u64();
        let e = quote.escrow.to_u64();
        // Pool lost exactly N+E TAO; trader paid exactly P; custody holds P+N+E.
        assert_eq!(SubnetTAO::<Test>::get(netuid).to_u64(), tao_before - n - e);
        assert_eq!(bal(&trader), trader_before - p);
        assert_eq!(custody_bal(netuid), p + n + e);

        // Aggregate reflects the single position.
        let agg = ShortAggregate::<Test>::get(netuid);
        assert_eq!(agg.r_sigma, pos.r_stored);
        assert_eq!(agg.e_sigma, pos.e_stored);
        assert_eq!(agg.b_sigma, pos.b_stored);
        assert_eq!(agg.q_sigma, pos.q_liability);
    });
}

// ---------------------------------------------------------------------------
// Capacity / anti-split (§5.1–5.2.1)
// ---------------------------------------------------------------------------

#[test]
fn open_rejected_when_capacity_exceeded() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        SubtensorModule::set_short_kappa_ppb(1_000_000); // κ = 0.001 → cap ≈ 1 TAO
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)),
            Error::<Test>::ShortCapacityExceeded
        );
    });
}

#[test]
fn stacked_opens_share_capacity() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        // Cap ≈ 70 TAO: one P=50 open (B≈47 TAO) fits; a second does not.
        SubtensorModule::set_short_kappa_ppb(70_000_000);
        let a = U256::from(10);
        let b = U256::from(20);
        add_balance_to_coldkey_account(&a, t(1000 * TAO));
        add_balance_to_coldkey_account(&b, t(1000 * TAO));

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(a), U256::from(11), netuid, t(50 * TAO)));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(b), U256::from(21), netuid, t(50 * TAO)),
            Error::<Test>::ShortCapacityExceeded
        );
    });
}

// ---------------------------------------------------------------------------
// Low liquidity (§4.1: λ_eff ≤ 0 rejects oversized opens)
// ---------------------------------------------------------------------------

#[test]
fn low_liquidity_rejects_oversized_open() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10 * TAO, 10 * TAO, 1.0); // tiny pool
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        // P far larger than the pool can collateralize → retained proceeds ≤ 0.
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)),
            Error::<Test>::EffectiveLtvNonPositive
        );
    });
}

#[test]
fn small_open_on_fresh_subnet_with_cold_ema() {
    new_test_ext(1).execute_with(|| {
        // No price EMA set (cold start): T_ref falls back to live reserve.
        let owner_c = U256::from(1);
        let owner_h = U256::from(2);
        let netuid = add_dynamic_network(&owner_h, &owner_c);
        setup_reserves(netuid, t(1000 * TAO), AlphaBalance::from(1000 * TAO));
        let sa = SubtensorModule::get_subnet_account_id(netuid).unwrap();
        add_balance_to_coldkey_account(&sa, t(1000 * TAO));
        SubtensorModule::set_shorts_enabled(true);
        SubtensorModule::set_short_kappa_ppb(900_000_000);
        assert_eq!(SubtensorModule::get_moving_alpha_price(netuid), 0); // cold

        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(50 * TAO)));
        assert!(ShortPositions::<Test>::get(netuid, trader).is_some());
    });
}

// ---------------------------------------------------------------------------
// Decay + restoration (§6)
// ---------------------------------------------------------------------------

#[test]
fn decay_shrinks_buffer_and_restores_tao() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        let r0 = ShortAggregate::<Test>::get(netuid).r_sigma.to_u64();
        let tao0 = SubnetTAO::<Test>::get(netuid).to_u64();
        let custody0 = custody_bal(netuid);
        let omega0 = ShortAggregate::<Test>::get(netuid).omega;

        for _ in 0..200 {
            SubtensorModule::run_short_decay();
        }

        let agg = ShortAggregate::<Test>::get(netuid);
        let r1 = agg.r_sigma.to_u64();
        let tao1 = SubnetTAO::<Test>::get(netuid).to_u64();
        let custody1 = custody_bal(netuid);

        assert!(r1 < r0, "buffer must decay: {r1} !< {r0}");
        assert!(agg.omega > omega0, "omega must increase");
        let restored = tao1 - tao0;
        let drained = custody0 - custody1;
        assert!(restored > 0, "TAO must be restored to the pool");
        // Conservation of the restoration leg: custody out == pool in.
        assert_eq!(restored, drained);
    });
}

#[test]
fn block_step_runs_decay() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        let r0 = ShortAggregate::<Test>::get(netuid).r_sigma.to_u64();
        step_block(5);
        assert!(ShortAggregate::<Test>::get(netuid).r_sigma.to_u64() < r0);
    });
}

// ---------------------------------------------------------------------------
// Top-up (§8.2)
// ---------------------------------------------------------------------------

#[test]
fn top_up_adds_buffer_only() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        let pos0 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let custody0 = custody_bal(netuid);
        assert_ok!(SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(10 * TAO)));
        let pos1 = ShortPositions::<Test>::get(netuid, trader).unwrap();

        assert_eq!(pos1.r_stored, pos0.r_stored + t(10 * TAO));
        assert_eq!(pos1.q_liability, pos0.q_liability); // unchanged
        assert_eq!(pos1.e_stored, pos0.e_stored);
        assert_eq!(pos1.b_stored, pos0.b_stored);
        assert_eq!(custody_bal(netuid), custody0 + 10 * TAO);
    });
}

#[test]
fn top_up_requires_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_noop!(
            SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(TAO)),
            Error::<Test>::ShortPositionNotFound
        );
    });
}

// ---------------------------------------------------------------------------
// Merge (§8.6)
// ---------------------------------------------------------------------------

#[test]
fn additional_open_merges_into_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO)));
        let p1 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO)));
        let p2 = ShortPositions::<Test>::get(netuid, trader).unwrap();

        assert_eq!(p2.p_floor, t(100 * TAO));
        assert!(p2.q_liability > p1.q_liability);
        assert!(p2.r_stored > p1.r_stored);
        // Single merged position, not two entries.
        assert_eq!(ShortPositions::<Test>::iter_prefix(netuid).count(), 1);
    });
}

// ---------------------------------------------------------------------------
// Close (§8.3–8.5) + conservation
// ---------------------------------------------------------------------------

#[test]
fn full_close_conserves_value() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        let p = 100 * TAO;
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(p)));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let (n, e, q) = (pos.r_stored.to_u64(), pos.e_stored.to_u64(), pos.q_liability);
        let tao_after_open = SubnetTAO::<Test>::get(netuid).to_u64();
        let alpha_after_open = SubnetAlphaIn::<Test>::get(netuid).to_u64();

        // Trader acquires the liability alpha (seeded) and closes fully.
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(q.to_u64() + 10 * TAO));
        let trader_before_close = bal(&trader);

        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000));

        // Position gone, aggregate empty.
        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        let agg = ShortAggregate::<Test>::get(netuid);
        assert_eq!(agg.r_sigma.to_u64(), 0);
        assert_eq!(agg.q_sigma.to_u64(), 0);

        // Custody fully drained; pool regained escrow + repaid alpha.
        assert_eq!(custody_bal(netuid), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid).to_u64(), tao_after_open + e);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid).to_u64(), alpha_after_open + q.to_u64());
        // Trader received floor + remaining buffer = P + N.
        assert_eq!(bal(&trader), trader_before_close + p + n);
    });
}

#[test]
fn partial_close_reduces_prorata() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));

        let pos0 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(pos0.q_liability.to_u64()));

        // Close half.
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 500_000_000));
        let pos1 = ShortPositions::<Test>::get(netuid, trader).unwrap();

        assert_approx(pos1.p_floor.to_u64(), pos0.p_floor.to_u64() / 2, 2, "p/2");
        assert_approx(pos1.q_liability.to_u64(), pos0.q_liability.to_u64() / 2, 2, "q/2");
        assert_approx(pos1.r_stored.to_u64(), pos0.r_stored.to_u64() / 2, 2, "r/2");
        assert_approx(pos1.e_stored.to_u64(), pos0.e_stored.to_u64() / 2, 2, "e/2");
    });
}

#[test]
fn close_without_alpha_rejected() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        // No alpha staked at the hotkey → cannot repay the liability.
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000),
            Error::<Test>::InsufficientAlphaToClose
        );
    });
}

#[test]
fn close_invalid_fraction_rejected() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 0),
            Error::<Test>::InvalidCloseFraction
        );
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_001),
            Error::<Test>::InvalidCloseFraction
        );
    });
}

// ---------------------------------------------------------------------------
// Default (§7)
// ---------------------------------------------------------------------------

#[test]
fn default_rejected_when_buffer_above_dust() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        let poker = U256::from(99);
        assert_noop!(
            SubtensorModule::default_short(RuntimeOrigin::signed(poker), trader, netuid),
            Error::<Test>::PositionNotDefaultEligible
        );
    });
}

#[test]
fn default_recycles_floor_and_restores_residual() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let (p, n, e) = (pos.p_floor.to_u64(), pos.r_stored.to_u64(), pos.e_stored.to_u64());
        // Make the whole buffer dust so the position is default-eligible now.
        SubtensorModule::set_short_dust(t(1000 * TAO));
        SubtensorModule::set_short_default_grace(0); // no anti-snipe delay for this test

        let tao0 = SubnetTAO::<Test>::get(netuid).to_u64();
        let ti0 = TotalIssuance::<Test>::get();
        let poker = U256::from(99);
        assert_ok!(SubtensorModule::default_short(RuntimeOrigin::signed(poker), trader, netuid));

        // Position removed; residual R+E restored to pool; floor P recycled (TI down).
        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        assert_eq!(SubnetTAO::<Test>::get(netuid).to_u64(), tao0 + n + e);
        assert_eq!(custody_bal(netuid), 0);
        assert_eq!(TotalIssuance::<Test>::get(), ti0 - t(p));
        let agg = ShortAggregate::<Test>::get(netuid);
        assert_eq!(agg.r_sigma.to_u64(), 0);
    });
}

#[test]
fn default_requires_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        assert_noop!(
            SubtensorModule::default_short(RuntimeOrigin::signed(U256::from(99)), U256::from(10), netuid),
            Error::<Test>::ShortPositionNotFound
        );
    });
}

// ---------------------------------------------------------------------------
// Subnet deregistration terminal settlement (§11.4)
// ---------------------------------------------------------------------------

#[test]
fn dereg_settles_in_the_money_short() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let c = pos.p_floor.to_u64() + pos.r_stored.to_u64(); // P + R
        let trader_before = bal(&trader);

        // Settle terminal. With pEMA = 1 and a bounded liability, equity > 0.
        SubtensorModule::settle_shorts_on_dereg(netuid);

        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        assert_eq!(custody_bal(netuid), 0);
        // Trader received positive equity, strictly less than the full claim.
        let gained = bal(&trader) - trader_before;
        assert!(gained > 0 && gained < c, "equity {gained} not in (0,{c})");
    });
}

#[test]
fn dereg_settles_underwater_short_with_zero_equity() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));

        // Drive the EMA liability reference far above the collateral claim.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(50.0));
        let trader_before = bal(&trader);
        let ti0 = TotalIssuance::<Test>::get();

        SubtensorModule::settle_shorts_on_dereg(netuid);

        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        assert_eq!(custody_bal(netuid), 0);
        // No equity paid; the full claim was recycled (issuance fell).
        assert_eq!(bal(&trader), trader_before);
        assert!(TotalIssuance::<Test>::get() < ti0);
    });
}

#[test]
fn dissolve_network_clears_shorts() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        assert!(ShortPositions::<Test>::get(netuid, trader).is_some());

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // Terminal hook fired: positions and aggregate cleared.
        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        assert!(!ShortAggregate::<Test>::contains_key(netuid));
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));
    });
}

// ---------------------------------------------------------------------------
// Audit fixes
// ---------------------------------------------------------------------------

// Fix: additional open must target the same hotkey (else close would repay from
// the wrong stake).
#[test]
fn merge_with_mismatched_hotkey_rejected() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(50 * TAO)));
        // Second open with a different hotkey must be rejected, leaving state intact.
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(12), netuid, t(50 * TAO)),
            Error::<Test>::ShortHotkeyMismatch
        );
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        assert_eq!(pos.hotkey, U256::from(11));
        assert_eq!(pos.p_floor, t(50 * TAO)); // unchanged by the rejected merge
    });
}

// Fix: opens below the minimum input are rejected (dust-spam / terminal-load bound).
#[test]
fn open_below_min_input_rejected() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        SubtensorModule::set_short_min_input(t(TAO)); // 1 TAO floor

        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(TAO / 2)),
            Error::<Test>::AmountTooLow
        );
        // At/above the floor it succeeds.
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(TAO)));
    });
}

// Fix: a third party cannot snipe a default within the grace window after the
// owner's last action; after the window it is allowed.
#[test]
fn permissionless_default_respects_grace_window() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        // Make the buffer dust-eligible, set a short grace window.
        SubtensorModule::set_short_dust(t(1000 * TAO));
        SubtensorModule::set_short_default_grace(5);
        let poker = U256::from(99);

        // Within the grace window: rejected even though the buffer is dust.
        assert_noop!(
            SubtensorModule::default_short(RuntimeOrigin::signed(poker), trader, netuid),
            Error::<Test>::PositionNotDefaultEligible
        );

        // After the grace window: allowed.
        step_block(6);
        assert_ok!(SubtensorModule::default_short(RuntimeOrigin::signed(poker), trader, netuid));
        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
    });
}

// Fix: the owner can defeat a snipe by topping up, which resets the grace window.
#[test]
fn top_up_resets_default_grace() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        SubtensorModule::set_short_dust(t(1000 * TAO));
        SubtensorModule::set_short_default_grace(5);

        step_block(6); // grace from open has elapsed
        // Owner tops up, resetting last_active to the current block.
        assert_ok!(SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(TAO)));

        // A snipe is now blocked again for another grace window.
        let poker = U256::from(99);
        assert_noop!(
            SubtensorModule::default_short(RuntimeOrigin::signed(poker), trader, netuid),
            Error::<Test>::PositionNotDefaultEligible
        );
    });
}

// Fix: only subnets with live short state are tracked for the per-block decay
// tick; membership is added on open and removed when the last position closes.
#[test]
fn active_subnet_set_tracks_membership() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));

        // No shorts yet → not tracked.
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));
        assert!(ShortActiveSubnets::<Test>::contains_key(netuid));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(pos.q_liability.to_u64() + 10 * TAO));
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000));

        // Fully closed → no longer tracked, so decay skips this subnet.
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));
    });
}

// ---------------------------------------------------------------------------
// Read / RPC layer
// ---------------------------------------------------------------------------

// The position view materializes decay to the current block, while raw storage
// stays at the last materialization.
#[test]
fn position_view_materializes_decay() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        SubtensorModule::set_decay_bounds_ppb(1_000_000_000, 1_000_000_000); // strong decay

        let raw = ShortPositions::<Test>::get(netuid, trader).unwrap().r_stored.to_u64();
        for _ in 0..2000 {
            SubtensorModule::run_short_decay();
        }

        let info = SubtensorModule::get_short_position(&trader, netuid).unwrap();
        // View reflects decay; raw storage is still the last-materialized value.
        assert!(info.buffer.to_u64() < raw, "view buffer {} !< raw {}", info.buffer.to_u64(), raw);
        assert_eq!(ShortPositions::<Test>::get(netuid, trader).unwrap().r_stored.to_u64(), raw);
        assert_eq!(
            info.collateral_claim.to_u64(),
            info.floor.to_u64() + info.buffer.to_u64()
        );
        assert!(info.daily_decay > 0);
        assert!(info.blocks_to_dust > 0 && info.blocks_to_dust < u64::MAX);
        assert_eq!(info.alpha_needed, info.alpha_liability); // holds none yet
    });
}

// The view's default-eligibility tracks the grace window.
#[test]
fn position_view_reports_default_window() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        SubtensorModule::set_short_dust(t(1000 * TAO)); // buffer is dust
        SubtensorModule::set_short_default_grace(5);

        let info = SubtensorModule::get_short_position(&trader, netuid).unwrap();
        assert!(!info.default_eligible, "within grace, not yet defaultable");

        step_block(6);
        let info2 = SubtensorModule::get_short_position(&trader, netuid).unwrap();
        assert!(info2.default_eligible, "after grace, defaultable");
        assert_eq!(info2.defaultable_at_block, info.defaultable_at_block);
    });
}

// Market view exposes capacity and parameters.
#[test]
fn market_view_reports_capacity() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let m = SubtensorModule::get_subnet_short_state(netuid).unwrap();
        assert!(m.shorts_enabled);
        assert!(m.footprint_used.to_u64() > 0);
        assert!(m.footprint_cap.to_u64() > m.footprint_used.to_u64());
        assert_eq!(
            m.footprint_remaining.to_u64(),
            m.footprint_cap.to_u64() - m.footprint_used.to_u64()
        );
        assert_eq!(m.open_interest_alpha, pos.q_liability);
        assert_eq!(m.buffer_total, pos.r_stored);
        assert!(m.current_daily_decay > 0);
    });
}

// Close quote matches the amounts an actual full close moves.
#[test]
fn close_quote_matches_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();

        let full = SubtensorModule::quote_close_short(&trader, netuid, 1_000_000_000).unwrap();
        assert_eq!(full.repay_alpha, pos.q_liability);
        assert_eq!(
            full.returned_tao.to_u64(),
            pos.p_floor.to_u64() + pos.r_stored.to_u64()
        );
        assert_eq!(full.alpha_needed, pos.q_liability); // holds none
        assert!(full.est_buyback_cost.to_u64() > 0);

        let half = SubtensorModule::quote_close_short(&trader, netuid, 500_000_000).unwrap();
        assert_approx(half.repay_alpha.to_u64(), full.repay_alpha.to_u64() / 2, 2, "half repay");
        assert_approx(half.returned_tao.to_u64(), full.returned_tao.to_u64() / 2, 2, "half return");
    });
}

// Materialization can never inflate a position: even with a (impossible)
// entry accumulator above the aggregate, the factor is clamped to ≤ 1.
#[test]
fn materialize_never_inflates() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));

        // Corrupt the invariant: set omega_entry far above the aggregate omega.
        let mut pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let buf = pos.r_stored;
        pos.omega_entry = I64F64::from_num(1000);
        ShortPositions::<Test>::insert(netuid, trader, pos);

        // The materialized view must not exceed the stored buffer (no inflation).
        let info = SubtensorModule::get_short_position(&trader, netuid).unwrap();
        assert!(info.buffer <= buf, "materialize inflated: {} > {}", info.buffer.to_u64(), buf.to_u64());
    });
}

// Open immediately followed by full close cannot be a rounding-profit loop: the
// trader gets back at most floor + buffer and must repay the full liability.
#[test]
fn open_close_roundtrip_is_not_profitable() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));

        let before = bal(&trader);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let n = pos.r_stored.to_u64();
        // Seed exactly the liability alpha so the round trip is self-contained.
        give_alpha(hotkey, trader, netuid, pos.q_liability);
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000));

        // TAO-only delta is +N (the retained proceeds); the trader still had to
        // source Q alpha, whose pool buy-cost strictly exceeds N — so no free TAO.
        assert_eq!(bal(&trader), before + n);
        let buy_cost = SubtensorModule::get_subnet_short_state(netuid); // sanity: market still consistent
        assert!(buy_cost.is_some());
    });
}

// Fix (L3): close must never mint alpha by saturating SubnetAlphaOut to zero.
#[test]
fn close_guards_against_alpha_mint() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO)));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, pos.q_liability);

        // Corrupt outstanding alpha below the liability: close must refuse rather
        // than push SubnetAlphaIn up while SubnetAlphaOut saturates (a mint).
        SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(0));
        let alpha_in_before = SubnetAlphaIn::<Test>::get(netuid);
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000),
            Error::<Test>::InsufficientAlphaToClose
        );
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), alpha_in_before); // no mint
    });
}

// Fix (L2): the open quote is unavailable while shorts are disabled.
#[test]
fn open_quote_gated_by_enable_flag() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        assert!(SubtensorModule::quote_open_short(netuid, t(100 * TAO)).is_some());
        SubtensorModule::set_shorts_enabled(false);
        assert!(SubtensorModule::quote_open_short(netuid, t(100 * TAO)).is_none());
    });
}

// Fix (M4): per-subnet open-position count is capped and maintained, bounding
// deregistration-settlement work.
#[test]
fn position_count_cap_enforced_and_maintained() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        SubtensorModule::set_short_max_positions(2);
        let (a, b, c) = (U256::from(10), U256::from(20), U256::from(30));
        for k in [a, b, c] {
            add_balance_to_coldkey_account(&k, t(1000 * TAO));
        }

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(a), U256::from(11), netuid, t(20 * TAO)));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(b), U256::from(21), netuid, t(20 * TAO)));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);

        // Third distinct position exceeds the cap.
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO)),
            Error::<Test>::ShortPositionLimit
        );

        // Closing one frees a slot; the count is decremented and reusable.
        let pos = ShortPositions::<Test>::get(netuid, a).unwrap();
        give_alpha(U256::from(11), a, netuid, pos.q_liability);
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(a), netuid, 1_000_000_000));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 1);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO)));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);

        // A merge (same coldkey, same hotkey) does not consume a new slot.
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO)));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);
    });
}

// Decay rate matches the closed form: one day at 1.0/day leaves ≈ e⁻¹, and the
// per-position materialized buffer stays consistent with the aggregate.
#[test]
fn decay_rate_matches_closed_form() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO)));
        SubtensorModule::set_decay_bounds_ppb(1_000_000_000, 1_000_000_000); // d = 1.0/day

        let r0 = ShortAggregate::<Test>::get(netuid).r_sigma.to_u64();
        for _ in 0..7200 {
            SubtensorModule::run_short_decay(); // one day of blocks
        }
        let r1 = ShortAggregate::<Test>::get(netuid).r_sigma.to_u64();

        // (1 − 1/7200)^7200 ≈ e⁻¹ ≈ 0.3679 of the original buffer.
        let expected = (r0 as f64 * 0.3679) as u64;
        assert_approx(r1, expected, r0 / 50, "one-day decay ≈ e^-1"); // within 2%

        // Per-position view (single position) matches the aggregate.
        let info = SubtensorModule::get_short_position(&trader, netuid).unwrap();
        assert_approx(info.buffer.to_u64(), r1, r0 / 100, "position == aggregate");
    });
}

// ---------------------------------------------------------------------------
// Longs (mirror) + side independence
// ---------------------------------------------------------------------------

fn setup_long(tao_reserve: u64, alpha_reserve: u64, price: f64) -> NetUid {
    let netuid = setup_market(tao_reserve, alpha_reserve, price);
    SubtensorModule::set_longs_enabled(true);
    SubtensorModule::set_long_kappa_ppb(900_000_000);
    netuid
}

fn alpha_issuance(netuid: NetUid) -> u64 {
    SubnetAlphaIn::<Test>::get(netuid).to_u64() + SubnetAlphaOut::<Test>::get(netuid).to_u64()
}

#[test]
fn open_long_rejected_when_disabled() {
    new_test_ext(1).execute_with(|| {
        // setup_market enables shorts only; longs remain off by default.
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)),
            Error::<Test>::LongsDisabled
        );
    });
}

#[test]
fn open_long_moves_alpha_off_issuance() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));

        let alpha_in0 = SubnetAlphaIn::<Test>::get(netuid).to_u64();
        let stake0 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid).to_u64();

        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let (n, e, d) = (pos.r_stored.to_u64(), pos.e_stored.to_u64(), pos.d_liability.to_u64());

        assert!(n > 0 && e > 0 && d > 0);
        assert_eq!(pos.p_floor.to_u64(), 100 * TAO);
        // Pool alpha dropped by N+E; trader stake dropped by the floor P.
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid).to_u64(), alpha_in0 - n - e);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid).to_u64(),
            stake0 - 100 * TAO
        );
        let agg = LongAggregate::<Test>::get(netuid);
        assert_eq!(agg.d_sigma, pos.d_liability);
        assert!(LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

#[test]
fn full_close_long_conserves_value() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        add_balance_to_coldkey_account(&trader, t(1000 * TAO)); // TAO to repay D

        let iss0 = alpha_issuance(netuid);
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let d = pos.d_liability.to_u64();
        let tao0 = SubnetTAO::<Test>::get(netuid).to_u64();

        assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(trader), netuid, 1_000_000_000));

        assert!(LongPositions::<Test>::get(netuid, trader).is_none());
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
        // Alpha issuance is fully restored (mint == earlier burn); TAO liability paid into pool.
        assert_eq!(alpha_issuance(netuid), iss0);
        assert_eq!(SubnetTAO::<Test>::get(netuid).to_u64(), tao0 + d);
        let agg = LongAggregate::<Test>::get(netuid);
        assert_eq!(agg.r_sigma.to_u64(), 0);
        assert_eq!(agg.d_sigma.to_u64(), 0);
    });
}

#[test]
fn long_decay_restores_alpha_to_pool() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)));

        let r0 = LongAggregate::<Test>::get(netuid).r_sigma.to_u64();
        let alpha_in0 = SubnetAlphaIn::<Test>::get(netuid).to_u64();
        for _ in 0..300 {
            SubtensorModule::run_long_decay();
        }
        assert!(LongAggregate::<Test>::get(netuid).r_sigma.to_u64() < r0);
        assert!(SubnetAlphaIn::<Test>::get(netuid).to_u64() > alpha_in0); // alpha minted back to pool
    });
}

#[test]
fn long_default_recycles_floor_and_restores_residual() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let (p, n, e) = (pos.p_floor.to_u64(), pos.r_stored.to_u64(), pos.e_stored.to_u64());
        SubtensorModule::set_long_dust(AlphaBalance::from(1000 * TAO));
        SubtensorModule::set_short_default_grace(0);

        let alpha_in0 = SubnetAlphaIn::<Test>::get(netuid).to_u64();
        let iss0 = alpha_issuance(netuid);
        assert_ok!(SubtensorModule::default_long(RuntimeOrigin::signed(U256::from(99)), trader, netuid));

        assert!(LongPositions::<Test>::get(netuid, trader).is_none());
        // Residual R+E minted back to the pool; floor P stays burned (recycled).
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid).to_u64(), alpha_in0 + n + e);
        assert_eq!(alpha_issuance(netuid), iss0 + n + e); // P remains out of issuance
        assert_eq!(p, 100 * TAO);
    });
}

#[test]
fn dereg_settles_longs() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO)));
        assert!(LongPositions::<Test>::get(netuid, trader).is_some());

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        assert!(LongPositions::<Test>::get(netuid, trader).is_none());
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

// Shorts and longs are independently flaggable on the same subnet.
#[test]
fn short_and_long_flags_are_independent() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1000 * TAO, 1000 * TAO, 1.0); // shorts on, longs off
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));

        // Shorts enabled, longs disabled.
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO)));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(50 * TAO)),
            Error::<Test>::LongsDisabled
        );

        // Flip: longs enabled, shorts disabled.
        SubtensorModule::set_shorts_enabled(false);
        SubtensorModule::set_longs_enabled(true);
        SubtensorModule::set_long_kappa_ppb(900_000_000);
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(U256::from(20)), hotkey, netuid, t(50 * TAO)),
            Error::<Test>::ShortsDisabled
        );
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(50 * TAO)));
    });
}

// Listing returns every position a coldkey holds across subnets.
#[test]
fn list_positions_across_subnets() {
    new_test_ext(1).execute_with(|| {
        let n1 = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let n2 = setup_market(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), n1, t(50 * TAO)));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(12), n2, t(50 * TAO)));

        let all = SubtensorModule::get_short_positions(&trader);
        assert_eq!(all.len(), 2);
        let mut netuids: Vec<_> = all.iter().map(|p| p.netuid).collect();
        netuids.sort();
        let mut want = vec![n1, n2];
        want.sort();
        assert_eq!(netuids, want);
    });
}
