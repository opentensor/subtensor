use super::*;
use frame_support::{dispatch::Pays, pallet_prelude::{DispatchResult, DispatchResultWithPostInfo, Weight}};
use frame_system::{ensure_signed, ensure_root};
use sp_core::Get;

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
		hotkey: &T::AccountId
	) -> DispatchResult {
		let coldkey = ensure_signed( origin )?; 
		log::info!("do_join_senate( coldkey: {:?}, hotkey: {:?} )", coldkey, hotkey );

		// Ensure that the pairing is correct.
		ensure!( Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );

		// Check all our senate requirements
		ensure!(Self::is_hotkey_registered_on_any_network(&hotkey), Error::<T>::NotRegistered);
		ensure!(!T::SenateMembers::is_member(&hotkey), Error::<T>::AlreadySenateMember);
		ensure!(Self::hotkey_is_delegate(&hotkey), Error::<T>::NotDelegate);

		let total_stake = Self::get_total_stake();
		let current_stake = Self::get_total_stake_for_hotkey(&hotkey);
		ensure!(total_stake > 0 && current_stake > 0, Error::<T>::BelowStakeThreshold);

		ensure!(current_stake * 100 / total_stake >= SenateRequiredStakePercentage::<T>::get(), Error::<T>::BelowStakeThreshold);

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

				ensure!(last_stake < current_stake, Error::<T>::BelowStakeThreshold);

				return T::SenateMembers::swap_member(last, &hotkey);
			}
		}

		// Since we're calling another extrinsic, we want to propagate our errors back up the call stack.
		T::SenateMembers::add_member(&hotkey)
	}

	pub fn do_leave_senate( 
		origin: T::RuntimeOrigin,
		hotkey: &T::AccountId
	) -> DispatchResult {
		let coldkey = ensure_signed( origin )?; 
		log::info!("do_leave_senate( coldkey: {:?} hotkey:{:?} )", coldkey, hotkey );

		// Ensure that the pairing is correct.
		ensure!( Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );

		// Check all our leave requirements
		ensure!(T::SenateMembers::is_member(&hotkey), Error::<T>::NotSenateMember);

		T::TriumvirateInterface::remove_votes(&hotkey)?;
		T::SenateMembers::remove_member(&hotkey)
	}

	pub fn do_vote_senate(
		origin: T::RuntimeOrigin,
		hotkey: &T::AccountId,
		proposal: T::Hash,
		index: u32,
		approve: bool
	) -> DispatchResultWithPostInfo {
		let coldkey = ensure_signed(origin.clone())?;
		// Ensure that the pairing is correct.
		ensure!( Self::coldkey_owns_hotkey( &coldkey, &hotkey ), Error::<T>::NonAssociatedColdKey );
		ensure!(T::SenateMembers::is_member(&hotkey), Error::<T>::NotSenateMember);

		let members = T::SenateMembers::members();
		// Detects first vote of the member in the motion
		let is_account_voting_first_time = T::TriumvirateInterface::add_vote(hotkey, proposal, index, approve)?;

		// Calculate extrinsic weight
		let member_count = members.len() as u32;
		let vote_weight = Weight::from_parts(20_528_275, 4980)
			.saturating_add(Weight::from_ref_time(48_856).saturating_mul(member_count.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_proof_size(128).saturating_mul(member_count.into()));

		Ok((Some(vote_weight), if is_account_voting_first_time { Pays::No } else { Pays::Yes }).into())
	}

	pub fn do_remove_votes(
		origin: T::RuntimeOrigin,
		who: &T::AccountId
	) -> DispatchResult {
		ensure_root(origin)?;

		ensure!(!T::SenateMembers::is_member(who), Error::<T>::SenateMember);
		T::TriumvirateInterface::remove_votes(who)?;

		Ok(())
	}
}