#![cfg_attr(not(feature = "std"), no_std)]
use core::ops::Neg;

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};

pub use order::*;

mod order;

pub trait SwapEngine<O: Order>: DefaultPriceLimit<O::PaidIn, O::PaidOut> {
    fn swap(
        netuid: NetUid,
        order: O,
        price_limit: TaoBalance,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>;
}

pub trait SwapHandler {
    fn swap<O: Order>(
        netuid: NetUid,
        order: O,
        price_limit: TaoBalance,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        Self: SwapEngine<O>;
    fn sim_swap<O: Order>(
        netuid: NetUid,
        order: O,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, DispatchError>
    where
        Self: SwapEngine<O>;

    fn approx_fee_amount<T: Token>(netuid: NetUid, amount: T) -> T;
    fn current_alpha_price(netuid: NetUid) -> U96F32;
    fn get_protocol_tao(netuid: NetUid) -> TaoBalance;
    fn max_price<C: Token>() -> C;
    fn min_price<C: Token>() -> C;
    fn adjust_protocol_liquidity(netuid: NetUid, tao_delta: TaoBalance, alpha_delta: AlphaBalance);
    fn is_user_liquidity_enabled(netuid: NetUid) -> bool;
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult;
    fn toggle_user_liquidity(netuid: NetUid, enabled: bool);
    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult;
    fn get_alpha_amount_for_tao(netuid: NetUid, tao_amount: TaoBalance) -> AlphaBalance;
}

/// Combined swap + balance execution interface for limit orders.
///
/// Wraps the complete buy/sell operation: AMM state update (via `SwapHandler`),
/// pool reserve accounting, and user balance changes (TAO free balance /
/// alpha staking). Implemented by `pallet_subtensor::Pallet<T>` using
/// `stake_into_subnet` / `unstake_from_subnet`.
pub trait OrderSwapInterface<AccountId> {
    /// Buy alpha with TAO: debit `tao_amount` from `coldkey`'s free balance,
    /// credit resulting alpha as stake at `hotkey` on `netuid`.
    ///
    /// When `validate` is `true` the implementation enforces subnet
    /// existence, hotkey registration, minimum stake amount, sufficient
    /// coldkey balance, and sets the staking rate-limit flag for `(hotkey,
    /// coldkey, netuid)` after a successful stake. Pass `false` for internal
    /// pallet-intermediary swaps that must bypass these user-facing guards.
    fn buy_alpha(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        limit_price: TaoBalance,
        validate: bool,
    ) -> Result<AlphaBalance, DispatchError>;

    /// Sell alpha for TAO: remove `alpha_amount` from `coldkey`'s stake at
    /// `hotkey` on `netuid`, credit resulting TAO to `coldkey`'s free balance.
    ///
    /// When `validate` is `true` the implementation enforces subnet
    /// existence, hotkey registration, minimum stake amount, sufficient alpha
    /// balance, and checks that the staking rate-limit flag is not set for
    /// `(hotkey, coldkey, netuid)` (i.e. the account did not stake this
    /// block). Pass `false` for internal pallet-intermediary swaps.
    fn sell_alpha(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        limit_price: TaoBalance,
        validate: bool,
    ) -> Result<TaoBalance, DispatchError>;

    /// Current spot price: TAO per alpha, same scale as
    /// `SwapHandler::current_alpha_price`.
    fn current_alpha_price(netuid: NetUid) -> U96F32;

    /// Transfer `amount` TAO from `from`'s free balance to `to`'s free balance.
    ///
    /// Used by the batch executor to collect TAO from buy-order signers into
    /// the pallet intermediary account and to distribute TAO to sell-order
    /// signers after internal matching.
    fn transfer_tao(from: &AccountId, to: &AccountId, amount: TaoBalance) -> DispatchResult;

    /// Move `amount` staked alpha directly between two (coldkey, hotkey) pairs
    /// on `netuid` **without going through the AMM pool**.
    ///
    /// This is a pure stake-accounting transfer used for internal order
    /// matching in `execute_batched_orders`: it lets the pallet collect alpha
    /// from sell-order signers into its intermediary account, and later
    /// distribute alpha to buy-order signers, all without touching the pool.
    ///
    /// When `validate_sender` is `true`, the sender side is validated before
    /// the transfer: subnet existence, subtoken enabled, minimum stake amount,
    /// and the staking rate-limit flag for `(from_hotkey, from_coldkey,
    /// netuid)` is checked — the transfer is rejected if `from_coldkey`
    /// already staked this block.
    ///
    /// When `validate_receiver` is `true`, the staking rate-limit flag for
    /// `(to_hotkey, to_coldkey, netuid)` is set after the transfer, marking
    /// that `to_coldkey` has received stake this block.
    ///
    /// The two flags are intentionally separate so that each call site can
    /// opt into only the half it needs:
    /// - Collecting alpha from users into the pallet intermediary:
    ///   `validate_sender: true, validate_receiver: false` — validates the
    ///   user but does not rate-limit the intermediary account.
    /// - Distributing alpha from the pallet intermediary to buyers:
    ///   `validate_sender: false, validate_receiver: true` — skips checking
    ///   the intermediary (which would fail) and rate-limits the buyer.
    fn transfer_staked_alpha(
        from_coldkey: &AccountId,
        from_hotkey: &AccountId,
        to_coldkey: &AccountId,
        to_hotkey: &AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
        validate_sender: bool,
        validate_receiver: bool,
    ) -> DispatchResult;
}

pub trait DefaultPriceLimit<PaidIn, PaidOut>
where
    PaidIn: Token,
    PaidOut: Token,
{
    fn default_price_limit<C: Token>() -> C;
}

/// Externally used swap result (for RPC)
#[freeze_struct("6a03533fc53ccfb8")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SwapResult<PaidIn, PaidOut>
where
    PaidIn: Token,
    PaidOut: Token,
{
    pub amount_paid_in: PaidIn,
    pub amount_paid_out: PaidOut,
    pub fee_paid: PaidIn,
    pub fee_to_block_author: PaidIn,
}

impl<PaidIn, PaidOut> SwapResult<PaidIn, PaidOut>
where
    PaidIn: Token,
    PaidOut: Token,
{
    pub fn paid_in_reserve_delta(&self) -> i128 {
        self.amount_paid_in.to_u64() as i128
    }

    pub fn paid_in_reserve_delta_i64(&self) -> i64 {
        self.paid_in_reserve_delta()
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }

    pub fn paid_out_reserve_delta(&self) -> i128 {
        (self.amount_paid_out.to_u64() as i128).neg()
    }

    pub fn paid_out_reserve_delta_i64(&self) -> i64 {
        (self.amount_paid_out.to_u64() as i128)
            .neg()
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }
}
