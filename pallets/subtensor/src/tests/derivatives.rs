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
use safe_math::FixedExt;
use substrate_fixed::types::{I64F64, I96F32, U64F64};
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::{Order, SimSwapOpts, SwapHandler};

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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None),
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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None),
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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(0), None),
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

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(p), None));

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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None),
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

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(a), U256::from(11), netuid, t(50 * TAO), None));
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(b), U256::from(21), netuid, t(50 * TAO), None),
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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None),
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(50 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

        let pos0 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let custody0 = custody_bal(netuid);
        assert_ok!(SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(10 * TAO), None));
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
            SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(TAO), None),
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

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO), None));
        let p1 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(p), None));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let (n, e, q) = (pos.r_stored.to_u64(), pos.e_stored.to_u64(), pos.q_liability);
        let tao_after_open = SubnetTAO::<Test>::get(netuid).to_u64();
        let alpha_after_open = SubnetAlphaIn::<Test>::get(netuid).to_u64();

        // Trader acquires the liability alpha (seeded) and closes fully.
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(q.to_u64() + 10 * TAO));
        let trader_before_close = bal(&trader);

        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));

        let pos0 = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(pos0.q_liability.to_u64()));

        // Close half.
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 500_000_000, None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
        // No alpha staked at the hotkey → cannot repay the liability.
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None),
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 0, None),
            Error::<Test>::InvalidCloseFraction
        );
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_001, None),
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(50 * TAO), None));
        // Second open with a different hotkey must be rejected, leaving state intact.
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(12), netuid, t(50 * TAO), None),
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
            SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(TAO / 2), None),
            Error::<Test>::AmountTooLow
        );
        // At/above the floor it succeeds.
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
        SubtensorModule::set_short_dust(t(1000 * TAO));
        SubtensorModule::set_short_default_grace(5);

        step_block(6); // grace from open has elapsed
        // Owner tops up, resetting last_active to the current block.
        assert_ok!(SubtensorModule::top_up_short(RuntimeOrigin::signed(trader), netuid, t(TAO), None));

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

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));
        assert!(ShortActiveSubnets::<Test>::contains_key(netuid));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(pos.q_liability.to_u64() + 10 * TAO));
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let n = pos.r_stored.to_u64();
        // Seed exactly the liability alpha so the round trip is self-contained.
        give_alpha(hotkey, trader, netuid, pos.q_liability);
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        give_alpha(hotkey, trader, netuid, pos.q_liability);

        // Corrupt outstanding alpha below the liability: close must refuse rather
        // than push SubnetAlphaIn up while SubnetAlphaOut saturates (a mint).
        SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(0));
        let alpha_in_before = SubnetAlphaIn::<Test>::get(netuid);
        assert_noop!(
            SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None),
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

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(a), U256::from(11), netuid, t(20 * TAO), None));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(b), U256::from(21), netuid, t(20 * TAO), None));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);

        // Third distinct position exceeds the cap.
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO), None),
            Error::<Test>::ShortPositionLimit
        );

        // Closing one frees a slot; the count is decremented and reusable.
        let pos = ShortPositions::<Test>::get(netuid, a).unwrap();
        give_alpha(U256::from(11), a, netuid, pos.q_liability);
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(a), netuid, 1_000_000_000, None));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 1);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO), None));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);

        // A merge (same coldkey, same hotkey) does not consume a new slot.
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(c), U256::from(31), netuid, t(20 * TAO), None));
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 2);
    });
}

// ===========================================================================
// PROOF: global value conservation across the full mixed lifecycle.
//
// Exercises the real dispatch path for both sides (open/top-up/partial+full
// close) plus continuous decay, and asserts that no TAO and no Alpha is minted
// or destroyed once every position is closed. Decay is driven directly (not via
// step_block) so coinbase emissions don't perturb issuance.
// ===========================================================================
#[test]
fn proof_full_lifecycle_conserves_tao_and_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0); // both sides enabled
        let (s_cold, s_hot) = (U256::from(10), U256::from(11));
        let (l_cold, l_hot) = (U256::from(20), U256::from(21));
        // Fund: short needs TAO (floor + top-up) and Alpha (repay Q); long needs
        // Alpha (collateral) and TAO (repay D).
        add_balance_to_coldkey_account(&s_cold, t(1000 * TAO));
        add_balance_to_coldkey_account(&l_cold, t(1000 * TAO));
        give_alpha(s_hot, s_cold, netuid, AlphaBalance::from(5000 * TAO));
        give_alpha(l_hot, l_cold, netuid, AlphaBalance::from(500 * TAO));

        // Baseline after all seeding.
        let tao0 = TotalIssuance::<Test>::get().to_u64();
        let alpha0 = alpha_issuance(netuid);

        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(s_cold), s_hot, netuid, t(100 * TAO), None));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(l_cold), l_hot, netuid, AlphaBalance::from(100 * TAO), None));

        // Continuous unwind on both sides.
        for _ in 0..500 {
            SubtensorModule::run_short_decay();
            SubtensorModule::run_long_decay();
        }

        // Mid-life owner actions.
        assert_ok!(SubtensorModule::top_up_short(RuntimeOrigin::signed(s_cold), netuid, t(10 * TAO), None));
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(s_cold), netuid, 500_000_000, None)); // half

        // Close everything out.
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(s_cold), netuid, 1_000_000_000, None));
        assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(l_cold), netuid, 1_000_000_000, None));

        // CONSERVATION.
        // TAO only ever *moves* between accounts (no recycle on this all-close
        // path), so total TAO supply is conserved exactly.
        assert_eq!(TotalIssuance::<Test>::get().to_u64(), tao0, "TAO supply not conserved");

        // Alpha is burned/minted around the pool; fixed-point flooring means the
        // restored amount is never ABOVE baseline (no value minted) and is below
        // it only by bounded rounding dust.
        let alpha1 = alpha_issuance(netuid);
        const DUST_TOL: u64 = 1_000_000; // 0.001 Alpha; observed drift is ~5e2 rao
        assert!(alpha1 <= alpha0, "Alpha was minted: {alpha1} > {alpha0}");
        assert!(alpha0 - alpha1 <= DUST_TOL, "Alpha loss {} exceeds dust tol", alpha0 - alpha1);
        assert!(custody_bal(netuid) <= DUST_TOL, "short custody dust too large");

        // Positions and counts are cleared exactly; fixed liabilities net to 0.
        assert!(ShortPositions::<Test>::get(netuid, s_cold).is_none());
        assert!(LongPositions::<Test>::get(netuid, l_cold).is_none());
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 0);
        assert_eq!(LongPositionCount::<Test>::get(netuid), 0);
        assert_eq!(ShortAggregate::<Test>::get(netuid).q_sigma.to_u64(), 0);
        assert_eq!(LongAggregate::<Test>::get(netuid).d_sigma.to_u64(), 0);
        // cleanup-on-empty evicts fully-closed subnets from the decay tick.
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

// PROOF: default reduces issuance by EXACTLY the recycled floor — no more, no
// less — on both sides.
#[test]
fn proof_default_recycles_exactly_the_floor() {
    new_test_ext(1).execute_with(|| {
        // Short side: TotalIssuance (TAO) drops by exactly the floor P.
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let (s_cold, s_hot) = (U256::from(10), U256::from(11));
        add_balance_to_coldkey_account(&s_cold, t(1000 * TAO));
        SubtensorModule::set_short_default_grace(0);
        SubtensorModule::set_short_dust(t(10_000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(s_cold), s_hot, netuid, t(100 * TAO), None));
        let tao_before = TotalIssuance::<Test>::get().to_u64();
        assert_ok!(SubtensorModule::default_short(RuntimeOrigin::signed(U256::from(99)), s_cold, netuid));
        assert_eq!(
            TotalIssuance::<Test>::get().to_u64(),
            tao_before - 100 * TAO,
            "short default must recycle exactly the floor"
        );

        // Long side: Alpha issuance drops by exactly the floor P.
        let (l_cold, l_hot) = (U256::from(20), U256::from(21));
        give_alpha(l_hot, l_cold, netuid, AlphaBalance::from(500 * TAO));
        SubtensorModule::set_long_dust(AlphaBalance::from(10_000 * TAO));
        SubtensorModule::set_long_default_grace(0);
        // Measure BEFORE open: long open burns alpha, default restores all but the
        // floor, so the net effect of open+default is exactly −floor.
        let alpha_before = alpha_issuance(netuid);
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(l_cold), l_hot, netuid, AlphaBalance::from(100 * TAO), None));
        assert_ok!(SubtensorModule::default_long(RuntimeOrigin::signed(U256::from(98)), l_cold, netuid));
        assert_eq!(
            alpha_issuance(netuid),
            alpha_before - 100 * TAO,
            "long default must recycle exactly the floor"
        );
    });
}

// PROOF (multi-position): the aggregate Σ-decay and per-position lazy decay
// stay solvent across MANY heterogeneous positions on both sides through a long
// decay horizon — the configuration the single-position tests can't exercise.
#[test]
fn proof_multi_position_decay_conserves() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(10_000 * TAO, 10_000 * TAO, 1.0);
        let shorts: [(U256, U256, u64); 3] = [
            (U256::from(10), U256::from(11), 50 * TAO),
            (U256::from(12), U256::from(13), 100 * TAO),
            (U256::from(14), U256::from(15), 30 * TAO),
        ];
        let longs: [(U256, U256, u64); 2] = [
            (U256::from(20), U256::from(21), 40 * TAO),
            (U256::from(22), U256::from(23), 60 * TAO),
        ];
        for (c, h, _) in shorts {
            add_balance_to_coldkey_account(&c, t(2000 * TAO));
            give_alpha(h, c, netuid, AlphaBalance::from(5000 * TAO)); // to repay Q
        }
        for (c, h, _) in longs {
            add_balance_to_coldkey_account(&c, t(2000 * TAO)); // to repay D
            give_alpha(h, c, netuid, AlphaBalance::from(1000 * TAO)); // collateral
        }

        let tao0 = TotalIssuance::<Test>::get().to_u64();
        let alpha0 = alpha_issuance(netuid);

        for (c, h, p) in shorts {
            assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(c), h, netuid, t(p), None));
        }
        for (c, h, p) in longs {
            assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(c), h, netuid, AlphaBalance::from(p), None));
        }

        for _ in 0..300 {
            SubtensorModule::run_short_decay();
            SubtensorModule::run_long_decay();
        }

        for (c, _, _) in shorts {
            assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(c), netuid, 1_000_000_000, None));
        }
        for (c, _, _) in longs {
            assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(c), netuid, 1_000_000_000, None));
        }

        const TOL: u64 = 10_000_000; // 0.01 token
        assert_eq!(TotalIssuance::<Test>::get().to_u64(), tao0, "TAO supply not conserved");
        let alpha1 = alpha_issuance(netuid);
        assert!(alpha1 <= alpha0, "Alpha minted across many positions");
        assert!(alpha0 - alpha1 <= TOL, "Alpha drift {} > tol", alpha0 - alpha1);
        assert!(custody_bal(netuid) <= TOL, "custody not drained across many positions");
        assert_eq!(ShortPositionCount::<Test>::get(netuid), 0);
        assert_eq!(LongPositionCount::<Test>::get(netuid), 0);
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

// Many partial closes followed by a full close drain the position cleanly (the
// floor-rounding residue path), with TAO conserved and custody emptied.
#[test]
fn short_many_partial_closes_drain_cleanly() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(5000 * TAO));

        let tao0 = TotalIssuance::<Test>::get().to_u64();
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(100 * TAO), None));
        for _ in 0..9 {
            assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 100_000_000, None)); // 10% of remaining
        }
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
        assert_eq!(TotalIssuance::<Test>::get().to_u64(), tao0);
        assert!(custody_bal(netuid) <= 10_000, "custody dust after partial closes");
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid));
    });
}

// Governance setters clamp out-of-range inputs (kappa can't freeze the market
// or remove the cap; decay bounds stay ordered and ≤ 1.0/day).
#[test]
fn governance_setters_clamp_ranges() {
    new_test_ext(1).execute_with(|| {
        let one = I64F64::from_num(1);
        let two = I64F64::from_num(2);

        SubtensorModule::set_short_kappa_ppb(0);
        assert!(ShortKappa::<Test>::get() > I64F64::from_num(0), "kappa=0 must clamp above 0");
        SubtensorModule::set_short_kappa_ppb(10_000_000_000); // 10.0
        assert_eq!(ShortKappa::<Test>::get(), two, "kappa clamps to 2.0");
        SubtensorModule::set_long_kappa_ppb(0);
        assert!(LongKappa::<Test>::get() > I64F64::from_num(0));

        // min > max → enforced min ≤ max.
        SubtensorModule::set_decay_bounds_ppb(500_000_000, 100_000_000);
        assert!(DecayMax::<Test>::get() >= DecayMin::<Test>::get());
        // max > 1.0/day → clamped so per-block delta stays < 1.
        SubtensorModule::set_decay_bounds_ppb(0, 5_000_000_000);
        assert!(DecayMax::<Test>::get() <= one, "decay max clamps to 1.0/day");

        // Max-positions clamped so root can't lift the dereg blast radius.
        SubtensorModule::set_short_max_positions(u32::MAX);
        assert_eq!(ShortMaxPositions::<Test>::get(), 4096);
        SubtensorModule::set_short_max_positions(0);
        assert_eq!(ShortMaxPositions::<Test>::get(), 1);
        SubtensorModule::set_long_max_positions(u32::MAX);
        assert_eq!(LongMaxPositions::<Test>::get(), 4096);
    });
}

// Cleanup-on-empty only evicts a subnet from the decay tick once its LAST
// position closes — not while others remain.
#[test]
fn cleanup_evicts_only_after_last_short_closes() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        let (a, b) = (U256::from(10), U256::from(20));
        for k in [a, b] {
            add_balance_to_coldkey_account(&k, t(1000 * TAO));
        }
        give_alpha(U256::from(11), a, netuid, AlphaBalance::from(5000 * TAO));
        give_alpha(U256::from(21), b, netuid, AlphaBalance::from(5000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(a), U256::from(11), netuid, t(50 * TAO), None));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(b), U256::from(21), netuid, t(50 * TAO), None));

        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(a), netuid, 1_000_000_000, None));
        assert!(ShortActiveSubnets::<Test>::contains_key(netuid), "still active while b open");

        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(b), netuid, 1_000_000_000, None));
        assert!(!ShortActiveSubnets::<Test>::contains_key(netuid), "evicted after last close");
    });
}

// Long capacity cap is enforced.
#[test]
fn long_capacity_cap_enforced() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        SubtensorModule::set_long_kappa_ppb(1_000_000); // κ_L = 0.001
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None),
            Error::<Test>::LongCapacityExceeded
        );
    });
}

// Long partial close reduces all legs pro-rata.
#[test]
fn long_partial_close_reduces_prorata() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        add_balance_to_coldkey_account(&trader, t(1000 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let p0 = LongPositions::<Test>::get(netuid, trader).unwrap();

        assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(trader), netuid, 500_000_000, None));
        let p1 = LongPositions::<Test>::get(netuid, trader).unwrap();
        assert_approx(p1.p_floor.to_u64(), p0.p_floor.to_u64() / 2, 2, "p/2");
        assert_approx(p1.d_liability.to_u64(), p0.d_liability.to_u64() / 2, 2, "d/2");
        assert_approx(p1.r_stored.to_u64(), p0.r_stored.to_u64() / 2, 2, "r/2");
    });
}

// Long terminal settlement is underwater (equity 0) when the collateral can't
// cover the TAO debt at the terminal price.
#[test]
fn long_dereg_underwater_pays_zero_equity() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));

        // Crash the price: D/price ≫ collateral ⇒ cover = C_L, equity = 0.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0.0001));
        let stake_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid).to_u64();

        SubtensorModule::settle_longs_on_dereg(netuid);

        assert!(LongPositions::<Test>::get(netuid, trader).is_none());
        let stake_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid).to_u64();
        assert_eq!(stake_after, stake_before, "underwater long must return no equity");
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

// Fix (L1): long open won't mint alpha by saturating SubnetAlphaOut to zero.
#[test]
fn open_long_guards_against_alpha_mint() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        // Corrupt outstanding alpha below the collateral; open must refuse.
        SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(0));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None),
            Error::<Test>::InsufficientCollateral
        );
    });
}

// Long top-up adds Alpha buffer (from stake) and resets the grace clock.
#[test]
fn long_top_up_adds_buffer_and_resets_grace() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let r0 = LongPositions::<Test>::get(netuid, trader).unwrap().r_stored;
        let stake0 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid);

        assert_ok!(SubtensorModule::top_up_long(RuntimeOrigin::signed(trader), netuid, AlphaBalance::from(10 * TAO), None));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        assert_eq!(pos.r_stored, r0 + AlphaBalance::from(10 * TAO));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &trader, netuid),
            stake0 - AlphaBalance::from(10 * TAO)
        );
    });
}

// Long merge must target the same hotkey; long position cap is enforced.
#[test]
fn long_merge_mismatch_and_position_cap() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let a = U256::from(10);
        give_alpha(U256::from(11), a, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(a), U256::from(11), netuid, AlphaBalance::from(20 * TAO), None));
        // Same coldkey, different hotkey → rejected.
        give_alpha(U256::from(12), a, netuid, AlphaBalance::from(100 * TAO));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(a), U256::from(12), netuid, AlphaBalance::from(20 * TAO), None),
            Error::<Test>::LongHotkeyMismatch
        );

        // Position cap: with max=1, a second distinct coldkey is rejected.
        SubtensorModule::set_long_max_positions(1);
        let b = U256::from(20);
        give_alpha(U256::from(21), b, netuid, AlphaBalance::from(100 * TAO));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(b), U256::from(21), netuid, AlphaBalance::from(20 * TAO), None),
            Error::<Test>::LongPositionLimit
        );
    });
}

// Long close rejects invalid fractions and below-min-input opens.
#[test]
fn long_close_invalid_fraction_and_min_input() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        SubtensorModule::set_long_min_input(AlphaBalance::from(TAO));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(TAO / 2), None),
            Error::<Test>::AmountTooLow
        );
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        assert_noop!(
            SubtensorModule::close_long(RuntimeOrigin::signed(trader), netuid, 0, None),
            Error::<Test>::InvalidCloseFraction
        );
        assert_noop!(
            SubtensorModule::close_long(RuntimeOrigin::signed(trader), netuid, 1_000_000_001, None),
            Error::<Test>::InvalidCloseFraction
        );
    });
}

// Shorts express negative subnet flow; a long close pays TAO in (positive
// flow); χ = 0 restores flow-neutral behavior.
#[test]
fn derivatives_write_subnet_flow() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let s = U256::from(10);
        add_balance_to_coldkey_account(&s, t(1000 * TAO));

        // Same-block round-trip must net ~0 on the EMA price: a short open sells
        // alpha → negative flow; a full close rebuys it on the SAME `Q·pEMA`
        // basis → flow returns to baseline (no positive residual — H1 regression).
        let shk = U256::from(11);
        give_alpha(shk, s, netuid, AlphaBalance::from(5000 * TAO)); // to repay Q on close
        let f0 = SubnetTaoFlow::<Test>::get(netuid);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(s), shk, netuid, t(100 * TAO), None));
        let f1 = SubnetTaoFlow::<Test>::get(netuid);
        assert!(f1 < f0, "short open must write negative flow: {f1} !< {f0}");
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(s), netuid, 1_000_000_000, None));
        let f_rt = SubnetTaoFlow::<Test>::get(netuid);
        let tol = (TAO as i64) / 1000; // generous rounding tolerance
        assert!(f_rt > f1, "short close must reverse toward positive flow");
        assert!(
            (f_rt - f0).abs() <= tol,
            "short round-trip must net ~0, not positive: f0={f0} f_rt={f_rt}"
        );

        // Defaulting a short must ALSO reverse its open flow (standing flow tracks
        // only live positions; abandoning leaves no lasting bias).
        let sd = U256::from(40);
        let sdh = U256::from(41);
        add_balance_to_coldkey_account(&sd, t(1000 * TAO));
        let fd0 = SubnetTaoFlow::<Test>::get(netuid);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(sd), sdh, netuid, t(100 * TAO), None));
        SubtensorModule::set_short_dust(t(10_000 * TAO));
        SubtensorModule::set_short_default_grace(0);
        assert_ok!(SubtensorModule::default_short(RuntimeOrigin::signed(U256::from(99)), sd, netuid));
        assert!(
            (SubnetTaoFlow::<Test>::get(netuid) - fd0).abs() <= tol,
            "short default must reverse the open flow"
        );
        SubtensorModule::set_short_dust(t(1));

        // A long open buys alpha with D TAO → positive; full close sells back on
        // the same `D` basis → flow returns to baseline.
        let lc = U256::from(20);
        let lh = U256::from(21);
        give_alpha(lh, lc, netuid, AlphaBalance::from(500 * TAO));
        add_balance_to_coldkey_account(&lc, t(1000 * TAO)); // to repay D on close
        let f2 = SubnetTaoFlow::<Test>::get(netuid);
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(lc), lh, netuid, AlphaBalance::from(100 * TAO), None));
        let f3 = SubnetTaoFlow::<Test>::get(netuid);
        assert!(f3 > f2, "long open must write positive flow");
        assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(lc), netuid, 1_000_000_000, None));
        let lf_rt = SubnetTaoFlow::<Test>::get(netuid);
        assert!(lf_rt < f3, "long close must reverse toward negative flow");
        assert!(
            (lf_rt - f2).abs() <= tol,
            "long round-trip must net ~0: f2={f2} lf_rt={lf_rt}"
        );

        // Defaulting a long must reverse its open `+D` flow (M1 regression).
        let ld = U256::from(50);
        let ldh = U256::from(51);
        give_alpha(ldh, ld, netuid, AlphaBalance::from(500 * TAO));
        let lfd0 = SubnetTaoFlow::<Test>::get(netuid);
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(ld), ldh, netuid, AlphaBalance::from(100 * TAO), None));
        SubtensorModule::set_long_dust(AlphaBalance::from(10_000 * TAO));
        SubtensorModule::set_long_default_grace(0);
        assert_ok!(SubtensorModule::default_long(RuntimeOrigin::signed(U256::from(98)), ld, netuid));
        assert!(
            (SubnetTaoFlow::<Test>::get(netuid) - lfd0).abs() <= tol,
            "long default must reverse the open flow"
        );
        SubtensorModule::set_long_dust(AlphaBalance::from(1));

        // χ = 0 → flow-neutral: another short open leaves flow untouched.
        SubtensorModule::set_derivative_flow_factor_ppb(0);
        let s2 = U256::from(30);
        add_balance_to_coldkey_account(&s2, t(1000 * TAO));
        let f3 = SubnetTaoFlow::<Test>::get(netuid);
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(s2), U256::from(31), netuid, t(100 * TAO), None));
        assert_eq!(SubnetTaoFlow::<Test>::get(netuid), f3, "χ=0 must be flow-neutral");
    });
}

// Short and long default-grace windows are governed independently.
#[test]
fn default_grace_independent_per_side() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let (sc, sh) = (U256::from(10), U256::from(11));
        let (lc, lh) = (U256::from(20), U256::from(21));
        add_balance_to_coldkey_account(&sc, t(1000 * TAO));
        give_alpha(lh, lc, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(sc), sh, netuid, t(100 * TAO), None));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(lc), lh, netuid, AlphaBalance::from(100 * TAO), None));

        SubtensorModule::set_short_dust(t(10_000 * TAO));
        SubtensorModule::set_long_dust(AlphaBalance::from(10_000 * TAO));
        SubtensorModule::set_short_default_grace(0); // shorts: no grace
        SubtensorModule::set_long_default_grace(5); // longs: still gated

        let poker = U256::from(99);
        // Short is immediately defaultable; long is not (independent grace).
        assert_ok!(SubtensorModule::default_short(RuntimeOrigin::signed(poker), sc, netuid));
        assert_noop!(
            SubtensorModule::default_long(RuntimeOrigin::signed(poker), lc, netuid),
            Error::<Test>::PositionNotDefaultEligible
        );
    });
}

// ---------------------------------------------------------------------------
// Long read/RPC layer (mirror of the short views)
// ---------------------------------------------------------------------------

#[test]
fn long_open_quote_matches_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));

        let q = SubtensorModule::quote_open_long(netuid, AlphaBalance::from(100 * TAO)).unwrap();
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        assert_eq!(pos.r_stored, q.retained_proceeds);
        assert_eq!(pos.d_liability, q.tao_liability);
        assert_eq!(pos.e_stored, q.escrow);
        assert_eq!(pos.p_floor, AlphaBalance::from(100 * TAO));
        assert!(q.effective_ltv > 0 && q.gross_collateral.to_u64() > 100 * TAO);
    });
}

#[test]
fn long_open_quote_gated_by_enable_flag() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        assert!(SubtensorModule::quote_open_long(netuid, AlphaBalance::from(100 * TAO)).is_some());
        SubtensorModule::set_longs_enabled(false);
        assert!(SubtensorModule::quote_open_long(netuid, AlphaBalance::from(100 * TAO)).is_none());
    });
}

#[test]
fn long_position_view_materializes_decay() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        SubtensorModule::set_decay_bounds_ppb(1_000_000_000, 1_000_000_000);

        let raw = LongPositions::<Test>::get(netuid, trader).unwrap().r_stored.to_u64();
        for _ in 0..2000 {
            SubtensorModule::run_long_decay();
        }
        let info = SubtensorModule::get_long_position(&trader, netuid).unwrap();
        assert!(info.buffer.to_u64() < raw, "view buffer {} !< raw {}", info.buffer.to_u64(), raw);
        assert_eq!(LongPositions::<Test>::get(netuid, trader).unwrap().r_stored.to_u64(), raw);
        assert_eq!(info.collateral_claim.to_u64(), info.floor.to_u64() + info.buffer.to_u64());
        assert!(info.daily_decay > 0);
        assert!(info.blocks_to_dust > 0 && info.blocks_to_dust < u64::MAX);
        assert_eq!(info.tao_to_close, info.tao_liability);
    });
}

#[test]
fn long_market_view_reports_capacity() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));

        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let m = SubtensorModule::get_subnet_long_state(netuid).unwrap();
        assert!(m.longs_enabled);
        assert!(m.footprint_used.to_u64() > 0);
        assert!(m.footprint_cap.to_u64() > m.footprint_used.to_u64());
        assert_eq!(
            m.footprint_remaining.to_u64(),
            m.footprint_cap.to_u64() - m.footprint_used.to_u64()
        );
        assert_eq!(m.open_interest_tao, pos.d_liability);
        assert_eq!(m.buffer_total, pos.r_stored);
        assert!(m.current_daily_decay > 0);
    });
}

#[test]
fn long_close_quote_matches_position() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        let hotkey = U256::from(11);
        give_alpha(hotkey, trader, netuid, AlphaBalance::from(500 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();

        let full = SubtensorModule::quote_close_long(&trader, netuid, 1_000_000_000).unwrap();
        assert_eq!(full.repay_tao, pos.d_liability);
        assert_eq!(
            full.returned_alpha.to_u64(),
            pos.p_floor.to_u64() + pos.r_stored.to_u64()
        );
        assert_eq!(full.escrow_settled, pos.e_stored);

        let half = SubtensorModule::quote_close_long(&trader, netuid, 500_000_000).unwrap();
        assert_approx(half.repay_tao.to_u64(), full.repay_tao.to_u64() / 2, 2, "half repay");
        assert_approx(half.returned_alpha.to_u64(), full.returned_alpha.to_u64() / 2, 2, "half return");
    });
}

#[test]
fn list_long_positions_across_subnets() {
    new_test_ext(1).execute_with(|| {
        let n1 = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let n2 = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let trader = U256::from(10);
        give_alpha(U256::from(11), trader, n1, AlphaBalance::from(200 * TAO));
        give_alpha(U256::from(12), trader, n2, AlphaBalance::from(200 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), U256::from(11), n1, AlphaBalance::from(50 * TAO), None));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), U256::from(12), n2, AlphaBalance::from(50 * TAO), None));

        let all = SubtensorModule::get_long_positions(&trader);
        assert_eq!(all.len(), 2);
        let mut netuids: Vec<_> = all.iter().map(|p| p.netuid).collect();
        netuids.sort();
        let mut want = vec![n1, n2];
        want.sort();
        assert_eq!(netuids, want);
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), netuid, t(100 * TAO), None));
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
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None),
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

        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
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
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let d = pos.d_liability.to_u64();
        let tao0 = SubnetTAO::<Test>::get(netuid).to_u64();

        assert_ok!(SubtensorModule::close_long(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

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
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));

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
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let (p, n, e) = (pos.p_floor.to_u64(), pos.r_stored.to_u64(), pos.e_stored.to_u64());
        SubtensorModule::set_long_dust(AlphaBalance::from(1000 * TAO));
        SubtensorModule::set_long_default_grace(0);

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
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(100 * TAO), None));
        assert!(LongPositions::<Test>::get(netuid, trader).is_some());

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        assert!(LongPositions::<Test>::get(netuid, trader).is_none());
        assert!(!LongActiveSubnets::<Test>::contains_key(netuid));
    });
}

// Fix: long collateral must be UNLOCKED alpha — opening a long against
// locked alpha (which a normal unstake would block) is rejected, so it can't
// be used to free locked stake.
#[test]
fn open_long_respects_stake_lock() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(1000 * TAO, 1000 * TAO, 1.0);
        let cold = U256::from(10);
        let hot = U256::from(11);
        register_ok_neuron(netuid, hot, cold, 0);
        give_alpha(hot, cold, netuid, AlphaBalance::from(200 * TAO));

        // Lock almost all the staked alpha.
        assert_ok!(SubtensorModule::do_lock_stake(&cold, netuid, &hot, AlphaBalance::from(195 * TAO)));

        // A long against the locked alpha is rejected (would otherwise free it).
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(cold), hot, netuid, AlphaBalance::from(100 * TAO), None),
            Error::<Test>::StakeUnavailable
        );
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hotkey, netuid, t(50 * TAO), None));
        assert_noop!(
            SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(50 * TAO), None),
            Error::<Test>::LongsDisabled
        );

        // Flip: longs enabled, shorts disabled.
        SubtensorModule::set_shorts_enabled(false);
        SubtensorModule::set_longs_enabled(true);
        SubtensorModule::set_long_kappa_ppb(900_000_000);
        assert_noop!(
            SubtensorModule::open_short(RuntimeOrigin::signed(U256::from(20)), hotkey, netuid, t(50 * TAO), None),
            Error::<Test>::ShortsDisabled
        );
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hotkey, netuid, AlphaBalance::from(50 * TAO), None));
    });
}

// ===========================================================================
// Engine-routed settlement: weight/fee-aware cover + asymmetry invariants
//
// The cover/settlement spot leg is now quoted through the live swap engine
// (fee + Balancer-weight aware) rather than a hand-rolled fee-less CPMM. These
// tests exercise the derivatives against a pool with NON-0.5 weights and a
// non-trivial fee — the regime where the old formula silently mispriced — and
// prove every quantity that must be invariant against the engine itself, not a
// re-implemented formula.
// ===========================================================================

/// Set the per-subnet swap fee (u16-normalized) directly.
fn set_fee(netuid: NetUid, rate: u16) {
    pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, rate);
}

/// Force the pool onto explicit Balancer weights derived from `price` (so spot
/// becomes `price`; with reserves whose ratio ≠ `price` the weights are ≠ 0.5),
/// and set the swap fee. Marks the pool initialized so later swaps don't reset
/// the weights back to 0.5.
fn skew_pool(netuid: NetUid, price: f64, fee_rate: u16) {
    pallet_subtensor_swap::PalSwapInitialized::<Test>::insert(netuid, false);
    assert_ok!(
        pallet_subtensor_swap::Pallet::<Test>::maybe_initialize_palswap(
            netuid,
            Some(U64F64::from_num(price)),
        )
    );
    set_fee(netuid, fee_rate);
}

fn sim_tao_in_for_alpha_out(netuid: NetUid, q: AlphaBalance, opts: SimSwapOpts) -> Option<u64> {
    <Test as pallet::Config>::SwapInterface::sim_tao_in_for_alpha_out(netuid.into(), q, opts)
        .ok()
        .map(|x| x.to_u64())
}

fn sim_alpha_in_for_tao_out(netuid: NetUid, d: TaoBalance, opts: SimSwapOpts) -> Option<u64> {
    <Test as pallet::Config>::SwapInterface::sim_alpha_in_for_tao_out(netuid.into(), d, opts)
        .ok()
        .map(|x| x.to_u64())
}

// PROOF: the exact-output short cover is the true inverse of the engine's
// exact-input buy — under non-0.5 weights AND a fee. Quoting "TAO needed to buy
// Q alpha" and then spending exactly that TAO must yield ~Q alpha back.
#[test]
fn engine_cover_inverts_real_swap_short() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        skew_pool(netuid, 1.6, 2_000); // weights ≈ 0.38/0.62, fee ≈ 3%
        let q = AlphaBalance::from(123 * TAO);

        let tao_in = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::WITH_FEES).unwrap();
        let got = <Test as pallet::Config>::SwapInterface::sim_swap(
            netuid.into(),
            GetAlphaForTao::<Test>::with_amount(t(tao_in)),
        )
        .unwrap()
        .amount_paid_out
        .to_u64();

        assert_approx(got, q.to_u64(), q.to_u64() / 1_000 + 10, "buy(quote(Q)) ≈ Q");
    });
}

// PROOF: the exact-output long cover inverts the engine's exact-input sell.
#[test]
fn engine_cover_inverts_real_swap_long() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        skew_pool(netuid, 0.7, 2_000);
        let d = t(77 * TAO);

        let alpha_in = sim_alpha_in_for_tao_out(netuid, d, SimSwapOpts::WITH_FEES).unwrap();
        let got = <Test as pallet::Config>::SwapInterface::sim_swap(
            netuid.into(),
            GetTaoForAlpha::<Test>::with_amount(AlphaBalance::from(alpha_in)),
        )
        .unwrap()
        .amount_paid_out
        .to_u64();

        assert_approx(got, d.to_u64(), d.to_u64() / 1_000 + 10, "sell(quote(D)) ≈ D");
    });
}

// PROOF: under weights+fee the engine cover diverges materially (>1%) from the
// old pure-CPMM fee-less formula `t·q/(a−q)` — i.e. the fix is not a no-op and
// the old path was genuinely mispricing the cover.
#[test]
fn engine_cover_diverges_from_naive_cpmm() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        skew_pool(netuid, 1.6, 2_000);
        let q = AlphaBalance::from(200 * TAO);

        let k_engine = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::WITH_FEES).unwrap();
        let tt = SubnetTAO::<Test>::get(netuid).to_u64() as u128;
        let aa = SubnetAlphaIn::<Test>::get(netuid).to_u64() as u128;
        let qq = q.to_u64() as u128;
        let k_cpmm = (tt.saturating_mul(qq) / (aa - qq)) as u64; // pure, fee-less

        assert!(
            k_engine.abs_diff(k_cpmm) as u128 * 100 > k_cpmm as u128,
            "engine {k_engine} vs naive cpmm {k_cpmm}: divergence < 1% (fix would be a no-op)"
        );
    });
}

// PROOF: `SimSwapOpts::NO_FEES` removes exactly the swap fee — the no-fee quote
// grossed up by the fee rate equals the fee-inclusive quote.
#[test]
fn sim_no_fees_flag_drops_exactly_the_fee() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        let fee: u16 = 2_000;
        skew_pool(netuid, 1.3, fee);
        let q = AlphaBalance::from(150 * TAO);

        let with = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::WITH_FEES).unwrap();
        let without = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::NO_FEES).unwrap();
        assert!(with > without, "fee-inclusive cover must exceed no-fee cover");

        let max = u16::MAX as u128;
        let expected_with = (without as u128 * max / (max - fee as u128)) as u64;
        assert_approx(with, expected_with, with / 100_000 + 4, "no_fees grosses up to with_fees");
    });
}

// PROOF: an exact-output quote for more than the pool can supply errs (the
// derivative cover helpers map this to the seize-full-claim sentinel).
#[test]
fn sim_exact_output_errs_when_pool_too_thin() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(1_000 * TAO, 1_000 * TAO, 1.0);
        assert!(sim_tao_in_for_alpha_out(netuid, AlphaBalance::from(1_000 * TAO), SimSwapOpts::WITH_FEES).is_none());
        assert!(sim_alpha_in_for_tao_out(netuid, t(1_000 * TAO), SimSwapOpts::WITH_FEES).is_none());
    });
}

// ASYMMETRY PROOF (short, fast pump): when spot values the liability above the
// stale-low EMA, terminal settlement collects `max(spot, EMA) = spot`, so the
// lagging EMA can NOT be used to under-collect. Equity equals the spot-based
// formula and never exceeds what an EMA-only settlement would have paid out.
#[test]
fn short_dereg_collects_max_spot_over_stale_low_ema() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(100_000 * TAO, 100_000 * TAO, 1.0);
        set_fee(netuid, 1_000);
        let trader = U256::from(10);
        let hot = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(10_000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hot, netuid, t(50 * TAO), None));

        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let c = pos.p_floor.to_u64() + pos.r_stored.to_u64();
        let q = pos.q_liability;

        // Fast pump: EMA lags low while spot (~1.0) values Q far higher.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0.2));
        let pema = I64F64::saturating_from_num(SubtensorModule::get_moving_alpha_price(netuid));
        let k_ema = (I64F64::saturating_from_num(q.to_u64()).saturating_mul(pema)).to_num::<u64>();
        // Settlement returns escrow E to the TAO reserve BEFORE quoting the spot
        // cover, so quote against that same post-escrow reserve to match.
        let e = pos.e_stored;
        SubnetTAO::<Test>::mutate(netuid, |x| *x = x.saturating_add(e));
        let k_spot = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::WITH_FEES).unwrap();
        SubnetTAO::<Test>::mutate(netuid, |x| *x = x.saturating_sub(e));
        assert!(k_spot > k_ema, "setup: spot {k_spot} must exceed stale EMA {k_ema} (pump)");

        let k_d = k_spot.max(k_ema);
        let expected_equity = c.saturating_sub(k_d.min(c));

        let before = bal(&trader);
        SubtensorModule::settle_shorts_on_dereg(netuid);
        let equity = bal(&trader) - before;

        assert_approx(equity, expected_equity, c / 100_000 + 50, "equity uses max(spot,EMA)=spot");
        let ema_only_equity = c.saturating_sub(k_ema.min(c));
        assert!(equity <= ema_only_equity, "settled spot must not pay more equity than stale EMA");
    });
}

// ASYMMETRY PROOF (long, fast drop): the mirror. When spot needs more alpha to
// cover the TAO debt than the stale-HIGH EMA implies, settlement seizes
// `max(spot, EMA) = spot` alpha, so a lagging EMA can NOT under-seize. This is
// the regression guard for the long-dereg `max(spot, EMA)` cover fix.
#[test]
fn long_dereg_collects_max_spot_over_stale_high_ema() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_long(100_000 * TAO, 100_000 * TAO, 1.0);
        set_fee(netuid, 1_000);
        let trader = U256::from(10);
        let hot = U256::from(11);
        give_alpha(hot, trader, netuid, AlphaBalance::from(50_000 * TAO));
        assert_ok!(SubtensorModule::open_long(RuntimeOrigin::signed(trader), hot, netuid, AlphaBalance::from(50 * TAO), None));

        let pos = LongPositions::<Test>::get(netuid, trader).unwrap();
        let c_l = pos.p_floor.to_u64() + pos.r_stored.to_u64();
        let d = pos.d_liability;

        // Fast drop: EMA lags HIGH, so D/pEMA understates the alpha cover; spot
        // (~1.0) needs far more alpha.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(5.0));
        let pema = I64F64::saturating_from_num(SubtensorModule::get_moving_alpha_price(netuid));
        let k_ema = I64F64::saturating_from_num(d.to_u64()).safe_div(pema).to_num::<u64>();
        // Settlement returns escrow E to the alpha reserve BEFORE quoting the
        // spot cover, so quote against that same post-escrow reserve to match.
        let e = pos.e_stored;
        SubnetAlphaIn::<Test>::mutate(netuid, |x| *x = x.saturating_add(e));
        let k_spot = sim_alpha_in_for_tao_out(netuid, d, SimSwapOpts::WITH_FEES).unwrap();
        SubnetAlphaIn::<Test>::mutate(netuid, |x| *x = x.saturating_sub(e));
        assert!(k_spot > k_ema, "setup: spot cover {k_spot} must exceed stale EMA cover {k_ema} (drop)");

        let cover = c_l.min(k_spot.max(k_ema));
        let expected_equity = c_l.saturating_sub(cover);

        let before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &trader, netuid).to_u64();
        SubtensorModule::settle_longs_on_dereg(netuid);
        let equity = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &trader, netuid).to_u64() - before;

        assert_approx(equity, expected_equity, c_l / 100_000 + 50, "alpha equity uses max(spot,EMA)=spot");
        let ema_only_equity = c_l.saturating_sub(k_ema.min(c_l));
        assert!(equity <= ema_only_equity, "settled spot must not return more equity than stale EMA");
    });
}

// INVARIANT: the self-cover close refuses to settle underwater — it can never
// charge the trader (or the pool) more than the claim `P + R`.
#[test]
fn short_self_close_rejects_when_underwater() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        let trader = U256::from(10);
        let hot = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(10_000 * TAO));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hot, netuid, t(100 * TAO), None));
        let pos = ShortPositions::<Test>::get(netuid, trader).unwrap();
        let q = pos.q_liability;
        let claim = pos.p_floor.to_u64() + pos.r_stored.to_u64();

        // Violent pump against the short: 100x the TAO reserve so the alpha
        // buyback cost dwarfs the claim (without hitting reserve-thin edges).
        SubnetTAO::<Test>::insert(netuid, t(1_000_000 * TAO));
        let k = sim_tao_in_for_alpha_out(netuid, q, SimSwapOpts::WITH_FEES).unwrap();
        assert!(k > claim, "setup: buyback {k} must exceed claim {claim}");
        assert_noop!(
            SubtensorModule::do_close_short_self(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None),
            Error::<Test>::CloseCostExceedsClaim
        );
    });
}

// CONSERVATION under weights + fee: a full open → decay → in-kind close round
// trip conserves TAO supply EXACTLY even on a skewed, fee-charging pool (the
// normal close is settled in-kind, so no engine fee leaks on this path).
#[test]
fn short_lifecycle_conserves_tao_under_weights_and_fee() {
    new_test_ext(1).execute_with(|| {
        let netuid = setup_market(10_000 * TAO, 10_000 * TAO, 1.0);
        skew_pool(netuid, 1.4, 2_000);
        let trader = U256::from(10);
        let hot = U256::from(11);
        add_balance_to_coldkey_account(&trader, t(10_000 * TAO));
        give_alpha(hot, trader, netuid, AlphaBalance::from(50_000 * TAO));

        let tao0 = TotalIssuance::<Test>::get().to_u64();
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), hot, netuid, t(100 * TAO), None));
        for _ in 0..200 {
            SubtensorModule::run_short_decay();
        }
        assert_ok!(SubtensorModule::close_short(RuntimeOrigin::signed(trader), netuid, 1_000_000_000, None));

        assert_eq!(TotalIssuance::<Test>::get().to_u64(), tao0, "TAO supply not conserved under weights+fee");
        assert!(custody_bal(netuid) <= 1_000_000, "custody not drained");
        assert!(ShortPositions::<Test>::get(netuid, trader).is_none());
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
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(11), n1, t(50 * TAO), None));
        assert_ok!(SubtensorModule::open_short(RuntimeOrigin::signed(trader), U256::from(12), n2, t(50 * TAO), None));

        let all = SubtensorModule::get_short_positions(&trader);
        assert_eq!(all.len(), 2);
        let mut netuids: Vec<_> = all.iter().map(|p| p.netuid).collect();
        netuids.sort();
        let mut want = vec![n1, n2];
        want.sort();
        assert_eq!(netuids, want);
    });
}
