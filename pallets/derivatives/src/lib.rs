//! # Derivatives Pallet
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec;
use codec::{Decode, Encode};
use frame_support::{
    PalletId,
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
use subtensor_runtime_common::{AlphaCurrency, BalanceOps, NetUid, TaoCurrency};

mod benchmarking;
mod mock;
mod tests;
pub mod weights;

// pub type CurrencyOf<T> = <T as Config>::Currency;

// pub type BalanceOf<T> =
//     <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(
    Encode,
    Clone,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum PositionType {
    Short,
}

/// Derivative position
#[freeze_struct("4c42c445dd17e071")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DerivativePosition<AccountId> {
    /// Subnet ID where the position is open
    pub netuid: NetUid,
    /// The coldkey of account holding this position
    pub owner_coldkey: AccountId,
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
    fn buy(netuid: NetUid, tao: TaoCurrency) -> AlphaCurrency;
    /// Buy tao with a given alpha amount
    fn sell(netuid: NetUid, alpha: AlphaCurrency) -> TaoCurrency;
    /// Get the amount of tao needed to buy the given amount of alpha
    fn get_tao_for_alpha_amount(netuid: NetUid, alpha: AlphaCurrency) -> TaoCurrency;
    /// Mint alpha
    fn mint_alpha(netuid: NetUid, alpha: AlphaCurrency);
    /// Burn alpha
    fn burn_alpha(netuid: NetUid, alpha: AlphaCurrency);
    /// Get alpha EMA price
    fn get_alpha_ema_price(netuid: NetUid) -> U96F32;
}

pub type PositionInfoOf<T> = DerivativePosition<<T as frame_system::Config>::AccountId>;

pub type DerivativePositionId = u64;

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

        /// The pallet id that will be used to keep collateral
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The mechanism to swap, mint, and burn
        type SwapInterface: DerivativeSwapInterface;

        /// Collateral ratio per billion
        type CollateralRatio: Get<u64>;
    }

    /// A map of open positions
    #[pallet::storage]
    pub type Positions<T: Config> =
        StorageMap<_, Twox64Concat, DerivativePositionId, PositionInfoOf<T>, OptionQuery>;

    /// Position ID counter
    #[pallet::storage]
    pub type LastPositionId<T> = StorageValue<_, DerivativePositionId, ValueQuery>;

    /// TODO: Structure that allows efficient search of positions by liquidation price
    // #[pallet::storage]
    // pub type PositionIndex<T: Config> =

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A position was opened
        Opened {
            position_id: DerivativePositionId,
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
            position_id: DerivativePositionId,
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
    pub enum Error<T> {}

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

            // Withdraw collateral
            let tao_collateral = T::BalanceOps::decrease_balance(&coldkey, tao_amount)?;

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
            let tao_proceeds = T::SwapInterface::sell(netuid, alpha_amount);

            // Calculate liquidation price
            // TBD
            let liquidation_price = U96F32::saturating_from_num(1000.);

            // Create position
            let mut position_id = LastPositionId::<T>::get();
            position_id = position_id.saturating_add(1);
            LastPositionId::<T>::set(position_id);
            Positions::<T>::insert(
                position_id,
                DerivativePosition {
                    netuid,
                    owner_coldkey: coldkey.clone(),
                    hotkey: hotkey.clone(),
                    pos_type: PositionType::Short,
                    liquidation_price,
                    tao_collateral,
                    tao_proceeds,
                    size: alpha_amount,
                },
            );

            // Emit event
            Self::deposit_event(Event::Opened {
                position_id,
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
        ///   - Buy alpha_amount from pool with collateral, burn it
        ///     - If the result is less alpha than was minted, take more alpha from
        ///       reserves to match
        ///     - If the result is more alpha than was minted, sell the difference
        ///       and credit the received TAO to coldkey balance
        ///   - Update position accordingly
        ///
        /// Parameters:
        /// - `hotkey`: The hotkey at which alpha is recorded
        /// - `netuid`: Subnet ID
        /// - `alpha_amount`: Position size in alpha
        #[pallet::call_index(1)]
        #[pallet::weight((
            Weight::from_parts(100_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Normal,
            Pays::Yes
        ))]
        pub fn close_short(
            _origin: T::RuntimeOrigin,
            _hotkey: T::AccountId,
            _netuid: NetUid,
            _alpha_amount: AlphaCurrency,
        ) -> DispatchResult {
            todo!()
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_collateral_ratio() -> U96F32 {
        U96F32::saturating_from_num(T::CollateralRatio::get())
            .safe_div(U96F32::saturating_from_num(1_000_000_000))
    }
}
