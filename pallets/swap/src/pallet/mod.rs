use core::num::NonZeroU64;

use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
// use safe_math::SafeDiv;
use subtensor_runtime_common::{
    AlphaCurrency, BalanceOps, CurrencyReserve, NetUid, SubnetInfo, TaoCurrency,
};

use crate::{pallet::balancer::Balancer, weights::WeightInfo};

pub use pallet::*;

mod balancer;
mod hooks;
mod impls;
pub mod migrations;
mod swap_step;
#[cfg(test)]
mod tests;

// Define a maximum length for the migration key
type MigrationKeyMaxLen = ConstU32<128>;

#[allow(clippy::module_inception)]
#[frame_support::pallet]
#[allow(clippy::expect_used)]
mod pallet {
    use super::*;
    use frame_system::ensure_root;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Implementor of
        /// [`SubnetInfo`](subtensor_swap_interface::SubnetInfo).
        type SubnetInfo: SubnetInfo<Self::AccountId>;

        /// Tao reserves info.
        type TaoReserve: CurrencyReserve<TaoCurrency>;

        /// Alpha reserves info.
        type AlphaReserve: CurrencyReserve<AlphaCurrency>;

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

    /// Storage for the current liquidity amount for each subnet.
    #[pallet::storage]
    pub type CurrentLiquidity<T> = StorageMap<_, Twox64Concat, NetUid, u64, ValueQuery>;

    /// Position ID counter.
    #[pallet::storage]
    pub type LastPositionId<T> = StorageValue<_, u128, ValueQuery>;

    ////////////////////////////////////////////////////
    // Balancer (PalSwap) maps and variables

    /// Default reserve weight
    #[pallet::type_value]
    pub fn DefaultBalancer() -> Balancer {
        Balancer::default()
    }
    /// u64-normalized reserve weight
    #[pallet::storage]
    pub type SwapBalancer<T> =
        StorageMap<_, Twox64Concat, NetUid, Balancer, ValueQuery, DefaultBalancer>;

    /// Storage to determine whether balancer swap was initialized for a specific subnet.
    #[pallet::storage]
    pub type PalSwapInitialized<T> = StorageMap<_, Twox64Concat, NetUid, bool, ValueQuery>;

    /// Total fees in TAO per subnet due to be paid to users / protocol
    #[pallet::storage]
    pub type FeesTao<T> = StorageMap<_, Twox64Concat, NetUid, TaoCurrency, ValueQuery>;

    /// Total fees in Alpha per subnet due to be paid to users / protocol
    #[pallet::storage]
    pub type FeesAlpha<T> = StorageMap<_, Twox64Concat, NetUid, AlphaCurrency, ValueQuery>;

    /// --- Storage for migration run status
    #[pallet::storage]
    pub type HasMigrationRun<T: Config> =
        StorageMap<_, Identity, BoundedVec<u8, MigrationKeyMaxLen>, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when the fee rate has been updated for a subnet
        FeeRateSet { netuid: NetUid, rate: u16 },
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
        MechanismDoesNotExist,

        /// The subnet does not have subtoken enabled
        SubtokenDisabled,

        /// Swap reserves are too imbalanced
        ReservesOutOfBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #![deny(clippy::expect_used)]

        /// Set the fee rate for swaps on a specific subnet (normalized value).
        /// For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_fee_rate())]
        pub fn set_fee_rate(origin: OriginFor<T>, netuid: NetUid, rate: u16) -> DispatchResult {
            ensure_root(origin)?;

            // Ensure that the subnet exists.
            ensure!(
                T::SubnetInfo::exists(netuid.into()),
                Error::<T>::MechanismDoesNotExist
            );

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

            Ok(())
        }
    }
}
