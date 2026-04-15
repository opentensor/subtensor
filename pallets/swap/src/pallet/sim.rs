use core::marker::PhantomData;

use frame_support::{ensure, pallet_prelude::Zero, traits::Get};
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token, TokenReserve};

use super::pallet::*;
use super::swap_step::SwapStepAction;
use crate::{
    SqrtPrice,
    tick::{ActiveTickIndexManager, TickIndex},
};
use subtensor_swap_interface::SwapResult;

const MAX_SIM_ITERATIONS: u16 = 1000;

/// Mutable AMM state carried through a pure simulation loop.
/// Values are read once from storage before the loop and updated locally
/// on each iteration — no storage writes occur.
pub(crate) struct SimState {
    pub sqrt_price: SqrtPrice,
    pub current_tick: TickIndex,
    pub current_liquidity: u64,
    /// True when pure simulation bootstraps an uninitialized V3 pool in memory only.
    pub virtual_full_range_liquidity: bool,
}

/// Output of one pure simulation step.
/// Mirrors `SwapStepResult` but also carries the updated AMM state so the
/// caller can thread it through the loop without touching storage.
pub(crate) struct PureStepResult<PaidIn, PaidOut>
where
    PaidIn: Token,
    PaidOut: Token,
{
    pub amount_to_take: PaidIn,
    pub fee_paid: PaidIn,
    pub delta_in: PaidIn,
    pub delta_out: PaidOut,
    pub fee_to_block_author: PaidIn,
    pub action: SwapStepAction,
    pub new_sqrt_price: SqrtPrice,
    pub new_tick: TickIndex,
    pub new_liquidity: u64,
}

/// Direction-specific pure methods.
///
/// This mirrors `SwapStep` but every method that previously read from storage
/// now takes an explicit `state: &SimState` argument instead.  The provided
/// `execute` method runs the full step (determine action + process swap)
/// returning a `PureStepResult` with updated state values instead of writing
/// them back to storage.
pub(crate) trait PureStep<T, PaidIn, PaidOut>
where
    T: Config,
    PaidIn: Token,
    PaidOut: Token,
{
    /// Get the input amount needed to reach the target price.
    fn delta_in(
        liquidity: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> PaidIn;

    /// Get the tick at the current tick edge (direction-specific).
    fn tick_edge(netuid: NetUid, state: &SimState) -> TickIndex;

    /// Get the target sqrt price based on the available input amount.
    fn sqrt_price_target(
        liquidity: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: PaidIn,
    ) -> SqrtPrice;

    /// Returns true if p1 is closer to the current price than p2
    /// in the direction of this step.
    fn price_is_closer(p1: &SqrtPrice, p2: &SqrtPrice) -> bool;

    /// The action to take when we land exactly on the edge price.
    fn action_on_edge_sqrt_price() -> SwapStepAction;

    /// Convert delta_in (input token) to delta_out (output token) using state values.
    fn convert_deltas(state: &SimState, delta_in: PaidIn) -> PaidOut;

    /// Compute the new liquidity after crossing a tick.
    /// Returns the updated liquidity value (does NOT write to storage).
    fn update_liquidity_at_crossing(
        netuid: NetUid,
        state: &SimState,
    ) -> Result<u64, Error<T>>;

    /// Execute a single pure simulation step.
    ///
    /// Mirrors `BasicSwapStep::new` + `determine_action` + `process_swap` but
    /// reads AMM state from `state` instead of storage, and returns updated
    /// state values in `PureStepResult` rather than writing them.
    fn execute(
        netuid: NetUid,
        amount_remaining: PaidIn,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
        state: &SimState,
    ) -> Result<PureStepResult<PaidIn, PaidOut>, Error<T>> {
        // --- Replicate BasicSwapStep::new ---
        let current_sqrt_price = state.sqrt_price;
        let edge_tick = Self::tick_edge(netuid, state);
        let edge_sqrt_price = edge_tick.as_sqrt_price_bounded();

        let fee =
            Pallet::<T>::calculate_fee_amount(netuid, amount_remaining, drop_fees);
        let possible_delta_in = amount_remaining.saturating_sub(fee);

        let current_liquidity =
            U64F64::saturating_from_num(state.current_liquidity);
        let target_sqrt_price =
            Self::sqrt_price_target(current_liquidity, current_sqrt_price, possible_delta_in);

        // --- Replicate determine_action ---
        let mut recalculate_fee = false;

        let (mut action, delta_in, final_price) =
            if Self::price_is_closer(&target_sqrt_price, &limit_sqrt_price)
                && Self::price_is_closer(&target_sqrt_price, &edge_sqrt_price)
            {
                // Case 1: stop within tick, use full possible_delta_in
                (SwapStepAction::Stop, possible_delta_in, target_sqrt_price)
            } else if Self::price_is_closer(&limit_sqrt_price, &target_sqrt_price)
                && Self::price_is_closer(&limit_sqrt_price, &edge_sqrt_price)
            {
                // Case 2: limit price is closest
                recalculate_fee = true;
                (
                    SwapStepAction::Stop,
                    Self::delta_in(current_liquidity, current_sqrt_price, limit_sqrt_price),
                    limit_sqrt_price,
                )
            } else {
                // Case 3: edge price is closest — tick crossing likely
                recalculate_fee = true;
                (
                    SwapStepAction::Crossing,
                    Self::delta_in(current_liquidity, current_sqrt_price, edge_sqrt_price),
                    edge_sqrt_price,
                )
            };

        let mut fee = fee;

        if recalculate_fee {
            let u16_max = U64F64::saturating_from_num(u16::MAX);
            let fee_rate = if drop_fees {
                U64F64::saturating_from_num(0)
            } else {
                U64F64::saturating_from_num(FeeRate::<T>::get(netuid))
            };
            let delta_fixed = U64F64::saturating_from_num(delta_in);
            fee = delta_fixed
                .saturating_mul(fee_rate.safe_div(u16_max.saturating_sub(fee_rate)))
                .saturating_to_num::<u64>()
                .into();
        }

        // Correct action when stopped exactly at the edge price
        let natural_reason_stop_price =
            if Self::price_is_closer(&limit_sqrt_price, &target_sqrt_price) {
                limit_sqrt_price
            } else {
                target_sqrt_price
            };
        if natural_reason_stop_price == edge_sqrt_price {
            action = Self::action_on_edge_sqrt_price();
        }

        // --- Replicate process_swap (no storage writes for fees/ticks/price) ---
        let delta_out = Self::convert_deltas(state, delta_in);

        let mut fee_to_block_author = PaidIn::ZERO;
        if delta_in > PaidIn::ZERO {
            ensure!(delta_out > PaidOut::ZERO, Error::<T>::ReservesTooLow);

            let fee_split = DefaultFeeSplit::get();
            let lp_fee: PaidIn = fee_split.mul_floor(fee.to_u64()).into();
            // lp_fee would be added to fee globals (skipped — pure simulation)
            fee_to_block_author = fee.saturating_sub(lp_fee);
        }

        // Determine updated state values
        let new_sqrt_price = final_price;
        let new_tick = TickIndex::from_sqrt_price_bounded(final_price);
        let new_liquidity = if action == SwapStepAction::Crossing {
            // Tick crossing: read liquidity_net from storage (read-only) to
            // compute the new liquidity, but do not write it.
            Self::update_liquidity_at_crossing(netuid, state)?
        } else {
            state.current_liquidity
        };

        Ok(PureStepResult {
            amount_to_take: delta_in.saturating_add(fee),
            fee_paid: fee,
            delta_in,
            delta_out,
            fee_to_block_author,
            action,
            new_sqrt_price,
            new_tick,
            new_liquidity,
        })
    }
}

// ---------------------------------------------------------------------------
// BuyStep: Tao → Alpha
// ---------------------------------------------------------------------------

pub(crate) struct BuyStep<T>(PhantomData<T>);

impl<T: Config> PureStep<T, TaoBalance, AlphaBalance> for BuyStep<T> {
    fn delta_in(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> TaoBalance {
        liquidity_curr
            .saturating_mul(sqrt_price_target.saturating_sub(sqrt_price_curr))
            .saturating_to_num::<u64>()
            .into()
    }

    fn tick_edge(netuid: NetUid, state: &SimState) -> TickIndex {
        if state.virtual_full_range_liquidity {
            return TickIndex::MAX;
        }

        ActiveTickIndexManager::<T>::find_closest_higher(
            netuid,
            state.current_tick.next().unwrap_or(TickIndex::MAX),
        )
        .unwrap_or(TickIndex::MAX)
    }

    fn sqrt_price_target(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: TaoBalance,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);

        if liquidity_curr == 0 {
            return SqrtPrice::saturating_from_num(
                Pallet::<T>::max_price_inner::<TaoBalance>().to_u64(),
            );
        }

        delta_fixed
            .safe_div(liquidity_curr)
            .saturating_add(sqrt_price_curr)
    }

    fn price_is_closer(sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool {
        sq_price1 <= sq_price2
    }

    fn action_on_edge_sqrt_price() -> SwapStepAction {
        SwapStepAction::Crossing
    }

    fn convert_deltas(state: &SimState, delta_in: TaoBalance) -> AlphaBalance {
        if delta_in.is_zero() {
            return AlphaBalance::ZERO;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(state.current_liquidity);
        let sqrt_price_curr = state.sqrt_price;
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in.to_u64());

        let result = {
            let a = liquidity_curr
                .saturating_mul(sqrt_price_curr)
                .saturating_add(delta_fixed)
                .saturating_mul(sqrt_price_curr);
            let b = liquidity_curr.safe_div(a);
            b.saturating_mul(delta_fixed)
        };

        result.saturating_to_num::<u64>().into()
    }

    fn update_liquidity_at_crossing(
        netuid: NetUid,
        state: &SimState,
    ) -> Result<u64, Error<T>> {
        if state.virtual_full_range_liquidity {
            // The only upper crossing in virtual bootstrap mode is TickIndex::MAX,
            // where protocol full-range position contributes negative liquidity_net.
            return Ok(0);
        }

        let mut liquidity_curr = state.current_liquidity;

        // For BuyStep, find the next active tick above current_tick
        let upper_tick = ActiveTickIndexManager::<T>::find_closest_higher(
            netuid,
            state.current_tick.next().unwrap_or(TickIndex::MAX),
        )
        .unwrap_or(TickIndex::MAX);

        let tick =
            Ticks::<T>::get(netuid, upper_tick).ok_or(Error::<T>::InsufficientLiquidity)?;

        let liquidity_update_abs_u64 = tick.liquidity_net_as_u64();

        liquidity_curr = if tick.liquidity_net >= 0 {
            liquidity_curr.saturating_add(liquidity_update_abs_u64)
        } else {
            liquidity_curr.saturating_sub(liquidity_update_abs_u64)
        };

        Ok(liquidity_curr)
    }
}

// ---------------------------------------------------------------------------
// SellStep: Alpha → Tao
// ---------------------------------------------------------------------------

pub(crate) struct SellStep<T>(PhantomData<T>);

impl<T: Config> PureStep<T, AlphaBalance, TaoBalance> for SellStep<T> {
    fn delta_in(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> AlphaBalance {
        let one = U64F64::saturating_from_num(1);

        liquidity_curr
            .saturating_mul(
                one.safe_div(sqrt_price_target.into())
                    .saturating_sub(one.safe_div(sqrt_price_curr)),
            )
            .saturating_to_num::<u64>()
            .into()
    }

    fn tick_edge(netuid: NetUid, state: &SimState) -> TickIndex {
        if state.virtual_full_range_liquidity {
            return TickIndex::MIN;
        }

        let current_tick = state.current_tick;
        let current_price: SqrtPrice = state.sqrt_price;
        let current_tick_price = current_tick.as_sqrt_price_bounded();
        let is_active = ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick);

        if is_active && current_price > current_tick_price {
            return ActiveTickIndexManager::<T>::find_closest_lower(netuid, current_tick)
                .unwrap_or(TickIndex::MIN);
        }

        ActiveTickIndexManager::<T>::find_closest_lower(
            netuid,
            current_tick.prev().unwrap_or(TickIndex::MIN),
        )
        .unwrap_or(TickIndex::MIN)
    }

    fn sqrt_price_target(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: AlphaBalance,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        if liquidity_curr == 0 {
            return SqrtPrice::saturating_from_num(
                Pallet::<T>::min_price_inner::<AlphaBalance>().to_u64(),
            );
        }

        one.safe_div(
            delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(one.safe_div(sqrt_price_curr)),
        )
    }

    fn price_is_closer(sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool {
        sq_price1 >= sq_price2
    }

    fn action_on_edge_sqrt_price() -> SwapStepAction {
        SwapStepAction::Stop
    }

    fn convert_deltas(state: &SimState, delta_in: AlphaBalance) -> TaoBalance {
        if delta_in.is_zero() {
            return TaoBalance::ZERO;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(state.current_liquidity);
        let sqrt_price_curr = state.sqrt_price;
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in.to_u64());

        let result = {
            let denom = liquidity_curr
                .safe_div(sqrt_price_curr)
                .saturating_add(delta_fixed);
            let a = liquidity_curr.safe_div(denom);
            let b = a.saturating_mul(sqrt_price_curr);
            delta_fixed.saturating_mul(b)
        };

        result.saturating_to_num::<u64>().into()
    }

    fn update_liquidity_at_crossing(
        netuid: NetUid,
        state: &SimState,
    ) -> Result<u64, Error<T>> {
        if state.virtual_full_range_liquidity {
            // The only lower crossing in virtual bootstrap mode is TickIndex::MIN,
            // where protocol full-range position contributes positive liquidity_net.
            return Ok(0);
        }

        let mut liquidity_curr = state.current_liquidity;

        // For SellStep, find the next active tick below current_tick
        // (mirrors the logic in BasicSwapStep<T, AlphaBalance, TaoBalance>::update_liquidity_at_crossing)
        let current_tick_index = state.current_tick;
        let current_price: SqrtPrice = state.sqrt_price;
        let current_tick_price = current_tick_index.as_sqrt_price_bounded();
        let is_active =
            ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick_index);

        let lower_tick = if is_active && current_price > current_tick_price {
            ActiveTickIndexManager::<T>::find_closest_lower(netuid, current_tick_index)
                .unwrap_or(TickIndex::MIN)
        } else {
            ActiveTickIndexManager::<T>::find_closest_lower(
                netuid,
                current_tick_index.prev().unwrap_or(TickIndex::MIN),
            )
            .unwrap_or(TickIndex::MIN)
        };

        let tick =
            Ticks::<T>::get(netuid, lower_tick).ok_or(Error::<T>::InsufficientLiquidity)?;

        let liquidity_update_abs_u64 = tick.liquidity_net_as_u64();

        liquidity_curr = if tick.liquidity_net >= 0 {
            liquidity_curr.saturating_sub(liquidity_update_abs_u64)
        } else {
            liquidity_curr.saturating_add(liquidity_update_abs_u64)
        };

        Ok(liquidity_curr)
    }
}

// ---------------------------------------------------------------------------
// Core pure simulation loop
// ---------------------------------------------------------------------------

/// Pure swap simulation loop.
///
/// Functionally equivalent to `swap_inner` but never writes to storage.
/// AMM state (`sqrt_price`, `current_tick`, `current_liquidity`) is read once
/// before the loop and carried as mutable local variables through each
/// iteration.
pub(crate) fn sim_swap_inner_pure<T, PaidIn, PaidOut, Step>(
    netuid: NetUid,
    amount: PaidIn,
    limit_sqrt_price: SqrtPrice,
    drop_fees: bool,
) -> Result<SwapResult<PaidIn, PaidOut>, Error<T>>
where
    T: Config,
    PaidIn: Token,
    PaidOut: Token,
    Step: PureStep<T, PaidIn, PaidOut>,
{
    // Read initial AMM state once. If V3 isn't initialized yet, bootstrap the
    // same effective state that `maybe_initialize_v3()` would create, but keep it
    // local (no storage writes).
    let mut state = if SwapV3Initialized::<T>::get(netuid) {
        SimState {
            sqrt_price: AlphaSqrtPrice::<T>::get(netuid),
            current_tick: CurrentTick::<T>::get(netuid),
            current_liquidity: CurrentLiquidity::<T>::get(netuid),
            virtual_full_range_liquidity: false,
        }
    } else {
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());

        let price = U64F64::saturating_from_num(tao_reserve)
            .safe_div(U64F64::saturating_from_num(alpha_reserve));
        let epsilon = U64F64::saturating_from_num(0.000000000001);
        let sqrt_price = price.checked_sqrt(epsilon).unwrap_or(U64F64::from_num(0));
        let current_tick = TickIndex::from_sqrt_price_bounded(sqrt_price);
        let current_liquidity = helpers_128bit::sqrt(
            (tao_reserve.to_u64() as u128).saturating_mul(alpha_reserve.to_u64() as u128),
        ) as u64;

        SimState {
            sqrt_price,
            current_tick,
            current_liquidity,
            virtual_full_range_liquidity: true,
        }
    };

    let mut amount_remaining = amount;
    let mut amount_paid_out = PaidOut::ZERO;
    let mut iteration_counter: u16 = 0;
    let mut in_acc = PaidIn::ZERO;
    let mut fee_acc = PaidIn::ZERO;
    let mut fee_to_block_author_acc = PaidIn::ZERO;

    while !amount_remaining.is_zero() {
        let step_result =
            Step::execute(netuid, amount_remaining, limit_sqrt_price, drop_fees, &state)?;

        // Thread updated state through the loop
        state.sqrt_price = step_result.new_sqrt_price;
        state.current_tick = step_result.new_tick;
        state.current_liquidity = step_result.new_liquidity;

        in_acc = in_acc.saturating_add(step_result.delta_in);
        fee_acc = fee_acc.saturating_add(step_result.fee_paid);
        fee_to_block_author_acc =
            fee_to_block_author_acc.saturating_add(step_result.fee_to_block_author);
        amount_remaining =
            amount_remaining.saturating_sub(step_result.amount_to_take);
        amount_paid_out = amount_paid_out.saturating_add(step_result.delta_out);

        if step_result.action == SwapStepAction::Stop {
            amount_remaining = PaidIn::ZERO;
        }

        if step_result.amount_to_take.is_zero() {
            amount_remaining = PaidIn::ZERO;
        }

        iteration_counter = iteration_counter.saturating_add(1);

        ensure!(
            iteration_counter <= MAX_SIM_ITERATIONS,
            Error::<T>::TooManySwapSteps
        );
    }

    Ok(SwapResult {
        amount_paid_in: in_acc,
        amount_paid_out,
        fee_paid: fee_acc,
        fee_to_block_author: fee_to_block_author_acc,
    })
}
