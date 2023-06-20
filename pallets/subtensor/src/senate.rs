use super::*;
use frame_support::{ pallet_prelude::DispatchResult};
use frame_system::{ensure_signed};

impl<T: Config> Pallet<T> {
	/// do_join_senate(RuntimeOrigin hotkey)
	/// 
	/// This extrinsic allows top delegates to join the senate, a group of accounts that votes on proposals from the Triumvirate.
	/// In order to join the senate, the hotkey must:
	/// 
	/// 1) Be registered as a delegate
	/// 2) Control greater than 2% of the total staked volume.
    pub fn do_join_senate( 
        origin: T::RuntimeOrigin,
    ) -> DispatchResult {
        let hotkey = ensure_signed( origin )?; 
        log::info!("do_join_senate( hotkey:{:?} )", hotkey );

		// Check all our senate requirements
		ensure!(Self::is_hotkey_registered_on_any_network(&hotkey), Error::<T>::NotRegistered);
		ensure!(!T::SenateMembers::is_member(&hotkey), Error::<T>::AlreadyRegistered);
        ensure!(Self::hotkey_is_delegate(&hotkey), Error::<T>::NotRegistered);

		let total_stake = Self::get_total_stake();
		let current_stake = Self::get_total_stake_for_hotkey(&hotkey);
		ensure!(total_stake > 0 && current_stake > 0, Error::<T>::NotEnoughStaketoWithdraw);

		ensure!(current_stake * 100 / total_stake >= SenateRequiredStakePercentage::<T>::get(), Error::<T>::NotEnoughStaketoWithdraw);

		// If we're full, we'll swap out the lowest stake member.
		let members = T::SenateMembers::members();
		if (members.len() as u32) == T::SenateMembers::max_members() {
			let mut sorted_members = members.clone();
			sorted_members.sort_by(|a, b| {
				let a_stake = Self::get_total_stake_for_hotkey(a);
				let b_stake = Self::get_total_stake_for_hotkey(b);

				b_stake.cmp(&a_stake)
			});

			if let Some(last) = sorted_members.last() {
				let last_stake = Self::get_total_stake_for_hotkey(last);

				ensure!(last_stake < current_stake, Error::<T>::NotEnoughStaketoWithdraw);

				return T::SenateMembers::swap_member(last, &hotkey);
			}
		}

		// Since we're calling another extrinsic, we want to propagate our errors back up the call stack.
        T::SenateMembers::add_member(&hotkey)
    }

	pub fn do_leave_senate( 
        origin: T::RuntimeOrigin,
    ) -> DispatchResult {
        let hotkey = ensure_signed( origin )?; 
        log::info!("do_leave_senate( hotkey:{:?} )", hotkey );

		// Check all our leave requirements
		ensure!(T::SenateMembers::is_member(&hotkey), Error::<T>::NotRegistered);

        T::SenateMembers::remove_member(&hotkey)
    }
}