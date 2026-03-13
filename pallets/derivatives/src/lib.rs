//! # Derivatives Pallet
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec;
use codec::{Decode, Encode};
use frame_support::{
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    sp_runtime::{RuntimeDebug, traits::Dispatchable},
    traits::{Get, IsSubType},
};
use frame_system::pallet_prelude::*;
use safe_math::*;
use scale_info::TypeInfo;
use weights::WeightInfo;

pub use pallet::*;
use substrate_fixed::types::U96F32;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaCurrency, BalanceOps, Currency, NetUid, TaoCurrency};

mod benchmarking;
mod mock;
mod tests;
pub mod weights;

#[derive(
    Encode,
    Clone,
    Decode,
    DecodeWithMemTracking,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum PositionType {
    #[default]
    Short,
}

/// Derivative position
#[freeze_struct("8a67a79bd2ec3369")]
#[derive(Encode, Decode, Default, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DerivativePosition<AccountId> {
    /// The hotkey against which the Alpha in this position is accounted
    pub hotkey: AccountId,
    /// Type of the position
    pub pos_type: PositionType,
    /// Liquidation price
    pub liquidation_price: U96F32,
    /// The position collateral
    pub tao_collateral: TaoCurrency,
    /// The tao received for selling alpha
    pub tao_proceeds: TaoCurrency,
    /// The position size in Alpha
    pub size: AlphaCurrency,
}

/// Trait for integration with the swap
pub trait DerivativeSwapInterface {
    /// Buy alpha with a given tao amount
    fn buy(netuid: NetUid, tao: TaoCurrency) -> Result<AlphaCurrency, DispatchError>;
    /// Buy tao with a given alpha amount
    fn sell(netuid: NetUid, alpha: AlphaCurrency) -> Result<TaoCurrency, DispatchError>;
    /// Get the amount of tao needed to buy the given amount of alpha
    fn get_tao_for_alpha_amount(netuid: NetUid, alpha: AlphaCurrency) -> TaoCurrency;
    /// Get the amount of alpha needed to buy the given amount of tao
    fn get_alpha_for_tao_amount(netuid: NetUid, tao: TaoCurrency) -> AlphaCurrency;
    /// Mint alpha
    fn mint_alpha(netuid: NetUid, alpha: AlphaCurrency);
    /// Burn alpha
    fn burn_alpha(netuid: NetUid, alpha: AlphaCurrency);
    /// Get alpha EMA price
    fn get_alpha_ema_price(netuid: NetUid) -> U96F32;
    /// Remove alpha from reserve and update price accordingly
    fn decrease_alpha_reserve(netuid: NetUid, alpha: AlphaCurrency) -> DispatchResult;
}

pub type PositionInfoOf<T> = DerivativePosition<<T as frame_system::Config>::AccountId>;

#[frame_support::pallet]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Operations with balances and stakes
        type BalanceOps: BalanceOps<Self::AccountId>;

        // /// The currency mechanism.
        // type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
        //     + fungible::Mutate<Self::AccountId>;

        /// The weight information for the pallet.
        type WeightInfo: WeightInfo;

        /// The mechanism to swap, mint, and burn
        type SwapInterface: DerivativeSwapInterface;

        /// Collateral ratio per billion
        type CollateralRatio: Get<u64>;

        /// Minimum position size in TAO
        type MinPositionSize: Get<TaoCurrency>;
    }

    /// A map of open positions
    #[pallet::storage]
    pub type Positions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // cold
            NMapKey<Identity, NetUid>,               // subnet
        ),
        PositionInfoOf<T>,
        OptionQuery,
    >;

    /// TODO: Structure that allows efficient search of positions by liquidation price
    // #[pallet::storage]
    // pub type PositionIndex<T: Config> =

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A position was opened
        Opened {
            netuid: NetUid,
            coldkey: T::AccountId,
            hotkey: T::AccountId,
            pos_type: PositionType,
            collateral: TaoCurrency,
            size: AlphaCurrency,
            open_price: U96F32,
        },
        /// A position was closed
        Closed {
            netuid: NetUid,
            coldkey: T::AccountId,
            hotkey: T::AccountId,
            pos_type: PositionType,
            size: AlphaCurrency,
            close_price: U96F32, // Average close price
            liquidation: bool,   // Whether position was liquidated or closed voluntarily
            partial: bool,       // Partial or full close
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No open position exists
        NoOpenPosition,
        /// Trying to close for greater size than open position
        InsufficientPositionSize,
        /// Position size is too low
        AmountTooLow,
        /// Insufficient TAO balance to open position
        InsufficientBalance,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let weight = frame_support::weights::Weight::from_parts(0, 0);

            // weight = weight
            //     .saturating_add(migrations::migrate_...::<T>());

            weight
        }

        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            // Execute liquidations here
            todo!();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #![deny(clippy::expect_used)]

        /// Open a short position at the specified subnet and hotkey
        ///
        ///   - Withdraw a collateral from the calling coldkey balance
        ///   - Mint and sell new alpha (tao_amount / ema_price), record alpha_amount
        ///   - Record received tao in tao_proceeds
        ///
        /// Parameters:
        /// - `hotkey`: The hotkey at which alpha is recorded
        /// - `netuid`: Subnet ID
        /// - `tao_amount`: Amount of TAO to spend on opening position
        #[pallet::call_index(0)]
        #[pallet::weight((
            Weight::from_parts(100_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Normal,
            Pays::Yes
        ))]
        pub fn open_short(
            origin: T::RuntimeOrigin,
            hotkey: T::AccountId,
            netuid: NetUid,
            tao_amount: TaoCurrency,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Make sure position size is above the limit
            ensure!(tao_amount >= T::MinPositionSize::get(), Error::<T>::AmountTooLow);

            // Withdraw collateral
            let tao_collateral = T::BalanceOps::decrease_balance(&coldkey, tao_amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // Set off the collateral ratio
            let collateral_ratio = Self::get_collateral_ratio();
            let position_tao: TaoCurrency = U96F32::saturating_from_num(tao_collateral)
                .safe_div(collateral_ratio)
                .saturating_to_num::<u64>()
                .into();

            // Get current alpha price and mint alpha_amount = (tao_amount / ema_price)
            let ema_price = T::SwapInterface::get_alpha_ema_price(netuid);
            let tao_amount_fixed = U96F32::saturating_from_num(position_tao);
            let alpha_amount_fixed = tao_amount_fixed.safe_div(ema_price);
            let alpha_amount = AlphaCurrency::from(alpha_amount_fixed.saturating_to_num::<u64>());
            T::SwapInterface::mint_alpha(netuid, alpha_amount);

            // Sell minted alpha
            let tao_proceeds = T::SwapInterface::sell(netuid, alpha_amount)?;

            // Create/update position
            Self::upsert_short_position_add(
                coldkey.clone(),
                hotkey.clone(), 
                netuid,
                tao_collateral,
                tao_proceeds,
                alpha_amount,
            );

            // Emit event
            Self::deposit_event(Event::Opened {
                netuid,
                coldkey,
                hotkey,
                pos_type: PositionType::Short,
                collateral: tao_collateral,
                size: alpha_amount,
                open_price: ema_price,
            });

            Ok(())
        }

        /// Close a short position at the specified subnet and hotkey.
        ///
        ///   - Buy position alpha amount from pool with collateral + proceeds, burn it
        ///     - If total tao is not enough, remove the alpha remainder from
        ///       the alpha reserve
        ///     - If any total tao left, credit the remainder to the coldkey
        ///       balance
        ///   - Update position accordingly
        ///
        /// Parameters:
        /// - `netuid`: Subnet ID
        #[pallet::call_index(1)]
        #[pallet::weight((
            Weight::from_parts(100_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Normal,
            Pays::Yes
        ))]
        pub fn close_short(
            origin: T::RuntimeOrigin,
            hotkey: T::AccountId,
            netuid: NetUid,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Get the current open short position
            let maybe_position = Positions::<T>::get((coldkey.clone(), netuid));
            let position = maybe_position.ok_or(Error::<T>::NoOpenPosition)?;

            // Get the position size
            let alpha_amount = position.size;

            // Calculate how much tao we need to buy the minted alpha back
            let tao_required_to_close = T::SwapInterface::get_tao_for_alpha_amount(netuid, alpha_amount);

            // Buy minted alpha back
            let alpha_amount_actual = T::SwapInterface::buy(netuid, tao_required_to_close)?;

            // Calculate the position tao remainder and act accordingly
            let total_tao = position.tao_collateral.saturating_add(position.tao_proceeds);
            if total_tao > tao_required_to_close {
                // Deposit remaining tao
                let tao_remainder = total_tao.saturating_sub(tao_required_to_close);
                T::BalanceOps::increase_balance(&coldkey, tao_remainder);
            } 

            // Calculate alpha needed to cover the loss if any and remove from alpha reserve
            if alpha_amount > alpha_amount_actual {
                let alpha_remainder = alpha_amount.saturating_sub(alpha_amount_actual);
                T::SwapInterface::decrease_alpha_reserve(netuid, alpha_remainder)?;
            }
            T::SwapInterface::burn_alpha(netuid, alpha_amount);

            // Calculate the average close price
            let close_price = U96F32::saturating_from_num(u64::from(tao_required_to_close)).safe_div(U96F32::saturating_from_num(alpha_amount));

            // Delete position
            Positions::<T>::remove((coldkey.clone(), netuid));

            // Emit event
            Self::deposit_event(Event::Closed {
                netuid,
                coldkey,
                hotkey,
                pos_type: PositionType::Short,
                size: alpha_amount,
                close_price,
                liquidation: false,
                partial: false,
            });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_collateral_ratio() -> U96F32 {
        U96F32::saturating_from_num(T::CollateralRatio::get())
            .safe_div(U96F32::saturating_from_num(1_000_000_000))
    }

    pub fn upsert_short_position_add(
        coldkey: T::AccountId,
        hotkey: T::AccountId, 
        netuid: NetUid,
        tao_collateral: TaoCurrency,
        tao_proceeds: TaoCurrency,
        size: AlphaCurrency,
    ) {
        let liquidation_price;
        let new_position = if let Some(position) = Positions::<T>::get((coldkey.clone(), netuid)) {
            // Update liquidation price
            // TBD
            liquidation_price = U96F32::saturating_from_num(1000.);

            let new_collateral = u64::from(tao_collateral).saturating_add(u64::from(position.tao_collateral));
            let new_proceeds = u64::from(tao_proceeds).saturating_add(u64::from(position.tao_proceeds));
            let new_size = u64::from(size).saturating_add(u64::from(position.size));

            DerivativePosition {
                hotkey,
                pos_type: PositionType::Short,
                liquidation_price,
                tao_collateral: new_collateral.into(),
                tao_proceeds: new_proceeds.into(),
                size: new_size.into(),
            }
        } else {
            // Calculate liquidation price
            // TBD
            liquidation_price = U96F32::saturating_from_num(1000.);

            DerivativePosition {
                hotkey,
                pos_type: PositionType::Short,
                liquidation_price,
                tao_collateral,
                tao_proceeds,
                size,
            }
        };

        Positions::<T>::insert((coldkey, netuid), new_position);
    }
}
