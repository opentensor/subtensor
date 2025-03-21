use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
use frame_system::pallet_prelude::*;
use core::marker::PhantomData;

use crate::tick::{Tick, TickIndex};

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

        /// The maximum fee rate that can be set
        #[pallet::constant]
        type MaxFeeRate: Get<u16>;
    }

    /// The fee rate applied to swaps, normalized value between 0 and u16::MAX
    ///
    /// For example, 0.3% is approximately 196
    #[pallet::storage]
    #[pallet::getter(fn fee_rate)]
    pub type FeeRate<T> = StorageValue<_, u16, ValueQuery>;
    
    /// Storage for all ticks, mapped by tick index
    #[pallet::storage]
    #[pallet::getter(fn ticks)]
    pub type Ticks<T> = StorageMap<_, Twox64Concat, TickIndex, Tick>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when the fee rate has been updated
        FeeRateSet { rate: u16 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The fee rate is too high
        FeeRateTooHigh,
    }

    /// Genesis configuration for the swap pallet
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Initial fee rate
        pub fee_rate: u16,
        /// Phantom data for unused generic
        pub _phantom: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                fee_rate: 0, // Default to 0% fee
                _phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Ensure the fee rate is within bounds
            assert!(
                self.fee_rate <= T::MaxFeeRate::get(),
                "Fee rate in genesis config is too high"
            );

            // Set the initial fee rate
            <FeeRate<T>>::put(self.fee_rate);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the fee rate for swaps (normalized value). For example, 0.3% is approximately 196.
        ///
        /// Only callable by the admin origin
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn set_fee_rate(origin: OriginFor<T>, rate: u16) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(rate <= T::MaxFeeRate::get(), Error::<T>::FeeRateTooHigh);

            <FeeRate<T>>::put(rate);

            Self::deposit_event(Event::FeeRateSet { rate });

            Ok(())
        }
    }
}
