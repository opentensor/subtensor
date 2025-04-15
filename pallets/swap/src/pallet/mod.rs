use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use substrate_fixed::types::U64F64;
use subtensor_swap_interface::LiquidityDataProvider;

use crate::{
    NetUid, SqrtPrice,
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

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_subtensor::pallet::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The origin which may configure the swap parameters
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Implementor of
        /// [`LiquidityDataProvider`](subtensor_swap_interface::LiquidityDataProvider).
        type LiquidityDataProvider: LiquidityDataProvider<Self::AccountId>;

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

    /// Storage for the current liquidity amount for each subnet.
    #[pallet::storage]
    pub type CurrentLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, u64, ValueQuery>;

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
        Position,
        OptionQuery,
    >;

    /// Position ID counter.
    #[pallet::storage]
    pub type NextPositionId<T> = StorageValue<_, u128, ValueQuery>;

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

        /// Event emitted when liquidity is added
        LiquidityAdded {
            coldkey: T::AccountId,
            hotkey: T::AccountId,
            netuid: NetUid,
            position_id: PositionId,
            liquidity: u64,
            tao: u64,
            alpha: u64,
        },

        /// Event emitted when liquidity is removed
        LiquidityRemoved {
            coldkey: T::AccountId,
            netuid: NetUid,
            position_id: PositionId,
            tao: u64,
            alpha: u64,
            fee_tao: u64,
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

        /// Subnet does not exist
        SubnetDoesNotExist,

        /// Hotkey account does not exist
        HotKeyAccountDoesNotExist
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
            T::AdminOrigin::ensure_origin(origin)?;

            // using u16 for compatibility
            let netuid = netuid.into();

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

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
            ensure!(pallet_subtensor::Pallet::<T>::if_subnet_exist(netuid), Error::<T>::SubnetDoesNotExist);

            let netuid = netuid.into();
            let tick_low_index =
                TickIndex::new(tick_low).map_err(|_| Error::<T>::InvalidTickRange)?;
            let tick_high_index =
                TickIndex::new(tick_high).map_err(|_| Error::<T>::InvalidTickRange)?;

            let (position_id, tao, alpha) = Self::do_add_liquidity(
                netuid,
                &coldkey,
                &hotkey,
                tick_low_index,
                tick_high_index,
                liquidity,
            )?;

            // Remove TAO and Alpha balances or fail transaction if they can't be removed exactly
            let tao_provided =
                pallet_subtensor::Pallet::<T>::remove_balance_from_coldkey_account(&coldkey, tao)?;
            ensure!(
                tao_provided == tao,
                Error::<T>::InsufficientBalance
            );

            let alpha_provided = pallet_subtensor::Pallet::<T>::decrease_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid.into(), alpha);
            ensure!(
                alpha_provided == alpha,
                Error::<T>::InsufficientBalance
            );

            // Emit an event
            Self::deposit_event(Event::LiquidityAdded {
                coldkey,
                hotkey,
                netuid,
                position_id,
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
            let netuid = netuid.into();
            let position_id = PositionId::from(position_id);

            // Ensure that the subnet exists.
            ensure!(pallet_subtensor::Pallet::<T>::if_subnet_exist(netuid), Error::<T>::SubnetDoesNotExist);

            // Ensure the hotkey account exists
            ensure!(
                pallet_subtensor::Pallet::<T>::hotkey_account_exists(&hotkey),
                Error::<T>::HotKeyAccountDoesNotExist
            );

            // Remove liquidity
            let result = Self::do_remove_liquidity(netuid.into(), &coldkey, position_id)?;

            // Credit the returned tao and alpha to the account
            pallet_subtensor::Pallet::<T>::add_balance_to_coldkey_account(&coldkey, result.tao.saturating_add(result.fee_tao));
            pallet_subtensor::Pallet::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid, result.alpha.saturating_add(result.fee_alpha));

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
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        NetUid,
        mock::*,
        pallet::{Error, FeeRate, Pallet as SwapModule},
    };
    use frame_support::{assert_noop, assert_ok};

    #[test]
    fn test_set_fee_rate() {
        new_test_ext().execute_with(|| {
            // Create a test subnet
            let netuid = 1u16;
            let fee_rate = 500; // 0.76% fee

            // Set fee rate (requires admin/root origin)
            assert_ok!(SwapModule::<Test>::set_fee_rate(
                RuntimeOrigin::root(),
                netuid,
                fee_rate
            ));

            // Check that fee rate was set correctly
            let netuid_struct = NetUid::from(netuid);
            assert_eq!(FeeRate::<Test>::get(netuid_struct), fee_rate);

            // Verify fee rate validation - should fail if too high
            let too_high_fee = MaxFeeRate::get() + 1;
            assert_noop!(
                SwapModule::<Test>::set_fee_rate(RuntimeOrigin::root(), netuid, too_high_fee),
                Error::<Test>::FeeRateTooHigh
            );
        });
    }
}
