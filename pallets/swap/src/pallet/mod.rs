use frame_support::{PalletId, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use pallet_subtensor_swap_interface::LiquidityDataProvider;
use substrate_fixed::types::U64F64;

use crate::Position;
use crate::tick::{Tick, TickIndex};

mod impls;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The origin which may configure the swap parameters
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Implementor of
        /// [`LiquidityDataProvider`](pallet_subtensor_swap_interface::LiquidityDataProvider).
        type LiquidityDataProvider: LiquidityDataProvider;

        /// This type is used to derive protocol accoun ID.
        #[pallet::constant]
        type ProtocolId: Get<PalletId>;

        /// The maximum fee rate that can be set
        #[pallet::constant]
        type MaxFeeRate: Get<u16>;

        /// The maximum number of positions a user can have
        #[pallet::constant]
        type MaxPositions: Get<u32>;
    }

    /// The fee rate applied to swaps per subnet, normalized value between 0 and u16::MAX
    ///
    /// For example, 0.3% is approximately 196
    #[pallet::storage]
    #[pallet::getter(fn fee_rate)]
    pub type FeeRate<T> = StorageMap<_, Twox64Concat, u16, u16, ValueQuery>;

    /// Storage for all ticks, using subnet ID as the primary key and tick index as the secondary key
    #[pallet::storage]
    #[pallet::getter(fn ticks)]
    pub type Ticks<T> = StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, TickIndex, Tick>;

    /// Storage to determine whether swap V3 was initialized for a specific subnet.
    #[pallet::storage]
    #[pallet::getter(fn swap_v3_initialized)]
    pub type SwapV3Initialized<T> = StorageMap<_, Twox64Concat, u16, bool, ValueQuery>;

    /// Storage for the square root price of Alpha token for each subnet.
    #[pallet::storage]
    #[pallet::getter(fn alpha_sqrt_price)]
    pub type AlphaSqrtPrice<T> = StorageMap<_, Twox64Concat, u16, U64F64, ValueQuery>;

    /// Storage for the current liquidity amount for each subnet.
    #[pallet::storage]
    #[pallet::getter(fn current_liquidity)]
    pub type CurrentLiquidity<T> = StorageMap<_, Twox64Concat, u16, u64, ValueQuery>;

    /// Storage for user positions, using subnet ID and account ID as keys
    /// The value is a bounded vector of Position structs with details about the liquidity positions
    #[pallet::storage]
    #[pallet::getter(fn positions)]
    pub type Positions<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u16,
        Twox64Concat,
        T::AccountId,
        BoundedVec<Position, T::MaxPositions>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when the fee rate has been updated for a subnet
        FeeRateSet { netuid: u16, rate: u16 },
    }

    #[pallet::error]
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the fee rate for swaps on a specific subnet (normalized value).
        /// For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn set_fee_rate(origin: OriginFor<T>, netuid: u16, rate: u16) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            FeeRate::<T>::insert(netuid, rate);

            Self::deposit_event(Event::FeeRateSet { netuid, rate });

            Ok(())
        }
    }
}
