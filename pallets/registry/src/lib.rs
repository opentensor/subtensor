#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

pub use pallet_identity::types::IdentityInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		// Interface to allow other pallets to control who can register identities
		type CanRegister: crate::CanRegisterIdentity<Self::AccountId>;
		type MaxAdditionalFields: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityRegistered {who: T::AccountId}, // Emitted when a user registers an identity
		IdentityDissolved {who: T::AccountId}, // Emitted when a user dissolves an identity
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		CannotRegister, // Error emitted when a user attempts to register an identity that is invalid
	}

	/// Identity data by account
	#[pallet::storage]
	#[pallet::getter(fn identityOf)]
	pub type IdentityData<T: Config> =
		StorageMap<_, Identity, T::AccountId, IdentityInfo<MaxAdditionalFields>, OptionQuery>;

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight((T::WeightInfo::register_identity(), DispatchClass::Operational))]
		pub fn register_identity(origin: OriginFor<T>, info: Box<IdentityInfo<T::MaxAdditionalFields>>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(T::CanRegisterIdentity::can_register(account), Error::<T>::CannotRegister);

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
// Interfaces to interact with other pallets
pub trait CanRegisterIdentity<AccountId> {
    fn can_register(address: AccountId) -> bool;
}

impl<A> CanRegisterIdentity<A> for () {
	fn can_register(_: A) -> bool {false}
}
