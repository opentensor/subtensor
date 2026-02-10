use core::ops::Neg;

use frame_support::storage::{TransactionOutcome, transactional};
use frame_support::{ensure, pallet_prelude::DispatchError, traits::Get};
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::{DispatchResult, Vec, traits::AccountIdConversion};
use substrate_fixed::types::{I64F64, U64F64, U96F32};
use subtensor_runtime_common::{
    AlphaCurrency, BalanceOps, Currency, CurrencyReserve, NetUid, SubnetInfo, TaoCurrency,
};
use subtensor_swap_interface::{
    DefaultPriceLimit, Order as OrderT, SwapEngine, SwapHandler, SwapResult,
};

use super::pallet::*;
use super::swap_step::{BasicSwapStep, SwapStep, SwapStepAction};
use crate::{
    SqrtPrice,
    position::{Position, PositionId},
    tick::{ActiveTickIndexManager, Tick, TickIndex},
};

const MAX_SWAP_ITERATIONS: u16 = 1000;

#[derive(Debug, PartialEq)]
pub struct UpdateLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub removed: bool,
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
}

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
    pub liquidity: u64,
}

impl<T: Config> Pallet<T> {
    pub fn current_price(netuid: NetUid) -> U96F32 {
        match T::SubnetInfo::mechanism(netuid.into()) {
            1 => {
                if SwapV3Initialized::<T>::get(netuid) {
                    let sqrt_price = AlphaSqrtPrice::<T>::get(netuid);
                    U96F32::saturating_from_num(sqrt_price.saturating_mul(sqrt_price))
                } else {
                    let tao_reserve = T::TaoReserve::reserve(netuid.into());
                    let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
                    if !alpha_reserve.is_zero() {
                        U96F32::saturating_from_num(tao_reserve)
                            .saturating_div(U96F32::saturating_from_num(alpha_reserve))
                    } else {
                        U96F32::saturating_from_num(0)
                    }
                }
            }
            _ => U96F32::saturating_from_num(1),
        }
    }

    // initializes V3 swap for a subnet if needed
    pub fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
        if SwapV3Initialized::<T>::get(netuid) {
            return Ok(());
        }

        // Initialize the v3:
        // Reserves are re-purposed, nothing to set, just query values for liquidity and price
        // calculation
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());

        // Set price
        let price = U64F64::saturating_from_num(tao_reserve)
            .safe_div(U64F64::saturating_from_num(alpha_reserve));

        let epsilon = U64F64::saturating_from_num(0.000000000001);

        let current_sqrt_price = price.checked_sqrt(epsilon).unwrap_or(U64F64::from_num(0));
        AlphaSqrtPrice::<T>::set(netuid, current_sqrt_price);

        // Set current tick
        let current_tick = TickIndex::from_sqrt_price_bounded(current_sqrt_price);
        CurrentTick::<T>::set(netuid, current_tick);

        // Set initial (protocol owned) liquidity and positions
        // Protocol liquidity makes one position from TickIndex::MIN to TickIndex::MAX
        // We are using the sp_arithmetic sqrt here, which works for u128
        let liquidity = helpers_128bit::sqrt(
            (tao_reserve.to_u64() as u128).saturating_mul(alpha_reserve.to_u64() as u128),
        ) as u64;
        let protocol_account_id = Self::protocol_account_id();

        let (position, _, _) = Self::add_liquidity_not_insert(
            netuid,
            &protocol_account_id,
            TickIndex::MIN,
            TickIndex::MAX,
            liquidity,
        )?;

        Positions::<T>::insert(&(netuid, protocol_account_id, position.id), position);

        Ok(())
    }

    pub(crate) fn get_proportional_alpha_tao_and_remainders(
        sqrt_alpha_price: U64F64,
        amount_tao: TaoCurrency,
        amount_alpha: AlphaCurrency,
    ) -> (TaoCurrency, AlphaCurrency, TaoCurrency, AlphaCurrency) {
        let price = sqrt_alpha_price.saturating_mul(sqrt_alpha_price);
        let tao_equivalent: u64 = U64F64::saturating_from_num(u64::from(amount_alpha))
            .saturating_mul(price)
            .saturating_to_num();
        let amount_tao_u64 = u64::from(amount_tao);

        if tao_equivalent <= amount_tao_u64 {
            // Too much or just enough TAO
            (
                tao_equivalent.into(),
                amount_alpha,
                amount_tao.saturating_sub(TaoCurrency::from(tao_equivalent)),
                0.into(),
            )
        } else {
            // Too much Alpha
            let alpha_equivalent: u64 = U64F64::saturating_from_num(u64::from(amount_tao))
                .safe_div(price)
                .saturating_to_num();
            (
                amount_tao,
                alpha_equivalent.into(),
                0.into(),
                u64::from(amount_alpha)
                    .saturating_sub(alpha_equivalent)
                    .into(),
            )
        }
    }

    /// Adjusts protocol liquidity with new values of TAO and Alpha reserve
    pub(super) fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        // Update protocol position with new liquidity
        let protocol_account_id = Self::protocol_account_id();
        let mut positions =
            Positions::<T>::iter_prefix_values((netuid, protocol_account_id.clone()))
                .collect::<sp_std::vec::Vec<_>>();

        if let Some(position) = positions.get_mut(0) {
            // Claim protocol fees and add them to liquidity
            let (tao_fees, alpha_fees) = position.collect_fees();

            // Add fee reservoirs and get proportional amounts
            let current_sqrt_price = AlphaSqrtPrice::<T>::get(netuid);
            let tao_reservoir = ScrapReservoirTao::<T>::get(netuid);
            let alpha_reservoir = ScrapReservoirAlpha::<T>::get(netuid);
            let (corrected_tao_delta, corrected_alpha_delta, tao_scrap, alpha_scrap) =
                Self::get_proportional_alpha_tao_and_remainders(
                    current_sqrt_price,
                    tao_delta
                        .saturating_add(TaoCurrency::from(tao_fees))
                        .saturating_add(tao_reservoir),
                    alpha_delta
                        .saturating_add(AlphaCurrency::from(alpha_fees))
                        .saturating_add(alpha_reservoir),
                );

            // Update scrap reservoirs
            ScrapReservoirTao::<T>::insert(netuid, tao_scrap);
            ScrapReservoirAlpha::<T>::insert(netuid, alpha_scrap);

            // Adjust liquidity
            let maybe_token_amounts = position.to_token_amounts(current_sqrt_price);
            if let Ok((tao, alpha)) = maybe_token_amounts {
                // Get updated reserves, calculate liquidity
                let new_tao_reserve = tao.saturating_add(corrected_tao_delta.to_u64());
                let new_alpha_reserve = alpha.saturating_add(corrected_alpha_delta.to_u64());
                let new_liquidity = helpers_128bit::sqrt(
                    (new_tao_reserve as u128).saturating_mul(new_alpha_reserve as u128),
                ) as u64;
                let liquidity_delta = new_liquidity.saturating_sub(position.liquidity);

                // Update current liquidity
                CurrentLiquidity::<T>::mutate(netuid, |current_liquidity| {
                    *current_liquidity = current_liquidity.saturating_add(liquidity_delta);
                });

                // Update protocol position
                position.liquidity = new_liquidity;
                Positions::<T>::insert(
                    (netuid, protocol_account_id, position.id),
                    position.clone(),
                );

                // Update position ticks
                Self::add_liquidity_at_index(netuid, position.tick_low, liquidity_delta, false);
                Self::add_liquidity_at_index(netuid, position.tick_high, liquidity_delta, true);
            }
        }
    }

    /// Executes a token swap on the specified subnet.
    ///
    /// # Parameters
    /// - `netuid`: The identifier of the subnet on which the swap is performed.
    /// - `order_type`: The type of the swap (e.g., Buy or Sell).
    /// - `amount`: The amount of tokens to swap.
    /// - `limit_sqrt_price`: A price limit (expressed as a square root) to bound the swap.
    /// - `simulate`: If `true`, the function runs in simulation mode and does not persist any
    ///   changes.
    ///
    /// # Returns
    /// Returns a [`Result`] with a [`SwapResult`] on success, or a [`DispatchError`] on failure.
    ///
    /// The [`SwapResult`] contains:
    /// - `amount_paid_out`: The amount of tokens received from the swap.
    /// - `refund`: Any unswapped portion of the input amount, refunded to the caller.
    ///
    /// # Simulation Mode
    /// When `simulate` is set to `true`, the function:
    /// 1. Executes all logic without persisting any state changes (i.e., performs a dry run).
    /// 2. Skips reserve checks — it may return an `amount_paid_out` greater than the available
    ///    reserve.
    ///
    /// Use simulation mode to preview the outcome of a swap without modifying the blockchain state.
    pub(crate) fn do_swap<Order>(
        netuid: NetUid,
        order: Order,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
        simulate: bool,
    ) -> Result<SwapResult<Order::PaidIn, Order::PaidOut>, DispatchError>
    where
        Order: OrderT,
        BasicSwapStep<T, Order::PaidIn, Order::PaidOut>: SwapStep<T, Order::PaidIn, Order::PaidOut>,
    {
        transactional::with_transaction(|| {
            let reserve = Order::ReserveOut::reserve(netuid.into());

            let result = Self::swap_inner::<Order>(netuid, order, limit_sqrt_price, drop_fees)
                .map_err(Into::into);

            if simulate || result.is_err() {
                // Simulation only
                TransactionOutcome::Rollback(result)
            } else {
                // Should persist changes

                // Check if reserves are overused
                if let Ok(ref swap_result) = result
                    && reserve < swap_result.amount_paid_out
                {
                    return TransactionOutcome::Commit(Err(
                        Error::<T>::InsufficientLiquidity.into()
                    ));
                }

                TransactionOutcome::Commit(result)
            }
        })
    }

    fn swap_inner<Order>(
        netuid: NetUid,
        order: Order,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
    ) -> Result<SwapResult<Order::PaidIn, Order::PaidOut>, Error<T>>
    where
        Order: OrderT,
        BasicSwapStep<T, Order::PaidIn, Order::PaidOut>: SwapStep<T, Order::PaidIn, Order::PaidOut>,
    {
        ensure!(
            Order::ReserveOut::reserve(netuid).to_u64() >= T::MinimumReserve::get().get(),
            Error::<T>::ReservesTooLow
        );

        Self::maybe_initialize_v3(netuid)?;

        // Because user specifies the limit price, check that it is in fact beoynd the current one
        ensure!(
            order.is_beyond_price_limit(AlphaSqrtPrice::<T>::get(netuid), limit_sqrt_price),
            Error::<T>::PriceLimitExceeded
        );

        let mut amount_remaining = order.amount();
        let mut amount_paid_out = Order::PaidOut::ZERO;
        let mut iteration_counter: u16 = 0;
        let mut in_acc = Order::PaidIn::ZERO;
        let mut fee_acc = Order::PaidIn::ZERO;

        log::trace!("======== Start Swap ========");
        log::trace!("Amount Remaining: {amount_remaining}");

        // Swap one tick at a time until we reach one of the stop conditions
        while !amount_remaining.is_zero() {
            log::trace!("\nIteration: {iteration_counter}");
            log::trace!(
                "\tCurrent Liquidity: {}",
                CurrentLiquidity::<T>::get(netuid)
            );

            // Create and execute a swap step
            let mut swap_step = BasicSwapStep::<T, Order::PaidIn, Order::PaidOut>::new(
                netuid,
                amount_remaining,
                limit_sqrt_price,
                drop_fees,
            );

            let swap_result = swap_step.execute()?;

            in_acc = in_acc.saturating_add(swap_result.delta_in);
            fee_acc = fee_acc.saturating_add(swap_result.fee_paid);
            amount_remaining = amount_remaining.saturating_sub(swap_result.amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(swap_result.delta_out);

            if swap_step.action() == SwapStepAction::Stop {
                amount_remaining = Order::PaidIn::ZERO;
            }

            // The swap step didn't exchange anything
            if swap_result.amount_to_take.is_zero() {
                amount_remaining = Order::PaidIn::ZERO;
            }

            iteration_counter = iteration_counter.saturating_add(1);

            ensure!(
                iteration_counter <= MAX_SWAP_ITERATIONS,
                Error::<T>::TooManySwapSteps
            );
        }

        log::trace!("\nAmount Paid Out: {amount_paid_out}");
        log::trace!("======== End Swap ========");

        Ok(SwapResult {
            amount_paid_in: in_acc,
            amount_paid_out,
            fee_paid: fee_acc,
        })
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    pub(crate) fn calculate_fee_amount<C: Currency>(
        netuid: NetUid,
        amount: C,
        drop_fees: bool,
    ) -> C {
        if drop_fees {
            return C::ZERO;
        }

        match T::SubnetInfo::mechanism(netuid) {
            1 => {
                let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(netuid))
                    .safe_div(U64F64::saturating_from_num(u16::MAX));
                U64F64::saturating_from_num(amount)
                    .saturating_mul(fee_rate)
                    .saturating_to_num::<u64>()
                    .into()
            }
            _ => C::ZERO,
        }
    }

    pub fn find_closest_lower_active_tick(netuid: NetUid, index: TickIndex) -> Option<Tick> {
        ActiveTickIndexManager::<T>::find_closest_lower(netuid, index)
            .and_then(|ti| Ticks::<T>::get(netuid, ti))
    }

    pub fn find_closest_higher_active_tick(netuid: NetUid, index: TickIndex) -> Option<Tick> {
        ActiveTickIndexManager::<T>::find_closest_higher(netuid, index)
            .and_then(|ti| Ticks::<T>::get(netuid, ti))
    }

    /// Here we subtract minimum safe liquidity from current liquidity to stay in the safe range
    pub(crate) fn current_liquidity_safe(netuid: NetUid) -> U64F64 {
        U64F64::saturating_from_num(
            CurrentLiquidity::<T>::get(netuid).saturating_sub(T::MinimumLiquidity::get()),
        )
    }

    /// Adds liquidity to the specified price range.
    ///
    /// This function allows an account to provide liquidity to a given range of price ticks. The
    /// amount of liquidity to be added can be determined using
    /// [`get_tao_based_liquidity`] and [`get_alpha_based_liquidity`], which compute the required
    /// liquidity based on TAO and Alpha balances for the current price tick.
    ///
    /// ### Behavior:
    /// - If the `protocol` flag is **not set** (`false`), the function will attempt to
    ///   **withdraw balances** from the account using `state_ops.withdraw_balances()`.
    /// - If the `protocol` flag is **set** (`true`), the liquidity is added without modifying balances.
    /// - If swap V3 was not initialized before, updates the value in storage.
    ///
    /// ### Parameters:
    /// - `coldkey_account_id`: A reference to the account coldkey that is providing liquidity.
    /// - `hotkey_account_id`: A reference to the account hotkey that is providing liquidity.
    /// - `tick_low`: The lower bound of the price tick range.
    /// - `tick_high`: The upper bound of the price tick range.
    /// - `liquidity`: The amount of liquidity to be added.
    ///
    /// ### Returns:
    /// - `Ok((u64, u64))`: (tao, alpha) amounts at new position
    /// - `Err(SwapError)`: If the operation fails due to insufficient balance, invalid tick range,
    ///   or other swap-related errors.
    ///
    /// ### Errors:
    /// - [`SwapError::InsufficientBalance`] if the account does not have enough balance.
    /// - [`SwapError::InvalidTickRange`] if `tick_low` is greater than or equal to `tick_high`.
    /// - Other [`SwapError`] variants as applicable.
    pub fn do_add_liquidity(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        hotkey_account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(PositionId, u64, u64), Error<T>> {
        ensure!(
            EnabledUserLiquidity::<T>::get(netuid),
            Error::<T>::UserLiquidityDisabled
        );

        let (position, tao, alpha) = Self::add_liquidity_not_insert(
            netuid,
            coldkey_account_id,
            tick_low,
            tick_high,
            liquidity,
        )?;
        let position_id = position.id;

        ensure!(
            T::BalanceOps::tao_balance(coldkey_account_id) >= TaoCurrency::from(tao)
                && T::BalanceOps::alpha_balance(
                    netuid.into(),
                    coldkey_account_id,
                    hotkey_account_id
                ) >= AlphaCurrency::from(alpha),
            Error::<T>::InsufficientBalance
        );

        // Small delta is not allowed
        ensure!(
            liquidity >= T::MinimumLiquidity::get(),
            Error::<T>::InvalidLiquidityValue
        );

        Positions::<T>::insert(&(netuid, coldkey_account_id, position.id), position);

        Ok((position_id, tao, alpha))
    }

    // add liquidity without inserting position into storage (used privately for v3 intiialization).
    // unlike Self::add_liquidity it also doesn't perform account's balance check.
    //
    // the public interface is [`Self::add_liquidity`]
    fn add_liquidity_not_insert(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(Position<T>, u64, u64), Error<T>> {
        ensure!(
            Self::count_positions(netuid, coldkey_account_id) < T::MaxPositions::get() as usize,
            Error::<T>::MaxPositionsExceeded
        );

        // Ensure that tick_high is actually higher than tick_low
        ensure!(tick_high > tick_low, Error::<T>::InvalidTickRange);

        // Add liquidity at tick
        Self::add_liquidity_at_index(netuid, tick_low, liquidity, false);
        Self::add_liquidity_at_index(netuid, tick_high, liquidity, true);

        // Update current tick liquidity
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        Self::clamp_sqrt_price(netuid, current_tick_index);

        Self::update_liquidity_if_needed(netuid, tick_low, tick_high, liquidity as i128);

        // New position
        let position_id = PositionId::new::<T>();
        let position = Position::new(position_id, netuid, tick_low, tick_high, liquidity);

        let current_price_sqrt = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price_sqrt)?;

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((position, tao, alpha))
    }

    /// Remove liquidity and credit balances back to (coldkey_account_id, hotkey_account_id) stake.
    /// Removing is allowed even when user liquidity is enabled.
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn do_remove_liquidity(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        position_id: PositionId,
    ) -> Result<RemoveLiquidityResult, Error<T>> {
        let Some(mut position) = Positions::<T>::get((netuid, coldkey_account_id, position_id))
        else {
            return Err(Error::<T>::LiquidityNotFound);
        };

        // Collect fees and get tao and alpha amounts
        let (fee_tao, fee_alpha) = position.collect_fees();
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price)?;

        // Update liquidity at position ticks
        Self::remove_liquidity_at_index(netuid, position.tick_low, position.liquidity, false);
        Self::remove_liquidity_at_index(netuid, position.tick_high, position.liquidity, true);

        // Update current tick liquidity
        Self::update_liquidity_if_needed(
            netuid,
            position.tick_low,
            position.tick_high,
            (position.liquidity as i128).neg(),
        );

        // Remove user position
        Positions::<T>::remove((netuid, coldkey_account_id, position_id));

        Ok(RemoveLiquidityResult {
            tao: tao.into(),
            alpha: alpha.into(),
            fee_tao: fee_tao.into(),
            fee_alpha: fee_alpha.into(),
            tick_low: position.tick_low,
            tick_high: position.tick_high,
            liquidity: position.liquidity,
        })
    }

    pub fn do_modify_position(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        hotkey_account_id: &T::AccountId,
        position_id: PositionId,
        liquidity_delta: i64,
    ) -> Result<UpdateLiquidityResult, Error<T>> {
        ensure!(
            EnabledUserLiquidity::<T>::get(netuid),
            Error::<T>::UserLiquidityDisabled
        );

        // Find the position
        let Some(mut position) = Positions::<T>::get((netuid, coldkey_account_id, position_id))
        else {
            return Err(Error::<T>::LiquidityNotFound);
        };

        // Small delta is not allowed
        ensure!(
            liquidity_delta.abs() >= T::MinimumLiquidity::get() as i64,
            Error::<T>::InvalidLiquidityValue
        );
        let mut delta_liquidity_abs = liquidity_delta.unsigned_abs();

        // Determine the effective price for token calculations
        let current_price_sqrt = AlphaSqrtPrice::<T>::get(netuid);
        let sqrt_pa: SqrtPrice = position
            .tick_low
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice = position
            .tick_high
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_price_box = if current_price_sqrt < sqrt_pa {
            sqrt_pa
        } else if current_price_sqrt > sqrt_pb {
            sqrt_pb
        } else {
            // Update current liquidity if price is in range
            let new_liquidity_curr = if liquidity_delta > 0 {
                CurrentLiquidity::<T>::get(netuid).saturating_add(delta_liquidity_abs)
            } else {
                CurrentLiquidity::<T>::get(netuid).saturating_sub(delta_liquidity_abs)
            };
            CurrentLiquidity::<T>::set(netuid, new_liquidity_curr);
            current_price_sqrt
        };

        // Calculate token amounts for the liquidity change
        let mul = SqrtPrice::from_num(1)
            .safe_div(sqrt_price_box)
            .saturating_sub(SqrtPrice::from_num(1).safe_div(sqrt_pb));
        let alpha = SqrtPrice::saturating_from_num(delta_liquidity_abs).saturating_mul(mul);
        let tao = SqrtPrice::saturating_from_num(delta_liquidity_abs)
            .saturating_mul(sqrt_price_box.saturating_sub(sqrt_pa));

        // Validate delta
        if liquidity_delta > 0 {
            // Check that user has enough balances
            ensure!(
                T::BalanceOps::tao_balance(coldkey_account_id)
                    >= TaoCurrency::from(tao.saturating_to_num::<u64>())
                    && T::BalanceOps::alpha_balance(netuid, coldkey_account_id, hotkey_account_id)
                        >= AlphaCurrency::from(alpha.saturating_to_num::<u64>()),
                Error::<T>::InsufficientBalance
            );
        } else {
            // Check that position has enough liquidity
            ensure!(
                position.liquidity >= delta_liquidity_abs,
                Error::<T>::InsufficientLiquidity
            );
        }

        // Collect fees
        let (fee_tao, fee_alpha) = position.collect_fees();

        // If delta brings the position liquidity below MinimumLiquidity, eliminate position and
        // withdraw full amounts
        let mut remove = false;
        if (liquidity_delta < 0)
            && (position.liquidity.saturating_sub(delta_liquidity_abs) < T::MinimumLiquidity::get())
        {
            delta_liquidity_abs = position.liquidity;
            remove = true;
        }

        // Adjust liquidity at the ticks based on the delta sign
        if liquidity_delta > 0 {
            // Add liquidity at tick
            Self::add_liquidity_at_index(netuid, position.tick_low, delta_liquidity_abs, false);
            Self::add_liquidity_at_index(netuid, position.tick_high, delta_liquidity_abs, true);

            // Add liquidity to user position
            position.liquidity = position.liquidity.saturating_add(delta_liquidity_abs);
        } else {
            // Remove liquidity at tick
            Self::remove_liquidity_at_index(netuid, position.tick_low, delta_liquidity_abs, false);
            Self::remove_liquidity_at_index(netuid, position.tick_high, delta_liquidity_abs, true);

            // Remove liquidity from user position
            position.liquidity = position.liquidity.saturating_sub(delta_liquidity_abs);
        }

        // Update or, in case if full liquidity is removed, remove the position
        if remove {
            Positions::<T>::remove((netuid, coldkey_account_id, position_id));
        } else {
            Positions::<T>::insert(&(netuid, coldkey_account_id, position.id), position.clone());
        }

        Ok(UpdateLiquidityResult {
            tao: tao.saturating_to_num::<u64>().into(),
            alpha: alpha.saturating_to_num::<u64>().into(),
            fee_tao: fee_tao.into(),
            fee_alpha: fee_alpha.into(),
            removed: remove,
            tick_low: position.tick_low,
            tick_high: position.tick_high,
        })
    }

    /// Adds or updates liquidity at a specific tick index for a subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet ID
    /// * `tick_index` - The tick index to add liquidity to
    /// * `liquidity` - The amount of liquidity to add
    fn add_liquidity_at_index(netuid: NetUid, tick_index: TickIndex, liquidity: u64, upper: bool) {
        // Convert liquidity to signed value, negating it for upper bounds
        let net_liquidity_change = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        Ticks::<T>::mutate(netuid, tick_index, |maybe_tick| match maybe_tick {
            Some(tick) => {
                tick.liquidity_net = tick.liquidity_net.saturating_add(net_liquidity_change);
                tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
            }
            None => {
                let current_tick = TickIndex::current_bounded::<T>(netuid);

                let (fees_out_tao, fees_out_alpha) = if tick_index > current_tick {
                    (
                        I64F64::saturating_from_num(FeeGlobalTao::<T>::get(netuid)),
                        I64F64::saturating_from_num(FeeGlobalAlpha::<T>::get(netuid)),
                    )
                } else {
                    (
                        I64F64::saturating_from_num(0),
                        I64F64::saturating_from_num(0),
                    )
                };
                *maybe_tick = Some(Tick {
                    liquidity_net: net_liquidity_change,
                    liquidity_gross: liquidity,
                    fees_out_tao,
                    fees_out_alpha,
                });
            }
        });

        // Update active ticks
        ActiveTickIndexManager::<T>::insert(netuid, tick_index);
    }

    /// Remove liquidity at tick index.
    fn remove_liquidity_at_index(
        netuid: NetUid,
        tick_index: TickIndex,
        liquidity: u64,
        upper: bool,
    ) {
        // Calculate net liquidity addition
        let net_reduction = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        Ticks::<T>::mutate_exists(netuid, tick_index, |maybe_tick| {
            if let Some(tick) = maybe_tick {
                tick.liquidity_net = tick.liquidity_net.saturating_sub(net_reduction);
                tick.liquidity_gross = tick.liquidity_gross.saturating_sub(liquidity);

                // If no liquidity is left at the tick, remove it
                if tick.liquidity_gross == 0 {
                    *maybe_tick = None;

                    // Update active ticks: Final liquidity is zero, remove this tick from active.
                    ActiveTickIndexManager::<T>::remove(netuid, tick_index);
                }
            }
        });
    }

    /// Updates the current liquidity for a subnet if the current tick index is within the specified
    /// range
    ///
    /// This function handles both increasing and decreasing liquidity based on the sign of the
    /// liquidity parameter. It uses i128 to safely handle values up to u64::MAX in both positive
    /// and negative directions.
    fn update_liquidity_if_needed(
        netuid: NetUid,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: i128,
    ) {
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        if (tick_low <= current_tick_index) && (current_tick_index < tick_high) {
            CurrentLiquidity::<T>::mutate(netuid, |current_liquidity| {
                let is_neg = liquidity.is_negative();
                let liquidity = liquidity.abs().min(u64::MAX as i128) as u64;
                if is_neg {
                    *current_liquidity = current_liquidity.saturating_sub(liquidity);
                } else {
                    *current_liquidity = current_liquidity.saturating_add(liquidity);
                }
            });
        }
    }

    /// Clamps the subnet's sqrt price when tick index is outside of valid bounds
    fn clamp_sqrt_price(netuid: NetUid, tick_index: TickIndex) {
        if tick_index >= TickIndex::MAX || tick_index <= TickIndex::MIN {
            let corrected_price = tick_index.as_sqrt_price_bounded();
            AlphaSqrtPrice::<T>::set(netuid, corrected_price);
        }
    }

    /// Returns the number of positions for an account in a specific subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet ID
    /// * `account_id` - The account ID
    ///
    /// # Returns
    /// The number of positions that the account has in the specified subnet
    pub(super) fn count_positions(netuid: NetUid, account_id: &T::AccountId) -> usize {
        Positions::<T>::iter_prefix_values((netuid, account_id.clone())).count()
    }

    /// Returns the protocol account ID
    ///
    /// # Returns
    /// The account ID of the protocol account
    pub fn protocol_account_id() -> T::AccountId {
        T::ProtocolId::get().into_account_truncating()
    }

    pub(crate) fn min_price_inner<C: Currency>() -> C {
        TickIndex::min_sqrt_price()
            .saturating_mul(TickIndex::min_sqrt_price())
            .saturating_mul(SqrtPrice::saturating_from_num(1_000_000_000))
            .saturating_to_num::<u64>()
            .into()
    }

    pub(crate) fn max_price_inner<C: Currency>() -> C {
        TickIndex::max_sqrt_price()
            .saturating_mul(TickIndex::max_sqrt_price())
            .saturating_mul(SqrtPrice::saturating_from_num(1_000_000_000))
            .saturating_round()
            .saturating_to_num::<u64>()
            .into()
    }

    /// Dissolve all LPs and clean state.
    pub fn do_dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        if SwapV3Initialized::<T>::get(netuid) {
            // 1) Snapshot only *non‑protocol* positions: (owner, position_id).
            struct CloseItem<A> {
                owner: A,
                pos_id: PositionId,
            }
            let protocol_account = Self::protocol_account_id();

            let mut to_close: sp_std::vec::Vec<CloseItem<T::AccountId>> = sp_std::vec::Vec::new();
            for ((owner, pos_id), _pos) in Positions::<T>::iter_prefix((netuid,)) {
                if owner != protocol_account {
                    to_close.push(CloseItem { owner, pos_id });
                }
            }

            if to_close.is_empty() {
                log::debug!(
                    "dissolve_all_lp: no user positions; netuid={netuid:?}, protocol liquidity untouched"
                );
                return Ok(());
            }

            let mut user_refunded_tao = TaoCurrency::ZERO;
            let mut user_staked_alpha = AlphaCurrency::ZERO;

            let trust: Vec<u16> = T::SubnetInfo::get_validator_trust(netuid.into());
            let permit: Vec<bool> = T::SubnetInfo::get_validator_permit(netuid.into());

            // Helper: pick target validator uid, only among permitted validators, by highest trust.
            let pick_target_uid = |trust: &Vec<u16>, permit: &Vec<bool>| -> Option<u16> {
                let mut best_uid: Option<usize> = None;
                let mut best_trust: u16 = 0;
                for (i, (&t, &p)) in trust.iter().zip(permit.iter()).enumerate() {
                    if p && (best_uid.is_none() || t > best_trust) {
                        best_uid = Some(i);
                        best_trust = t;
                    }
                }
                best_uid.map(|i| i as u16)
            };

            for CloseItem { owner, pos_id } in to_close.into_iter() {
                match Self::do_remove_liquidity(netuid, &owner, pos_id) {
                    Ok(rm) => {
                        // α withdrawn from the pool = principal + accrued fees
                        let alpha_total_from_pool: AlphaCurrency =
                            rm.alpha.saturating_add(rm.fee_alpha);

                        // ---------------- USER: refund τ and convert α → stake ----------------

                        // 1) Refund τ principal directly.
                        let tao_total_from_pool: TaoCurrency = rm.tao.saturating_add(rm.fee_tao);
                        if tao_total_from_pool > TaoCurrency::ZERO {
                            T::BalanceOps::increase_balance(&owner, tao_total_from_pool);
                            user_refunded_tao =
                                user_refunded_tao.saturating_add(tao_total_from_pool);
                            T::TaoReserve::decrease_provided(netuid, tao_total_from_pool);
                        }

                        // 2) Stake ALL withdrawn α (principal + fees) to the best permitted validator.
                        if alpha_total_from_pool > AlphaCurrency::ZERO {
                            if let Some(target_uid) = pick_target_uid(&trust, &permit) {
                                let validator_hotkey: T::AccountId =
                                    T::SubnetInfo::hotkey_of_uid(netuid.into(), target_uid).ok_or(
                                        sp_runtime::DispatchError::Other(
                                            "validator_hotkey_missing",
                                        ),
                                    )?;

                                // Stake α from LP owner (coldkey) to chosen validator (hotkey).
                                T::BalanceOps::increase_stake(
                                    &owner,
                                    &validator_hotkey,
                                    netuid,
                                    alpha_total_from_pool,
                                )?;

                                user_staked_alpha =
                                    user_staked_alpha.saturating_add(alpha_total_from_pool);

                                log::debug!(
                                    "dissolve_all_lp: user dissolved & staked α: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_staked={alpha_total_from_pool:?}, target_uid={target_uid}"
                                );
                            } else {
                                // No permitted validators; burn to avoid balance drift.
                                log::debug!(
                                    "dissolve_all_lp: no permitted validators; α burned: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_total={alpha_total_from_pool:?}"
                                );
                            }

                            T::AlphaReserve::decrease_provided(netuid, alpha_total_from_pool);
                        }
                    }
                    Err(e) => {
                        log::debug!(
                            "dissolve_all_lp: force-close failed: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, err={e:?}"
                        );
                        continue;
                    }
                }
            }

            log::debug!(
                "dissolve_all_liquidity_providers (users-only): netuid={netuid:?}, users_refunded_total_τ={user_refunded_tao:?}, users_staked_total_α={user_staked_alpha:?}; protocol liquidity untouched"
            );

            return Ok(());
        }

        log::debug!(
            "dissolve_all_liquidity_providers: netuid={netuid:?}, mode=V2-or-nonV3, leaving all liquidity/state intact"
        );

        Ok(())
    }

    /// Clear **protocol-owned** liquidity and wipe all swap state for `netuid`.
    pub fn do_clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        let protocol_account = Self::protocol_account_id();

        // 1) Force-close only protocol positions, burning proceeds.
        let mut burned_tao = TaoCurrency::ZERO;
        let mut burned_alpha = AlphaCurrency::ZERO;

        // Collect protocol position IDs first to avoid mutating while iterating.
        let protocol_pos_ids: sp_std::vec::Vec<PositionId> = Positions::<T>::iter_prefix((netuid,))
            .filter_map(|((owner, pos_id), _)| {
                if owner == protocol_account {
                    Some(pos_id)
                } else {
                    None
                }
            })
            .collect();

        for pos_id in protocol_pos_ids {
            match Self::do_remove_liquidity(netuid, &protocol_account, pos_id) {
                Ok(rm) => {
                    let alpha_total_from_pool: AlphaCurrency =
                        rm.alpha.saturating_add(rm.fee_alpha);
                    let tao_total_from_pool: TaoCurrency = rm.tao.saturating_add(rm.fee_tao);

                    if tao_total_from_pool > TaoCurrency::ZERO {
                        burned_tao = burned_tao.saturating_add(tao_total_from_pool);
                    }
                    if alpha_total_from_pool > AlphaCurrency::ZERO {
                        burned_alpha = burned_alpha.saturating_add(alpha_total_from_pool);
                    }

                    log::debug!(
                        "clear_protocol_liquidity: burned protocol pos: netuid={netuid:?}, pos_id={pos_id:?}, τ={tao_total_from_pool:?}, α_total={alpha_total_from_pool:?}"
                    );
                }
                Err(e) => {
                    log::debug!(
                        "clear_protocol_liquidity: force-close failed: netuid={netuid:?}, pos_id={pos_id:?}, err={e:?}"
                    );
                    continue;
                }
            }
        }

        // 2) Clear active tick index entries, then all swap state (idempotent even if empty/non‑V3).
        let active_ticks: sp_std::vec::Vec<TickIndex> =
            Ticks::<T>::iter_prefix(netuid).map(|(ti, _)| ti).collect();
        for ti in active_ticks {
            ActiveTickIndexManager::<T>::remove(netuid, ti);
        }

        let _ = Positions::<T>::clear_prefix((netuid,), u32::MAX, None);
        let _ = Ticks::<T>::clear_prefix(netuid, u32::MAX, None);

        FeeGlobalTao::<T>::remove(netuid);
        FeeGlobalAlpha::<T>::remove(netuid);
        CurrentLiquidity::<T>::remove(netuid);
        CurrentTick::<T>::remove(netuid);
        AlphaSqrtPrice::<T>::remove(netuid);
        SwapV3Initialized::<T>::remove(netuid);

        let _ = TickIndexBitmapWords::<T>::clear_prefix((netuid,), u32::MAX, None);
        FeeRate::<T>::remove(netuid);
        EnabledUserLiquidity::<T>::remove(netuid);
        ScrapReservoirTao::<T>::remove(netuid);
        ScrapReservoirAlpha::<T>::remove(netuid);

        log::debug!(
            "clear_protocol_liquidity: netuid={netuid:?}, protocol_burned: τ={burned_tao:?}, α={burned_alpha:?}; state cleared"
        );

        Ok(())
    }
}

impl<T: Config> DefaultPriceLimit<TaoCurrency, AlphaCurrency> for Pallet<T> {
    fn default_price_limit<C: Currency>() -> C {
        Self::max_price_inner::<C>()
    }
}

impl<T: Config> DefaultPriceLimit<AlphaCurrency, TaoCurrency> for Pallet<T> {
    fn default_price_limit<C: Currency>() -> C {
        Self::min_price_inner::<C>()
    }
}

impl<T, O> SwapEngine<O> for Pallet<T>
where
    T: Config,
    O: OrderT,
    BasicSwapStep<T, O::PaidIn, O::PaidOut>: SwapStep<T, O::PaidIn, O::PaidOut>,
    Self: DefaultPriceLimit<O::PaidIn, O::PaidOut>,
{
    fn swap(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError> {
        let limit_sqrt_price = SqrtPrice::saturating_from_num(price_limit.to_u64())
            .safe_div(SqrtPrice::saturating_from_num(1_000_000_000))
            .checked_sqrt(SqrtPrice::saturating_from_num(0.0000000001))
            .ok_or(Error::<T>::PriceLimitExceeded)?;

        Self::do_swap::<O>(
            NetUid::from(netuid),
            order,
            limit_sqrt_price,
            drop_fees,
            should_rollback,
        )
        .map_err(Into::into)
    }
}

impl<T: Config> SwapHandler for Pallet<T> {
    fn swap<O>(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        O: OrderT,
        Self: SwapEngine<O>,
    {
        <Self as SwapEngine<O>>::swap(netuid, order, price_limit, drop_fees, should_rollback)
    }

    fn sim_swap<O>(
        netuid: NetUid,
        order: O,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        O: OrderT,
        Self: SwapEngine<O>,
    {
        match T::SubnetInfo::mechanism(netuid) {
            1 => {
                let price_limit = Self::default_price_limit::<TaoCurrency>();

                <Self as SwapEngine<O>>::swap(netuid, order, price_limit, false, true)
            }
            _ => {
                let actual_amount = if T::SubnetInfo::exists(netuid) {
                    order.amount()
                } else {
                    O::PaidIn::ZERO
                };
                Ok(SwapResult {
                    amount_paid_in: actual_amount,
                    amount_paid_out: actual_amount.to_u64().into(),
                    fee_paid: 0.into(),
                })
            }
        }
    }

    fn approx_fee_amount<C: Currency>(netuid: NetUid, amount: C) -> C {
        Self::calculate_fee_amount(netuid, amount, false)
    }

    fn current_alpha_price(netuid: NetUid) -> U96F32 {
        Self::current_price(netuid.into())
    }

    fn get_protocol_tao(netuid: NetUid) -> TaoCurrency {
        let protocol_account_id = Self::protocol_account_id();
        let mut positions =
            Positions::<T>::iter_prefix_values((netuid, protocol_account_id.clone()))
                .collect::<sp_std::vec::Vec<_>>();

        if let Some(position) = positions.get_mut(0) {
            let current_sqrt_price = AlphaSqrtPrice::<T>::get(netuid);
            // Adjust liquidity
            let maybe_token_amounts = position.to_token_amounts(current_sqrt_price);
            if let Ok((tao, _)) = maybe_token_amounts {
                return tao.into();
            }
        }

        TaoCurrency::ZERO
    }

    fn min_price<C: Currency>() -> C {
        Self::min_price_inner()
    }

    fn max_price<C: Currency>() -> C {
        Self::max_price_inner()
    }

    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        Self::adjust_protocol_liquidity(netuid, tao_delta, alpha_delta);
    }

    fn is_user_liquidity_enabled(netuid: NetUid) -> bool {
        EnabledUserLiquidity::<T>::get(netuid)
    }
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        Self::do_dissolve_all_liquidity_providers(netuid)
    }
    fn toggle_user_liquidity(netuid: NetUid, enabled: bool) {
        EnabledUserLiquidity::<T>::insert(netuid, enabled)
    }
    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        Self::do_clear_protocol_liquidity(netuid)
    }
}
