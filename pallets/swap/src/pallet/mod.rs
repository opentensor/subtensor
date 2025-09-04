use core::num::NonZeroU64;
use core::ops::Neg;

use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{
    AlphaCurrency, BalanceOps, Currency, NetUid, SubnetInfo, TaoCurrency,
};

use crate::{
    position::{Position, PositionId},
    tick::{LayerLevel, Tick, TickIndex},
    weights::WeightInfo,
};

pub use pallet::*;

mod impls;
#[cfg(test)]
mod tests;

#[allow(clippy::module_inception)]
#[frame_support::pallet]
mod pallet {
    use super::*;
    use frame_system::{ensure_root, ensure_signed};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Implementor of
        /// [`SubnetInfo`](subtensor_swap_interface::SubnetInfo).
        type SubnetInfo: SubnetInfo<Self::AccountId>;

        /// Implementor of
        /// [`BalanceOps`](subtensor_swap_interface::BalanceOps).
        type BalanceOps: BalanceOps<Self::AccountId>;

        /// This type is used to derive protocol accoun ID.
        #[pallet::constant]
        type ProtocolId: Get<PalletId>;

        /// The maximum fee rate that can be set
        #[pallet::constant]
        type MaxFeeRate: Get<u16>;

        /// The maximum number of positions a user can have
        #[pallet::constant]
        type MaxPositions: Get<u32>;

        /// Minimum liquidity that is safe for rounding and integer math.
        #[pallet::constant]
        type MinimumLiquidity: Get<u64>;

        /// Minimum reserve for tao and alpha
        #[pallet::constant]
        type MinimumReserve: Get<NonZeroU64>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Default fee rate if not set
    #[pallet::type_value]
    pub fn DefaultFeeRate() -> u16 {
        33 // ~0.05 %
    }

    /// The fee rate applied to swaps per subnet, normalized value between 0 and u16::MAX
    #[pallet::storage]
    pub type FeeRate<T> = StorageMap<_, Twox64Concat, NetUid, u16, ValueQuery, DefaultFeeRate>;

    // Global accrued fees in tao per subnet
    #[pallet::storage]
    pub type FeeGlobalTao<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    // Global accrued fees in alpha per subnet
    #[pallet::storage]
    pub type FeeGlobalAlpha<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    /// Storage for all ticks, using subnet ID as the primary key and tick index as the secondary key
    #[pallet::storage]
    pub type Ticks<T> = StorageDoubleMap<_, Twox64Concat, NetUid, Twox64Concat, TickIndex, Tick>;

    /// Storage to determine whether swap V3 was initialized for a specific subnet.
    #[pallet::storage]
    pub type SwapV3Initialized<T> = StorageMap<_, Twox64Concat, NetUid, bool, ValueQuery>;

    /// Storage for the square root price of Alpha token for each subnet.
    #[pallet::storage]
    pub type AlphaSqrtPrice<T> = StorageMap<_, Twox64Concat, NetUid, U64F64, ValueQuery>;

    /// Storage for the current price tick.
    #[pallet::storage]
    pub type CurrentTick<T> = StorageMap<_, Twox64Concat, NetUid, TickIndex, ValueQuery>;

    /// Storage for the current liquidity amount for each subnet.
    #[pallet::storage]
    pub type CurrentLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, u64, ValueQuery>;

    /// Indicates whether a subnet has been switched to V3 swap from V2.
    /// If `true`, the subnet is permanently on V3 swap mode allowing add/remove liquidity
    /// operations. Once set to `true` for a subnet, it cannot be changed back to `false`.
    #[pallet::storage]
    pub type EnabledUserLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, bool, ValueQuery>;

    /// Storage for user positions, using subnet ID and account ID as keys
    /// The value is a bounded vector of Position structs with details about the liquidity positions
    #[pallet::storage]
    pub type Positions<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, NetUid>,       // Subnet ID
            NMapKey<Twox64Concat, T::AccountId>, // Account ID
            NMapKey<Twox64Concat, PositionId>,   // Position ID
        ),
        Position<T>,
        OptionQuery,
    >;

    /// Position ID counter.
    #[pallet::storage]
    pub type LastPositionId<T> = StorageValue<_, u128, ValueQuery>;

    /// Tick index bitmap words storage
    #[pallet::storage]
    pub type TickIndexBitmapWords<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Twox64Concat, NetUid>,     // Subnet ID
            NMapKey<Twox64Concat, LayerLevel>, // Layer level
            NMapKey<Twox64Concat, u32>,        // word index
        ),
        u128,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when the fee rate has been updated for a subnet
        FeeRateSet { netuid: NetUid, rate: u16 },

        /// Event emitted when user liquidity operations are enabled for a subnet.
        /// First enable even indicates a switch from V2 to V3 swap.
        UserLiquidityToggled { netuid: NetUid, enable: bool },

        /// Event emitted when a liquidity position is added to a subnet's liquidity pool.
        LiquidityAdded {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha comes from
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity added to the position
            liquidity: u64,
            /// The amount of TAO tokens committed to the position
            tao: TaoCurrency,
            /// The amount of Alpha tokens committed to the position
            alpha: AlphaCurrency,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },

        /// Event emitted when a liquidity position is removed from a subnet's liquidity pool.
        LiquidityRemoved {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha goes to
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity removed from the position
            liquidity: u64,
            /// The amount of TAO tokens returned to the user
            tao: TaoCurrency,
            /// The amount of Alpha tokens returned to the user
            alpha: AlphaCurrency,
            /// The amount of TAO fees earned from the position
            fee_tao: TaoCurrency,
            /// The amount of Alpha fees earned from the position
            fee_alpha: AlphaCurrency,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },

        /// Event emitted when a liquidity position is modified in a subnet's liquidity pool.
        /// Modifying causes the fees to be claimed.
        LiquidityModified {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account where Alpha comes from or goes to
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: NetUid,
            /// Unique identifier for the liquidity position
            position_id: PositionId,
            /// The amount of liquidity added to or removed from the position
            liquidity: i64,
            /// The amount of TAO tokens returned to the user
            tao: i64,
            /// The amount of Alpha tokens returned to the user
            alpha: i64,
            /// The amount of TAO fees earned from the position
            fee_tao: TaoCurrency,
            /// The amount of Alpha fees earned from the position
            fee_alpha: AlphaCurrency,
            /// the lower tick
            tick_low: TickIndex,
            /// the upper tick
            tick_high: TickIndex,
        },
    }

    #[pallet::error]
    #[derive(PartialEq)]
    pub enum Error<T> {
        /// The fee rate is too high
        FeeRateTooHigh,

        /// The provided amount is insufficient for the swap.
        InsufficientInputAmount,

        /// The provided liquidity is insufficient for the operation.
        InsufficientLiquidity,

        /// The operation would exceed the price limit.
        PriceLimitExceeded,

        /// The caller does not have enough balance for the operation.
        InsufficientBalance,

        /// Attempted to remove liquidity that does not exist.
        LiquidityNotFound,

        /// The provided tick range is invalid.
        InvalidTickRange,

        /// Maximum user positions exceeded
        MaxPositionsExceeded,

        /// Too many swap steps
        TooManySwapSteps,

        /// Provided liquidity parameter is invalid (likely too small)
        InvalidLiquidityValue,

        /// Reserves too low for operation.
        ReservesTooLow,

        /// The subnet does not exist.
        SubNetworkDoesNotExist,

        /// User liquidity operations are disabled for this subnet
        UserLiquidityDisabled,

        /// The subnet does not have subtoken enabled
        SubtokenDisabled,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the fee rate for swaps on a specific subnet (normalized value).
        /// For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_fee_rate())]
        pub fn set_fee_rate(origin: OriginFor<T>, netuid: NetUid, rate: u16) -> DispatchResult {
            if ensure_root(origin.clone()).is_err() {
                let account_id: T::AccountId = ensure_signed(origin)?;
                ensure!(
                    T::SubnetInfo::is_owner(&account_id, netuid.into()),
                    DispatchError::BadOrigin
                );
            }

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubNetworkDoesNotExist
            );

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

            Ok(())
        }

        /// Enable user liquidity operations for a specific subnet. This switches the
        /// subnet from V2 to V3 swap mode. Thereafter, adding new user liquidity can be disabled
        /// by toggling this flag to false, but the swap mode will remain V3 because of existing
        /// user liquidity until all users withdraw their liquidity.
        ///
        /// Only sudo or subnet owner can enable user liquidity.
        /// Only sudo can disable user liquidity.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::toggle_user_liquidity())]
        pub fn toggle_user_liquidity(
            origin: OriginFor<T>,
            netuid: NetUid,
            enable: bool,
        ) -> DispatchResult {
            if ensure_root(origin.clone()).is_err() {
                let account_id: T::AccountId = ensure_signed(origin)?;
                // Only enabling is allowed to subnet owner
                ensure!(
                    T::SubnetInfo::is_owner(&account_id, netuid.into()) && enable,
                    DispatchError::BadOrigin
                );
            }

            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubNetworkDoesNotExist
            );

            EnabledUserLiquidity::<T>::insert(netuid, enable);

            Self::deposit_event(Event::UserLiquidityToggled { netuid, enable });

            Ok(())
        }

        /// Add liquidity to a specific price range for a subnet.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - tick_low: Lower bound of the price range
        /// - tick_high: Upper bound of the price range
        /// - liquidity: Amount of liquidity to add
        ///
        /// Emits `Event::LiquidityAdded` on success
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            netuid: NetUid,
            tick_low: TickIndex,
            tick_high: TickIndex,
            liquidity: u64,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubNetworkDoesNotExist
            );

            ensure!(
                T::SubnetInfo::is_subtoken_enabled(netuid.into()),
                Error::<T>::SubtokenDisabled
            );

            let (position_id, tao, alpha) = Self::do_add_liquidity(
                netuid.into(),
                &coldkey,
                &hotkey,
                tick_low,
                tick_high,
                liquidity,
            )?;
            let alpha = AlphaCurrency::from(alpha);
            let tao = TaoCurrency::from(tao);

            // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
            let tao_provided = T::BalanceOps::decrease_balance(&coldkey, tao)?;
            ensure!(tao_provided == tao, Error::<T>::InsufficientBalance);

            let alpha_provided =
                T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid.into(), alpha)?;
            ensure!(alpha_provided == alpha, Error::<T>::InsufficientBalance);

            // Add provided liquidity to user-provided reserves
            T::BalanceOps::increase_provided_tao_reserve(netuid.into(), tao_provided);
            T::BalanceOps::increase_provided_alpha_reserve(netuid.into(), alpha_provided);

            // Emit an event
            Self::deposit_event(Event::LiquidityAdded {
                coldkey,
                hotkey,
                netuid,
                position_id,
                liquidity,
                tao,
                alpha,
                tick_low,
                tick_high,
            });

            Ok(())
        }

        /// Remove liquidity from a specific position.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - position_id: ID of the position to remove
        ///
        /// Emits `Event::LiquidityRemoved` on success
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::remove_liquidity())]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            netuid: NetUid,
            position_id: PositionId,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubNetworkDoesNotExist
            );

            ensure!(
                T::SubnetInfo::is_subtoken_enabled(netuid.into()),
                Error::<T>::SubtokenDisabled
            );

            // Remove liquidity
            let result = Self::do_remove_liquidity(netuid, &coldkey, position_id)?;

            // Credit the returned tao and alpha to the account
            T::BalanceOps::increase_balance(&coldkey, result.tao.saturating_add(result.fee_tao));
            T::BalanceOps::increase_stake(
                &coldkey,
                &hotkey,
                netuid.into(),
                result.alpha.saturating_add(result.fee_alpha),
            )?;

            // Remove withdrawn liquidity from user-provided reserves
            T::BalanceOps::decrease_provided_tao_reserve(netuid.into(), result.tao);
            T::BalanceOps::decrease_provided_alpha_reserve(netuid.into(), result.alpha);

            // Emit an event
            Self::deposit_event(Event::LiquidityRemoved {
                coldkey,
                hotkey,
                netuid: netuid.into(),
                position_id,
                liquidity: result.liquidity,
                tao: result.tao,
                alpha: result.alpha,
                fee_tao: result.fee_tao,
                fee_alpha: result.fee_alpha,
                tick_low: result.tick_low.into(),
                tick_high: result.tick_high.into(),
            });

            Ok(())
        }

        /// Modify a liquidity position.
        ///
        /// Parameters:
        /// - origin: The origin of the transaction
        /// - netuid: Subnet ID
        /// - position_id: ID of the position to remove
        /// - liquidity_delta: Liquidity to add (if positive) or remove (if negative)
        ///
        /// Emits `Event::LiquidityRemoved` on success
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::modify_position())]
        pub fn modify_position(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            netuid: NetUid,
            position_id: PositionId,
            liquidity_delta: i64,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::SubNetworkDoesNotExist
            );

            ensure!(
                T::SubnetInfo::is_subtoken_enabled(netuid.into()),
                Error::<T>::SubtokenDisabled
            );

            // Add or remove liquidity
            let result =
                Self::do_modify_position(netuid, &coldkey, &hotkey, position_id, liquidity_delta)?;

            if liquidity_delta > 0 {
                // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
                let tao_provided = T::BalanceOps::decrease_balance(&coldkey, result.tao)?;
                ensure!(tao_provided == result.tao, Error::<T>::InsufficientBalance);

                let alpha_provided =
                    T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid.into(), result.alpha)?;
                ensure!(
                    alpha_provided == result.alpha,
                    Error::<T>::InsufficientBalance
                );

                // Emit an event
                Self::deposit_event(Event::LiquidityModified {
                    coldkey: coldkey.clone(),
                    hotkey: hotkey.clone(),
                    netuid,
                    position_id,
                    liquidity: liquidity_delta,
                    tao: result.tao.to_u64() as i64,
                    alpha: result.alpha.to_u64() as i64,
                    fee_tao: result.fee_tao,
                    fee_alpha: result.fee_alpha,
                    tick_low: result.tick_low,
                    tick_high: result.tick_high,
                });
            } else {
                // Credit the returned tao and alpha to the account
                T::BalanceOps::increase_balance(&coldkey, result.tao);
                T::BalanceOps::increase_stake(&coldkey, &hotkey, netuid.into(), result.alpha)?;

                // Emit an event
                if result.removed {
                    Self::deposit_event(Event::LiquidityRemoved {
                        coldkey: coldkey.clone(),
                        hotkey: hotkey.clone(),
                        netuid,
                        position_id,
                        liquidity: liquidity_delta.unsigned_abs(),
                        tao: result.tao,
                        alpha: result.alpha,
                        fee_tao: result.fee_tao,
                        fee_alpha: result.fee_alpha,
                        tick_low: result.tick_low,
                        tick_high: result.tick_high,
                    });
                } else {
                    Self::deposit_event(Event::LiquidityModified {
                        coldkey: coldkey.clone(),
                        hotkey: hotkey.clone(),
                        netuid,
                        position_id,
                        liquidity: liquidity_delta,
                        tao: (result.tao.to_u64() as i64).neg(),
                        alpha: (result.alpha.to_u64() as i64).neg(),
                        fee_tao: result.fee_tao,
                        fee_alpha: result.fee_alpha,
                        tick_low: result.tick_low,
                        tick_high: result.tick_high,
                    });
                }
            }

            // Credit accrued fees to user account (no matter if liquidity is added or removed)
            if result.fee_tao > TaoCurrency::ZERO {
                T::BalanceOps::increase_balance(&coldkey, result.fee_tao);
            }
            if !result.fee_alpha.is_zero() {
                T::BalanceOps::increase_stake(
                    &coldkey,
                    &hotkey.clone(),
                    netuid.into(),
                    result.fee_alpha,
                )?;
            }

            Ok(())
        }
    }
}
