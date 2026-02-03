use frame_support::storage::{TransactionOutcome, transactional};
use frame_support::{ensure, pallet_prelude::DispatchError, traits::Get};
use safe_math::*;
use sp_arithmetic::{
    //helpers_128bit,
    Perquintill,
};
use sp_runtime::{DispatchResult, traits::AccountIdConversion};
use sp_std::vec::Vec;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{
    AlphaCurrency,
    // BalanceOps,
    Currency,
    CurrencyReserve,
    NetUid,
    SubnetInfo,
    TaoCurrency,
};
use subtensor_swap_interface::{
    DefaultPriceLimit, Order as OrderT, SwapEngine, SwapHandler, SwapResult,
};

use super::pallet::*;
use super::swap_step::{BasicSwapStep, SwapStep};
use crate::{pallet::Balancer, pallet::balancer::BalancerError, position::PositionId};

#[derive(Debug, PartialEq)]
pub struct UpdateLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub removed: bool,
}

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub liquidity: u64,
}

impl<T: Config> Pallet<T> {
    pub fn current_price(netuid: NetUid) -> U64F64 {
        match T::SubnetInfo::mechanism(netuid.into()) {
            1 => {
                let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
                if !alpha_reserve.is_zero() {
                    let tao_reserve = T::TaoReserve::reserve(netuid.into());
                    let balancer = SwapBalancer::<T>::get(netuid);
                    balancer.calculate_price(alpha_reserve.into(), tao_reserve.into())
                } else {
                    U64F64::saturating_from_num(0)
                }
            }
            _ => U64F64::saturating_from_num(1),
        }
    }

    // initializes pal-swap (balancer) for a subnet if needed
    pub fn maybe_initialize_palswap(
        netuid: NetUid,
        maybe_price: Option<U64F64>,
    ) -> Result<(), Error<T>> {
        if PalSwapInitialized::<T>::get(netuid) {
            return Ok(());
        }

        // Query reserves
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());

        // Create balancer based on price
        let balancer = Balancer::new(if let Some(price) = maybe_price {
            // Price is given, calculate weights:
            // w_quote = y / (px + y)
            let px_high = (price.saturating_to_num::<u64>() as u128)
                .saturating_mul(u64::from(alpha_reserve) as u128);
            let px_low = U64F64::saturating_from_num(alpha_reserve)
                .saturating_mul(price.frac())
                .saturating_to_num::<u128>();
            let px_plus_y = px_high
                .saturating_add(px_low)
                .saturating_add(u64::from(tao_reserve) as u128);

            // If price is given and both reserves are zero, the swap doesn't initialize
            if px_plus_y == 0u128 {
                return Err(Error::<T>::ReservesOutOfBalance);
            }
            Perquintill::from_rational(u64::from(tao_reserve) as u128, px_plus_y)
        } else {
            // No price = insert 0.5 into SwapBalancer
            Perquintill::from_rational(1_u64, 2_u64)
        })
        .map_err(|err| match err {
            BalancerError::InvalidValue => Error::<T>::ReservesOutOfBalance,
        })?;
        SwapBalancer::<T>::insert(netuid, balancer.clone());

        // Insert current liquidity
        let liquidity =
            balancer.calculate_current_liquidity(u64::from(tao_reserve), u64::from(alpha_reserve));
        CurrentLiquidity::<T>::insert(netuid, liquidity);

        // TODO: Review when/if we have user liquidity
        // Initialize the pal-swap:

        // Set initial (protocol owned) liquidity and positions
        // Protocol liquidity makes one position
        // We are using the sp_arithmetic sqrt here, which works for u128
        // let liquidity = helpers_128bit::sqrt(
        //     (tao_reserve.to_u64() as u128).saturating_mul(alpha_reserve.to_u64() as u128),
        // ) as u64;
        // let protocol_account_id = Self::protocol_account_id();

        // let (position, _, _) = Self::add_liquidity_not_insert(
        //     netuid,
        //     &protocol_account_id,
        //     TickIndex::MIN,
        //     TickIndex::MAX,
        //     liquidity,
        // )?;

        // Positions::<T>::insert(&(netuid, protocol_account_id, position.id), position);

        PalSwapInitialized::<T>::insert(netuid, true);

        Ok(())
    }

    /// Adjusts protocol liquidity with new values of TAO and Alpha reserve
    /// Returns actually added Tao and Alpha, which includes fees
    pub(super) fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) -> (TaoCurrency, AlphaCurrency) {
        // Collect fees
        // TODO: Revise when user liquidity is available
        let tao_fees = FeesTao::<T>::get(netuid);
        let alpha_fees = FeesAlpha::<T>::get(netuid);
        FeesTao::<T>::insert(netuid, TaoCurrency::ZERO);
        FeesAlpha::<T>::insert(netuid, AlphaCurrency::ZERO);
        let actual_tao_delta = tao_delta.saturating_add(tao_fees);
        let actual_alpha_delta = alpha_delta.saturating_add(alpha_fees);

        // Get reserves
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let mut balancer = SwapBalancer::<T>::get(netuid);

        // Update weights and log errors if they go out of range
        if balancer
            .update_weights_for_added_liquidity(
                u64::from(tao_reserve),
                u64::from(alpha_reserve),
                u64::from(actual_tao_delta),
                u64::from(actual_alpha_delta),
            )
            .is_err()
        {
            log::error!(
                "Reserves are out of range for emission: netuid = {}, tao = {}, alpha = {}, tao_delta = {}, alpha_delta = {}",
                netuid,
                tao_reserve,
                alpha_reserve,
                actual_tao_delta,
                actual_alpha_delta
            );
            // Return fees back into fee storage and return zeroes
            FeesTao::<T>::insert(netuid, tao_fees);
            FeesAlpha::<T>::insert(netuid, alpha_fees);
            (TaoCurrency::ZERO, AlphaCurrency::ZERO)
        } else {
            SwapBalancer::<T>::insert(netuid, balancer);
            (actual_tao_delta, actual_alpha_delta)
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
        limit_price: U64F64,
        drop_fees: bool,
        simulate: bool,
    ) -> Result<SwapResult<Order::PaidIn, Order::PaidOut>, DispatchError>
    where
        Order: OrderT,
        BasicSwapStep<T, Order::PaidIn, Order::PaidOut>: SwapStep<T, Order::PaidIn, Order::PaidOut>,
    {
        transactional::with_transaction(|| {
            let reserve = Order::ReserveOut::reserve(netuid.into());

            let result = Self::swap_inner::<Order>(netuid, order, limit_price, drop_fees)
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
        limit_price: U64F64,
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

        Self::maybe_initialize_palswap(netuid, None)?;

        // Because user specifies the limit price, check that it is in fact beoynd the current one
        ensure!(
            order.is_beyond_price_limit(Self::current_price(netuid), limit_price),
            Error::<T>::PriceLimitExceeded
        );

        log::trace!("======== Start Swap ========");
        let amount_to_swap = order.amount();
        log::trace!("Amount to swap:  {amount_to_swap}");

        // Create and execute a swap step
        let mut swap_step = BasicSwapStep::<T, Order::PaidIn, Order::PaidOut>::new(
            netuid,
            amount_to_swap,
            limit_price,
            drop_fees,
        );

        let swap_result = swap_step.execute()?;

        log::trace!("Delta out: {}", swap_result.delta_out);
        log::trace!("Fees: {}", swap_result.fee_paid);
        log::trace!("======== End Swap ========");

        Ok(SwapResult {
            amount_paid_in: swap_result.delta_in,
            amount_paid_out: swap_result.delta_out,
            fee_paid: swap_result.fee_paid,
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
        _netuid: NetUid,
        _coldkey_account_id: &T::AccountId,
        _hotkey_account_id: &T::AccountId,
        _liquidity: u64,
    ) -> Result<(PositionId, u64, u64), Error<T>> {
        // ensure!(
        //     EnabledUserLiquidity::<T>::get(netuid),
        //     Error::<T>::UserLiquidityDisabled
        // );

        // TODO: Revise when user liquidity is enabled
        Err(Error::<T>::UserLiquidityDisabled)
    }

    /// Remove liquidity and credit balances back to (coldkey_account_id, hotkey_account_id) stake.
    /// Removing is allowed even when user liquidity is enabled.
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn do_remove_liquidity(
        _netuid: NetUid,
        _coldkey_account_id: &T::AccountId,
        _position_id: PositionId,
    ) -> Result<RemoveLiquidityResult, Error<T>> {
        // TODO: Revise when user liquidity is enabled
        Err(Error::<T>::UserLiquidityDisabled)
    }

    pub fn do_modify_position(
        _netuid: NetUid,
        _coldkey_account_id: &T::AccountId,
        _hotkey_account_id: &T::AccountId,
        _position_id: PositionId,
        _liquidity_delta: i64,
    ) -> Result<UpdateLiquidityResult, Error<T>> {
        // ensure!(
        //     EnabledUserLiquidity::<T>::get(netuid),
        //     Error::<T>::UserLiquidityDisabled
        // );

        // TODO: Revise when user liquidity is enabled
        Err(Error::<T>::UserLiquidityDisabled)
    }

    // /// Returns the number of positions for an account in a specific subnet
    // ///
    // /// # Arguments
    // /// * `netuid` - The subnet ID
    // /// * `account_id` - The account ID
    // ///
    // /// # Returns
    // /// The number of positions that the account has in the specified subnet
    // pub(super) fn count_positions(netuid: NetUid, account_id: &T::AccountId) -> usize {
    //     PositionsV2::<T>::iter_prefix_values((netuid, account_id.clone())).count()
    // }

    pub(crate) fn min_price_inner<C: Currency>() -> C {
        u64::from(1_000_u64).into()
    }

    pub(crate) fn max_price_inner<C: Currency>() -> C {
        u64::from(1_000_000_000_000_000_u64).into()
    }

    /// Returns the protocol account ID
    ///
    /// # Returns
    /// The account ID of the protocol account
    pub fn protocol_account_id() -> T::AccountId {
        T::ProtocolId::get().into_account_truncating()
    }

    /// Dissolve all LPs and clean state.
    pub fn do_dissolve_all_liquidity_providers(_netuid: NetUid) -> DispatchResult {
        // TODO: Revise when user liquidity is available
        Ok(())

        // if PalSwapInitialized::<T>::get(netuid) {
        //     // 1) Snapshot only *non‑protocol* positions: (owner, position_id).
        //     struct CloseItem<A> {
        //         owner: A,
        //         pos_id: PositionId,
        //     }
        //     let protocol_account = Self::protocol_account_id();

        //     let mut to_close: sp_std::vec::Vec<CloseItem<T::AccountId>> = sp_std::vec::Vec::new();
        //     for ((owner, pos_id), _pos) in Positions::<T>::iter_prefix((netuid,)) {
        //         if owner != protocol_account {
        //             to_close.push(CloseItem { owner, pos_id });
        //         }
        //     }

        //     if to_close.is_empty() {
        //         log::debug!(
        //             "dissolve_all_lp: no user positions; netuid={netuid:?}, protocol liquidity untouched"
        //         );
        //         return Ok(());
        //     }

        //     let mut user_refunded_tao = TaoCurrency::ZERO;
        //     let mut user_staked_alpha = AlphaCurrency::ZERO;

        //     let trust: Vec<u16> = T::SubnetInfo::get_validator_trust(netuid.into());
        //     let permit: Vec<bool> = T::SubnetInfo::get_validator_permit(netuid.into());

        //     // Helper: pick target validator uid, only among permitted validators, by highest trust.
        //     let pick_target_uid = |trust: &Vec<u16>, permit: &Vec<bool>| -> Option<u16> {
        //         let mut best_uid: Option<usize> = None;
        //         let mut best_trust: u16 = 0;
        //         for (i, (&t, &p)) in trust.iter().zip(permit.iter()).enumerate() {
        //             if p && (best_uid.is_none() || t > best_trust) {
        //                 best_uid = Some(i);
        //                 best_trust = t;
        //             }
        //         }
        //         best_uid.map(|i| i as u16)
        //     };

        //     for CloseItem { owner, pos_id } in to_close.into_iter() {
        //         match Self::do_remove_liquidity(netuid, &owner, pos_id) {
        //             Ok(rm) => {
        //                 // α withdrawn from the pool = principal + accrued fees
        //                 let alpha_total_from_pool: AlphaCurrency =
        //                     rm.alpha.saturating_add(rm.fee_alpha);

        //                 // ---------------- USER: refund τ and convert α → stake ----------------

        //                 // 1) Refund τ principal directly.
        //                 let tao_total_from_pool: TaoCurrency = rm.tao.saturating_add(rm.fee_tao);
        //                 if tao_total_from_pool > TaoCurrency::ZERO {
        //                     T::BalanceOps::increase_balance(&owner, tao_total_from_pool);
        //                     user_refunded_tao =
        //                         user_refunded_tao.saturating_add(tao_total_from_pool);
        //                     T::TaoReserve::decrease_provided(netuid, tao_total_from_pool);
        //                 }

        //                 // 2) Stake ALL withdrawn α (principal + fees) to the best permitted validator.
        //                 if alpha_total_from_pool > AlphaCurrency::ZERO {
        //                     if let Some(target_uid) = pick_target_uid(&trust, &permit) {
        //                         let validator_hotkey: T::AccountId =
        //                             T::SubnetInfo::hotkey_of_uid(netuid.into(), target_uid).ok_or(
        //                                 sp_runtime::DispatchError::Other(
        //                                     "validator_hotkey_missing",
        //                                 ),
        //                             )?;

        //                         // Stake α from LP owner (coldkey) to chosen validator (hotkey).
        //                         T::BalanceOps::increase_stake(
        //                             &owner,
        //                             &validator_hotkey,
        //                             netuid,
        //                             alpha_total_from_pool,
        //                         )?;

        //                         user_staked_alpha =
        //                             user_staked_alpha.saturating_add(alpha_total_from_pool);

        //                         log::debug!(
        //                             "dissolve_all_lp: user dissolved & staked α: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_staked={alpha_total_from_pool:?}, target_uid={target_uid}"
        //                         );
        //                     } else {
        //                         // No permitted validators; burn to avoid balance drift.
        //                         log::debug!(
        //                             "dissolve_all_lp: no permitted validators; α burned: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_total={alpha_total_from_pool:?}"
        //                         );
        //                     }

        //                     T::AlphaReserve::decrease_provided(netuid, alpha_total_from_pool);
        //                 }
        //             }
        //             Err(e) => {
        //                 log::debug!(
        //                     "dissolve_all_lp: force-close failed: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, err={e:?}"
        //                 );
        //                 continue;
        //             }
        //         }
        //     }

        //     log::debug!(
        //         "dissolve_all_liquidity_providers (users-only): netuid={netuid:?}, users_refunded_total_τ={user_refunded_tao:?}, users_staked_total_α={user_staked_alpha:?}; protocol liquidity untouched"
        //     );

        //     return Ok(());
        // }

        // log::debug!(
        //     "dissolve_all_liquidity_providers: netuid={netuid:?}, mode=V2-or-nonV3, leaving all liquidity/state intact"
        // );

        // Ok(())
    }

    /// TODO: Revise when user liquidity is available
    /// Clear **protocol-owned** liquidity and wipe all swap state for `netuid`.
    pub fn do_clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        // let protocol_account = Self::protocol_account_id();

        // 1) Force-close only protocol positions, burning proceeds.
        let burned_tao = T::TaoReserve::reserve(netuid.into());
        let burned_alpha = T::AlphaReserve::reserve(netuid.into());

        T::TaoReserve::decrease_provided(netuid.into(), burned_tao);
        T::AlphaReserve::decrease_provided(netuid.into(), burned_alpha);

        // // Collect protocol position IDs first to avoid mutating while iterating.
        // let protocol_pos_ids: sp_std::vec::Vec<PositionId> = Positions::<T>::iter_prefix((netuid,))
        //     .filter_map(|((owner, pos_id), _)| {
        //         if owner == protocol_account {
        //             Some(pos_id)
        //         } else {
        //             None
        //         }
        //     })
        //     .collect();

        // for pos_id in protocol_pos_ids {
        //     match Self::do_remove_liquidity(netuid, &protocol_account, pos_id) {
        //         Ok(rm) => {
        //             let alpha_total_from_pool: AlphaCurrency =
        //                 rm.alpha.saturating_add(rm.fee_alpha);
        //             let tao_total_from_pool: TaoCurrency = rm.tao.saturating_add(rm.fee_tao);

        //             if tao_total_from_pool > TaoCurrency::ZERO {
        //                 burned_tao = burned_tao.saturating_add(tao_total_from_pool);
        //             }
        //             if alpha_total_from_pool > AlphaCurrency::ZERO {
        //                 burned_alpha = burned_alpha.saturating_add(alpha_total_from_pool);
        //             }

        //             log::debug!(
        //                 "clear_protocol_liquidity: burned protocol pos: netuid={netuid:?}, pos_id={pos_id:?}, τ={tao_total_from_pool:?}, α_total={alpha_total_from_pool:?}"
        //             );
        //         }
        //         Err(e) => {
        //             log::debug!(
        //                 "clear_protocol_liquidity: force-close failed: netuid={netuid:?}, pos_id={pos_id:?}, err={e:?}"
        //             );
        //             continue;
        //         }
        //     }
        // }

        let _ = PositionsV2::<T>::clear_prefix((netuid,), u32::MAX, None);

        FeesTao::<T>::remove(netuid);
        FeesAlpha::<T>::remove(netuid);
        PalSwapInitialized::<T>::remove(netuid);
        FeeRate::<T>::remove(netuid);
        EnabledUserLiquidity::<T>::remove(netuid);
        SwapBalancer::<T>::remove(netuid);

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
        let limit_price = U64F64::saturating_from_num(price_limit.to_u64())
            .safe_div(U64F64::saturating_from_num(1_000_000_000_u64));

        Self::do_swap::<O>(
            NetUid::from(netuid),
            order,
            limit_price,
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

    fn current_alpha_price(netuid: NetUid) -> U64F64 {
        Self::current_price(netuid.into())
    }

    fn get_protocol_tao(netuid: NetUid) -> TaoCurrency {
        let protocol_account_id = Self::protocol_account_id();
        let mut positions =
            PositionsV2::<T>::iter_prefix_values((netuid, protocol_account_id.clone()))
                .collect::<sp_std::vec::Vec<_>>();

        if let Some(position) = positions.get_mut(0) {
            let price = Self::current_price(netuid);
            // Adjust liquidity
            let maybe_token_amounts = position.to_token_amounts(price);
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
    ) -> (TaoCurrency, AlphaCurrency) {
        Self::adjust_protocol_liquidity(netuid, tao_delta, alpha_delta)
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
    fn init_swap(netuid: NetUid, maybe_price: Option<U64F64>) {
        Self::maybe_initialize_palswap(netuid, maybe_price).unwrap_or_default();
    }
    fn get_all_subnet_netuids() -> Vec<NetUid> {
        PalSwapInitialized::<T>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }
}
