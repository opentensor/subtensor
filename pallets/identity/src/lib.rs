#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_runtime::{
	traits::Member,
	RuntimeAppPublic,
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::BoundedVec;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Aura: crate::AuraInterface<Self::AuthorityId, Self::MaxAuthorities>;

		/// The identifier type for an authority.
		type AuthorityId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;
		/// The maximum number of authorities that the pallet can hold.
		type MaxAuthorities: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn swap_authorities(origin: OriginFor<T>, new_authorities: BoundedVec<T::AuthorityId, T::MaxAuthorities>) -> DispatchResult {
			ensure_root(origin)?;

			T::Aura::change_authorities(new_authorities);
			
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

// Interfaces to interact with other pallets

use sp_runtime::BoundedVec;

pub trait AuraInterface<AuthorityId, MaxAuthorities> {
    fn change_authorities(new: BoundedVec<AuthorityId, MaxAuthorities>);
}

impl<A, M> AuraInterface<A, M> for () {
	fn change_authorities(_: BoundedVec<A, M>) {}
}
