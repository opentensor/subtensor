//! Covered continuous-unwind LONGS — the mirror of shorts with Alpha and TAO
//! swapped (spec §9). Collateral/buffer/escrow are Alpha; the fixed liability
//! `D` is TAO.
//!
//! Unlike shorts, longs need no TAO custody account: the parked Alpha is
//! tracked purely via issuance accounting (removed from `SubnetAlphaIn` /
//! `SubnetAlphaOut` at open, minted back on restoration/close, left burned =
//! recycled on default/cover). The only TAO movement is the trader paying the
//! `D` liability into the pool at close. Shared math (`solve_collateral`,
//! `solve_phi`, `neg_ln_one_minus`, `decay_curve`, conversions) is reused from
//! the parent module.

use super::*;
use safe_math::FixedExt;
use substrate_fixed::types::I64F64;
use subtensor_runtime_common::Token;

const BLOCKS_PER_DAY: u64 = 7200;

impl<T: Config> Pallet<T> {
    /// Conservative Alpha reference `A_ref = min(A_live, A_EMA)`, where `A_EMA`
    /// is the block-lagged Alpha-reserve EMA (`SubnetAlphaInMovingReserve`, ticked
    /// once per block in `update_moving_price`). The lagged factor caps `A_ref`
    /// from above so an in-block reserve nudge cannot inflate capacity; the live
    /// `A_live` floor can only pull it *down* (conservative). Cold EMA falls back
    /// to the live reserve until it warms.
    fn long_a_ref(netuid: NetUid) -> I64F64 {
        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let a_ema = I64F64::saturating_from_num(SubnetAlphaInMovingReserve::<T>::get(netuid));
        if a_ema <= I64F64::from_num(0) {
            return a_live;
        }
        a_live.min(a_ema)
    }

    /// Current long daily decay rate at the live long footprint.
    fn long_daily_decay(netuid: NetUid, b_sigma: AlphaBalance) -> I64F64 {
        let cap = LongKappa::<T>::get().saturating_mul(Self::long_a_ref(netuid));
        Self::decay_curve(Self::utilization(Self::alpha_f(b_sigma), cap))
    }

    fn materialize_long(pos: &mut LongPosition<T::AccountId>, omega_now: I64F64) {
        let arg = pos
            .omega_entry
            .saturating_sub(omega_now)
            .min(I64F64::from_num(0));
        let f = arg.checked_exp().unwrap_or_else(|| I64F64::from_num(0));
        pos.r_stored = Self::mul_alpha(pos.r_stored, f);
        pos.e_stored = Self::mul_alpha(pos.e_stored, f);
        pos.b_stored = Self::mul_alpha(pos.b_stored, f);
        pos.omega_entry = omega_now;
    }

    /// Drop the aggregate + active-set entry once the last long position closes,
    /// so the decay tick stops visiting a subnet that only holds rounding dust.
    fn cleanup_long_if_empty(netuid: NetUid) {
        if LongPositionCount::<T>::get(netuid) == 0 {
            LongAggregate::<T>::remove(netuid);
            LongActiveSubnets::<T>::remove(netuid);
        }
    }

    fn sync_active_long(netuid: NetUid, agg: &LongAgg) {
        if agg.r_sigma.is_zero()
            && agg.e_sigma.is_zero()
            && agg.b_sigma.is_zero()
            && agg.d_sigma.is_zero()
        {
            LongActiveSubnets::<T>::remove(netuid);
        } else {
            LongActiveSubnets::<T>::insert(netuid, ());
        }
    }

    // ---- user operations (spec §9, mirror of §8) -----------------------

    /// Open (or merge into) a covered long. Trader posts `position_input` Alpha
    /// (drawn from stake at `hotkey`).
    pub fn do_open_long(
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
        position_input: AlphaBalance,
        limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(LongsEnabled::<T>::get(), Error::<T>::LongsDisabled);
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            SubnetMechanism::<T>::get(netuid) == 1,
            Error::<T>::SubnetNotDynamic
        );
        ensure!(
            position_input >= LongMinInput::<T>::get(),
            Error::<T>::AmountTooLow
        );

        let mut agg = LongAggregate::<T>::get(netuid);
        let a_ref = Self::long_a_ref(netuid);
        let p = Self::alpha_f(position_input);
        let (c, n) =
            Self::solve_collateral(p, a_ref, Self::alpha_f(agg.b_sigma), LongBaseLtv::<T>::get())
                .ok_or(Error::<T>::EffectiveLtvNonPositive)?;
        let b = LongBaseLtv::<T>::get().saturating_mul(c);

        ensure!(
            Self::alpha_f(agg.b_sigma).saturating_add(b) <= LongKappa::<T>::get().saturating_mul(a_ref),
            Error::<T>::LongCapacityExceeded
        );

        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let t_live = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let phi = Self::solve_phi(n, a_live).ok_or(Error::<T>::ReserveDomainExceeded)?;

        let n_alpha = Self::to_alpha(n);
        let e_alpha = Self::to_alpha(phi.saturating_mul(a_live));
        let b_alpha = Self::to_alpha(b);
        let d_tao = Self::to_tao(phi.saturating_mul(t_live));
        ensure!(!n_alpha.is_zero(), Error::<T>::RetainedProceedsNonPositive);

        // Trader posts P Alpha from stake; remove N+E Alpha from the pool. All
        // of this leaves issuance (held off-chain in the position numbers).
        ensure!(
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                >= position_input,
            Error::<T>::InsufficientCollateral
        );
        // Respect stake locks: collateral must be unlocked alpha, exactly as a
        // normal unstake requires (otherwise a long open+close would free locked
        // alpha and bypass the subnet-ownership lock).
        Self::ensure_available_to_unstake(&coldkey, netuid, position_input)?;
        // Symmetric to the short-close guard: never `saturating_sub` below the
        // collateral, which would later let close mint alpha back unbacked.
        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= position_input,
            Error::<T>::InsufficientCollateral
        );
        Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid, position_input);
        SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_sub(position_input));
        Self::decrease_provided_alpha_reserve(netuid, n_alpha.saturating_add(e_alpha));
        // Bullish flow: the long's `D` TAO liability is the positive signal at
        // open; close/default reverse it on the same `D` basis (round-trip ~0).
        Self::record_derivative_inflow(netuid, d_tao);

        let block = Self::get_current_block_as_u64();
        let pos = match LongPositions::<T>::get(netuid, &coldkey) {
            Some(mut existing) => {
                ensure!(existing.hotkey == hotkey, Error::<T>::LongHotkeyMismatch);
                Self::materialize_long(&mut existing, agg.omega);
                existing.p_floor = existing.p_floor.saturating_add(position_input);
                existing.d_liability = existing.d_liability.saturating_add(d_tao);
                existing.r_stored = existing.r_stored.saturating_add(n_alpha);
                existing.e_stored = existing.e_stored.saturating_add(e_alpha);
                existing.b_stored = existing.b_stored.saturating_add(b_alpha);
                existing.last_active = block;
                existing
            }
            None => {
                let count = LongPositionCount::<T>::get(netuid);
                ensure!(count < LongMaxPositions::<T>::get(), Error::<T>::LongPositionLimit);
                LongPositionCount::<T>::insert(netuid, count.saturating_add(1));
                LongPosition {
                    hotkey,
                    p_floor: position_input,
                    d_liability: d_tao,
                    r_stored: n_alpha,
                    e_stored: e_alpha,
                    b_stored: b_alpha,
                    omega_entry: agg.omega,
                    last_active: block,
                }
            }
        };
        LongPositions::<T>::insert(netuid, &coldkey, pos);

        agg.r_sigma = agg.r_sigma.saturating_add(n_alpha);
        agg.e_sigma = agg.e_sigma.saturating_add(e_alpha);
        agg.b_sigma = agg.b_sigma.saturating_add(b_alpha);
        agg.d_sigma = agg.d_sigma.saturating_add(d_tao);
        LongAggregate::<T>::insert(netuid, agg);
        LongActiveSubnets::<T>::insert(netuid, ());

        // Slippage guard: a long raises the price, so reject if it ended up above
        // the caller's ceiling (sandwich/MEV protection). `None` = no bound.
        Self::ensure_price_at_most(netuid, limit_price)?;

        Self::deposit_event(Event::LongOpened {
            coldkey,
            netuid,
            position_input,
            retained_proceeds: n_alpha,
            tao_liability: d_tao,
            escrow: e_alpha,
        });
        Ok(())
    }

    /// Top up the carry buffer `R` with fresh Alpha (drawn from stake).
    pub fn do_top_up_long(
        origin: OriginFor<T>,
        netuid: NetUid,
        amount: AlphaBalance,
        // Accepted for interface symmetry; top-up never touches the pool, so there
        // is no execution price to bound. Intentionally unused.
        _limit_price: Option<u64>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(!amount.is_zero(), Error::<T>::AmountTooLow);
        let mut pos =
            LongPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::LongPositionNotFound)?;
        let mut agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);

        ensure!(
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid) >= amount,
            Error::<T>::InsufficientCollateral
        );
        Self::ensure_available_to_unstake(&coldkey, netuid, amount)?;
        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= amount,
            Error::<T>::InsufficientCollateral
        );
        Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid, amount);
        SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_sub(amount));

        pos.r_stored = pos.r_stored.saturating_add(amount);
        pos.last_active = Self::get_current_block_as_u64();
        agg.r_sigma = agg.r_sigma.saturating_add(amount);
        LongPositions::<T>::insert(netuid, &coldkey, pos);
        LongAggregate::<T>::insert(netuid, agg);
        Self::deposit_event(Event::LongToppedUp {
            coldkey,
            netuid,
            amount,
        });
        Ok(())
    }

    /// Partial or full close. Trader repays `ρD` TAO into the pool and receives
    /// `ρ(P+R)` Alpha back as stake.
    pub fn do_close_long(
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
            LongPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::LongPositionNotFound)?;
        let mut agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);

        let d_close = Self::mul_tao(pos.d_liability, rho);
        let r_close = Self::mul_alpha(pos.r_stored, rho);
        let e_close = Self::mul_alpha(pos.e_stored, rho);
        let p_close = Self::mul_alpha(pos.p_floor, rho);
        let b_close = Self::mul_alpha(pos.b_stored, rho);

        // Trader repays ρD TAO into the pool (strict transfer).
        if !d_close.is_zero() {
            let subnet_account =
                Self::get_subnet_account_id(netuid).ok_or(Error::<T>::SubnetNotExists)?;
            Self::transfer_tao(&coldkey, &subnet_account, d_close.into())?;
            Self::increase_provided_tao_reserve(netuid, d_close);
            TotalStake::<T>::mutate(|t| *t = t.saturating_add(d_close));
        }
        // Closing a long sells the alpha exposure back for TAO: negative flow,
        // reversing the open buy.
        Self::record_derivative_outflow(netuid, d_close);
        // Settle escrow back to the pool; return floor+buffer as stake (mint).
        Self::increase_provided_alpha_reserve(netuid, e_close);
        let returned = p_close.saturating_add(r_close);
        if !returned.is_zero() {
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid, returned);
            SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_add(returned));
        }
        // Slippage guard: a long unwind pushes the price down, so reject if it
        // ended up below the caller's floor (sandwich/MEV protection).
        Self::ensure_price_at_least(netuid, limit_price)?;

        pos.d_liability = pos.d_liability.saturating_sub(d_close);
        pos.r_stored = pos.r_stored.saturating_sub(r_close);
        pos.e_stored = pos.e_stored.saturating_sub(e_close);
        pos.p_floor = pos.p_floor.saturating_sub(p_close);
        pos.b_stored = pos.b_stored.saturating_sub(b_close);

        agg.d_sigma = agg.d_sigma.saturating_sub(d_close);
        agg.r_sigma = agg.r_sigma.saturating_sub(r_close);
        agg.e_sigma = agg.e_sigma.saturating_sub(e_close);
        agg.b_sigma = agg.b_sigma.saturating_sub(b_close);
        Self::sync_active_long(netuid, &agg);
        LongAggregate::<T>::insert(netuid, agg);

        if fraction_ppb == 1_000_000_000 || pos.p_floor.is_zero() {
            LongPositions::<T>::remove(netuid, &coldkey);
            LongPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
            Self::cleanup_long_if_empty(netuid);
        } else {
            LongPositions::<T>::insert(netuid, &coldkey, pos);
        }
        Self::deposit_event(Event::LongClosed {
            coldkey,
            netuid,
            fraction_ppb,
            repaid_tao: d_close,
            returned,
        });
        Ok(())
    }

    /// Fee+weight-aware Alpha that must be sold into the live pool to raise `d`
    /// TAO, quoted through the swap engine (exact-output). Mirrors
    /// `short_spot_close_cost`. Saturates to a sentinel when the pool can't
    /// yield `d`.
    fn long_spot_close_cost(netuid: NetUid, d: TaoBalance) -> I64F64 {
        match T::SwapInterface::sim_alpha_in_for_tao_out(netuid.into(), d, SimSwapOpts::WITH_FEES) {
            Ok(alpha) => Self::alpha_f(alpha),
            Err(_) => I64F64::from_num(1e18),
        }
    }

    /// Self-covering close (cash-settled): the protocol sells just enough of the
    /// `ρ(P+R)` Alpha claim into the pool to raise the `ρD` TAO liability and
    /// settle it, so **no pre-held TAO is required** — a long is Alpha-in /
    /// Alpha-out. Selling `K'` Alpha for `ρD` TAO and returning the TAO to settle
    /// is TAO-neutral, netting to a one-sided Alpha injection (`K'` + escrow).
    /// Rejected when `K'` exceeds the claim (underwater).
    pub fn do_close_long_self(
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
            LongPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::LongPositionNotFound)?;
        let mut agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);

        let d_close = Self::mul_tao(pos.d_liability, rho);
        let r_close = Self::mul_alpha(pos.r_stored, rho);
        let e_close = Self::mul_alpha(pos.e_stored, rho);
        let p_close = Self::mul_alpha(pos.p_floor, rho);
        let b_close = Self::mul_alpha(pos.b_stored, rho);

        // Alpha that must be sold to raise `ρD` TAO, charged to the claim.
        let claim = p_close.saturating_add(r_close);
        let k = Self::to_alpha(Self::long_spot_close_cost(netuid, d_close));
        ensure!(k <= claim, Error::<T>::CloseCostExceedsClaim);

        // The sell-for-D-and-settle is TAO-neutral; `K'` (sale) + escrow both
        // restore the pool's Alpha reserve. No `SubnetTAO` movement occurs.
        Self::increase_provided_alpha_reserve(netuid, k.saturating_add(e_close));
        // Closing sells the Alpha exposure back for TAO: negative flow, reversing
        // the open buy on the same `D` basis.
        Self::record_derivative_outflow(netuid, d_close);

        // Return the remaining claim as Alpha stake (mint), like the explicit close.
        let returned = claim.saturating_sub(k);
        if !returned.is_zero() {
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &pos.hotkey,
                &coldkey,
                netuid,
                returned,
            );
            SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_add(returned));
        }
        // Slippage guard: the self-cover sale pushes the price down, so reject if
        // it ended up below the caller's floor (sandwich/MEV protection).
        Self::ensure_price_at_least(netuid, limit_price)?;

        pos.d_liability = pos.d_liability.saturating_sub(d_close);
        pos.r_stored = pos.r_stored.saturating_sub(r_close);
        pos.e_stored = pos.e_stored.saturating_sub(e_close);
        pos.p_floor = pos.p_floor.saturating_sub(p_close);
        pos.b_stored = pos.b_stored.saturating_sub(b_close);

        agg.d_sigma = agg.d_sigma.saturating_sub(d_close);
        agg.r_sigma = agg.r_sigma.saturating_sub(r_close);
        agg.e_sigma = agg.e_sigma.saturating_sub(e_close);
        agg.b_sigma = agg.b_sigma.saturating_sub(b_close);
        Self::sync_active_long(netuid, &agg);
        LongAggregate::<T>::insert(netuid, agg);

        if fraction_ppb == 1_000_000_000 || pos.p_floor.is_zero() {
            LongPositions::<T>::remove(netuid, &coldkey);
            LongPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
            Self::cleanup_long_if_empty(netuid);
        } else {
            LongPositions::<T>::insert(netuid, &coldkey, pos);
        }
        Self::deposit_event(Event::LongClosed {
            coldkey,
            netuid,
            fraction_ppb,
            repaid_tao: d_close,
            returned,
        });
        Ok(())
    }

    /// Permissionless default once the buffer is dust and the grace window has
    /// elapsed. Restores residual Alpha, recycles the floor (left burned),
    /// extinguishes `D`.
    pub fn do_default_long(
        origin: OriginFor<T>,
        coldkey: T::AccountId,
        netuid: NetUid,
    ) -> DispatchResult {
        ensure_signed(origin)?;
        let mut pos =
            LongPositions::<T>::get(netuid, &coldkey).ok_or(Error::<T>::LongPositionNotFound)?;
        let mut agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);
        ensure!(
            pos.r_stored <= LongDust::<T>::get(),
            Error::<T>::PositionNotDefaultEligible
        );
        ensure!(
            Self::get_current_block_as_u64()
                >= pos.last_active.saturating_add(LongDefaultGrace::<T>::get()),
            Error::<T>::PositionNotDefaultEligible
        );

        // Restore residual R+E Alpha to the pool; floor stays burned (recycled).
        Self::increase_provided_alpha_reserve(netuid, pos.r_stored.saturating_add(pos.e_stored));

        // Default ends the position: reverse its remaining open-side `+D` flow so
        // an abandoned long can't leave a lasting positive-flow bias for only the
        // cost of the forfeited floor.
        Self::record_derivative_outflow(netuid, pos.d_liability);

        agg.r_sigma = agg.r_sigma.saturating_sub(pos.r_stored);
        agg.e_sigma = agg.e_sigma.saturating_sub(pos.e_stored);
        agg.b_sigma = agg.b_sigma.saturating_sub(pos.b_stored);
        agg.d_sigma = agg.d_sigma.saturating_sub(pos.d_liability);
        Self::sync_active_long(netuid, &agg);
        LongAggregate::<T>::insert(netuid, agg);
        LongPositions::<T>::remove(netuid, &coldkey);
        LongPositionCount::<T>::mutate(netuid, |c| *c = c.saturating_sub(1));
        Self::cleanup_long_if_empty(netuid);

        Self::deposit_event(Event::LongDefaulted { coldkey, netuid });
        Ok(())
    }

    // ---- per-block decay + restoration ---------------------------------

    /// O(1)-per-subnet long decay tick; restores decayed Alpha to the pool by
    /// minting it back into `SubnetAlphaIn`.
    pub fn run_long_decay() {
        let active: Vec<NetUid> = LongActiveSubnets::<T>::iter_keys().collect();
        for netuid in active {
            let mut agg = LongAggregate::<T>::get(netuid);
            if agg.r_sigma.is_zero() && agg.e_sigma.is_zero() && agg.b_sigma.is_zero() {
                continue;
            }
            let delta = Self::long_daily_decay(netuid, agg.b_sigma)
                .safe_div(I64F64::from_num(BLOCKS_PER_DAY));
            if delta <= I64F64::from_num(0) {
                continue;
            }
            let dr = Self::mul_alpha(agg.r_sigma, delta);
            let de = Self::mul_alpha(agg.e_sigma, delta);
            let db = Self::mul_alpha(agg.b_sigma, delta);
            agg.r_sigma = agg.r_sigma.saturating_sub(dr);
            agg.e_sigma = agg.e_sigma.saturating_sub(de);
            agg.b_sigma = agg.b_sigma.saturating_sub(db);
            agg.omega = agg.omega.saturating_add(Self::neg_ln_one_minus(delta));
            LongAggregate::<T>::insert(netuid, agg);

            // Restoration: mint decayed R+E Alpha back into the pool reserve.
            Self::increase_provided_alpha_reserve(netuid, dr.saturating_add(de));
        }
    }

    // ---- terminal deregistration settlement (spec §11.5) ---------------

    /// Settle all longs on a subnet at deregistration: escrow Alpha rejoins the
    /// pool; the `D` TAO debt is covered at the conservative
    /// `max(spot, EMA)` alpha cost (mirror of the short's `K_D`); the cover
    /// alpha stays burned (recycled); the equity remainder returns as stake.
    pub fn settle_longs_on_dereg(netuid: NetUid) {
        let agg = LongAggregate::<T>::get(netuid);
        let price = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
        let positions: Vec<(T::AccountId, LongPosition<T::AccountId>)> =
            LongPositions::<T>::iter_prefix(netuid).collect();
        for (coldkey, mut pos) in positions {
            Self::materialize_long(&mut pos, agg.omega);
            // Escrow rejoins the pool / terminal distribution.
            Self::increase_provided_alpha_reserve(netuid, pos.e_stored);

            let c_l = Self::alpha_f(pos.p_floor.saturating_add(pos.r_stored));
            // Cover the `D` TAO debt at the price that is conservative for the
            // pool: the MORE alpha-expensive of the live spot cost and the EMA
            // valuation. Mirrors the short's `K_D = max(K_spot, Q·pEMA)`. Using
            // the EMA alone (`D/pEMA`) would let a stale-high EMA after a fast
            // price drop under-seize, leaking equity to the long.
            let k_spot = Self::long_spot_close_cost(netuid, pos.d_liability);
            // Flat EMA cover `D/pEMA`; zero when the EMA is cold so the `max`
            // falls back to spot (mirrors the short's `Q·pEMA == 0` case).
            let k_ema = if price > I64F64::from_num(0) {
                Self::tao_f(pos.d_liability).safe_div(price)
            } else {
                I64F64::from_num(0)
            };
            let cover = c_l.min(k_spot.max(k_ema));
            let equity = Self::to_alpha(c_l.saturating_sub(cover));
            if !equity.is_zero() {
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(&pos.hotkey, &coldkey, netuid, equity);
                SubnetAlphaOut::<T>::mutate(netuid, |o| *o = o.saturating_add(equity));
            }
            // The cover portion of the collateral stays burned (recycled).
            LongPositions::<T>::remove(netuid, &coldkey);
            Self::deposit_event(Event::LongTerminalSettled {
                coldkey,
                netuid,
                equity,
            });
        }
        LongAggregate::<T>::remove(netuid);
        LongActiveSubnets::<T>::remove(netuid);
        LongPositionCount::<T>::remove(netuid);
    }

    // ---- governance setters --------------------------------------------

    pub fn set_long_kappa_ppb(kappa_ppb: u64) {
        let k = kappa_ppb.clamp(1, 2_000_000_000);
        LongKappa::<T>::put(I64F64::from_num(k).safe_div(I64F64::from_num(1_000_000_000u64)));
    }
    pub fn set_long_base_ltv_ppb(ltv_ppb: u64) {
        let ltv = ltv_ppb.clamp(1, 999_999_999);
        LongBaseLtv::<T>::put(I64F64::from_num(ltv).safe_div(I64F64::from_num(1_000_000_000u64)));
    }
    pub fn set_long_dust(dust: AlphaBalance) {
        LongDust::<T>::put(dust);
    }
    pub fn set_long_min_input(min_input: AlphaBalance) {
        LongMinInput::<T>::put(min_input);
    }
    /// Clamped to `[1, 4096]` (see `set_short_max_positions`).
    pub fn set_long_max_positions(max: u32) {
        LongMaxPositions::<T>::put(max.clamp(1, 4096));
    }

    // ---- read-only views (mirror of the short read layer) --------------

    /// Estimated blocks until `r_current` (Alpha) decays to dust at the current
    /// rate. `u64::MAX` when decay is effectively zero.
    fn long_blocks_to_dust(netuid: NetUid, r_current: AlphaBalance, b_sigma: AlphaBalance) -> u64 {
        let dust = LongDust::<T>::get();
        if r_current <= dust || dust.is_zero() {
            return if r_current <= dust { 0 } else { u64::MAX };
        }
        let delta = Self::long_daily_decay(netuid, b_sigma).safe_div(I64F64::from_num(BLOCKS_PER_DAY));
        if delta <= I64F64::from_num(0) {
            return u64::MAX;
        }
        let neg_ln_g = Self::neg_ln_one_minus(delta);
        if neg_ln_g <= I64F64::from_num(0) {
            return u64::MAX;
        }
        let ratio = Self::alpha_f(r_current).safe_div(Self::alpha_f(dust));
        match ratio.checked_ln() {
            Some(ln_ratio) if ln_ratio > I64F64::from_num(0) => {
                ln_ratio.safe_div(neg_ln_g).saturating_to_num::<u64>()
            }
            _ => 0,
        }
    }

    /// Pure pre-open long quote. `None` when longs are disabled or the subnet is
    /// not a dynamic market.
    pub fn quote_open_long(netuid: NetUid, position_input: AlphaBalance) -> Option<LongOpenQuote> {
        if !LongsEnabled::<T>::get() || SubnetMechanism::<T>::get(netuid) != 1 {
            return None;
        }
        let agg = LongAggregate::<T>::get(netuid);
        let a_ref = Self::long_a_ref(netuid);
        let p = Self::alpha_f(position_input);
        let (c, n) =
            Self::solve_collateral(p, a_ref, Self::alpha_f(agg.b_sigma), LongBaseLtv::<T>::get())?;
        let a_live = Self::alpha_f(SubnetAlphaIn::<T>::get(netuid));
        let t_live = Self::tao_f(SubnetTAO::<T>::get(netuid));
        let phi = Self::solve_phi(n, a_live)?;

        let scale = I64F64::from_num(1_000_000_000u64);
        let effective_ltv = n.safe_div(c).saturating_mul(scale).saturating_to_num::<u64>();
        let daily_decay = Self::long_daily_decay(netuid, agg.b_sigma)
            .saturating_mul(scale)
            .saturating_to_num::<u64>();
        Some(LongOpenQuote {
            gross_collateral: Self::to_alpha(c),
            retained_proceeds: Self::to_alpha(n),
            tao_liability: Self::to_tao(phi.saturating_mul(t_live)),
            escrow: Self::to_alpha(phi.saturating_mul(a_live)),
            effective_ltv,
            daily_decay,
        })
    }

    /// Materialized, health-rich view of one long position.
    pub fn get_long_position(
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> Option<LongPositionInfo<T::AccountId>> {
        let mut pos = LongPositions::<T>::get(netuid, coldkey)?;
        let agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);

        let scale = I64F64::from_num(1_000_000_000u64);
        let daily_decay = Self::long_daily_decay(netuid, agg.b_sigma)
            .saturating_mul(scale)
            .saturating_to_num::<u64>();
        let now = Self::get_current_block_as_u64();
        let defaultable_at_block = pos.last_active.saturating_add(LongDefaultGrace::<T>::get());
        let default_eligible = pos.r_stored <= LongDust::<T>::get() && now >= defaultable_at_block;

        Some(LongPositionInfo {
            netuid,
            hotkey: pos.hotkey.clone(),
            floor: pos.p_floor,
            tao_liability: pos.d_liability,
            buffer: pos.r_stored,
            escrow: pos.e_stored,
            collateral_claim: pos.p_floor.saturating_add(pos.r_stored),
            daily_decay,
            blocks_to_dust: Self::long_blocks_to_dust(netuid, pos.r_stored, agg.b_sigma),
            default_eligible,
            defaultable_at_block,
            tao_to_close: pos.d_liability,
        })
    }

    /// All of a coldkey's long positions across subnets.
    pub fn get_long_positions(coldkey: &T::AccountId) -> Vec<LongPositionInfo<T::AccountId>> {
        Self::get_all_subnet_netuids()
            .into_iter()
            .filter_map(|netuid| Self::get_long_position(coldkey, netuid))
            .collect()
    }

    /// Per-subnet long market state for sizing and capacity decisions.
    pub fn get_subnet_long_state(netuid: NetUid) -> Option<LongMarketInfo> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }
        let agg = LongAggregate::<T>::get(netuid);
        let a_ref = Self::long_a_ref(netuid);
        let cap = LongKappa::<T>::get().saturating_mul(a_ref);
        let used = Self::alpha_f(agg.b_sigma);
        let scale = I64F64::from_num(1_000_000_000u64);
        let ppb = |x: I64F64| x.saturating_mul(scale).saturating_to_num::<u64>();

        Some(LongMarketInfo {
            longs_enabled: LongsEnabled::<T>::get(),
            base_ltv: ppb(LongBaseLtv::<T>::get()),
            kappa: ppb(LongKappa::<T>::get()),
            decay_min: ppb(DecayMin::<T>::get()),
            decay_max: ppb(DecayMax::<T>::get()),
            current_daily_decay: ppb(Self::long_daily_decay(netuid, agg.b_sigma)),
            a_ref: Self::to_alpha(a_ref),
            footprint_used: agg.b_sigma,
            footprint_cap: Self::to_alpha(cap),
            footprint_remaining: Self::to_alpha(cap.saturating_sub(used)),
            open_interest_tao: agg.d_sigma,
            buffer_total: agg.r_sigma,
            escrow_total: agg.e_sigma,
            dust_threshold: LongDust::<T>::get(),
            min_input: LongMinInput::<T>::get(),
            default_grace: LongDefaultGrace::<T>::get(),
        })
    }

    /// Pre-close quote for `fraction_ppb / 1e9` of a long position.
    pub fn quote_close_long(
        coldkey: &T::AccountId,
        netuid: NetUid,
        fraction_ppb: u64,
    ) -> Option<CloseLongQuote> {
        if fraction_ppb == 0 || fraction_ppb > 1_000_000_000 {
            return None;
        }
        let mut pos = LongPositions::<T>::get(netuid, coldkey)?;
        let agg = LongAggregate::<T>::get(netuid);
        Self::materialize_long(&mut pos, agg.omega);
        let rho = I64F64::from_num(fraction_ppb).safe_div(I64F64::from_num(1_000_000_000u64));

        Some(CloseLongQuote {
            repay_tao: Self::mul_tao(pos.d_liability, rho),
            returned_alpha: Self::mul_alpha(pos.p_floor, rho)
                .saturating_add(Self::mul_alpha(pos.r_stored, rho)),
            escrow_settled: Self::mul_alpha(pos.e_stored, rho),
        })
    }
}
