use core::marker::PhantomData;
use core::ops::Neg;

use pallet_subtensor_swap_interface::OrderType;
use safe_math::*;
use substrate_fixed::types::U64F64;

use crate::{
    RemoveLiquidityResult, SqrtPrice, SwapError, SwapResult, SwapStepAction, SwapStepResult,
    position::Position,
    tick::{LayerLevel, Tick, TickIndex, TickIndexBitmap},
};

/// This trait implementation depends on Runtime and it needs to be implemented
/// in the pallet to be able to work with chain state and per subnet. All subnet
/// swaps are independent and hence netuid is abstracted away from swap implementation.
///
pub trait SwapDataOperations<AccountIdType> {
    /// Tells if v3 swap is initialized in the state. v2 only provides base and quote
    /// reserves, while v3 also stores ticks and positions, which need to be initialized
    /// at the first pool creation.
    fn is_v3_initialized(&self) -> bool;
    fn set_v3_initialized(&mut self);
    /// Returns u16::MAX normalized fee rate. For example, 0.3% is approximately 196.
    fn get_fee_rate(&self) -> u16;
    /// Minimum liquidity that is safe for rounding and integer math.
    fn get_minimum_liquidity(&self) -> u64;
    fn get_tick_by_index(&self, tick_index: TickIndex) -> Option<Tick>;
    fn insert_tick_by_index(&mut self, tick_index: TickIndex, tick: Tick);
    fn remove_tick_by_index(&mut self, tick_index: TickIndex);
    /// Minimum sqrt price across all active ticks
    fn get_min_sqrt_price(&self) -> SqrtPrice;
    /// Maximum sqrt price across all active ticks
    fn get_max_sqrt_price(&self) -> SqrtPrice;
    fn get_tao_reserve(&self) -> u64;
    fn set_tao_reserve(&mut self, tao: u64) -> u64;
    fn get_alpha_reserve(&self) -> u64;
    fn set_alpha_reserve(&mut self, alpha: u64) -> u64;
    fn get_alpha_sqrt_price(&self) -> SqrtPrice;
    fn set_alpha_sqrt_price(&mut self, sqrt_price: SqrtPrice);

    // Getters/setters for global accrued fees in alpha and tao per subnet
    fn get_fee_global_tao(&self) -> U64F64;
    fn set_fee_global_tao(&mut self, fee: U64F64);
    fn get_fee_global_alpha(&self) -> U64F64;
    fn set_fee_global_alpha(&mut self, fee: U64F64);

    /// Get current tick liquidity
    fn get_current_liquidity(&self) -> u64;
    /// Set current tick liquidity
    fn set_current_liquidity(&mut self, liquidity: u64);

    // User account operations
    fn get_protocol_account_id(&self) -> AccountIdType;
    fn get_max_positions(&self) -> u16;
    fn withdraw_balances(
        &mut self,
        account_id: &AccountIdType,
        tao: u64,
        alpha: u64,
    ) -> Result<(u64, u64), SwapError>;
    fn deposit_balances(&mut self, account_id: &AccountIdType, tao: u64, alpha: u64);
    fn get_position_count(&self, account_id: &AccountIdType) -> u16;
    fn get_position(&self, account_id: &AccountIdType, position_id: u16) -> Option<Position>;
    fn create_position(&mut self, account_id: &AccountIdType, positions: Position) -> u16;
    fn update_position(
        &mut self,
        account_id: &AccountIdType,
        position_id: u16,
        positions: Position,
    );
    fn remove_position(&mut self, account_id: &AccountIdType, position_id: u16);

    // Tick index storage
    // Storage is organized in 3 layers:
    //    Layer 0 consists of one u128 that stores 55 bits. Each bit indicates which layer 1 words are active.
    //    Layer 1 consists of up to 55 u128's that store 6932 bits for the layer 2 words.
    //    Layer 2 consists of up to 6932 u128's that store 887272 bits for active/inactive ticks.
    fn get_layer0_word(&self, word_index: u32) -> u128;
    fn get_layer1_word(&self, word_index: u32) -> u128;
    fn get_layer2_word(&self, word_index: u32) -> u128;
    fn set_layer0_word(&mut self, word_index: u32, word: u128);
    fn set_layer1_word(&mut self, word_index: u32, word: u128);
    fn set_layer2_word(&mut self, word_index: u32, word: u128);
}

/// All main swapping logic abstracted from Runtime implementation is concentrated
/// in this struct
///
#[derive(Debug)]
pub struct Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations<AccountIdType>,
{
    pub(crate) state_ops: Ops,
    phantom_key: PhantomData<AccountIdType>,
}

impl<AccountIdType, Ops> Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations<AccountIdType>,
{
    pub fn new(mut ops: Ops) -> Self {
        todo!("transfered to Pallet::maybe_initialize_v3")
    }

    /// Auxiliary method to calculate Alpha amount to match given TAO
    /// amount at the current price for liquidity.
    ///
    /// Returns (Alpha, Liquidity) tuple
    ///
    pub fn get_tao_based_liquidity(&self, _tao: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();
        todo!()
    }

    /// Auxiliary method to calculate TAO amount to match given Alpha
    /// amount at the current price for liquidity.
    ///
    /// Returns (TAO, Liquidity) tuple
    ///
    pub fn get_alpha_based_liquidity(&self, _alpha: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();

        todo!()
    }

    /// Add liquidity at tick index. Creates new tick if it doesn't exist
    ///
    fn add_liquidity_at_index(&mut self, tick_index: TickIndex, liquidity: u64, upper: bool) {
        todo!("tranfered to Pallet, but still needed here for code to compile")
    }

    /// Remove liquidity at tick index.
    ///
    fn remove_liquidity_at_index(&mut self, tick_index: TickIndex, liquidity: u64, upper: bool) {
        todo!("moved to Pallet::remove_liquidity_at_index");
    }

    /// Adds liquidity to the specified price range.
    ///
    /// This function allows an account to provide liquidity to a given range of price ticks.
    /// The amount of liquidity to be added can be determined using the functions
    /// [`get_tao_based_liquidity`] and [`get_alpha_based_liquidity`], which compute the
    /// required liquidity based on TAO and Alpha balances for the current price tick.
    ///
    /// ### Behavior:
    /// - If the `protocol` flag is **not set** (`false`), the function will attempt to
    ///   **withdraw balances** from the account using `state_ops.withdraw_balances()`.
    /// - If the `protocol` flag is **set** (`true`), the liquidity is added without modifying balances.
    ///
    /// ### Parameters:
    /// - `account_id`: A reference to the account that is providing liquidity.
    /// - `tick_low`: The lower bound of the price tick range.
    /// - `tick_high`: The upper bound of the price tick range.
    /// - `liquidity`: The amount of liquidity to be added.
    /// - `protocol`: A boolean flag indicating whether the operation is protocol-managed:
    ///   - `true` -> Do not use this value outside of this implementation. Liquidity is added **without**
    ///               withdrawing balances.
    ///   - `false` -> Use this value for all user transactions. Liquidity is added
    ///               **after withdrawing balances**.
    ///
    /// ### Returns:
    /// - `Ok(u64)`: The final liquidity amount added.
    /// - `Err(SwapError)`: If the operation fails due to insufficient balance, invalid tick range,
    ///   or other swap-related errors.
    ///
    /// ### Errors:
    /// - [`SwapError::InsufficientBalance`] if the account does not have enough balance.
    /// - [`SwapError::InvalidTickRange`] if `tick_low` is greater than or equal to `tick_high`.
    /// - Other [`SwapError`] variants as applicable.
    ///
    pub fn add_liquidity(
        &mut self,
        account_id: &AccountIdType,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
        protocol: bool,
    ) -> Result<(), SwapError> {
        todo!("transfered to Pallet::add_liquidity")
    }

    /// Remove liquidity and credit balances back to account_id
    ///
    /// Account ID and Position ID identify position in the storage map
    ///
    pub fn remove_liquidity(
        &mut self,
        account_id: &AccountIdType,
        position_id: u16,
    ) -> Result<RemoveLiquidityResult, SwapError> {
        todo!("moved to Pallet::remove_liquidity")
    }

    /// Perform a swap
    ///
    /// Returns a tuple (amount, refund), where amount is the resulting paid out amount
    ///
    pub fn swap(
        &mut self,
        order_type: &OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<SwapResult, SwapError> {
        todo!("moved to Pallet::swap")
    }

    fn get_current_tick_index(&mut self) -> TickIndex {
        todo!("moved to TickIndex::current_bounded")
    }

    /// Process a single step of a swap
    ///
    fn swap_step(
        &mut self,
        order_type: &OrderType,
        delta_in: u64,
        sqrt_price_final: SqrtPrice,
        action: SwapStepAction,
    ) -> Result<SwapStepResult, SwapError> {
        todo!("moved to Pallet::swap_step")
    }

    /// Get the square root price at the current tick edge for the given direction (order type)
    /// If order type is Buy, then price edge is the high tick bound price, otherwise it is
    /// the low tick bound price.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e.
    /// return the edge that is impossible to execute
    ///
    fn get_sqrt_price_edge(&self, order_type: &OrderType) -> SqrtPrice {
        todo!("moved to Pallet::sqrt_price_edge")
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    ///
    fn get_fee_amount(&self, amount: u64) -> u64 {
        todo!("moved to Pallet::calculate_fee_amount")
    }

    /// Here we subtract minimum safe liquidity from current liquidity to stay in the
    /// safe range
    ///
    fn get_safe_current_liquidity(&self) -> U64F64 {
        todo!("moved to Pallet::current_liquidity_safe")
    }

    /// Get the target square root price based on the input amount
    ///
    fn get_sqrt_price_target(&self, order_type: &OrderType, delta_in: u64) -> SqrtPrice {
        todo!("moved to Pallet::sqrt_price_target")
    }

    /// Get the target quantity, which is
    ///     `1 / (target square root price)` in case of sell order
    ///     `target square root price` in case of buy order
    ///
    /// ...based on the input amount, current liquidity, and current alpha price
    ///
    fn get_target_quantity(&self, order_type: &OrderType, delta_in: u64) -> SqrtPrice {
        todo!("moved to Pallet::target_quantity")
    }

    /// Get the input amount needed to reach the target price
    ///
    fn get_delta_in(&self, order_type: &OrderType, sqrt_price_target: SqrtPrice) -> u64 {
        todo!("moved to Pallet::delat_in")
    }

    /// Add fees to the global fee counters
    fn add_fees(&mut self, order_type: &OrderType, fee: u64) {
        todo!("moved to Pallet::add_fees")
    }

    /// Convert input amount (delta_in) to output amount (delta_out)
    ///
    /// This is the core method of uniswap V3 that tells how much
    /// output token is given for an amount of input token within one
    /// price tick.
    ///
    fn convert_deltas(&self, order_type: &OrderType, delta_in: u64) -> u64 {
        todo!("moved to Pallet::convert_deltas")
    }

    /// Multiplies a `u64` by a `U64F64` and returns a `u128` result without overflow.
    // pub fn mul_u64_u64f64(a: u64, b: U64F64) -> u128 {
    //     // Multiply a by integer part of b in integer math.
    //     // Result doesn't overflow u128 because both multipliers are 64 bit
    //     let int_b: u64 = b.saturating_to_num::<u64>();
    //     let high = (a as u128).saturating_mul(int_b as u128);

    //     // Multiply a by fractional part of b using U64F64
    //     let frac_b = b.saturating_sub(U64F64::saturating_from_num(int_b));
    //     let low = U64F64::saturating_from_num(a).saturating_mul(frac_b);

    //     // The only possible overflow (that is cut off by saturating math)
    //     // is when a is u64::MAX, int_b is u64::MAX, and frac_b is non-zero,
    //     // which is negligible error if we saturate and return u128::MAX
    //     high.saturating_add(low).saturating_to_num::<u128>()
    // }

    /// Update token reserves after a swap
    ///
    fn update_reserves(&mut self, order_type: &OrderType, amount_in: u64, amount_out: u64) {
        let (new_tao_reserve, new_alpha_reserve) = match order_type {
            OrderType::Sell => (
                self.state_ops.get_tao_reserve().saturating_add(amount_in),
                self.state_ops
                    .get_alpha_reserve()
                    .saturating_sub(amount_out),
            ),
            OrderType::Buy => (
                self.state_ops.get_tao_reserve().saturating_sub(amount_in),
                self.state_ops
                    .get_alpha_reserve()
                    .saturating_add(amount_out),
            ),
        };

        self.state_ops.set_tao_reserve(new_tao_reserve);
        self.state_ops.set_alpha_reserve(new_alpha_reserve);
    }

    fn get_liquidity_update_u64(&self, tick: &Tick) -> u64 {
        todo!("moved to Tick::liquidity_net_as_u64")
    }

    /// Update liquidity when crossing a tick
    ///
    fn update_liquidity_at_crossing(&mut self, order_type: &OrderType) -> Result<(), SwapError> {
        todo!("moved to Pallet::update_liquidity_at_crossing")
    }

    /// Collect fees for a position
    /// Updates the position
    ///
    fn collect_fees(&mut self, position: &mut Position) -> (u64, u64) {
        todo!("moved to Position::collect_fees")
    }

    /// Get fees in a position's range
    ///
    /// If quote flag is true, Tao is returned, otherwise alpha.
    ///
    fn get_fees_in_range(&mut self, position: &Position, quote: bool) -> u64 {
        todo!("moved to Position::fees_in_range")
    }

    /// Get fees above a tick
    ///
    fn get_fees_above(&mut self, tick_index: TickIndex, quote: bool) -> U64F64 {
        todo!("moved to TickIndex::fees_above")
    }

    /// Get fees below a tick
    fn get_fees_below(&mut self, tick_index: TickIndex, quote: bool) -> U64F64 {
        todo!("moved to TickIndex::fees_below")
    }

    /// Active tick operations
    ///
    /// Data structure:
    ///    Active ticks are stored in three hash maps, each representing a "Level"
    ///    Level 0 stores one u128 word, where each bit represents one Level 1 word.
    ///    Level 1 words each store also a u128 word, where each bit represents one Level 2 word.
    ///    Level 2 words each store u128 word, where each bit represents a tick.
    ///
    /// Insertion: 3 reads, 3 writes
    /// Search: 3-5 reads
    /// Deletion: 2 reads, 1-3 writes
    ///

    // Use TickIndexBitmap::layer_to_index instead

    pub fn insert_active_tick(&mut self, index: TickIndex) {
        todo!("moved to ActiveTickIndexManager::insert")
    }

    pub fn remove_active_tick(&mut self, index: TickIndex) {
        todo!("moved to ActiveTickIndexManager::remove")
    }

    pub fn find_closest_active_tick_index(
        &self,
        index: TickIndex,
        lower: bool,
    ) -> Option<TickIndex> {
        todo!("moved to ActiveTickIndexManager::find_closet")
    }

    pub fn find_closest_lower_active_tick_index(&self, index: TickIndex) -> Option<TickIndex> {
        todo!("moved to ActiveTickIndexManager::find_closest_lower")
    }

    pub fn find_closest_higher_active_tick_index(&self, index: TickIndex) -> Option<TickIndex> {
        todo!("moved to ActiveTickIndexManager::find_closest_higher")
    }

    pub fn find_closest_lower_active_tick(&self, index: TickIndex) -> Option<Tick> {
        todo!("moved to Pallet::find_closest_lower")
    }

    pub fn find_closest_higher_active_tick(&self, index: TickIndex) -> Option<Tick> {
        todo!("moved to Pallet::find_closest_higher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use sp_arithmetic::helpers_128bit::sqrt;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct MockSwapDataOperations {
        is_initialized: bool,
        fee_rate: u16,
        minimum_liquidity: u64,
        ticks: HashMap<TickIndex, Tick>,
        min_sqrt_price: SqrtPrice,
        max_sqrt_price: SqrtPrice,
        tao_reserve: u64,
        alpha_reserve: u64,
        alpha_sqrt_price: SqrtPrice,
        fee_global_tao: U64F64,
        fee_global_alpha: U64F64,
        current_liquidity: u64,
        max_positions: u16,
        balances: HashMap<u16, (u64, u64)>,
        positions: HashMap<u16, HashMap<u16, Position>>,
        tick_index_l0: HashMap<u32, u128>,
        tick_index_l1: HashMap<u32, u128>,
        tick_index_l2: HashMap<u32, u128>,
    }

    impl MockSwapDataOperations {
        pub fn new() -> Self {
            Self {
                is_initialized: false,
                fee_rate: 196,
                minimum_liquidity: 1000,
                ticks: HashMap::new(),
                min_sqrt_price: SqrtPrice::from_num(0.01),
                max_sqrt_price: SqrtPrice::from_num(10),
                tao_reserve: 0,
                alpha_reserve: 0,
                alpha_sqrt_price: SqrtPrice::from_num(0),
                fee_global_tao: U64F64::from_num(0),
                fee_global_alpha: U64F64::from_num(0),
                current_liquidity: 0,
                max_positions: 100,
                balances: HashMap::new(),
                positions: HashMap::new(),
                tick_index_l0: HashMap::new(),
                tick_index_l1: HashMap::new(),
                tick_index_l2: HashMap::new(),
            }
        }
    }

    impl SwapDataOperations<u16> for MockSwapDataOperations {
        fn is_v3_initialized(&self) -> bool {
            self.is_initialized
        }

        fn set_v3_initialized(&mut self) {
            self.is_initialized = true;
        }

        fn get_fee_rate(&self) -> u16 {
            self.fee_rate
        }

        fn get_minimum_liquidity(&self) -> u64 {
            self.minimum_liquidity
        }

        fn get_tick_by_index(&self, tick_index: TickIndex) -> Option<Tick> {
            self.ticks.get(&tick_index).cloned()
        }

        fn insert_tick_by_index(&mut self, tick_index: TickIndex, tick: Tick) {
            self.ticks.insert(tick_index, tick);
        }

        fn remove_tick_by_index(&mut self, tick_index: TickIndex) {
            self.ticks.remove(&tick_index);
        }

        fn get_min_sqrt_price(&self) -> SqrtPrice {
            self.min_sqrt_price
        }

        fn get_max_sqrt_price(&self) -> SqrtPrice {
            self.max_sqrt_price
        }

        fn get_tao_reserve(&self) -> u64 {
            self.tao_reserve
        }

        fn set_tao_reserve(&mut self, tao: u64) -> u64 {
            self.tao_reserve = tao;
            tao
        }

        fn get_alpha_reserve(&self) -> u64 {
            self.alpha_reserve
        }

        fn set_alpha_reserve(&mut self, alpha: u64) -> u64 {
            self.alpha_reserve = alpha;
            alpha
        }

        fn get_alpha_sqrt_price(&self) -> SqrtPrice {
            self.alpha_sqrt_price
        }

        fn set_alpha_sqrt_price(&mut self, sqrt_price: SqrtPrice) {
            self.alpha_sqrt_price = sqrt_price;
        }

        fn get_fee_global_tao(&self) -> U64F64 {
            self.fee_global_tao
        }

        fn set_fee_global_tao(&mut self, fee: U64F64) {
            self.fee_global_tao = fee;
        }

        fn get_fee_global_alpha(&self) -> U64F64 {
            self.fee_global_alpha
        }

        fn set_fee_global_alpha(&mut self, fee: U64F64) {
            self.fee_global_alpha = fee;
        }

        fn get_current_liquidity(&self) -> u64 {
            self.current_liquidity
        }

        fn set_current_liquidity(&mut self, liquidity: u64) {
            self.current_liquidity = liquidity;
        }

        fn get_max_positions(&self) -> u16 {
            self.max_positions
        }

        fn withdraw_balances(
            &mut self,
            account_id: &u16,
            tao: u64,
            alpha: u64,
        ) -> Result<(u64, u64), SwapError> {
            let (current_tao, current_alpha) =
                self.balances.get(account_id).cloned().unwrap_or((0, 0));

            if (tao > current_tao) || (alpha > current_alpha) {
                return Err(SwapError::InsufficientBalance);
            }

            self.balances
                .insert(*account_id, (current_tao - tao, current_alpha - alpha));

            Ok((tao, alpha))
        }

        fn deposit_balances(&mut self, account_id: &u16, tao: u64, alpha: u64) {
            let (current_tao, current_alpha) =
                self.balances.get(account_id).cloned().unwrap_or((0, 0));
            self.balances.insert(
                account_id.clone(),
                (current_tao + tao, current_alpha + alpha),
            );
        }

        fn get_protocol_account_id(&self) -> u16 {
            0xFFFF
        }

        fn get_position_count(&self, account_id: &u16) -> u16 {
            self.positions.get(account_id).map_or(0, |p| p.len() as u16)
        }

        fn get_position(&self, account_id: &u16, position_id: u16) -> Option<Position> {
            self.positions
                .get(account_id)
                .and_then(|p| p.get(&position_id).cloned())
        }

        fn create_position(&mut self, account_id: &u16, position: Position) -> u16 {
            let entry = self
                .positions
                .entry(account_id.clone())
                .or_insert_with(HashMap::new);

            // Find the next available position ID
            let new_position_id = entry.keys().max().map_or(0, |max_id| max_id + 1);

            entry.insert(new_position_id, position);
            new_position_id
        }

        fn update_position(&mut self, account_id: &u16, position_id: u16, position: Position) {
            if let Some(account_positions) = self.positions.get_mut(account_id) {
                account_positions.insert(position_id, position);
            }
        }

        fn remove_position(&mut self, account_id: &u16, position_id: u16) {
            if let Some(account_positions) = self.positions.get_mut(account_id) {
                account_positions.remove(&position_id);
            }
        }

        fn get_layer0_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l0.get(&word_index).unwrap_or(&0_u128)
        }
        fn get_layer1_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l1.get(&word_index).unwrap_or(&0_u128)
        }
        fn get_layer2_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l2.get(&word_index).unwrap_or(&0_u128)
        }
        fn set_layer0_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l0.insert(word_index, word);
        }
        fn set_layer1_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l1.insert(word_index, word);
        }
        fn set_layer2_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l2.insert(word_index, word);
        }
    }

    #[test]
    fn test_swap_initialization() {
        let tao = 1_000_000_000;
        let alpha = 4_000_000_000;

        let mut mock_ops = MockSwapDataOperations::new();
        mock_ops.set_tao_reserve(tao);
        mock_ops.set_alpha_reserve(alpha);
        let swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        // Active ticks
        let tick_low = swap.state_ops.get_tick_by_index(TickIndex::MIN).unwrap();
        let tick_high = swap.state_ops.get_tick_by_index(TickIndex::MAX).unwrap();
        let liquidity = sqrt(alpha as u128 * tao as u128) as u64;
        let expected_liquidity_net_low: i128 = liquidity as i128;
        let expected_liquidity_gross_low: u64 = liquidity;
        let expected_liquidity_net_high: i128 = (liquidity as i128).neg();
        let expected_liquidity_gross_high: u64 = liquidity;
        assert_eq!(tick_low.liquidity_net, expected_liquidity_net_low,);
        assert_eq!(tick_low.liquidity_gross, expected_liquidity_gross_low,);
        assert_eq!(tick_high.liquidity_net, expected_liquidity_net_high,);
        assert_eq!(tick_high.liquidity_gross, expected_liquidity_gross_high,);

        // Liquidity position at correct ticks
        let account_id = swap.state_ops.get_protocol_account_id();
        assert_eq!(swap.state_ops.get_position_count(&account_id), 1);

        let position = swap.state_ops.get_position(&account_id, 0).unwrap();
        assert_eq!(position.liquidity, liquidity);
        assert_eq!(position.tick_low, TickIndex::MIN);
        assert_eq!(position.tick_high, TickIndex::MAX);
        assert_eq!(position.fees_alpha, 0);
        assert_eq!(position.fees_tao, 0);

        // Current liquidity
        assert_eq!(swap.state_ops.get_current_liquidity(), liquidity);

        // Current price
        let sqrt_price = swap.state_ops.get_alpha_sqrt_price();
        assert_abs_diff_eq!(sqrt_price.to_num::<f64>(), 0.50, epsilon = 0.00001,);
    }

    fn price_to_tick(price: f64) -> TickIndex {
        let price_sqrt: SqrtPrice = SqrtPrice::from_num(price.sqrt());
        // Handle potential errors in the conversion
        match TickIndex::try_from_sqrt_price(price_sqrt) {
            Ok(mut tick) => {
                // Ensure the tick is within bounds
                if tick > TickIndex::MAX {
                    tick = TickIndex::MAX;
                } else if tick < TickIndex::MIN {
                    tick = TickIndex::MIN;
                }
                tick
            }
            // Default to a reasonable value when conversion fails
            Err(_) => {
                if price > 1.0 {
                    TickIndex::MAX
                } else {
                    TickIndex::MIN
                }
            }
        }
    }

    fn tick_to_price(tick: TickIndex) -> f64 {
        // Handle errors gracefully
        match tick.try_to_sqrt_price() {
            Ok(price_sqrt) => (price_sqrt * price_sqrt).to_num::<f64>(),
            Err(_) => {
                // Return a sensible default based on whether the tick is above or below the valid range
                if tick > TickIndex::MAX {
                    tick_to_price(TickIndex::MAX) // Use the max valid tick price
                } else {
                    tick_to_price(TickIndex::MIN) // Use the min valid tick price
                }
            }
        }
    }

    #[test]
    fn test_tick_price_sanity_check() {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        assert!(min_price > 0.);
        assert!(max_price > 0.);
        assert!(max_price > min_price);
        assert!(min_price < 0.000001);
        assert!(max_price > 10.);

        // Roundtrip conversions
        let min_price_sqrt: SqrtPrice = TickIndex::MIN.try_to_sqrt_price().unwrap();
        let min_tick = TickIndex::try_from_sqrt_price(min_price_sqrt).unwrap();
        assert_eq!(min_tick, TickIndex::MIN);

        let max_price_sqrt: SqrtPrice = TickIndex::MAX.try_to_sqrt_price().unwrap();
        let max_tick = TickIndex::try_from_sqrt_price(max_price_sqrt).unwrap();
        assert_eq!(max_tick, TickIndex::MAX);
    }

    // Test adding liquidity on top of the existing protocol liquidity
    #[test]
    fn test_add_liquidity_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        let current_price = 0.25;
        assert_eq!(max_tick, TickIndex::MAX);

        // As a user add liquidity with all possible corner cases
        //   - Initial price is 0.25
        //   - liquidity is expressed in RAO units
        // Test case is (price_low, price_high, liquidity, tao, alpha)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (
                min_price,
                max_price,
                2_000_000_000_u64,
                1_000_000_000_u64,
                4_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range: Expect the same alpha
            (0.25, max_price, 2_000_000_000_u64, 0, 4_000_000_000),
            // Repeat the protocol liquidity at min to current range: Expect all the same tao
            (min_price, 0.24999, 2_000_000_000_u64, 1_000_000_000, 0),
            // Half to double price - just some sane wothdraw amounts
            (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
            // Both below price - tao is non-zero, alpha is zero
            (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
            // Both above price - tao is zero, alpha is non-zero
            (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity, tao, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Get tick infos and liquidity before adding (to account for protocol liquidity)
            let tick_low_info_before = swap
                .state_ops
                .get_tick_by_index(tick_low)
                .unwrap_or_default();
            let tick_high_info_before = swap
                .state_ops
                .get_tick_by_index(tick_high)
                .unwrap_or_default();
            let liquidity_before = swap.state_ops.get_current_liquidity();

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Check that low and high ticks appear in the state and are properly updated
            let tick_low_info = swap.state_ops.get_tick_by_index(tick_low).unwrap();
            let tick_high_info = swap.state_ops.get_tick_by_index(tick_high).unwrap();
            let expected_liquidity_net_low: i128 = *liquidity as i128;
            let expected_liquidity_gross_low: u64 = *liquidity;
            let expected_liquidity_net_high: i128 = (*liquidity as i128).neg();
            let expected_liquidity_gross_high: u64 = *liquidity;
            assert_eq!(
                tick_low_info.liquidity_net - tick_low_info_before.liquidity_net,
                expected_liquidity_net_low,
            );
            assert_eq!(
                tick_low_info.liquidity_gross - tick_low_info_before.liquidity_gross,
                expected_liquidity_gross_low,
            );
            assert_eq!(
                tick_high_info.liquidity_net - tick_high_info_before.liquidity_net,
                expected_liquidity_net_high,
            );
            assert_eq!(
                tick_high_info.liquidity_gross - tick_high_info_before.liquidity_gross,
                expected_liquidity_gross_high,
            );

            // Balances are withdrawn
            let (user_tao_after, user_alpha_after) =
                swap.state_ops.balances.get(&account_id).unwrap();
            let tao_withdrawn = user_tao - user_tao_after;
            let alpha_withdrawn = user_alpha - user_alpha_after;
            assert_abs_diff_eq!(tao_withdrawn, *tao, epsilon = *tao / 1000);
            assert_abs_diff_eq!(alpha_withdrawn, *alpha, epsilon = *alpha / 1000);

            // Liquidity position at correct ticks
            assert_eq!(swap.state_ops.get_position_count(&account_id), 1);

            let position = swap.state_ops.get_position(&account_id, 0).unwrap();
            assert_eq!(position.liquidity, *liquidity);
            assert_eq!(position.tick_low, tick_low);
            assert_eq!(position.tick_high, tick_high);
            assert_eq!(position.fees_alpha, 0);
            assert_eq!(position.fees_tao, 0);

            // Current liquidity is updated only when price range includes the current price
            if (*price_high >= current_price) && (*price_low <= current_price) {
                assert_eq!(
                    swap.state_ops.get_current_liquidity(),
                    liquidity_before + *liquidity
                );
            } else {
                assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);
            }

            // Reserves are updated
            assert_eq!(
                swap.state_ops.get_tao_reserve(),
                tao_withdrawn + protocol_tao,
            );
            assert_eq!(
                swap.state_ops.get_alpha_reserve(),
                alpha_withdrawn + protocol_alpha,
            );
        });
    }

    #[test]
    fn test_add_liquidity_out_of_bounds() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 2_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;

        [
            // For our tests, we'll construct TickIndex values that are intentionally
            // outside the valid range for testing purposes only
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                TickIndex::MAX,
                1_000_000_000_u64,
            ),
            (
                TickIndex::MIN,
                TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                1_000_000_000_u64,
            ),
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                1_000_000_000_u64,
            ),
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 100),
                TickIndex::new_unchecked(TickIndex::MAX.get() + 100),
                1_000_000_000_u64,
            ),
        ]
        .iter()
        .for_each(|(tick_low, tick_high, liquidity)| {
            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert_eq!(
                swap.add_liquidity(&account_id, *tick_low, *tick_high, *liquidity, false),
                Err(SwapError::InvalidTickRange),
            );
        });
    }

    #[test]
    fn test_add_liquidity_over_balance() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 1_000_000_000;
        let user_alpha = 1_000_000_000;
        let account_id = 1;

        [
            // Lower than price (not enough alpha)
            (0.1, 0.2, 100_000_000_000_u64),
            // Higher than price (not enough tao)
            (0.3, 0.4, 100_000_000_000_u64),
            // Around the price (not enough both)
            (0.1, 0.4, 100_000_000_000_u64),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity)| {
            // Calculate ticks
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert_eq!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false),
                Err(SwapError::InsufficientBalance),
            );
        });
    }

    // Test removing liquidity
    #[test]
    fn test_remove_liquidity_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick, TickIndex::MAX);

        // As a user add liquidity with all possible corner cases
        //   - Initial price is 0.25
        //   - liquidity is expressed in RAO units
        // Test case is (price_low, price_high, liquidity, tao, alpha)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (
                min_price,
                max_price,
                2_000_000_000_u64,
                1_000_000_000_u64,
                4_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range: Expect the same alpha
            (0.25, max_price, 2_000_000_000_u64, 0, 4_000_000_000),
            // Repeat the protocol liquidity at min to current range: Expect all the same tao
            (min_price, 0.24999, 2_000_000_000_u64, 1_000_000_000, 0),
            // Half to double price - just some sane wothdraw amounts
            (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
            // Both below price - tao is non-zero, alpha is zero
            (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
            // Both above price - tao is zero, alpha is non-zero
            (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity, tao, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
            let liquidity_before = swap.state_ops.get_current_liquidity();

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Remove liquidity
            let remove_result = swap.remove_liquidity(&account_id, 0).unwrap();
            assert_abs_diff_eq!(remove_result.tao, *tao, epsilon = *tao / 1000);
            assert_abs_diff_eq!(remove_result.alpha, *alpha, epsilon = *alpha / 1000);
            assert_eq!(remove_result.fee_tao, 0);
            assert_eq!(remove_result.fee_alpha, 0);

            // Balances are returned
            let (user_tao_after, user_alpha_after) =
                swap.state_ops.balances.get(&account_id).unwrap();
            assert_eq!(user_tao, *user_tao_after);
            assert_eq!(user_alpha, *user_alpha_after);

            // Liquidity position is removed
            assert_eq!(swap.state_ops.get_position_count(&account_id), 0);
            assert!(swap.state_ops.get_position(&account_id, 0).is_none());

            // Current liquidity is updated (back where it was)
            assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);

            // Reserves are updated (back where they were)
            assert_eq!(swap.state_ops.get_tao_reserve(), protocol_tao,);
            assert_eq!(swap.state_ops.get_alpha_reserve(), protocol_alpha,);
        });
    }

    #[test]
    fn test_remove_liquidity_nonexisting_position() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick.get(), TickIndex::MAX.get());

        // Test case is (price_low, price_high, liquidity)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (min_price, max_price, 2_000_000_000_u64),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Remove liquidity
            assert_eq!(
                swap.remove_liquidity(&account_id, 1),
                Err(SwapError::LiquidityNotFound),
            );
        });
    }

    // Test swapping against protocol liquidity only
    #[test]
    fn test_swap_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;

        // Current price is 0.25
        // Test case is (order_type, liquidity, limit_price, output_amount)
        [(OrderType::Buy, 500_000_000u64, 1000.0_f64, 3990_u64)]
            .iter()
            .for_each(|(order_type, liquidity, limit_price, output_amount)| {
                // Consumed liquidity ticks
                let tick_low = TickIndex::MIN;
                let tick_high = TickIndex::MAX;

                // Setup swap
                let mut mock_ops = MockSwapDataOperations::new();
                mock_ops.set_tao_reserve(protocol_tao);
                mock_ops.set_alpha_reserve(protocol_alpha);
                let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

                // Get tick infos before the swap
                let tick_low_info_before = swap
                    .state_ops
                    .get_tick_by_index(tick_low)
                    .unwrap_or_default();
                let tick_high_info_before = swap
                    .state_ops
                    .get_tick_by_index(tick_high)
                    .unwrap_or_default();
                let liquidity_before = swap.state_ops.get_current_liquidity();

                // Swap
                let sqrt_limit_price: SqrtPrice = SqrtPrice::from_num((*limit_price).sqrt());
                let swap_result = swap.swap(order_type, *liquidity, sqrt_limit_price);
                // assert_abs_diff_eq!(
                //     swap_result.unwrap().amount_paid_out,
                //     *output_amount,
                //     epsilon = *output_amount/1000
                // );

                // Check that low and high ticks' fees were updated properly, and liquidity values were not updated
                let tick_low_info = swap.state_ops.get_tick_by_index(tick_low).unwrap();
                let tick_high_info = swap.state_ops.get_tick_by_index(tick_high).unwrap();
                let expected_liquidity_net_low: i128 = tick_low_info_before.liquidity_net;
                let expected_liquidity_gross_low: u64 = tick_low_info_before.liquidity_gross;
                let expected_liquidity_net_high: i128 = tick_high_info_before.liquidity_net;
                let expected_liquidity_gross_high: u64 = tick_high_info_before.liquidity_gross;
                assert_eq!(tick_low_info.liquidity_net, expected_liquidity_net_low,);
                assert_eq!(tick_low_info.liquidity_gross, expected_liquidity_gross_low,);
                assert_eq!(tick_high_info.liquidity_net, expected_liquidity_net_high,);
                assert_eq!(
                    tick_high_info.liquidity_gross,
                    expected_liquidity_gross_high,
                );

                // Expected fee amount
                let fee_rate = swap.state_ops.get_fee_rate() as f64 / u16::MAX as f64;
                let expected_fee = output_amount * fee_rate as u64;

                // Global fees should be updated
                let actual_global_fee: U64F64 = match order_type {
                    OrderType::Buy => swap.state_ops.get_fee_global_alpha(),
                    OrderType::Sell => swap.state_ops.get_fee_global_tao(),
                };
                println!("actual_global_fee {:?}", actual_global_fee);
                assert_eq!(actual_global_fee, expected_fee);

                // Tick fees should be updated

                // Liquidity position should not be updated
                let protocol_id = swap.state_ops.get_protocol_account_id();

                let position = swap.state_ops.get_position(&protocol_id, 0).unwrap();
                assert_eq!(
                    position.liquidity,
                    sqrt(protocol_tao as u128 * protocol_alpha as u128) as u64
                );
                assert_eq!(position.tick_low, tick_low);
                assert_eq!(position.tick_high, tick_high);
                assert_eq!(position.fees_alpha, 0);
                assert_eq!(position.fees_tao, 0);

                // // Current liquidity is updated only when price range includes the current price
                // if (*price_high >= current_price) && (*price_low <= current_price) {
                //     assert_eq!(
                //         swap.state_ops.get_current_liquidity(),
                //         liquidity_before + *liquidity
                //     );
                // } else {
                //     assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);
                // }

                // // Reserves are updated
                // assert_eq!(
                //     swap.state_ops.get_tao_reserve(),
                //     tao_withdrawn + protocol_tao,
                // );
                // assert_eq!(
                //     swap.state_ops.get_alpha_reserve(),
                //     alpha_withdrawn + protocol_alpha,
                // );
            });
    }

    // cargo test --package pallet-subtensor-swap --lib -- tests::test_tick_search_basic --exact --show-output
    #[test]
    fn test_tick_search_basic() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        swap.insert_active_tick(TickIndex::MIN);
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.saturating_div(2))
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.prev().unwrap())
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.next().unwrap())
                .unwrap(),
            TickIndex::MIN
        );

        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX.saturating_div(2)),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX.prev().unwrap()),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN.next().unwrap()),
            None
        );

        swap.insert_active_tick(TickIndex::MAX);
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX)
                .unwrap(),
            TickIndex::MAX
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.saturating_div(2))
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.prev().unwrap())
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.next().unwrap())
                .unwrap(),
            TickIndex::MIN
        );
    }

    #[test]
    fn test_tick_search_sparse_queries() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        let active_index = TickIndex::MIN.saturating_add(10);
        swap.insert_active_tick(active_index);
        assert_eq!(
            swap.find_closest_lower_active_tick_index(active_index)
                .unwrap(),
            active_index
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.saturating_add(11))
                .unwrap(),
            active_index
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.saturating_add(12))
                .unwrap(),
            active_index
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN),
            None
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.saturating_add(9)),
            None
        );

        assert_eq!(
            swap.find_closest_higher_active_tick_index(active_index)
                .unwrap(),
            active_index
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN.saturating_add(11)),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN.saturating_add(12)),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN)
                .unwrap(),
            active_index
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN.saturating_add(9))
                .unwrap(),
            active_index
        );
    }

    #[test]
    fn test_tick_search_many_lows() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        (0..1000).for_each(|i| {
            swap.insert_active_tick(TickIndex::MIN.saturating_add(i));
        });

        for i in 0..1000 {
            let test_index = TickIndex::MIN.saturating_add(i);
            assert_eq!(
                swap.find_closest_lower_active_tick_index(test_index)
                    .unwrap(),
                test_index
            );
            assert_eq!(
                swap.find_closest_higher_active_tick_index(test_index)
                    .unwrap(),
                test_index
            );
        }
    }

    #[test]
    fn test_tick_search_many_sparse() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for i in 0..=count {
            swap.insert_active_tick(TickIndex::new_unchecked(i * 10));
        }
        for i in 1..count {
            let tick = TickIndex::new_unchecked(i * 10);
            assert_eq!(
                swap.find_closest_lower_active_tick_index(tick).unwrap(),
                tick
            );
            assert_eq!(
                swap.find_closest_higher_active_tick_index(tick).unwrap(),
                tick
            );
            for j in 1..=9 {
                let before_tick = TickIndex::new_unchecked(i * 10 - j);
                let after_tick = TickIndex::new_unchecked(i * 10 + j);
                let prev_tick = TickIndex::new_unchecked((i - 1) * 10);
                let next_tick = TickIndex::new_unchecked((i + 1) * 10);
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(before_tick)
                        .unwrap(),
                    prev_tick
                );
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(after_tick)
                        .unwrap(),
                    tick
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(before_tick)
                        .unwrap(),
                    tick
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(after_tick)
                        .unwrap(),
                    next_tick
                );
            }
        }
    }

    #[test]
    fn test_tick_search_many_lows_sparse_reversed() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for i in (0..=count).rev() {
            swap.insert_active_tick(TickIndex::new_unchecked(i * 10));
        }
        for i in 1..count {
            let tick = TickIndex::new_unchecked(i * 10);
            assert_eq!(
                swap.find_closest_lower_active_tick_index(tick).unwrap(),
                tick
            );
            assert_eq!(
                swap.find_closest_higher_active_tick_index(tick).unwrap(),
                tick
            );
            for j in 1..=9 {
                let before_tick = TickIndex::new_unchecked(i * 10 - j);
                let after_tick = TickIndex::new_unchecked(i * 10 + j);
                let prev_tick = TickIndex::new_unchecked((i - 1) * 10);
                let next_tick = TickIndex::new_unchecked((i + 1) * 10);

                assert_eq!(
                    swap.find_closest_lower_active_tick_index(before_tick)
                        .unwrap(),
                    prev_tick
                );
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(after_tick)
                        .unwrap(),
                    tick
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(before_tick)
                        .unwrap(),
                    tick
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(after_tick)
                        .unwrap(),
                    next_tick
                );
            }
        }
    }

    #[test]
    fn test_tick_search_repeated_insertions() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for _ in 0..10 {
            for i in 0..=count {
                let tick = TickIndex::new_unchecked(i * 10);
                swap.insert_active_tick(tick);
            }
            for i in 1..count {
                let tick = TickIndex::new_unchecked(i * 10);
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(tick).unwrap(),
                    tick
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(tick).unwrap(),
                    tick
                );
                for j in 1..=9 {
                    let before_tick = TickIndex::new_unchecked(i * 10 - j);
                    let after_tick = TickIndex::new_unchecked(i * 10 + j);
                    let prev_tick = TickIndex::new_unchecked((i - 1) * 10);
                    let next_tick = TickIndex::new_unchecked((i + 1) * 10);

                    assert_eq!(
                        swap.find_closest_lower_active_tick_index(before_tick)
                            .unwrap(),
                        prev_tick
                    );
                    assert_eq!(
                        swap.find_closest_lower_active_tick_index(after_tick)
                            .unwrap(),
                        tick
                    );
                    assert_eq!(
                        swap.find_closest_higher_active_tick_index(before_tick)
                            .unwrap(),
                        tick
                    );
                    assert_eq!(
                        swap.find_closest_higher_active_tick_index(after_tick)
                            .unwrap(),
                        next_tick
                    );
                }
            }
        }
    }

    #[test]
    fn test_tick_search_full_range() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let step = 1019;
        // Get the full valid tick range by subtracting MIN from MAX
        let count = (TickIndex::MAX.get() - TickIndex::MIN.get()) / step;

        for i in 0..=count {
            let index = TickIndex::MIN.saturating_add(i * step);
            swap.insert_active_tick(index);
        }
        for i in 1..count {
            let index = TickIndex::MIN.saturating_add(i * step);

            let prev_index = TickIndex::new_unchecked(index.get() - step);
            let next_minus_one = TickIndex::new_unchecked(index.get() + step - 1);

            assert_eq!(
                swap.find_closest_lower_active_tick_index(prev_index)
                    .unwrap(),
                prev_index
            );
            assert_eq!(
                swap.find_closest_lower_active_tick_index(index).unwrap(),
                index
            );
            assert_eq!(
                swap.find_closest_lower_active_tick_index(next_minus_one)
                    .unwrap(),
                index
            );

            let mid_next = TickIndex::new_unchecked(index.get() + step / 2);
            assert_eq!(
                swap.find_closest_lower_active_tick_index(mid_next).unwrap(),
                index
            );

            assert_eq!(
                swap.find_closest_higher_active_tick_index(index).unwrap(),
                index
            );

            let next_index = TickIndex::new_unchecked(index.get() + step);
            assert_eq!(
                swap.find_closest_higher_active_tick_index(next_index)
                    .unwrap(),
                next_index
            );

            let mid_next = TickIndex::new_unchecked(index.get() + step / 2);
            assert_eq!(
                swap.find_closest_higher_active_tick_index(mid_next)
                    .unwrap(),
                next_index
            );

            let next_minus_1 = TickIndex::new_unchecked(index.get() + step - 1);
            assert_eq!(
                swap.find_closest_higher_active_tick_index(next_minus_1)
                    .unwrap(),
                next_index
            );
            for j in 1..=9 {
                let before_index = TickIndex::new_unchecked(index.get() - j);
                let after_index = TickIndex::new_unchecked(index.get() + j);

                assert_eq!(
                    swap.find_closest_lower_active_tick_index(before_index)
                        .unwrap(),
                    prev_index
                );
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(after_index)
                        .unwrap(),
                    index
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(before_index)
                        .unwrap(),
                    index
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(after_index)
                        .unwrap(),
                    next_index
                );
            }
        }
    }

    #[test]
    fn test_tick_remove_basic() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        swap.insert_active_tick(TickIndex::MIN);
        swap.insert_active_tick(TickIndex::MAX);
        swap.remove_active_tick(TickIndex::MAX);

        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.saturating_div(2))
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MAX.prev().unwrap())
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_lower_active_tick_index(TickIndex::MIN.next().unwrap())
                .unwrap(),
            TickIndex::MIN
        );

        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN)
                .unwrap(),
            TickIndex::MIN
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX.saturating_div(2)),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MAX.prev().unwrap()),
            None
        );
        assert_eq!(
            swap.find_closest_higher_active_tick_index(TickIndex::MIN.next().unwrap()),
            None
        );
    }

    #[test]
    fn test_tick_remove_full_range() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let step = 1019;
        // Get the full valid tick range by subtracting MIN from MAX
        let count = (TickIndex::MAX.get() - TickIndex::MIN.get()) / step;
        let remove_frequency = 5; // Remove every 5th tick

        // Insert ticks
        for i in 0..=count {
            let index = TickIndex::MIN.saturating_add(i * step);
            swap.insert_active_tick(index);
        }

        // Remove some ticks
        for i in 1..count {
            if i % remove_frequency == 0 {
                let index = TickIndex::MIN.saturating_add(i * step);
                swap.remove_active_tick(index);
            }
        }

        // Verify
        for i in 1..count {
            let index = TickIndex::MIN.saturating_add(i * step);

            if i % remove_frequency == 0 {
                let lower = swap.find_closest_lower_active_tick_index(index);
                let higher = swap.find_closest_higher_active_tick_index(index);
                assert!(lower != Some(index));
                assert!(higher != Some(index));
            } else {
                assert_eq!(
                    swap.find_closest_lower_active_tick_index(index).unwrap(),
                    index
                );
                assert_eq!(
                    swap.find_closest_higher_active_tick_index(index).unwrap(),
                    index
                );
            }
        }
    }
}
