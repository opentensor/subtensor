#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn anonymous_vote(_origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn remove_anonymous_vote(_origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}
