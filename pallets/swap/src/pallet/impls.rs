use frame_support::{ensure, traits::Get};
use pallet_subtensor_swap_interface::LiquidityDataProvider;
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::traits::AccountIdConversion;
use substrate_fixed::types::U64F64;

use super::pallet::*;
use crate::{
    NetUid, RemoveLiquidityResult,
    position::{Position, PositionId},
    tick::{Tick, TickIndex},
};

impl<T: Config> Pallet<T> {
    // initializes V3 swap for a subnet if needed
    fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
        if SwapV3Initialized::<T>::get(netuid) {
            return Ok(());
        }

        // Initialize the v3:
        // Reserves are re-purposed, nothing to set, just query values for liquidity and price calculation
        let tao_reserve = <T as Config>::LiquidityDataProvider::tao_reserve();
        let alpha_reserve = <T as Config>::LiquidityDataProvider::alpha_reserve();

        // Set price
        let price = U64F64::saturating_from_num(tao_reserve)
            .safe_div(U64F64::saturating_from_num(alpha_reserve));

        let epsilon = U64F64::saturating_from_num(0.000001);

        AlphaSqrtPrice::<T>::set(
            netuid,
            price.checked_sqrt(epsilon).unwrap_or(U64F64::from_num(0)),
        );

        // Set initial (protocol owned) liquidity and positions
        // Protocol liquidity makes one position from TickIndex::MIN to TickIndex::MAX
        // We are using the sp_arithmetic sqrt here, which works for u128
        let liquidity =
            helpers_128bit::sqrt((tao_reserve as u128).saturating_mul(alpha_reserve as u128))
                as u64;
        let protocol_account_id = T::ProtocolId::get().into_account_truncating();

        let _ = Self::add_liquidity(
            netuid,
            &protocol_account_id,
            TickIndex::MIN,
            TickIndex::MAX,
            liquidity,
        )?;

        Ok(())
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
    /// - `account_id`: A reference to the account that is providing liquidity.
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
    pub fn add_liquidity(
        netuid: NetUid,
        account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(u64, u64), Error<T>> {
        ensure!(
            Self::count_positions(netuid, account_id) <= T::MaxPositions::get() as usize,
            Error::<T>::MaxPositionsExceeded
        );

        // Add liquidity at tick
        Self::add_liquidity_at_index(netuid, tick_low, liquidity, false);
        Self::add_liquidity_at_index(netuid, tick_high, liquidity, true);

        // Update current tick liquidity
        let current_tick_index = Self::bounded_current_tick_index(netuid);
        Self::clamp_sqrt_price(netuid, current_tick_index);

        Self::update_liquidity_if_needed(netuid, tick_low, tick_high, liquidity as i128);

        // New position
        let position = Position {
            id: PositionId::new(),
            tick_low,
            tick_high,
            liquidity,
            fees_tao: 0,
            fees_alpha: 0,
        };

        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price)?;

        // If this is a user transaction, withdraw balances and update reserves
        // TODO this should be returned (tao, alpha) from this function to prevent
        // mutation of outside storage - the logic should be passed to the user of
        // pallet_subtensor_swap_interface
        // if !protocol {
        //     let current_price = self.state_ops.get_alpha_sqrt_price();
        //     let (tao, alpha) = position.to_token_amounts(current_price)?;
        //     self.state_ops.withdraw_balances(account_id, tao, alpha)?;

        //     // Update reserves
        //     let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        //     self.state_ops.set_tao_reserve(new_tao_reserve);
        //     let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_add(alpha);
        //     self.state_ops.set_alpha_reserve(new_alpha_reserve);
        // }

        Positions::<T>::insert(&(netuid, account_id, position.id), position);

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((tao, alpha))
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
            -(liquidity as i128)
        } else {
            liquidity as i128
        };

        Ticks::<T>::mutate(netuid, tick_index, |maybe_tick| match maybe_tick {
            Some(tick) => {
                tick.liquidity_net = tick.liquidity_net.saturating_add(net_liquidity_change);
                tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
            }
            None => {
                *maybe_tick = Some(Tick {
                    liquidity_net: net_liquidity_change,
                    liquidity_gross: liquidity,
                    fees_out_tao: U64F64::from_num(0),
                    fees_out_alpha: U64F64::from_num(0),
                });
            }
        });
    }

    /// Remove liquidity and credit balances back to account_id
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn remove_liquidity(
        netuid: NetUid,
        account_id: &T::AccountId,
        position_id: PositionId,
    ) -> Result<RemoveLiquidityResult, Error<T>> {
        let Some(mut pos) = Positions::<T>::get((netuid, account_id, position_id)) else {
            return Err(Error::<T>::LiquidityNotFound);
        };
        let current_tick_index = CurrentTickIndex::<T>::get(netuid);

        // Collect fees and get tao and alpha amounts
        // let (fee_tao, fee_alpha) = self.collect_fees(&mut pos);
        //     let current_price: SqrtPrice = self.state_ops.get_alpha_sqrt_price();
        //     let (tao, alpha) = pos.to_token_amounts(current_price)?;

        //     // Update liquidity at position ticks
        //     self.remove_liquidity_at_index(pos.tick_low, pos.liquidity, false);
        //     self.remove_liquidity_at_index(pos.tick_high, pos.liquidity, true);

        //     // Update current tick liquidity
        //     if (pos.tick_low <= current_tick_index) && (current_tick_index <= pos.tick_high) {
        //         let new_current_liquidity = self
        //             .state_ops
        //             .get_current_liquidity()
        //             .saturating_sub(pos.liquidity);
        //         self.state_ops.set_current_liquidity(new_current_liquidity);
        //     }

        //     // Remove user position
        //     self.state_ops.remove_position(account_id, position_id);

        //     // Deposit balances
        //     self.state_ops.deposit_balances(account_id, tao, alpha);

        //     // Update reserves
        //     let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_sub(tao);
        //     self.state_ops.set_tao_reserve(new_tao_reserve);
        //     let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_sub(alpha);
        //     self.state_ops.set_alpha_reserve(new_alpha_reserve);

        //     // TODO: Clear with R&D
        //     // Update current price (why?)
        //     // self.state_ops.set_alpha_sqrt_price(sqrt_price);

        //     // Return Ok result
        //     Ok(RemoveLiquidityResult {
        //         tao,
        //         alpha,
        //         fee_tao,
        //         fee_alpha,
        //     })
        todo!()
    }

    /// Gets the current tick index for a subnet, ensuring it's within valid bounds
    fn bounded_current_tick_index(netuid: NetUid) -> TickIndex {
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        TickIndex::from_sqrt_price_bounded(current_price)
    }

    /// Clamps the subnet's sqrt price when tick index is outside of valid bounds
    fn clamp_sqrt_price(netuid: NetUid, tick_index: TickIndex) {
        if tick_index >= TickIndex::MAX || tick_index <= TickIndex::MIN {
            let corrected_price = tick_index.to_sqrt_price_bounded();
            AlphaSqrtPrice::<T>::set(netuid, corrected_price);
        }
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
        let current_tick_index = Self::bounded_current_tick_index(netuid);
        if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
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

    /// Returns the number of positions for an account in a specific subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet ID
    /// * `account_id` - The account ID
    ///
    /// # Returns
    /// The number of positions that the account has in the specified subnet
    fn count_positions(netuid: NetUid, account_id: &T::AccountId) -> usize {
        Positions::<T>::iter_prefix_values((netuid, account_id.clone())).count()
    }
}
