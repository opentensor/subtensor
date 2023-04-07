use super::*;
use crate::system::ensure_root;

use pallet_collective::Config;

use frame_support::dispatch::DispatchResult;

// ~/.cargo/git/checkouts/substrate-7e08433d4c370a21/1837f42/frame/support/src/traits/members.rs
// 277
//
impl<T: Config> Pallet<T> { 
	//
	pub fn do_add_admin(origin: <T as frame_system::Config>::RuntimeOrigin) -> DispatchResult {
		ensure_root(origin)?;
		T::ChangeAdmins::change_members_sorted();
		todo!();
	}

	//
	pub fn do_remove_admin(origin: <T as frame_system::Config>::RuntimeOrigin) -> DispatchResult {
		ensure_root(origin)?;
		todo!();
	}
}

