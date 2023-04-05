use super::*;
use crate::system::ensure_root;

use pallet_collective::Config;

use frame_support::dispatch::DispatchResult;

//
impl<T: Config> pallet::Pallet<T> { 
	//
	pub fn do_add_admin(origin: <T as frame_system::Config>::RuntimeOrigin) -> DispatchResult {
		ensure_root(origin)?;
		todo!();
	}

	//
	pub fn do_remove_admin(origin: <T as frame_system::Config>::RuntimeOrigin) -> DispatchResult {
		ensure_root(origin)?;
		todo!();
	}
}

