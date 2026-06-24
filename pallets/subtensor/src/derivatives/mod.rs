//! Fixed-liability covered continuous-unwind derivatives (spec v3.6.1).
//!
//! Both sides are implemented and independently gated (`ShortsEnabled` /
//! `LongsEnabled`, both default-off). Shorts live here; the long mirror is in
//! `long.rs`. Both sides expose a symmetric client/RPC read layer (`quote_*`,
//! `get_*` via `DerivativesRuntimeApi`), so the two are equivalent to clients.
//!
//! Custody model. Shorts park floor/buffer/escrow TAO in a dedicated per-subnet
//! custody account; longs have no custody account and instead track parked Alpha
//! via issuance accounting (burned at open, minted back on restore/close). Pool
//! reserves, `TotalStake`, and issuance move in lockstep.
//!
//! Emissions flow. Open and position-end (close/default) write opposite TaoFlow
//! signals on a SINGLE per-side basis, scaled by the governance factor `χ`
//! (`DerivativeFlowFactor`), so a round-trip nets ~0 and standing flow reflects
//! only live positions:
//!   - short: `Q·pEMA` notional — open writes `−χ·Q·pEMA`; close/default write
//!     `+χ·(ρQ)·pEMA`. (Same basis at both ends; EMA price, not spot, so it
//!     can't be flash-manipulated. A nonzero residual survives only if the EMA
//!     moved while open — the realized directional impact.)
//!   - long: `D` TAO liability — open writes `+χ·D`; close/default write `−χ·ρD`.
//! Decay and dereg do not write flow (decay is gradual; dereg removes the
//! ledger). `χ = 0` restores flow-neutral behavior (spec §4.5).
//!
//! Custody solvency invariant. `custody_balance(netuid)` (shorts) and the burned
//! Alpha (longs) equal `Σ materialized (P + R(t) + E(t))` **to within per-block
//! floor rounding**. The aggregate Σ-decay floors faster than the per-position
//! `exp` decay, so the drift is always in the safe direction (custody ≥
//! obligations); residual dust is reclaimed by the terminal sweep at dereg.

use super::*;
use frame_support::traits::tokens::{Fortitude, Precision, Preservation, fungible::Balanced};
use safe_math::FixedExt;
use sp_runtime::traits::AccountIdConversion;
use substrate_fixed::types::I64F64;
use subtensor_runtime_common::Token;

pub mod long;
pub mod types;
pub use types::*;

/// 12s blocks → 7200 per day. Decay rates are pro-rated per block.
const BLOCKS_PER_DAY: u64 = 7200;
/// Bisection tolerance for fixed-point square roots.
fn sqrt_eps() -> I64F64 {
    I64F64::from_num(0.000_000_001)
}

impl<T: Config> Pallet<T> {
    // ---- conversions ----------------------------------------------------

    fn tao_f(t: TaoBalance) -> I64F64 {
        // `saturating_from_num`, not `from_num`: these run in the non-transactional
        // `on_initialize` decay path, so a panic would halt consensus. Saturating
        // is safe (balances are supply-capped well below I64F64's range).
        I64F64::saturating_from_num(t.to_u64())
    }
    fn alpha_f(a: AlphaBalance) -> I64F64 {
        I64F64::saturating_from_num(a.to_u64())
    }
    fn to_tao(x: I64F64) -> TaoBalance {
        TaoBalance::from(x.max(I64F64::from_num(0)).saturating_to_num::<u64>())
    }
    fn to_alpha(x: I64F64) -> AlphaBalance {
        AlphaBalance::from(x.max(I64F64::from_num(0)).saturating_to_num::<u64>())
    }
    fn mul_tao(t: TaoBalance, f: I64F64) -> TaoBalance {
        Self::to_tao(Self::tao_f(t).saturating_mul(f))
    }
    fn mul_alpha(a: AlphaBalance, f: I64F64) -> AlphaBalance {
        Self::to_alpha(Self::alpha_f(a).saturating_mul(f))
    }

    // ---- accounts -------------------------------------------------------

    /// Per-subnet account holding parked derivative TAO (floor + buffer + escrow).
    /// Distinct from the subnet pool account so pool reserves are never polluted.
    pub fn short_custody_account(netuid: NetUid) -> T::AccountId {
        T::SubtensorPalletId::get().into_sub_account_truncating(("shrt", u16::from(netuid)))
    }

    /// Recycle TAO out of the protocol custody account (reduce issuance). Unlike
    /// `recycle_tao`, this does not preserve an existential deposit, so the
    /// custody account can be drained to zero.
    fn recycle_custody_tao(custody: &T::AccountId, amount: TaoBalance) {
        if amount.is_zero() {
            return;
        }
        // Never recycle (and never reduce issuance by) more than is actually
        // held: caps an `Exact` withdraw failure that would desync issuance.
        let amt = Self::get_coldkey_balance(custody).min(amount.into());
        TotalIssuance::<T>::mutate(|ti| *ti = ti.saturating_sub(amt));
        let _ = <T as Config>::Currency::withdraw(
            custody,
            amt,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Force,
        );
    }

    // ---- emissions-flow accounting (spec §4.5) -------------------------

    /// `χ`-scaled TAO amount for TaoFlow writes. `χ = 0` ⇒ flow-neutral.
    fn scale_flow(tao: TaoBalance) -> TaoBalance {
        Self::to_tao(Self::tao_f(tao).saturating_mul(DerivativeFlowFactor::<T>::get()))
    }

    /// Negative TaoFlow for TAO a derivative removes from the subnet pool
    /// (a short open expresses bearish demand on the subnet).
    fn record_derivative_outflow(netuid: NetUid, tao: TaoBalance) {
        let amt = Self::scale_flow(tao);
        if !amt.is_zero() {
            Self::record_tao_outflow(netuid, amt);
        }
    }

    /// Positive TaoFlow for TAO a derivative returns to the subnet pool
    /// (short unwinds, and a long close pays its TAO liability into the pool).
    fn record_derivative_inflow(netuid: NetUid, tao: TaoBalance) {
        let amt = Self::scale_flow(tao);
        if !amt.is_zero() {
            Self::record_tao_inflow(netuid, amt);
        }
    }

    // ---- references (spec §3, §4) --------------------------------------

    /// Conservative TAO reference `T_ref = min(T_live, T_EMA)`, with
    /// `T_EMA = pEMA · A_live` reconstructed from the existing price EMA.
    fn short_t_ref(netuid: NetUid) -> I64F64 {
        let t_live = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        let t_ema = pema.saturating_mul(a_live);
        // A cold price EMA (`pema == 0`, e.g. a freshly created subnet) must not
        // lock the market; fall back to the live reserve until it warms up.
        if t_ema <= I64F64::from_num(0) {
            t_live
        } else {
            t_live.min(t_ema)
        }
    }

    /// Convex decay curve `d(u) = d_min + (d_max − d_min)·u²` (spec §6.2),
    /// shared by both sides (the rate is denomination-agnostic).
    fn decay_curve(u: I64F64) -> I64F64 {
        let dmin = DecayMin::<T>::get();
        let dmax = DecayMax::<T>::get();
        dmin.saturating_add(dmax.saturating_sub(dmin).saturating_mul(u).saturating_mul(u))
    }

    /// Utilization ratio `min(1, S / cap)`.
    fn utilization(s: I64F64, cap: I64F64) -> I64F64 {
        if cap > I64F64::from_num(0) {
            s.safe_div(cap).min(I64F64::from_num(1))
        } else {
            I64F64::from_num(0)
        }
    }

    /// Current short daily decay rate at the live short footprint.
    fn short_daily_decay(netuid: NetUid, b_sigma: TaoBalance) -> I64F64 {
        let cap = ShortKappa::<T>::get().saturating_mul(Self::short_t_ref(netuid));
        Self::decay_curve(Self::utilization(Self::tao_f(b_sigma), cap))
    }

    // ---- open-time math (spec §4.1–4.3, Appendix A.1) -------------------

    /// Solve gross collateral `C` and retained proceeds `N` from input `P`
    /// (spec §4.2). Side-agnostic: `ref_reserve` is `T_ref` for shorts / `A_ref`
    /// for longs, `lambda` the per-side base LTV. Returns `None` if `N ≤ 0`.
    fn solve_collateral(
        p: I64F64,
        ref_reserve: I64F64,
        s: I64F64,
        lambda: I64F64,
    ) -> Option<(I64F64, I64F64)> {
        let t_ref = ref_reserve;
        if t_ref <= I64F64::from_num(0) || lambda <= I64F64::from_num(0) {
            return None;
        }
        let one = I64F64::from_num(1);
        let two = I64F64::from_num(2);
        let four = I64F64::from_num(4);
        // a = λ²/T_ref ; b = 1 − λ + 2λS/T_ref
        let a = lambda.saturating_mul(lambda).safe_div(t_ref);
        let b = one
            .saturating_sub(lambda)
            .saturating_add(two.saturating_mul(lambda).saturating_mul(s).safe_div(t_ref));
        // C = (−b + √(b² + 4aP)) / 2a
        let disc = b
            .saturating_mul(b)
            .saturating_add(four.saturating_mul(a).saturating_mul(p));
        let root = disc.checked_sqrt(sqrt_eps())?;
        let c = root
            .saturating_sub(b)
            .safe_div(two.saturating_mul(a));
        let n = c.saturating_sub(p);
        if n <= I64F64::from_num(0) || c <= I64F64::from_num(0) {
            return None;
        }
        Some((c, n))
    }

    /// Pool fraction `ϕ = (1 − √(1 − 4N/T))/2` (spec §4.3). Returns `None` if the
    /// remove-and-sell-back domain `4N ≤ T` fails.
    fn solve_phi(n: I64F64, t_live: I64F64) -> Option<I64F64> {
        if t_live <= I64F64::from_num(0) {
            return None;
        }
        let one = I64F64::from_num(1);
        let four = I64F64::from_num(4);
        let frac = four.saturating_mul(n).safe_div(t_live);
        if frac > one {
            return None;
        }
        let root = one.saturating_sub(frac).checked_sqrt(sqrt_eps())?;
        Some(one.saturating_sub(root).safe_div(I64F64::from_num(2)))
    }

    /// Keep the active-short-subnet set in sync with the aggregate: a subnet is
    /// tracked iff it still has any live short state. The per-block decay tick
    /// iterates only this set instead of every subnet.
    fn sync_active_short(netuid: NetUid, agg: &ShortAgg) {
        if agg.r_sigma.is_zero()
            && agg.e_sigma.is_zero()
            && agg.b_sigma.is_zero()
            && agg.q_sigma.is_zero()
        {
            ShortActiveSubnets::<T>::remove(netuid);
        } else {
            ShortActiveSubnets::<T>::insert(netuid, ());
        }
    }

    /// `−ln(1 − δ) = δ + δ²/2 + δ³/3 + …` for the small per-block decay `δ`.
    ///
    /// Computed directly from the series rather than `checked_ln(1 − δ)`, which
    /// is imprecise (and can return the wrong sign) for arguments just below 1.
    /// This keeps the aggregate factor `g = 1 − δ` and the per-position factor
    /// `exp(−ΔΩ) = Π g` consistent to within per-block floor rounding (the
    /// 3-term series and `checked_exp`'s 7-term series are both truncations).
    fn neg_ln_one_minus(delta: I64F64) -> I64F64 {
        let d2 = delta.saturating_mul(delta);
        let d3 = d2.saturating_mul(delta);
        delta
            .saturating_add(d2.saturating_mul(I64F64::from_num(0.5)))
            .saturating_add(d3.saturating_mul(I64F64::from_num(1.0 / 3.0)))
    }

    /// When the last position on a subnet closes, drop the aggregate and the
    /// active-set entry so the per-block decay tick stops visiting it (otherwise
    /// floor-rounding dust in `r_sigma` keeps the subnet "active" forever). Any
    /// residual custody dust is reclaimed by the terminal sweep at dereg.
    fn cleanup_short_if_empty(netuid: NetUid) {
        if ShortPositionCount::<T>::get(netuid) == 0 {
            ShortAggregate::<T>::remove(netuid);
            ShortActiveSubnets::<T>::remove(netuid);
        }
    }

    /// Materialize a position to the current accumulator: `f = exp(−(Ω − Ω_entry))`.
    fn materialize_short(pos: &mut ShortPosition<T::AccountId>, omega_now: I64F64) {
        // `Ω` only ever grows, so `arg ≤ 0` and `f ≤ 1` (decay never inflates).
        // The `unwrap_or(0)` below is correct, not a silent failure: a large
        // negative `arg` legitimately decays the buffer toward 0. Clamp `arg ≤ 0`
        // defensively so an (impossible) positive `arg` can't yield `f > 1`.
        let arg = pos
            .omega_entry
            .saturating_sub(omega_now)
            .min(I64F64::from_num(0));
        let f = arg.checked_exp().unwrap_or_else(|| I64F64::from_num(0));
        pos.r_stored = Self::mul_tao(pos.r_stored, f);
        pos.e_stored = Self::mul_tao(pos.e_stored, f);
        pos.b_stored = Self::mul_tao(pos.b_stored, f);
        pos.omega_entry = omega_now;
    }

    // ---- user operations (spec §8) -------------------------------------

    /// Open (or merge into) a covered short (spec §8.1, §8.6).
    pub fn do_open_short(
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
        position_input: TaoBalance,
        limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(ShortsEnabled::<T>::get(), Error::<T>::ShortsDisabled);
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            SubnetMechanism::<T>::get(netuid) == 1,
            Error::<T>::SubnetNotDynamic
        );
        ensure!(
            position_input >= ShortMinInput::<T>::get(),
            Error::<T>::AmountTooLow
        );

        let mut agg = ShortAggregate::<T>::get(netuid);
        let t_ref = Self::short_t_ref(netuid);
        let p = Self::tao_f(position_input);

        let (c, n) = Self::solve_collateral(p, t_ref, Self::tao_f(agg.b_sigma), ShortBaseLtv::<T>::get())
            .ok_or(Error::<T>::EffectiveLtvNonPositive)?;
        let b = ShortBaseLtv::<T>::get().saturating_mul(c);

        // Capacity: S + B ≤ κ_S · T_ref (also bounds same-block stacked opens).
        ensure!(
            Self::tao_f(agg.b_sigma).saturating_add(b) <= ShortKappa::<T>::get().saturating_mul(t_ref),
            Error::<T>::ShortCapacityExceeded
        );

        let t_live = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let phi = Self::solve_phi(n, t_live).ok_or(Error::<T>::ReserveDomainExceeded)?;

        let n_tao = Self::to_tao(n);
        let e_tao = Self::to_tao(phi.saturating_mul(t_live));
        let b_tao = Self::to_tao(b);
        let q_alpha = Self::to_alpha(phi.saturating_mul(a_live));
        ensure!(!n_tao.is_zero(), Error::<T>::RetainedProceedsNonPositive);

        let custody = Self::short_custody_account(netuid);
        let subnet_account =
            Self::get_subnet_account_id(netuid).ok_or(Error::<T>::SubnetNotExists)?;

        // 1. Trader posts floor P into custody (fails early if underfunded).
        Self::transfer_tao(&coldkey, &custody, position_input.into())?;
        // 2. Remove N+E TAO from the pool into custody (the downward price impact).
        let removed = n_tao.saturating_add(e_tao);
        Self::transfer_tao(&subnet_account, &custody, removed.into())?;
        Self::decrease_provided_tao_reserve(netuid, removed);
        TotalStake::<T>::mutate(|t| *t = t.saturating_sub(removed));
        // Bearish flow: the short sells `Q` alpha, marked at the EMA price. Open
        // and close/default use the SAME `Q·pEMA` basis so a round-trip nets ~0
        // (a residual only survives if the EMA price moved while the short was
        // open — i.e. the realized directional impact). EMA, not spot, so it
        // can't be flash-manipulated.
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        Self::record_derivative_outflow(netuid, Self::to_tao(Self::alpha_f(q_alpha).saturating_mul(pema)));

        let block = Self::get_current_block_as_u64();
        let pos = match ShortPositions::<T>::get(netuid, &coldkey) {
            Some(mut existing) => {
                // A merge must target the same hotkey, otherwise the liability
                // alpha repaid on close would be drawn from the wrong stake.
                ensure!(existing.hotkey == hotkey, Error::<T>::ShortHotkeyMismatch);
                Self::materialize_short(&mut existing, agg.omega);
                existing.p_floor = existing.p_floor.saturating_add(position_input);
                existing.q_liability = existing.q_liability.saturating_add(q_alpha);
                existing.r_stored = existing.r_stored.saturating_add(n_tao);
                existing.e_stored = existing.e_stored.saturating_add(e_tao);
                existing.b_stored = existing.b_stored.saturating_add(b_tao);
                existing.last_active = block;
                existing
            }
            None => {
                // New position: enforce and bump the per-subnet position count
                // so deregistration settlement work stays bounded.
                let count = ShortPositionCount::<T>::get(netuid);
                ensure!(
                    count < ShortMaxPositions::<T>::get(),
                    Error::<T>::ShortPositionLimit
                );
                ShortPositionCount::<T>::insert(netuid, count.saturating_add(1));
                ShortPosition {
                    hotkey,
                    p_floor: position_input,
                    q_liability: q_alpha,
                    r_stored: n_tao,
                    e_stored: e_tao,
                    b_stored: b_tao,
                    omega_entry: agg.omega,
                    last_active: block,
                }
            }
        };
        ShortPositions::<T>::insert(netuid, &coldkey, pos);

        agg.r_sigma = agg.r_sigma.saturating_add(n_tao);
        agg.e_sigma = agg.e_sigma.saturating_add(e_tao);
        agg.b_sigma = agg.b_sigma.saturating_add(b_tao);
        agg.q_sigma = agg.q_sigma.saturating_add(q_alpha);
        ShortAggregate::<T>::insert(netuid, agg);
        ShortActiveSubnets::<T>::insert(netuid, ());

        // Slippage guard: a short lowers the price, so reject if it ended up
        // below the caller's floor (sandwich/MEV protection). `None` = no bound.
        Self::ensure_price_at_least(netuid, limit_price)?;

        Self::deposit_event(Event::ShortOpened {
            coldkey,
            netuid,
            position_input,
            retained_proceeds: n_tao,
            alpha_liability: q_alpha,
            escrow: e_tao,
        });
        Ok(())
    }

    /// Top up the carry buffer `R` with fresh capital (spec §8.2).
    pub fn do_top_up_short(
        origin: OriginFor<T>,
        netuid: NetUid,
        amount: TaoBalance,
        // Accepted for CLI/interface symmetry. Top-up only credits the carry
        // buffer in custody and never touches the pool, so there is no execution
        // price and nothing to bound; the parameter is intentionally unused.
        _limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(!amount.is_zero(), Error::<T>::AmountTooLow);
        let mut pos =
            ShortPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::ShortPositionNotFound)?;
        let mut agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);

        Self::transfer_tao(&coldkey, &Self::short_custody_account(netuid), amount.into())?;
        pos.r_stored = pos.r_stored.saturating_add(amount);
        pos.last_active = Self::get_current_block_as_u64();
        agg.r_sigma = agg.r_sigma.saturating_add(amount);

        ShortPositions::<T>::insert(netuid, &coldkey, pos);
        ShortAggregate::<T>::insert(netuid, agg);
        Self::deposit_event(Event::ShortToppedUp {
            coldkey,
            netuid,
            amount,
        });
        Ok(())
    }

    /// Partial (`fraction_ppb < 1e9`) or full (`= 1e9`) close (spec §8.3–8.5).
    pub fn do_close_short(
        origin: OriginFor<T>,
        netuid: NetUid,
        fraction_ppb: u64,
        limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(
            fraction_ppb > 0 && fraction_ppb <= 1_000_000_000,
            Error::<T>::InvalidCloseFraction
        );
        let rho = I64F64::from_num(fraction_ppb).safe_div(I64F64::from_num(1_000_000_000u64));

        let mut pos =
            ShortPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::ShortPositionNotFound)?;
        let mut agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);

        let q_close = Self::mul_alpha(pos.q_liability, rho);
        let r_close = Self::mul_tao(pos.r_stored, rho);
        let e_close = Self::mul_tao(pos.e_stored, rho);
        let p_close = Self::mul_tao(pos.p_floor, rho);
        let b_close = Self::mul_tao(pos.b_stored, rho);

        // Trader repays ρQ alpha from staked balance at the position hotkey.
        ensure!(
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid)
                >= q_close,
            Error::<T>::InsufficientAlphaToClose
        );
        // Guard against minting alpha: the repaid `q_close` must come out of
        // outstanding stake, never saturate `SubnetAlphaOut` to zero.
        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= q_close,
            Error::<T>::InsufficientAlphaToClose
        );
        // The repayment alpha must be unlocked (respect stake locks like unstake).
        Self::ensure_available_to_unstake(&coldkey, netuid, q_close)?;
        Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid, q_close);
        SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_sub(q_close));
        Self::increase_provided_alpha_reserve(netuid, q_close);
        // Closing rebuys `ρQ` alpha: positive flow on the same `Q·pEMA` basis as
        // the open, reversing it proportionally.
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        Self::record_derivative_inflow(netuid, Self::to_tao(Self::alpha_f(q_close).saturating_mul(pema)));

        let custody = Self::short_custody_account(netuid);
        let subnet_account =
            Self::get_subnet_account_id(netuid).ok_or(Error::<T>::SubnetNotExists)?;
        // Settle escrow ρE back to the pool, return ρ(P+R) to the trader.
        if !e_close.is_zero() {
            Self::transfer_tao(&custody, &subnet_account, e_close.into())?;
            Self::increase_provided_tao_reserve(netuid, e_close);
            TotalStake::<T>::mutate(|t| *t = t.saturating_add(e_close));
        }
        let returned = p_close.saturating_add(r_close);
        if !returned.is_zero() {
            Self::transfer_tao(&custody, &coldkey, returned.into())?;
        }
        // Slippage guard: settling escrow raises the price, so reject if it ended
        // up above the caller's ceiling (sandwich/MEV protection). `None` = no bound.
        Self::ensure_price_at_most(netuid, limit_price)?;

        pos.q_liability = pos.q_liability.saturating_sub(q_close);
        pos.r_stored = pos.r_stored.saturating_sub(r_close);
        pos.e_stored = pos.e_stored.saturating_sub(e_close);
        pos.p_floor = pos.p_floor.saturating_sub(p_close);
        pos.b_stored = pos.b_stored.saturating_sub(b_close);

        agg.q_sigma = agg.q_sigma.saturating_sub(q_close);
        agg.r_sigma = agg.r_sigma.saturating_sub(r_close);
        agg.e_sigma = agg.e_sigma.saturating_sub(e_close);
        agg.b_sigma = agg.b_sigma.saturating_sub(b_close);
        Self::sync_active_short(netuid, &agg);
        ShortAggregate::<T>::insert(netuid, agg);

        if fraction_ppb == 1_000_000_000 || pos.p_floor.is_zero() {
            ShortPositions::<T>::remove(netuid, &coldkey);
            ShortPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
            Self::cleanup_short_if_empty(netuid);
        } else {
            ShortPositions::<T>::insert(netuid, &coldkey, pos);
        }
        Self::deposit_event(Event::ShortClosed {
            coldkey,
            netuid,
            fraction_ppb,
            repaid_alpha: q_close,
            returned,
        });
        Ok(())
    }

    /// Self-covering close (cash-settled): the protocol rebuys the `ρQ` Alpha
    /// liability from the pool and charges the TAO cost against the trader's own
    /// floor+buffer, so **no pre-held Alpha is required** — a short is TAO-in /
    /// TAO-out. Buying `ρQ` and returning it to settle the synthetic debt is
    /// Alpha-neutral, so it nets to a one-sided injection of `K` TAO into the
    /// pool (`K` = current CPMM buyback cost). Rejected when `K` exceeds the
    /// claim `ρ(P+R)` (underwater): close with own funds or let it default.
    pub fn do_close_short_self(
        origin: OriginFor<T>,
        netuid: NetUid,
        fraction_ppb: u64,
        limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(
            fraction_ppb > 0 && fraction_ppb <= 1_000_000_000,
            Error::<T>::InvalidCloseFraction
        );
        let rho = I64F64::from_num(fraction_ppb).safe_div(I64F64::from_num(1_000_000_000u64));

        let mut pos =
            ShortPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::ShortPositionNotFound)?;
        let mut agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);

        let q_close = Self::mul_alpha(pos.q_liability, rho);
        let r_close = Self::mul_tao(pos.r_stored, rho);
        let e_close = Self::mul_tao(pos.e_stored, rho);
        let p_close = Self::mul_tao(pos.p_floor, rho);
        let b_close = Self::mul_tao(pos.b_stored, rho);

        // Buyback cost to rebuy `ρQ` Alpha at the live pool, charged to the claim.
        let claim = p_close.saturating_add(r_close);
        let k = Self::to_tao(Self::short_spot_close_cost(netuid, q_close));
        ensure!(k <= claim, Error::<T>::CloseCostExceedsClaim);

        let custody = Self::short_custody_account(netuid);
        let subnet_account =
            Self::get_subnet_account_id(netuid).ok_or(Error::<T>::SubnetNotExists)?;

        // K (buyback) + escrow E both restore the pool's TAO reserve. The rebuy is
        // Alpha-neutral, so no Alpha reserve / `SubnetAlphaOut` movement occurs.
        let to_pool = k.saturating_add(e_close);
        if !to_pool.is_zero() {
            Self::transfer_tao(&custody, &subnet_account, to_pool.into())?;
            Self::increase_provided_tao_reserve(netuid, to_pool);
            TotalStake::<T>::mutate(|t| *t = t.saturating_add(to_pool));
        }
        // Closing rebuys `ρQ` Alpha: positive flow on the same `Q·pEMA` basis as
        // the open, reversing it proportionally.
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        Self::record_derivative_inflow(
            netuid,
            Self::to_tao(Self::alpha_f(q_close).saturating_mul(pema)),
        );

        let returned = claim.saturating_sub(k);
        if !returned.is_zero() {
            Self::transfer_tao(&custody, &coldkey, returned.into())?;
        }
        // Slippage guard: the buyback raises the price, so reject if it ended up
        // above the caller's ceiling (sandwich/MEV protection). `None` = no bound.
        Self::ensure_price_at_most(netuid, limit_price)?;

        pos.q_liability = pos.q_liability.saturating_sub(q_close);
        pos.r_stored = pos.r_stored.saturating_sub(r_close);
        pos.e_stored = pos.e_stored.saturating_sub(e_close);
        pos.p_floor = pos.p_floor.saturating_sub(p_close);
        pos.b_stored = pos.b_stored.saturating_sub(b_close);

        agg.q_sigma = agg.q_sigma.saturating_sub(q_close);
        agg.r_sigma = agg.r_sigma.saturating_sub(r_close);
        agg.e_sigma = agg.e_sigma.saturating_sub(e_close);
        agg.b_sigma = agg.b_sigma.saturating_sub(b_close);
        Self::sync_active_short(netuid, &agg);
        ShortAggregate::<T>::insert(netuid, agg);

        if fraction_ppb == 1_000_000_000 || pos.p_floor.is_zero() {
            ShortPositions::<T>::remove(netuid, &coldkey);
            ShortPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
            Self::cleanup_short_if_empty(netuid);
        } else {
            ShortPositions::<T>::insert(netuid, &coldkey, pos);
        }
        Self::deposit_event(Event::ShortClosed {
            coldkey,
            netuid,
            fraction_ppb,
            repaid_alpha: q_close,
            returned,
        });
        Ok(())
    }

    /// Permissionless default once the buffer has decayed to dust (spec §7.4).
    pub fn do_default_short(
        origin: OriginFor<T>,
        coldkey: T::AccountId,
        netuid: NetUid,
    ) -> DispatchResult {
        ensure_signed(origin)?;
        let mut pos =
            ShortPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::ShortPositionNotFound)?;
        let mut agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);
        ensure!(
            pos.r_stored <= ShortDust::<T>::get(),
            Error::<T>::PositionNotDefaultEligible
        );
        // Anti-snipe: a third party cannot default within the grace window after
        // the owner's last action, so the owner always has time to top up.
        ensure!(
            Self::get_current_block_as_u64()
                >= pos.last_active.saturating_add(ShortDefaultGrace::<T>::get()),
            Error::<T>::PositionNotDefaultEligible
        );

        let custody = Self::short_custody_account(netuid);
        let subnet_account =
            Self::get_subnet_account_id(netuid).ok_or(Error::<T>::SubnetNotExists)?;
        // Restore residual R+E to the pool; recycle the floor P; extinguish Q.
        let residual = pos.r_stored.saturating_add(pos.e_stored);
        if !residual.is_zero() {
            Self::transfer_tao(&custody, &subnet_account, residual.into())?;
            Self::increase_provided_tao_reserve(netuid, residual);
            TotalStake::<T>::mutate(|t| *t = t.saturating_add(residual));
        }
        Self::recycle_custody_tao(&custody, pos.p_floor);

        // Default ends the position: reverse its remaining open-side flow on the
        // same `Q·pEMA` basis, so standing flow only reflects live positions
        // (abandoning can't cheaply leave a lasting flow bias).
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        Self::record_derivative_inflow(
            netuid,
            Self::to_tao(Self::alpha_f(pos.q_liability).saturating_mul(pema)),
        );

        agg.r_sigma = agg.r_sigma.saturating_sub(pos.r_stored);
        agg.e_sigma = agg.e_sigma.saturating_sub(pos.e_stored);
        agg.b_sigma = agg.b_sigma.saturating_sub(pos.b_stored);
        agg.q_sigma = agg.q_sigma.saturating_sub(pos.q_liability);
        Self::sync_active_short(netuid, &agg);
        ShortAggregate::<T>::insert(netuid, agg);
        ShortPositions::<T>::remove(netuid, &coldkey);
        ShortPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
        Self::cleanup_short_if_empty(netuid);

        Self::deposit_event(Event::ShortDefaulted { coldkey, netuid });
        Ok(())
    }

    // ---- per-block decay + restoration (spec §6.4–6.5, §12.4) ----------

    /// O(1)-per-subnet aggregate decay tick with one-sided TAO restoration zap.
    /// Iterates only subnets with live short state (`ShortActiveSubnets`).
    pub fn run_short_decay() {
        let active: Vec<NetUid> = ShortActiveSubnets::<T>::iter_keys().collect();
        for netuid in active {
            let mut agg = ShortAggregate::<T>::get(netuid);
            if agg.r_sigma.is_zero() && agg.e_sigma.is_zero() && agg.b_sigma.is_zero() {
                continue;
            }
            let d_day = Self::short_daily_decay(netuid, agg.b_sigma);
            let delta = d_day.safe_div(I64F64::from_num(BLOCKS_PER_DAY));
            if delta <= I64F64::from_num(0) {
                continue;
            }
            let dr = Self::mul_tao(agg.r_sigma, delta);
            let de = Self::mul_tao(agg.e_sigma, delta);
            let db = Self::mul_tao(agg.b_sigma, delta);
            agg.r_sigma = agg.r_sigma.saturating_sub(dr);
            agg.e_sigma = agg.e_sigma.saturating_sub(de);
            agg.b_sigma = agg.b_sigma.saturating_sub(db);
            // Ω ← Ω + (−ln(1−δ)), so a later exp(−ΔΩ) reproduces Π(1−δ) exactly.
            agg.omega = agg.omega.saturating_add(Self::neg_ln_one_minus(delta));
            ShortAggregate::<T>::insert(netuid, agg);

            // Restoration zap: decayed R+E flows back into the pool (price drifts up).
            // Credit reserves ONLY if the TAO actually moved, so a short custody
            // can never inflate `SubnetTAO` / `TotalStake`.
            let restore = dr.saturating_add(de);
            if !restore.is_zero()
                && let Some(subnet_account) = Self::get_subnet_account_id(netuid)
                && Self::transfer_tao(
                    &Self::short_custody_account(netuid),
                    &subnet_account,
                    restore.into(),
                )
                .is_ok()
            {
                Self::increase_provided_tao_reserve(netuid, restore);
                TotalStake::<T>::mutate(|t| *t = t.saturating_add(restore));
            }
        }
    }

    // ---- terminal deregistration settlement (spec §11.4) ---------------

    /// Settle all shorts on a subnet at deregistration. Must run before the
    /// pool is drained so restored escrow joins the terminal distribution.
    pub fn settle_shorts_on_dereg(netuid: NetUid) {
        let agg = ShortAggregate::<T>::get(netuid);
        let pema = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        let custody = Self::short_custody_account(netuid);
        let subnet_account = match Self::get_subnet_account_id(netuid) {
            Some(a) => a,
            None => return,
        };

        let positions: Vec<(T::AccountId, ShortPosition<T::AccountId>)> =
            ShortPositions::<T>::iter_prefix(netuid).collect();
        for (coldkey, mut pos) in positions {
            Self::materialize_short(&mut pos, agg.omega);

            // Escrow returns to the pool (joins terminal distribution). Credit
            // reserves only on a successful transfer.
            if !pos.e_stored.is_zero()
                && Self::transfer_tao(&custody, &subnet_account, pos.e_stored.into()).is_ok()
            {
                Self::increase_provided_tao_reserve(netuid, pos.e_stored);
                TotalStake::<T>::mutate(|t| *t = t.saturating_add(pos.e_stored));
            }

            // K_D(Q) = max(K_spot,last(Q), Q·pEMA).
            let c = Self::tao_f(pos.p_floor).saturating_add(Self::tao_f(pos.r_stored));
            let k_ema = Self::alpha_f(pos.q_liability).saturating_mul(pema);
            let k_spot = Self::short_spot_close_cost(netuid, pos.q_liability);
            let k_d = k_ema.max(k_spot);

            let equity = Self::to_tao(c.saturating_sub(k_d));
            let cover = Self::to_tao(c.min(k_d));
            if !equity.is_zero() {
                let _ = Self::transfer_tao(&custody, &coldkey, equity.into());
            }
            Self::recycle_custody_tao(&custody, cover);

            ShortPositions::<T>::remove(netuid, &coldkey);
            Self::deposit_event(Event::ShortTerminalSettled {
                coldkey,
                netuid,
                equity,
                liability_cover: cover,
            });
        }
        // Sweep any residual custody dust (rounding drift) so no TAO is orphaned
        // in the per-subnet custody account after the subnet is gone.
        Self::recycle_custody_tao(&custody, TaoBalance::MAX);
        ShortAggregate::<T>::remove(netuid);
        ShortActiveSubnets::<T>::remove(netuid);
        ShortPositionCount::<T>::remove(netuid);
    }

    /// Slippage-aware TAO cost to buy `q` alpha on the live pool (CPMM core).
    fn short_spot_close_cost(netuid: NetUid, q: AlphaBalance) -> I64F64 {
        let t = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let a = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let qf = Self::alpha_f(q);
        if a <= qf {
            // Liability un-buyable from the pool: saturate so cover = C, equity = 0.
            return I64F64::from_num(1e18);
        }
        // Compute the ratio `q/(a−q)` (which is O(1)) BEFORE multiplying by `t`.
        // The naive `t·q` overflows: `t` and `q` are both rao-scale (~1e13–1e15),
        // so the product (~1e27) saturates I64F64 (int range ~9.2e18) and collapses
        // the cost to a garbage near-zero value — making the close return only the
        // escrow to the pool (permanent ~N drain) and defeating the underwater guard.
        t.saturating_mul(qf.safe_div(a.saturating_sub(qf)))
    }

    // ---- slippage / limit-price protection (caller-supplied) -----------

    /// Executable alpha price (TAO per alpha) scaled by 1e9, computed in `u128`
    /// to avoid the rao×1e9 overflow. `u64::MAX` when the pool has no alpha.
    pub fn executable_price_ppb(netuid: NetUid) -> u64 {
        let t = u128::from(SubnetTAO::<T>::get(netuid).to_u64());
        let a = u128::from(SubnetAlphaIn::<T>::get(netuid).to_u64());
        if a == 0 {
            return u64::MAX;
        }
        // `a > 0` here, so plain division is safe (and `u128` has no `safe_div`).
        u64::try_from(t.saturating_mul(1_000_000_000u128) / a).unwrap_or(u64::MAX)
    }

    /// Reject if the post-trade executable price fell below `limit` (used by the
    /// price-lowering legs: short open, long close). `None` = no protection.
    fn ensure_price_at_least(netuid: NetUid, limit: Option<u64>) -> DispatchResult {
        if let Some(min) = limit {
            ensure!(
                Self::executable_price_ppb(netuid) >= min,
                Error::<T>::SlippageExceeded
            );
        }
        Ok(())
    }

    /// Reject if the post-trade executable price rose above `limit` (used by the
    /// price-raising legs: short close, long open). `None` = no protection.
    fn ensure_price_at_most(netuid: NetUid, limit: Option<u64>) -> DispatchResult {
        if let Some(max) = limit {
            ensure!(
                Self::executable_price_ppb(netuid) <= max,
                Error::<T>::SlippageExceeded
            );
        }
        Ok(())
    }

    // ---- governance setters (spec §14.6) -------------------------------

    pub fn set_shorts_enabled(enabled: bool) {
        ShortsEnabled::<T>::put(enabled);
    }
    pub fn set_longs_enabled(enabled: bool) {
        LongsEnabled::<T>::put(enabled);
    }
    /// `κ_S`, supplied scaled by 1e9. Clamped to `(0, 2.0]` so governance can't
    /// freeze the market (`κ=0`) or remove the capacity guard entirely.
    pub fn set_short_kappa_ppb(kappa_ppb: u64) {
        let k = kappa_ppb.clamp(1, 2_000_000_000);
        ShortKappa::<T>::put(I64F64::from_num(k).safe_div(I64F64::from_num(1_000_000_000u64)));
    }
    /// `λ`, supplied scaled by 1e9. Clamped to `(0, 1)` so the open quadratic
    /// stays well-formed.
    pub fn set_short_base_ltv_ppb(ltv_ppb: u64) {
        let ltv = ltv_ppb.clamp(1, 999_999_999);
        ShortBaseLtv::<T>::put(I64F64::from_num(ltv).safe_div(I64F64::from_num(1_000_000_000u64)));
    }
    /// `d_min`, `d_max`, supplied scaled by 1e9. Each is clamped to `[0, 1.0]`
    /// per day (so the per-block factor `g = 1 − d/blocks_per_day` stays in
    /// `(0, 1]`) and `d_min ≤ d_max` is enforced.
    pub fn set_decay_bounds_ppb(min_ppb: u64, max_ppb: u64) {
        let scale = I64F64::from_num(1_000_000_000u64);
        let lo = min_ppb.min(1_000_000_000);
        let hi = max_ppb.clamp(lo, 1_000_000_000);
        DecayMin::<T>::put(I64F64::from_num(lo).safe_div(scale));
        DecayMax::<T>::put(I64F64::from_num(hi).safe_div(scale));
    }
    pub fn set_short_dust(dust: TaoBalance) {
        ShortDust::<T>::put(dust);
    }
    pub fn set_short_default_grace(blocks: u64) {
        ShortDefaultGrace::<T>::put(blocks);
    }
    /// Emissions-flow factor `χ`, supplied scaled by 1e9. Clamped to `[0, 1.0]`;
    /// `0` restores flow-neutral behavior.
    pub fn set_derivative_flow_factor_ppb(chi_ppb: u64) {
        let c = chi_ppb.min(1_000_000_000);
        DerivativeFlowFactor::<T>::put(
            I64F64::from_num(c).safe_div(I64F64::from_num(1_000_000_000u64)),
        );
    }
    pub fn set_long_default_grace(blocks: u64) {
        LongDefaultGrace::<T>::put(blocks);
    }
    pub fn set_short_min_input(min_input: TaoBalance) {
        ShortMinInput::<T>::put(min_input);
    }
    /// Clamped to `[1, 4096]` so governance can't lift the dereg-settlement
    /// blast radius to a chain-halting size (terminal settlement is O(positions)
    /// in a single block until incremental settlement lands).
    pub fn set_short_max_positions(max: u32) {
        ShortMaxPositions::<T>::put(max.clamp(1, 4096));
    }

    // ---- read-only quote (spec §1.2) -----------------------------------

    /// Pure pre-open quote for a given input `P`. Returns `None` when shorts are
    /// disabled or the subnet is not a dynamic market.
    pub fn quote_open_short(netuid: NetUid, position_input: TaoBalance) -> Option<ShortOpenQuote> {
        if !ShortsEnabled::<T>::get() || SubnetMechanism::<T>::get(netuid) != 1 {
            return None;
        }
        let agg = ShortAggregate::<T>::get(netuid);
        let t_ref = Self::short_t_ref(netuid);
        let p = Self::tao_f(position_input);
        let (c, n) = Self::solve_collateral(p, t_ref, Self::tao_f(agg.b_sigma), ShortBaseLtv::<T>::get())?;
        let t_live = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let phi = Self::solve_phi(n, t_live)?;

        let q_alpha = Self::to_alpha(phi.saturating_mul(a_live));
        let scale = I64F64::from_num(1_000_000_000u64);
        let lambda_eff = n.safe_div(c).saturating_mul(scale).saturating_to_num::<u64>();
        let daily_decay = Self::short_daily_decay(netuid, agg.b_sigma)
            .saturating_mul(scale)
            .saturating_to_num::<u64>();
        Some(ShortOpenQuote {
            gross_collateral: Self::to_tao(c),
            retained_proceeds: Self::to_tao(n),
            alpha_liability: q_alpha,
            escrow: Self::to_tao(phi.saturating_mul(t_live)),
            effective_ltv: lambda_eff,
            daily_decay,
            est_close_cost: Self::to_tao(Self::short_spot_close_cost(netuid, q_alpha)),
        })
    }

    /// Estimated blocks until `r_current` decays to dust at the current rate.
    /// `u64::MAX` when decay is effectively zero.
    fn short_blocks_to_dust(netuid: NetUid, r_current: TaoBalance, b_sigma: TaoBalance) -> u64 {
        let dust = ShortDust::<T>::get();
        if r_current <= dust || dust.is_zero() {
            return if r_current <= dust { 0 } else { u64::MAX };
        }
        let delta = Self::short_daily_decay(netuid, b_sigma)
            .safe_div(I64F64::from_num(BLOCKS_PER_DAY));
        if delta <= I64F64::from_num(0) {
            return u64::MAX;
        }
        let neg_ln_g = Self::neg_ln_one_minus(delta);
        if neg_ln_g <= I64F64::from_num(0) {
            return u64::MAX;
        }
        let ratio = Self::tao_f(r_current).safe_div(Self::tao_f(dust));
        match ratio.checked_ln() {
            Some(ln_ratio) if ln_ratio > I64F64::from_num(0) => ln_ratio
                .safe_div(neg_ln_g)
                .saturating_to_num::<u64>(),
            _ => 0,
        }
    }

    /// Materialized, health-rich view of one position (decayed to the current block).
    pub fn get_short_position(
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> Option<ShortPositionInfo<T::AccountId>> {
        let mut pos = ShortPositions::<T>::get(netuid, coldkey)?;
        let agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);

        let scale = I64F64::from_num(1_000_000_000u64);
        let daily_decay = Self::short_daily_decay(netuid, agg.b_sigma)
            .saturating_mul(scale)
            .saturating_to_num::<u64>();
        let now = Self::get_current_block_as_u64();
        let defaultable_at_block = pos.last_active.saturating_add(ShortDefaultGrace::<T>::get());
        let default_eligible = pos.r_stored <= ShortDust::<T>::get() && now >= defaultable_at_block;
        let alpha_held =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, coldkey, netuid);

        Some(ShortPositionInfo {
            netuid,
            hotkey: pos.hotkey.clone(),
            floor: pos.p_floor,
            alpha_liability: pos.q_liability,
            buffer: pos.r_stored,
            escrow: pos.e_stored,
            collateral_claim: pos.p_floor.saturating_add(pos.r_stored),
            daily_decay,
            blocks_to_dust: Self::short_blocks_to_dust(netuid, pos.r_stored, agg.b_sigma),
            default_eligible,
            defaultable_at_block,
            est_close_cost: Self::to_tao(Self::short_spot_close_cost(netuid, pos.q_liability)),
            alpha_held,
            alpha_needed: AlphaBalance::from(
                pos.q_liability.to_u64().saturating_sub(alpha_held.to_u64()),
            ),
        })
    }

    /// All of a coldkey's short positions across subnets.
    pub fn get_short_positions(coldkey: &T::AccountId) -> Vec<ShortPositionInfo<T::AccountId>> {
        Self::get_all_subnet_netuids()
            .into_iter()
            .filter_map(|netuid| Self::get_short_position(coldkey, netuid))
            .collect()
    }

    /// Per-subnet short market state for sizing and capacity decisions.
    pub fn get_subnet_short_state(netuid: NetUid) -> Option<ShortMarketInfo> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }
        let agg = ShortAggregate::<T>::get(netuid);
        let t_ref = Self::short_t_ref(netuid);
        let cap = ShortKappa::<T>::get().saturating_mul(t_ref);
        let used = Self::tao_f(agg.b_sigma);
        let scale = I64F64::from_num(1_000_000_000u64);
        let ppb = |x: I64F64| x.saturating_mul(scale).saturating_to_num::<u64>();

        Some(ShortMarketInfo {
            shorts_enabled: ShortsEnabled::<T>::get(),
            base_ltv: ppb(ShortBaseLtv::<T>::get()),
            kappa: ppb(ShortKappa::<T>::get()),
            decay_min: ppb(DecayMin::<T>::get()),
            decay_max: ppb(DecayMax::<T>::get()),
            current_daily_decay: ppb(Self::short_daily_decay(netuid, agg.b_sigma)),
            t_ref: Self::to_tao(t_ref),
            footprint_used: agg.b_sigma,
            footprint_cap: Self::to_tao(cap),
            footprint_remaining: Self::to_tao(cap.saturating_sub(used)),
            open_interest_alpha: agg.q_sigma,
            buffer_total: agg.r_sigma,
            escrow_total: agg.e_sigma,
            dust_threshold: ShortDust::<T>::get(),
            min_input: ShortMinInput::<T>::get(),
            default_grace: ShortDefaultGrace::<T>::get(),
        })
    }

    /// Pre-close quote for `fraction_ppb / 1e9` of a position.
    pub fn quote_close_short(
        coldkey: &T::AccountId,
        netuid: NetUid,
        fraction_ppb: u64,
    ) -> Option<CloseShortQuote> {
        if fraction_ppb == 0 || fraction_ppb > 1_000_000_000 {
            return None;
        }
        let mut pos = ShortPositions::<T>::get(netuid, coldkey)?;
        let agg = ShortAggregate::<T>::get(netuid);
        Self::materialize_short(&mut pos, agg.omega);
        let rho = I64F64::from_num(fraction_ppb).safe_div(I64F64::from_num(1_000_000_000u64));

        let repay_alpha = Self::mul_alpha(pos.q_liability, rho);
        let returned_tao =
            Self::mul_tao(pos.p_floor, rho).saturating_add(Self::mul_tao(pos.r_stored, rho));
        let escrow_settled = Self::mul_tao(pos.e_stored, rho);
        let alpha_held =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, coldkey, netuid);

        Some(CloseShortQuote {
            repay_alpha,
            returned_tao,
            escrow_settled,
            est_buyback_cost: Self::to_tao(Self::short_spot_close_cost(netuid, repay_alpha)),
            alpha_held,
            alpha_needed: AlphaBalance::from(
                repay_alpha.to_u64().saturating_sub(alpha_held.to_u64()),
            ),
        })
    }
}
