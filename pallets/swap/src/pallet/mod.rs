use core::num::NonZeroU64;

use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use substrate_fixed::types::U64F64;
use subtensor_swap_interface::{BalanceOps, SubnetInfo};

use crate::{
    NetUid,
    position::{Position, PositionId},
    tick::{LayerLevel, Tick, TickIndex},
    weights::WeightInfo,
};

pub use pallet::*;

mod impls;

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
        196 // 0.3 %
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
        /// This indicates a permanent switch from V2 to V3 swap.
        UserLiquidityEnabled { netuid: NetUid },

        /// Event emitted when liquidity is added to a subnet's liquidity pool.
        LiquidityAdded {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The hotkey account associated with the position
            hotkey: T::AccountId,
            /// The subnet identifier
            netuid: u16,
            /// Unique identifier for the liquidity position
            position_id: u128,
            /// The amount of liquidity added to the position
            liquidity: u64,
            /// The amount of TAO tokens committed to the position
            tao: u64,
            /// The amount of Alpha tokens committed to the position
            alpha: u64,
        },

        /// Event emitted when liquidity is removed from a subnet's liquidity pool.
        LiquidityRemoved {
            /// The coldkey account that owns the position
            coldkey: T::AccountId,
            /// The subnet identifier
            netuid: u16,
            /// Unique identifier for the liquidity position
            position_id: u128,
            /// The amount of TAO tokens returned to the user
            tao: u64,
            /// The amount of Alpha tokens returned to the user
            alpha: u64,
            /// The amount of TAO fees earned from the position
            fee_tao: u64,
            /// The amount of Alpha fees earned from the position
            fee_alpha: u64,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the fee rate for swaps on a specific subnet (normalized value).
        /// For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_fee_rate())]
        pub fn set_fee_rate(origin: OriginFor<T>, netuid: u16, rate: u16) -> DispatchResult {
            if ensure_root(origin.clone()).is_err() {
                let account_id: T::AccountId = ensure_signed(origin)?;
                ensure!(
                    T::SubnetInfo::is_owner(&account_id, netuid),
                    DispatchError::BadOrigin
                );
            }

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            // using u16 for compatibility
            let netuid = netuid.into();

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

            Ok(())
        }

        /// Enable user liquidity operations for a specific subnet. This permanently switches the
        /// subnet from V2 to V3 swap mode. Once enabled, it cannot be disabled.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_enabled_user_liquidity())]
        pub fn set_enabled_user_liquidity(origin: OriginFor<T>, netuid: u16) -> DispatchResult {
            if ensure_root(origin.clone()).is_err() {
                let account_id: T::AccountId = ensure_signed(origin)?;
                ensure!(
                    T::SubnetInfo::is_owner(&account_id, netuid),
                    DispatchError::BadOrigin
                );
            }

            ensure!(
                T::SubnetInfo::exists(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            let netuid = netuid.into();

            EnabledUserLiquidity::<T>::insert(netuid, true);

            Self::deposit_event(Event::UserLiquidityEnabled { netuid });

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
            netuid: u16,
            tick_low: i32,
            tick_high: i32,
            liquidity: u64,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            let tick_low = TickIndex::new(tick_low).map_err(|_| Error::<T>::InvalidTickRange)?;
            let tick_high = TickIndex::new(tick_high).map_err(|_| Error::<T>::InvalidTickRange)?;
            let (position_id, tao, alpha) = Self::do_add_liquidity(
                netuid.into(),
                &coldkey,
                &hotkey,
                tick_low,
                tick_high,
                liquidity,
            )?;

            // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
            let tao_provided = T::BalanceOps::decrease_balance(&coldkey, tao)?;
            ensure!(tao_provided == tao, Error::<T>::InsufficientBalance);

            let alpha_provided = T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid, alpha)?;
            ensure!(alpha_provided == alpha, Error::<T>::InsufficientBalance);

            // Emit an event
            Self::deposit_event(Event::LiquidityAdded {
                coldkey,
                hotkey,
                netuid,
                position_id: position_id.into(),
                liquidity,
                tao,
                alpha,
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
            netuid: u16,
            position_id: u128,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            // Remove liquidity
            let result = Self::do_remove_liquidity(netuid.into(), &coldkey, position_id.into())?;

            // Credit the returned tao and alpha to the account
            T::BalanceOps::increase_balance(&coldkey, result.tao.saturating_add(result.fee_tao));
            T::BalanceOps::increase_stake(
                &coldkey,
                &hotkey,
                netuid,
                result.alpha.saturating_add(result.fee_alpha),
            )?;

            // Emit an event
            Self::deposit_event(Event::LiquidityRemoved {
                coldkey,
                netuid: netuid.into(),
                position_id,
                tao: result.tao,
                alpha: result.alpha,
                fee_tao: result.fee_tao,
                fee_alpha: result.fee_alpha,
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
            netuid: u16,
            position_id: u128,
            liquidity_delta: i64,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            // Add or remove liquidity
            let result = Self::do_modify_position(
                netuid.into(),
                &coldkey,
                &hotkey,
                position_id.into(),
                liquidity_delta,
            )?;

            if liquidity_delta > 0 {
                // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
                let tao_provided = T::BalanceOps::decrease_balance(&coldkey, result.tao)?;
                ensure!(tao_provided == result.tao, Error::<T>::InsufficientBalance);

                let alpha_provided =
                    T::BalanceOps::decrease_stake(&coldkey, &hotkey, netuid, result.alpha)?;
                ensure!(
                    alpha_provided == result.alpha,
                    Error::<T>::InsufficientBalance
                );

                // Emit an event
                Self::deposit_event(Event::LiquidityAdded {
                    coldkey,
                    hotkey,
                    netuid,
                    position_id,
                    liquidity: liquidity_delta as u64,
                    tao: result.tao,
                    alpha: result.alpha,
                });
            } else {
                // Credit the returned tao and alpha to the account
                T::BalanceOps::increase_balance(
                    &coldkey,
                    result.tao.saturating_add(result.fee_tao),
                );
                T::BalanceOps::increase_stake(
                    &coldkey,
                    &hotkey,
                    netuid,
                    result.alpha.saturating_add(result.fee_alpha),
                )?;

                // Emit an event
                Self::deposit_event(Event::LiquidityRemoved {
                    coldkey,
                    netuid: netuid.into(),
                    position_id,
                    tao: result.tao,
                    alpha: result.alpha,
                    fee_tao: result.fee_tao,
                    fee_alpha: result.fee_alpha,
                });
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::DispatchError;

    use crate::{
        NetUid,
        mock::*,
        pallet::{EnabledUserLiquidity, Error, FeeRate},
    };

    #[test]
    fn test_set_fee_rate() {
        new_test_ext().execute_with(|| {
            let netuid = 1u16;
            let fee_rate = 500; // 0.76% fee

            assert_noop!(
                Swap::set_fee_rate(RuntimeOrigin::signed(666), netuid.into(), fee_rate),
                DispatchError::BadOrigin
            );

            assert_ok!(Swap::set_fee_rate(RuntimeOrigin::root(), netuid, fee_rate));

            // Check that fee rate was set correctly
            assert_eq!(FeeRate::<Test>::get(NetUid::from(netuid)), fee_rate);

            let fee_rate = fee_rate * 2;
            assert_ok!(Swap::set_fee_rate(
                RuntimeOrigin::signed(1),
                netuid,
                fee_rate
            ));
            assert_eq!(FeeRate::<Test>::get(NetUid::from(netuid)), fee_rate);

            // Verify fee rate validation - should fail if too high
            let too_high_fee = MaxFeeRate::get() + 1;
            assert_noop!(
                Swap::set_fee_rate(RuntimeOrigin::root(), netuid, too_high_fee),
                Error::<Test>::FeeRateTooHigh
            );
        });
    }

    #[test]
    fn test_set_enabled_user_liquidity() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(101);

            assert!(!EnabledUserLiquidity::<Test>::get(netuid));

            assert_ok!(Swap::set_enabled_user_liquidity(
                RuntimeOrigin::root(),
                netuid.into()
            ));

            assert!(EnabledUserLiquidity::<Test>::get(netuid));

            assert_noop!(
                Swap::set_enabled_user_liquidity(RuntimeOrigin::signed(666), netuid.into()),
                DispatchError::BadOrigin
            );

            assert_ok!(Swap::set_enabled_user_liquidity(
                RuntimeOrigin::signed(1),
                netuid.into()
            ));

            assert_noop!(
                Swap::set_enabled_user_liquidity(RuntimeOrigin::root(), NON_EXISTENT_NETUID),
                Error::<Test>::SubNetworkDoesNotExist
            );
        });
    }
}
