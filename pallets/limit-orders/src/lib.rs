#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::{IdentifyAccount, Verify};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::OrderSwapInterface;

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// The canonical order payload that users sign off-chain.
/// Only its H256 hash is stored on-chain; the full struct is submitted by the
/// admin at execution time (or by the user at cancellation time).
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
pub struct Order<AccountId: Encode + Decode + TypeInfo + MaxEncodedLen + Clone> {
    /// The coldkey that authorised this order (pays TAO for buys; owns the
    /// staked alpha for sells).
    pub signer: AccountId,
    /// The hotkey to stake to (buy) or unstake from (sell).
    pub hotkey: AccountId,
    /// Target subnet.
    pub netuid: NetUid,
    /// Buy or Sell.
    pub side: OrderSide,
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
/// Signature verification is performed against `order.signer` (the AccountId)
/// directly, which works because in Substrate sr25519/ed25519 AccountIds are
/// the raw public keys.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
pub struct SignedOrder<
    AccountId: Encode + Decode + TypeInfo + MaxEncodedLen + Clone,
    Signature: Encode + Decode + TypeInfo + MaxEncodedLen + Clone,
> {
    pub order: Order<AccountId>,
    /// Signature over `SCALE_ENCODE(order)`.
    pub signature: Signature,
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
pub enum OrderStatus {
    /// The order was successfully executed.
    Fulfilled,
    /// The user registered a cancellation intent before execution.
    Cancelled,
}

// ── Pallet ───────────────────────────────────────────────────────────────────

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Get, UnixTime},
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Signature type used to verify off-chain order authorisations.
        ///
        /// The `Verify::verify` method is called with the order's `signer`
        /// (`T::AccountId`) as the expected signer, which works for
        /// sr25519/ed25519 where AccountId == public key.
        ///
        /// For the subtensor runtime, set this to `sp_runtime::MultiSignature`.
        type Signature: Verify<Signer: IdentifyAccount<AccountId = Self::AccountId>>
            + Encode
            + Decode
            + DecodeWithMemTracking
            + TypeInfo
            + MaxEncodedLen
            + Clone
            + PartialEq
            + core::fmt::Debug;

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
    }

    // ── Storage ───────────────────────────────────────────────────────────────

    /// Protocol fee in parts-per-billion (PPB). e.g. 1_000_000 PPB = 0.1%.
    #[pallet::storage]
    pub type ProtocolFee<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Tracks the on-chain status of a known `OrderId`.
    /// Absent ⇒ never seen (still executable if valid).
    /// Present ⇒ Fulfilled or Cancelled (both are terminal).
    #[pallet::storage]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, H256, OrderStatus, OptionQuery>;

    /// The privileged account allowed to call `execute_orders` and `set_protocol_fee`.
    #[pallet::storage]
    pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    // ── Events ────────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A limit order was successfully executed.
        OrderExecuted {
            order_id: H256,
            signer: T::AccountId,
            netuid: NetUid,
            side: OrderSide,
        },
        /// A user registered a cancellation intent for their order.
        OrderCancelled {
            order_id: H256,
            signer: T::AccountId,
        },
        /// The admin account was updated.
        AdminSet { admin: T::AccountId },
        /// The protocol fee was updated.
        ProtocolFeeSet { fee: u32 },
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
        /// Caller is not the configured admin.
        NotAdmin,
        /// Caller is not the order signer (required for cancellation).
        Unauthorized,
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
            orders: BoundedVec<SignedOrder<T::AccountId, T::Signature>, T::MaxOrdersPerBatch>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Admin::<T>::get().as_ref() == Some(&who), Error::<T>::NotAdmin);

            for signed_order in orders {
                // Best-effort: individual order failures do not revert the batch.
                let _ = Self::try_execute_order(signed_order);
            }

            Ok(())
        }

        /// Register a cancellation intent for an order.
        ///
        /// Must be called by the order's signer. The full `Order` payload is
        /// provided so the pallet can derive the `OrderId`. Once marked
        /// Cancelled, the order can never be executed.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1)))]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order: Order<T::AccountId>,
        ) -> DispatchResult {
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

        /// Set the admin account. Requires root.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1)))]
        pub fn set_admin(origin: OriginFor<T>, new_admin: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            Admin::<T>::put(&new_admin);
            Self::deposit_event(Event::AdminSet { admin: new_admin });
            Ok(())
        }

        /// Set the protocol fee in parts-per-billion. Admin-gated.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0).saturating_add(T::DbWeight::get().writes(1)))]
        pub fn set_protocol_fee(origin: OriginFor<T>, fee: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Admin::<T>::get().as_ref() == Some(&who), Error::<T>::NotAdmin);
            ProtocolFee::<T>::put(fee);
            Self::deposit_event(Event::ProtocolFeeSet { fee });
            Ok(())
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Derive the on-chain `OrderId` as blake2_256 over the SCALE-encoded order.
        pub fn derive_order_id(order: &Order<T::AccountId>) -> H256 {
            H256(sp_core::hashing::blake2_256(&order.encode()))
        }

        /// Attempt to execute one signed order. Returns an error on any
        /// validation or execution failure without panicking.
        fn try_execute_order(
            signed_order: SignedOrder<T::AccountId, T::Signature>,
        ) -> DispatchResult {
            let order = &signed_order.order;
            let order_id = Self::derive_order_id(order);

            // 1. Verify the signature over the SCALE-encoded order.
            let message = order.encode();
            ensure!(
                signed_order
                    .signature
                    .verify(message.as_slice(), &order.signer),
                Error::<T>::InvalidSignature
            );

            // 2. Check the order has not already been processed.
            ensure!(
                Orders::<T>::get(order_id).is_none(),
                Error::<T>::OrderAlreadyProcessed
            );

            // 3. Check expiry.
            let now_ms = T::TimeProvider::now().as_millis() as u64;
            ensure!(now_ms <= order.expiry, Error::<T>::OrderExpired);

            // 4. Check price condition.
            let current_price = T::SwapInterface::current_alpha_price(order.netuid);
            let limit_price = U96F32::saturating_from_num(order.limit_price);
            match order.side {
                // Buy: only execute if alpha is at or below the limit price.
                OrderSide::Buy => ensure!(
                    current_price <= limit_price,
                    Error::<T>::PriceConditionNotMet
                ),
                // Sell: only execute if alpha is at or above the limit price.
                OrderSide::Sell => ensure!(
                    current_price >= limit_price,
                    Error::<T>::PriceConditionNotMet
                ),
            }

            // 5. Execute the swap, taking protocol fee from the input.
            let fee_ppb = ProtocolFee::<T>::get();
            match order.side {
                OrderSide::Buy => {
                    let tao_in = TaoBalance::from(order.amount);
                    // Deduct protocol fee from TAO input before swapping.
                    let fee_tao = Self::ppb_of_tao(tao_in, fee_ppb);
                    let tao_after_fee = tao_in.saturating_sub(fee_tao);

                    T::SwapInterface::buy_alpha(
                        &order.signer,
                        &order.hotkey,
                        order.netuid,
                        tao_after_fee,
                        TaoBalance::from(order.limit_price),
                    )?;

                    // Route the fee TAO to the fee collector as staked alpha.
                    if !fee_tao.is_zero() {
                        T::SwapInterface::buy_alpha(
                            &order.signer,
                            &T::FeeCollector::get(),
                            order.netuid,
                            fee_tao,
                            T::SwapInterface::current_alpha_price(order.netuid)
                                .saturating_to_num::<u64>()
                                .into(),
                        )
                        .ok();
                    }
                }
                OrderSide::Sell => {
                    let alpha_in = AlphaBalance::from(order.amount);
                    let fee_alpha = Self::ppb_of_alpha(alpha_in, fee_ppb);
                    let alpha_after_fee = alpha_in.saturating_sub(fee_alpha);

                    T::SwapInterface::sell_alpha(
                        &order.signer,
                        &order.hotkey,
                        order.netuid,
                        alpha_after_fee,
                        TaoBalance::from(order.limit_price),
                    )?;

                    // Sell fee alpha separately; TAO proceeds go to fee collector.
                    if !fee_alpha.is_zero() {
                        let fee_tao = T::SwapInterface::sell_alpha(
                            &order.signer,
                            &order.hotkey,
                            order.netuid,
                            fee_alpha,
                            TaoBalance::ZERO,
                        )
                        .unwrap_or(TaoBalance::ZERO);

                        if !fee_tao.is_zero() {
                            // The sell_alpha implementation is expected to credit TAO to
                            // the signer; transferring to fee collector requires a
                            // runtime-level BalanceOps call outside this pallet's scope.
                            // TODO: integrate BalanceOps to move fee TAO to FeeCollector.
                            let _ = fee_tao;
                        }
                    }
                }
            }

            // 6. Mark as fulfilled and emit event.
            Orders::<T>::insert(order_id, OrderStatus::Fulfilled);
            Self::deposit_event(Event::OrderExecuted {
                order_id,
                signer: order.signer.clone(),
                netuid: order.netuid,
                side: order.side.clone(),
            });

            Ok(())
        }

        fn ppb_of_tao(amount: TaoBalance, ppb: u32) -> TaoBalance {
            let result = (amount.to_u64() as u128)
                .saturating_mul(ppb as u128)
                .saturating_div(1_000_000_000);
            TaoBalance::from(result as u64)
        }

        fn ppb_of_alpha(amount: AlphaBalance, ppb: u32) -> AlphaBalance {
            let result = (amount.to_u64() as u128)
                .saturating_mul(ppb as u128)
                .saturating_div(1_000_000_000);
            AlphaBalance::from(result as u64)
        }
    }
}
