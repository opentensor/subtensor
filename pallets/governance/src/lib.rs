#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, WeightInfo};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	//#[pallet::error]
	//pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Vote for a new coldkey to join the Triumvirate (Subnets)
		#[pallet::call_index(0)]
		#[pallet::weight((
			0,
			DispatchClass::Operational
		))]
		pub fn nominate(origin: OriginFor<T>) -> DispatchResult {


			Ok(().into())
		}

		/// Remove all emissions from a subnet (Triumvirate)
		#[pallet::call_index(1)]
		#[pallet::weight((
			0,
			DispatchClass::Operational
		))]
		pub fn censor(origin: OriginFor<T>) -> DispatchResult {


			Ok(().into())
		}

		/// Vote on a proposal created by the Triumvirate (Root)
		#[pallet::call_index(2)]
		#[pallet::weight((
			0,
			DispatchClass::Operational
		))]
		pub fn trenchant(origin: OriginFor<T>) -> DispatchResult {


			Ok(().into())
		}

		/// Propose an administrative action (Triumvirate)
		#[pallet::call_index(3)]
		#[pallet::weight((
			0,
			DispatchClass::Operational
		))]
		pub fn sudo(origin: OriginFor<T>) -> DispatchResult {


			Ok(().into())
		}

		/// Punish a delegate validator (Subnets)
		#[pallet::call_index(4)]
		#[pallet::weight((
			0,
			DispatchClass::Operational
		))]
		pub fn ostraca(origin: OriginFor<T>) -> DispatchResult {


			Ok(().into())
		}
	}
}
