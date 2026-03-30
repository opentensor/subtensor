#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{AccountId32, MultiSignature, traits::Verify};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::OrderSwapInterface;

// ── Data structures ──────────────────────────────────────────────────────────

/// Internal direction of a net pool trade. Used only for `GroupExecutionSummary`
/// and pool-swap bookkeeping; not part of the public order payload.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// The user-facing order type. Each variant encodes both the execution action
/// (buy alpha / sell alpha) and the price-trigger direction.
///
/// | Variant      | Action | Triggers when       |
/// |--------------|--------|---------------------|
/// | `LimitBuy`   | Buy    | price ≤ limit_price |
/// | `TakeProfit` | Sell   | price ≥ limit_price |
/// | `StopLoss`   | Sell   | price ≤ limit_price |
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum OrderType {
    LimitBuy,
    TakeProfit,
    StopLoss,
}

impl OrderType {
    /// `true` if this order results in buying alpha (staking into subnet).
    pub fn is_buy(&self) -> bool {
        matches!(self, OrderType::LimitBuy)
    }
}

/// The canonical order payload that users sign off-chain.
/// Only its H256 hash is stored on-chain; the full struct is submitted by the
/// admin at execution time (or by the user at cancellation time).
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub struct Order<AccountId: Encode + Decode + TypeInfo + MaxEncodedLen + Clone> {
    /// The coldkey that authorised this order (pays TAO for buys; owns the
    /// staked alpha for sells).
    pub signer: AccountId,
    /// The hotkey to stake to (buy) or unstake from (sell).
    pub hotkey: AccountId,
    /// Target subnet.
    pub netuid: NetUid,
    /// Order type (LimitBuy, TakeProfit, or StopLoss).
    pub order_type: OrderType,
    /// Input amount: TAO (raw) for Buy, alpha (raw) for Sell.
    pub amount: u64,
    /// Price threshold in TAO/alpha (raw units, same scale as
    /// `OrderSwapInterface::current_alpha_price`).
    /// Buy: maximum acceptable price.  Sell: minimum acceptable price.
    pub limit_price: u64,
    /// Unix timestamp in milliseconds after which this order must not be executed.
    pub expiry: u64,
}

/// The envelope the admin submits on-chain: the order payload plus the user's
/// signature over the SCALE-encoded `Order`.
///
/// TODO: evaluate cross-chain replay protection. The signature covers only the
/// SCALE-encoded `Order` with no chain-specific domain separator (genesis hash,
/// chain ID, or pallet prefix). A signed order is therefore valid on any chain
/// that shares the same runtime types (e.g. a testnet fork). Consider prepending
/// a domain tag to the signed payload or adding the genesis hash as an `Order` field.
///
/// Signature verification is performed against `order.signer` (the AccountId)
/// directly. Only sr25519 signatures are accepted; ed25519 and ecdsa variants
/// of `MultiSignature` are rejected at validation time.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub struct SignedOrder<AccountId: Encode + Decode + TypeInfo + MaxEncodedLen + Clone> {
    pub order: Order<AccountId>,
    /// Sr25519 signature over `SCALE_ENCODE(order)`.
    pub signature: MultiSignature,
}

#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum OrderStatus {
    /// The order was successfully executed.
    Fulfilled,
    /// The user registered a cancellation intent before execution.
    Cancelled,
}

/// Classified, fee-adjusted entry produced by `validate_and_classify`.
/// Used in every in-memory batch pipeline step; never stored on-chain.
#[derive(Debug)]
pub(crate) struct OrderEntry<AccountId> {
    pub(crate) order_id: H256,
    pub(crate) signer: AccountId,
    pub(crate) hotkey: AccountId,
    pub(crate) side: OrderType,
    /// Gross input amount (before fee).
    pub(crate) gross: u64,
    /// Net input amount (after fee).
    pub(crate) net: u64,
    /// Fee amount (TAO for buys; 0 for sells – applied on TAO output).
    pub(crate) fee: u64,
}

// ── Pallet ───────────────────────────────────────────────────────────────────

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        PalletId,
        pallet_prelude::*,
        traits::{Get, UnixTime},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AccountIdConversion;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config<AccountId = AccountId32> {
        /// Full swap + balance execution interface (see [`OrderSwapInterface`]).
        type SwapInterface: OrderSwapInterface<Self::AccountId>;

        /// Time provider for expiry checks.
        type TimeProvider: UnixTime;

        /// Account that collects protocol fees.
        #[pallet::constant]
        type FeeCollector: Get<Self::AccountId>;

        /// Maximum number of orders in a single `execute_orders` call.
        /// Should equal `floor(max_block_weight / per_order_weight)`.
        #[pallet::constant]
        type MaxOrdersPerBatch: Get<u32>;

        /// PalletId used to derive the intermediary account for batch execution.
        ///
        /// The derived account temporarily holds pooled TAO and staked alpha
        /// during `execute_batched_orders` before distributing to order signers.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Hotkey registered in each subnet that the pallet's intermediary
        /// account stakes to/from during batch execution.
        ///
        /// This must be a hotkey registered on every subnet the pallet may
        /// operate on. Operators should register a dedicated hotkey and set
        /// this in the runtime configuration.
        #[pallet::constant]
        type PalletHotkey: Get<Self::AccountId>;
    }

    // ── Storage ───────────────────────────────────────────────────────────────

    /// Protocol fee in parts-per-billion (PPB). e.g. 1_000_000 PPB = 0.1%.
    #[pallet::storage]
    pub type ProtocolFee<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The privileged account that may call `set_protocol_fee`.
    /// Absent ⇒ no admin set; only root can change the fee.
    /// Set by root via `set_admin`.
    #[pallet::storage]
    pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Tracks the on-chain status of a known `OrderId`.
    /// Absent ⇒ never seen (still executable if valid).
    /// Present ⇒ Fulfilled or Cancelled (both are terminal).
    #[pallet::storage]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, H256, OrderStatus, OptionQuery>;

    // ── Events ────────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A limit order was successfully executed.
        OrderExecuted {
            order_id: H256,
            signer: T::AccountId,
            netuid: NetUid,
            order_type: OrderType,
            /// Input amount: TAO (raw) for Buy orders, alpha (raw) for Sell orders.
            amount_in: u64,
            /// Output amount: alpha (raw) received for Buy orders, TAO (raw) received for Sell orders (after fee).
            amount_out: u64,
        },
        /// An order was skipped during batch execution (invalid signature,
        /// expired, already processed, wrong netuid, or price not met).
        OrderSkipped { order_id: H256 },
        /// A user registered a cancellation intent for their order.
        OrderCancelled {
            order_id: H256,
            signer: T::AccountId,
        },
        /// The protocol fee was updated.
        ProtocolFeeSet { fee: u32 },
        /// The admin account was updated by root.
        AdminSet { new_admin: Option<T::AccountId> },
        /// Summary emitted once per `execute_batched_orders` call.
        GroupExecutionSummary {
            /// The subnet all orders in this batch belong to.
            netuid: NetUid,
            /// Direction of the net pool trade (Buy = net TAO into pool).
            net_side: OrderSide,
            /// Net amount sent to the pool (TAO for Buy, alpha for Sell).
            /// Zero when buys and sells perfectly offset each other.
            net_amount: u64,
            /// Tokens received back from the pool.
            /// Zero when `net_amount` is zero.
            actual_out: u64,
            /// Number of orders that were successfully executed.
            executed_count: u32,
        },
    }

    // ── Errors ────────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The provided signature does not match the order payload and signer.
        InvalidSignature,
        /// The order has already been Fulfilled or Cancelled.
        OrderAlreadyProcessed,
        /// The order's expiry timestamp is in the past.
        OrderExpired,
        /// The current market price does not satisfy the order's limit price.
        PriceConditionNotMet,
        /// Caller is not the order signer (required for cancellation).
        Unauthorized,
        /// Caller is neither root nor the current admin.
        NotAdmin,
        /// The pool swap returned zero output for a non-zero input.
        SwapReturnedZero,
    }

    // ── Extrinsics ────────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Execute a batch of signed limit orders. Admin-gated.
        ///
        /// Orders whose price condition is not yet met are silently skipped so
        /// that a single stale order cannot block the rest of the batch.
        /// Orders that fail for any other reason (expired, bad signature, etc.)
        /// are also skipped; the admin is expected to filter these off-chain.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(
            T::DbWeight::get().reads_writes(2, 1).saturating_mul(orders.len() as u64)
        ))]
        pub fn execute_orders(
            origin: OriginFor<T>,
            orders: BoundedVec<SignedOrder<T::AccountId>, T::MaxOrdersPerBatch>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            for signed_order in orders {
                // Best-effort: individual order failures do not revert the batch.
                let _ = Self::try_execute_order(signed_order);
            }

            Ok(())
        }

        /// Execute a batch of signed limit orders for a single subnet using
        /// aggregated (netted) pool interaction.
        ///
        /// Unlike `execute_orders`, which hits the pool once per order, this
        /// extrinsic:
        ///
        /// 1. Validates all orders (bad signature / expired / already processed /
        ///    price-not-met orders are skipped and emit `OrderSkipped`).
        /// 2. Fetches the current price once.
        /// 3. Aggregates all valid buy inputs (TAO) and sell inputs (alpha).
        /// 4. Nets the two sides: only the residual amount touches the pool in
        ///    a single swap, minimising price impact.
        /// 5. Distributes outputs pro-rata:
        ///    - Dominant-side orders split the pool output proportionally to
        ///      their individual net amounts.
        ///    - Offset-side orders are filled internally at the current price
        ///      (no pool interaction for them).
        /// 6. Collects protocol fees (TAO for buy orders, alpha → TAO for sell
        ///    orders) and routes them to `FeeCollector`.
        ///
        /// All orders in the batch must target `netuid`. Orders for a different
        /// subnet are skipped.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(
            T::DbWeight::get().reads_writes(3, 2).saturating_mul(orders.len() as u64)
        ))]
        pub fn execute_batched_orders(
            origin: OriginFor<T>,
            netuid: NetUid,
            orders: BoundedVec<SignedOrder<T::AccountId>, T::MaxOrdersPerBatch>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::do_execute_batched_orders(netuid, orders)
        }

        /// Register a cancellation intent for an order.
        ///
        /// Must be called by the order's signer. The full `Order` payload is
        /// provided so the pallet can derive the `OrderId`. Once marked
        /// Cancelled, the order can never be executed.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1)))]
        pub fn cancel_order(origin: OriginFor<T>, order: Order<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(order.signer == who, Error::<T>::Unauthorized);

            let order_id = Self::derive_order_id(&order);

            ensure!(
                Orders::<T>::get(order_id).is_none(),
                Error::<T>::OrderAlreadyProcessed
            );

            Orders::<T>::insert(order_id, OrderStatus::Cancelled);
            Self::deposit_event(Event::OrderCancelled {
                order_id,
                signer: who,
            });

            Ok(())
        }

        /// Set the protocol fee in parts-per-billion.
        ///
        /// May be called by root or the current admin account.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1)))]
        pub fn set_protocol_fee(origin: OriginFor<T>, fee: u32) -> DispatchResult {
            let is_root = ensure_root(origin.clone()).is_ok();
            if !is_root {
                let who = ensure_signed(origin)?;
                ensure!(
                    Admin::<T>::get().as_ref() == Some(&who),
                    Error::<T>::NotAdmin
                );
            }
            ProtocolFee::<T>::put(fee);
            Self::deposit_event(Event::ProtocolFeeSet { fee });
            Ok(())
        }

        /// Set or clear the admin account. Requires root.
        ///
        /// Pass `None` to remove the admin, leaving only root able to change fees.
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1)))]
        pub fn set_admin(origin: OriginFor<T>, new_admin: Option<T::AccountId>) -> DispatchResult {
            ensure_root(origin)?;
            match &new_admin {
                Some(a) => Admin::<T>::put(a),
                None => Admin::<T>::kill(),
            }
            Self::deposit_event(Event::AdminSet { new_admin });
            Ok(())
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Derive the on-chain `OrderId` as blake2_256 over the SCALE-encoded order.
        pub fn derive_order_id(order: &Order<T::AccountId>) -> H256 {
            H256(sp_core::hashing::blake2_256(&order.encode()))
        }

        /// Account derived from the pallet's `PalletId`.
        fn pallet_account() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Returns `true` if `signed_order` passes all execution preconditions:
        /// valid signature, not yet processed, not expired, and price condition met.
        /// Netuid is intentionally not checked here; callers handle that separately.
        fn is_order_valid(
            signed_order: &SignedOrder<T::AccountId>,
            order_id: H256,
            now_ms: u64,
            current_price: U96F32,
        ) -> bool {
            let order = &signed_order.order;
            matches!(signed_order.signature, MultiSignature::Sr25519(_))
                && signed_order
                    .signature
                    .verify(order.encode().as_slice(), &order.signer)
                && Orders::<T>::get(order_id).is_none()
                && now_ms <= order.expiry
                && match order.order_type {
                    OrderType::TakeProfit => {
                        current_price >= U96F32::saturating_from_num(order.limit_price)
                    }
                    OrderType::StopLoss | OrderType::LimitBuy => {
                        current_price <= U96F32::saturating_from_num(order.limit_price)
                    }
                }
        }

        /// Attempt to execute one signed order. Returns an error on any
        /// validation or execution failure without panicking.
        fn try_execute_order(
            signed_order: SignedOrder<T::AccountId>,
        ) -> DispatchResult {
            let order = &signed_order.order;
            let order_id = Self::derive_order_id(order);
            let now_ms = T::TimeProvider::now().as_millis() as u64;
            let current_price = T::SwapInterface::current_alpha_price(order.netuid);

            ensure!(
                Self::is_order_valid(&signed_order, order_id, now_ms, current_price),
                Error::<T>::InvalidSignature
            );

            // 5. Execute the swap, taking protocol fee from the input.
            let fee_ppb = ProtocolFee::<T>::get();
            let (amount_in, amount_out) = if order.order_type.is_buy() {
                let tao_in = TaoBalance::from(order.amount);
                // Deduct protocol fee from TAO input before swapping.
                let fee_tao = Self::ppb_of_tao(tao_in, fee_ppb);
                let tao_after_fee = tao_in.saturating_sub(fee_tao);

                let alpha_out = T::SwapInterface::buy_alpha(
                    &order.signer,
                    &order.hotkey,
                    order.netuid,
                    tao_after_fee,
                    TaoBalance::from(order.limit_price),
                )?;

                // Forward the fee TAO directly to FeeCollector.
                if !fee_tao.is_zero() {
                    T::SwapInterface::transfer_tao(&order.signer, &T::FeeCollector::get(), fee_tao)
                        .ok();
                }
                (order.amount, alpha_out.to_u64())
            } else {
                // Sell the full alpha amount; fee is taken from the TAO output.
                let tao_out = T::SwapInterface::sell_alpha(
                    &order.signer,
                    &order.hotkey,
                    order.netuid,
                    AlphaBalance::from(order.amount),
                    TaoBalance::from(order.limit_price),
                )?;

                // Deduct protocol fee from TAO output and forward to FeeCollector.
                let fee_tao = Self::ppb_of_tao(tao_out, fee_ppb);
                if !fee_tao.is_zero() {
                    T::SwapInterface::transfer_tao(&order.signer, &T::FeeCollector::get(), fee_tao)
                        .ok();
                }
                (order.amount, tao_out.saturating_sub(fee_tao).to_u64())
            };

            // 6. Mark as fulfilled and emit event.
            Orders::<T>::insert(order_id, OrderStatus::Fulfilled);
            Self::deposit_event(Event::OrderExecuted {
                order_id,
                signer: order.signer.clone(),
                netuid: order.netuid,
                order_type: order.order_type.clone(),
                amount_in,
                amount_out,
            });

            Ok(())
        }

        /// Thin orchestrator for `execute_batched_orders`.
        fn do_execute_batched_orders(
            netuid: NetUid,
            orders: BoundedVec<SignedOrder<T::AccountId>, T::MaxOrdersPerBatch>,
        ) -> DispatchResult {
            let now_ms = T::TimeProvider::now().as_millis() as u64;
            let fee_ppb = ProtocolFee::<T>::get();
            let current_price = T::SwapInterface::current_alpha_price(netuid);

            // Filter invalid/expired/price-missed orders; classify the rest into buys and sells.
            let (valid_buys, valid_sells) =
                Self::validate_and_classify(netuid, &orders, now_ms, fee_ppb, current_price);

            let executed_count = (valid_buys.len() + valid_sells.len()) as u32;
            if executed_count == 0 {
                return Ok(());
            }

            let total_buy_net: u128 = valid_buys.iter().map(|e| e.net as u128).sum();
            let total_sell_net: u128 = valid_sells.iter().map(|e| e.net as u128).sum();
            let total_sell_tao_equiv: u128 = Self::alpha_to_tao(total_sell_net, current_price);

            let pallet_acct = Self::pallet_account();
            let pallet_hotkey = T::PalletHotkey::get();

            // Pull all input assets into the pallet intermediary before touching the pool.
            Self::collect_assets(
                &valid_buys,
                &valid_sells,
                &pallet_acct,
                &pallet_hotkey,
                netuid,
            )?;

            // Execute a single pool swap for the residual (buy TAO minus sell TAO-equiv, or vice versa).
            let (net_side, actual_out) = Self::net_pool_swap(
                total_buy_net,
                total_sell_net,
                total_sell_tao_equiv,
                current_price,
                &pallet_acct,
                &pallet_hotkey,
                netuid,
            )?;

            // Give every buyer their pro-rata share of (pool alpha output + offset sell alpha).
            Self::distribute_alpha_pro_rata(
                &valid_buys,
                actual_out,
                total_buy_net,
                total_sell_net,
                &net_side,
                current_price,
                &pallet_acct,
                &pallet_hotkey,
                netuid,
            )?;

            // Give every seller their pro-rata share of (pool TAO output + offset buy TAO),
            // deducting the fee from each payout; returns the total sell-side fee in TAO.
            let sell_fee_tao = Self::distribute_tao_pro_rata(
                &valid_sells,
                actual_out,
                total_buy_net,
                total_sell_tao_equiv,
                &net_side,
                current_price,
                fee_ppb,
                &pallet_acct,
                netuid,
            )?;

            // Forward all accumulated TAO fees (buy input fees + sell output fees) to FeeCollector.
            Self::collect_fees(&valid_buys, sell_fee_tao, &pallet_acct);

            let net_amount = Self::net_amount_for_event(
                &net_side,
                total_buy_net,
                total_sell_net,
                total_sell_tao_equiv,
                current_price,
            );
            Self::deposit_event(Event::GroupExecutionSummary {
                netuid,
                net_side,
                net_amount,
                actual_out: actual_out as u64,
                executed_count,
            });

            Ok(())
        }

        /// Validate every order against `netuid`, signature, expiry, and price.
        /// Valid orders are split into two BoundedVecs by side.
        /// Each entry is `(order_id, signer, hotkey, gross, net, fee)`.
        pub(crate) fn validate_and_classify(
            netuid: NetUid,
            orders: &BoundedVec<SignedOrder<T::AccountId>, T::MaxOrdersPerBatch>,
            now_ms: u64,
            fee_ppb: u32,
            current_price: U96F32,
        ) -> (
            BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
        ) {
            let mut buys = BoundedVec::new();
            let mut sells = BoundedVec::new();

            orders
                .iter()
                .filter_map(|signed_order| {
                    let order = &signed_order.order;
                    let order_id = Self::derive_order_id(order);

                    let valid = order.netuid == netuid
                        && Self::is_order_valid(signed_order, order_id, now_ms, current_price);

                    if !valid {
                        Self::deposit_event(Event::OrderSkipped { order_id });
                        return None;
                    }

                    let (net, fee) = if order.order_type.is_buy() {
                        // Buy: fee on TAO input — buyer contributes less TAO to the pool.
                        let f = Self::ppb_of_tao(TaoBalance::from(order.amount), fee_ppb).to_u64();
                        (order.amount.saturating_sub(f), f)
                    } else {
                        // Sell: fee on TAO output — seller contributes full alpha; the fee
                        // is deducted from their TAO payout in `distribute_tao_pro_rata`.
                        // No alpha is withheld here, so fee is recorded as 0 in the entry.
                        (order.amount, 0u64)
                    };

                    Some(OrderEntry {
                        order_id,
                        signer: order.signer.clone(),
                        hotkey: order.hotkey.clone(),
                        side: order.order_type.clone(),
                        gross: order.amount,
                        net,
                        fee,
                    })
                })
                .for_each(|entry| {
                    // try_push cannot fail: both vecs share the same bound as `orders`.
                    if entry.side.is_buy() {
                        let _ = buys.try_push(entry);
                    } else {
                        let _ = sells.try_push(entry);
                    }
                });

            (buys, sells)
        }

        /// Pull gross TAO from each buyer and gross staked alpha from each seller
        /// into the pallet intermediary account, bypassing the pool.
        fn collect_assets(
            buys: &BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            sells: &BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            pallet_acct: &T::AccountId,
            pallet_hotkey: &T::AccountId,
            netuid: NetUid,
        ) -> DispatchResult {
            for e in buys.iter() {
                T::SwapInterface::transfer_tao(&e.signer, pallet_acct, TaoBalance::from(e.gross))?;
            }
            for e in sells.iter() {
                T::SwapInterface::transfer_staked_alpha(
                    &e.signer,
                    &e.hotkey,
                    pallet_acct,
                    pallet_hotkey,
                    netuid,
                    AlphaBalance::from(e.gross),
                )?;
            }
            Ok(())
        }

        /// Execute a single pool swap for the net (residual) amount.
        /// Returns `(net_side, actual_out)` where `actual_out` is in the output
        /// token units (alpha for Buy, TAO for Sell).
        fn net_pool_swap(
            total_buy_net: u128,
            total_sell_net: u128,
            total_sell_tao_equiv: u128,
            current_price: U96F32,
            pallet_acct: &T::AccountId,
            pallet_hotkey: &T::AccountId,
            netuid: NetUid,
        ) -> Result<(OrderSide, u128), DispatchError> {
            if total_buy_net >= total_sell_tao_equiv {
                let net_tao = (total_buy_net.saturating_sub(total_sell_tao_equiv)) as u64;
                let actual_alpha = if net_tao > 0 {
                    let out = T::SwapInterface::buy_alpha(
                        pallet_acct,
                        pallet_hotkey,
                        netuid,
                        TaoBalance::from(net_tao),
                        TaoBalance::ZERO,
                    )?
                    .to_u64() as u128;
                    ensure!(out > 0, Error::<T>::SwapReturnedZero);
                    out
                } else {
                    0u128
                };
                Ok((OrderSide::Buy, actual_alpha))
            } else {
                let total_buy_alpha_equiv = Self::tao_to_alpha(total_buy_net, current_price);
                let net_alpha = (total_sell_net.saturating_sub(total_buy_alpha_equiv)) as u64;
                let actual_tao = if net_alpha > 0 {
                    let out = T::SwapInterface::sell_alpha(
                        pallet_acct,
                        pallet_hotkey,
                        netuid,
                        AlphaBalance::from(net_alpha),
                        TaoBalance::ZERO,
                    )?
                    .to_u64() as u128;
                    ensure!(out > 0, Error::<T>::SwapReturnedZero);
                    out
                } else {
                    0u128
                };
                Ok((OrderSide::Sell, actual_tao))
            }
        }

        /// Distribute alpha pro-rata to ALL buyers and mark their orders fulfilled.
        ///
        /// - Buy-dominant: total alpha = pool output + sell-side alpha (passed through).
        /// - Sell-dominant: total alpha = buy-side TAO converted at `current_price`.
        pub(crate) fn distribute_alpha_pro_rata(
            buys: &BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            actual_out: u128,
            total_buy_net: u128,
            total_sell_net: u128,
            net_side: &OrderSide,
            current_price: U96F32,
            pallet_acct: &T::AccountId,
            pallet_hotkey: &T::AccountId,
            netuid: NetUid,
        ) -> DispatchResult {
            let total_alpha: u128 = match net_side {
                OrderSide::Buy => actual_out.saturating_add(total_sell_net),
                OrderSide::Sell => Self::tao_to_alpha(total_buy_net, current_price),
            };

            for e in buys.iter() {
                let share: u64 = if total_buy_net > 0 {
                    (total_alpha.saturating_mul(e.net as u128) / total_buy_net) as u64
                } else {
                    0
                };
                if share > 0 {
                    T::SwapInterface::transfer_staked_alpha(
                        pallet_acct,
                        pallet_hotkey,
                        &e.signer,
                        &e.hotkey,
                        netuid,
                        AlphaBalance::from(share),
                    )?;
                }
                Orders::<T>::insert(e.order_id, OrderStatus::Fulfilled);
                Self::deposit_event(Event::OrderExecuted {
                    order_id: e.order_id,
                    signer: e.signer.clone(),
                    netuid,
                    order_type: e.side.clone(),
                    amount_in: e.gross,
                    amount_out: share,
                });
            }
            Ok(())
        }

        /// Distribute TAO pro-rata to ALL sellers and mark their orders fulfilled.
        ///
        /// - Sell-dominant: total TAO = pool output + buy-side TAO (passed through).
        /// - Buy-dominant: each seller receives their alpha valued at `current_price`.
        ///
        /// Fee on TAO output: `ppb(share)` is withheld from each seller's payout and
        /// left in the pallet account. Returns the total sell-side fee TAO accumulated.
        pub(crate) fn distribute_tao_pro_rata(
            sells: &BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            actual_out: u128,
            total_buy_net: u128,
            total_sell_tao_equiv: u128,
            net_side: &OrderSide,
            current_price: U96F32,
            fee_ppb: u32,
            pallet_acct: &T::AccountId,
            netuid: NetUid,
        ) -> Result<u64, DispatchError> {
            let total_tao: u128 = match net_side {
                OrderSide::Sell => actual_out.saturating_add(total_buy_net),
                OrderSide::Buy => total_sell_tao_equiv,
            };

            let mut total_sell_fee_tao: u64 = 0;

            for e in sells.iter() {
                let sell_tao_equiv = Self::alpha_to_tao(e.net as u128, current_price);
                let gross_share: u64 = if total_sell_tao_equiv > 0 {
                    (total_tao.saturating_mul(sell_tao_equiv) / total_sell_tao_equiv) as u64
                } else {
                    0u64
                };
                let fee = Self::ppb_of_tao(TaoBalance::from(gross_share), fee_ppb).to_u64();
                let net_share = gross_share.saturating_sub(fee);
                total_sell_fee_tao = total_sell_fee_tao.saturating_add(fee);

                T::SwapInterface::transfer_tao(
                    pallet_acct,
                    &e.signer,
                    TaoBalance::from(net_share),
                )?;
                Orders::<T>::insert(e.order_id, OrderStatus::Fulfilled);
                Self::deposit_event(Event::OrderExecuted {
                    order_id: e.order_id,
                    signer: e.signer.clone(),
                    netuid,
                    order_type: e.side.clone(),
                    amount_in: e.gross,
                    amount_out: net_share,
                });
            }
            Ok(total_sell_fee_tao)
        }

        /// Route accumulated protocol fees to `FeeCollector`.
        ///
        /// Both buy and sell fees are always in TAO by this point:
        /// - Buy fees: withheld from TAO input in `validate_and_classify`.
        /// - Sell fees: withheld from TAO output in `distribute_tao_pro_rata`
        ///   (passed in as `sell_fee_tao`).
        ///
        /// Both transfers are best-effort and do not revert the batch on failure.
        pub(crate) fn collect_fees(
            buys: &BoundedVec<OrderEntry<T::AccountId>, T::MaxOrdersPerBatch>,
            sell_fee_tao: u64,
            pallet_acct: &T::AccountId,
        ) {
            let fee_collector = T::FeeCollector::get();

            let total_buy_fee: u64 = buys.iter().map(|e| e.fee).sum();
            let total_fee = total_buy_fee.saturating_add(sell_fee_tao);
            if total_fee > 0 {
                T::SwapInterface::transfer_tao(
                    pallet_acct,
                    &fee_collector,
                    TaoBalance::from(total_fee),
                )
                .ok();
            }

            // TODO: sweep rounding dust and any emissions accrued on the pallet account.
            // Pro-rata integer division leaves small alpha residuals in (pallet_account,
            // pallet_hotkey) after each batch.  Over time these accumulate and, if an
            // emission epoch fires while the dust is present, the pallet earns emissions
            // it never distributes.  Fix: add `staked_alpha(coldkey, hotkey, netuid) ->
            // AlphaBalance` to `OrderSwapInterface`, then sell the full remaining balance
            // here and forward the TAO to `FeeCollector`.
        }

        /// Compute the net amount field for the `GroupExecutionSummary` event.
        pub(crate) fn net_amount_for_event(
            net_side: &OrderSide,
            total_buy_net: u128,
            total_sell_net: u128,
            total_sell_tao_equiv: u128,
            current_price: U96F32,
        ) -> u64 {
            match net_side {
                OrderSide::Buy => (total_buy_net.saturating_sub(total_sell_tao_equiv)) as u64,
                OrderSide::Sell => {
                    let buy_alpha_equiv = Self::tao_to_alpha(total_buy_net, current_price) as u64;
                    (total_sell_net as u64).saturating_sub(buy_alpha_equiv)
                }
            }
        }

        /// Convert a TAO amount to alpha at `price` (TAO/alpha).
        /// Returns 0 when `price` is zero.
        fn tao_to_alpha(tao: u128, price: U96F32) -> u128 {
            if price == U96F32::from_num(0u32) {
                return 0u128;
            }
            (U96F32::from_num(tao) / price).saturating_to_num::<u128>()
        }

        /// Convert an alpha amount to TAO at `price` (TAO/alpha).
        fn alpha_to_tao(alpha: u128, price: U96F32) -> u128 {
            price
                .saturating_mul(U96F32::from_num(alpha))
                .saturating_to_num::<u128>()
        }

        pub(crate) fn ppb_of_tao(amount: TaoBalance, ppb: u32) -> TaoBalance {
            let result = (amount.to_u64() as u128)
                .saturating_mul(ppb as u128)
                .saturating_div(1_000_000_000);
            TaoBalance::from(result as u64)
        }

        pub(crate) fn ppb_of_alpha(amount: AlphaBalance, ppb: u32) -> AlphaBalance {
            let result = (amount.to_u64() as u128)
                .saturating_mul(ppb as u128)
                .saturating_div(1_000_000_000);
            AlphaBalance::from(result as u64)
        }
    }
}
