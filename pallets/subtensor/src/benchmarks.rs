//! Subtensor pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

use crate::Pallet as Subtensor;
use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;
use frame_system::RawOrigin;
pub use pallet::*;
//use mock::{Test, new_test_ext};

benchmarks! {
  // Add individual benchmarks here
  benchmark_register {
	// Lets create a single network.
	let n: u16 = 10;
	let netuid: u16 = 1; //11 is the benchmark network.
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let seed : u32 = 1;

	let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
	let start_nonce: u64 = (39420842u64 + 100u64*netuid as u64).into();
	let hotkey: T::AccountId = account("Alice", 0, seed);
	let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number( netuid, block_number, start_nonce, &hotkey);

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into());

	let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
	let coldkey: T::AccountId = account("Test", 0, seed);
  }: register( RawOrigin::Signed( hotkey.clone() ), netuid, block_number, nonce, work, hotkey.clone(), coldkey.clone() )

  benchmark_set_weights {

	// This is a whitelisted caller who can make transaction without weights.
	let netuid: u16 = 1;
	let version_key: u64 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );

	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into() );
	Subtensor::<T>::set_max_registrations_per_block( netuid.try_into().unwrap(), 4096 );
	Subtensor::<T>::set_target_registrations_per_interval( netuid.try_into().unwrap(), 4096 );

	let mut seed : u32 = 1;
	let mut dests: Vec<u16> = vec![];
	let mut weights: Vec<u16> = vec![];
	let signer : T::AccountId = account("Alice", 0, seed);

	for id in 0..4096 as u16 {
	  let hotkey: T::AccountId = account("Alice", 0, seed);
	  let coldkey: T::AccountId = account("Test", 0, seed);
	  seed = seed +1;

		Subtensor::<T>::set_burn(netuid, 1);
		let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000 );
	  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

	  Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone())?;

	  let uid = Subtensor::<T>::get_uid_for_net_and_hotkey(netuid, &hotkey.clone()).unwrap();
	  Subtensor::<T>::set_validator_permit_for_uid(netuid, uid.clone(), true);
	  dests.push(id.clone());
	  weights.push(id.clone());
	}

  }: set_weights(RawOrigin::Signed( signer.clone() ), netuid, dests, weights, version_key)


  benchmark_become_delegate {
	// This is a whitelisted caller who can make transaction without weights.
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let version_key: u64 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let seed : u32 = 1;

	Subtensor::<T>::init_new_network(netuid, tempo);
	  Subtensor::<T>::set_burn(netuid, 1);
	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );

	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into());
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	let coldkey: T::AccountId = account("Test", 0, seed);
	let hotkey: T::AccountId = account("Alice", 0, seed);

	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  }: become_delegate(RawOrigin::Signed( coldkey.clone() ), hotkey.clone())

  benchmark_add_stake {
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let version_key: u64 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let seed : u32 = 1;

	Subtensor::<T>::init_new_network(netuid, tempo);

	Subtensor::<T>::set_burn(netuid, 1);
	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into() );

	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	let coldkey: T::AccountId = account("Test", 0, seed);
	let hotkey: T::AccountId = account("Alice", 0, seed);

	let amount: u64 = 1;
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  }: add_stake(RawOrigin::Signed( coldkey.clone() ), hotkey, amount)

  benchmark_remove_stake{
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let version_key: u64 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let seed : u32 = 1;

	// Set our total stake to 1000 TAO
	Subtensor::<T>::increase_total_stake(1_000_000_000_000);

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into() );

	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	let coldkey: T::AccountId = account("Test", 0, seed);
	let hotkey: T::AccountId = account("Alice", 0, seed);
	  Subtensor::<T>::set_burn(netuid, 1);

	let wallet_bal = Subtensor::<T>::u64_to_balance(1000000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), wallet_bal.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
	assert_ok!(Subtensor::<T>::do_become_delegate(RawOrigin::Signed(coldkey.clone()).into(), hotkey.clone(), Subtensor::<T>::get_default_take()));

	  // Stake 10% of our current total staked TAO
	  let u64_staked_amt = 100_000_000_000;
	let amount_to_be_staked = Subtensor::<T>::u64_to_balance(u64_staked_amt);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked.unwrap());

	assert_ok!( Subtensor::<T>::add_stake(RawOrigin::Signed( coldkey.clone() ).into() , hotkey.clone(), u64_staked_amt));

	let amount_unstaked: u64 = u64_staked_amt - 1;
  }: remove_stake(RawOrigin::Signed( coldkey.clone() ), hotkey.clone(), amount_unstaked)

  benchmark_serve_axon{
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	let version: u32 =  2;
	let ip: u128 = 1676056785;
	let port: u16 = 128;
	let ip_type: u8 = 4;
	let protocol: u8 = 0;
	let placeholder1: u8 = 0;
	let placeholder2: u8 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	Subtensor::<T>::set_burn(netuid, 1);
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000 );
	Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amoun_to_be_staked.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), netuid, caller.clone()));

	Subtensor::<T>::set_serving_rate_limit(netuid, 0);

  }: serve_axon(RawOrigin::Signed( caller.clone() ), netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2)

  benchmark_serve_prometheus {
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	let version: u32 = 2;
	let ip: u128 = 1676056785;
	let port: u16 = 128;
	let ip_type: u8 = 4;

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	Subtensor::<T>::set_burn(netuid, 1);
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000 );
	Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amoun_to_be_staked.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), netuid, caller.clone()));
	Subtensor::<T>::set_serving_rate_limit(netuid, 0);

  }: serve_prometheus(RawOrigin::Signed( caller.clone() ), netuid, version, ip, port, ip_type)

  /*
  benchmark_sudo_register {
	let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
	let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
	let netuid: u16 = 1;
	let tempo: u16 = 0;
	let modality: u16 = 0;
	let stake: u64 = 10;
	let balance: u64 = 1000000000;

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	let seed : u32 = 1;
	let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
	let hotkey: T::AccountId = account("Alice", 0, seed);
	let coldkey: T::AccountId = account("Test", 0, seed);

	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( balance );
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

  }: sudo_register(RawOrigin::<AccountIdOf<T>>::Root, netuid, hotkey, coldkey, stake, balance)

  benchmark_sudo_set_default_take {
	let default_take: u16 = 100;

  }: sudo_set_default_take(RawOrigin::<AccountIdOf<T>>::Root, default_take)

  benchmark_sudo_set_serving_rate_limit {
	let serving_rate_limit: u64 = 100;
	let netuid: u16 = 1;

  }: sudo_set_serving_rate_limit(RawOrigin::<AccountIdOf<T>>::Root, netuid, serving_rate_limit)

  benchmark_sudo_set_max_difficulty {
	let netuid: u16 = 1;
	let max_difficulty: u64 = 100000;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_difficulty)

  benchmark_sudo_set_min_difficulty {
	let netuid: u16 = 1;
	let min_difficulty: u64 = 1000;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_min_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, min_difficulty)

  benchmark_sudo_set_weights_set_rate_limit {
	let netuid: u16 = 1;
	let weights_set_rate_limit: u64 = 3;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_weights_set_rate_limit(RawOrigin::<AccountIdOf<T>>::Root, netuid, weights_set_rate_limit)

  benchmark_sudo_set_weights_version_key {
	let netuid: u16 = 1;
	let weights_version_key: u64 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_weights_version_key(RawOrigin::<AccountIdOf<T>>::Root, netuid, weights_version_key)

  benchmark_sudo_set_bonds_moving_average {
	let netuid: u16 = 1;
	let bonds_moving_average: u64 = 100;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_bonds_moving_average(RawOrigin::<AccountIdOf<T>>::Root, netuid, bonds_moving_average)

  benchmark_sudo_set_max_allowed_validators {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let max_allowed_validators: u16 = 10;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_allowed_validators(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_allowed_validators)

  benchmark_sudo_set_difficulty {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let difficulty: u64 = 1200000;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, difficulty)

  benchmark_sudo_set_adjustment_interval {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let adjustment_interval: u16 = 12;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_adjustment_interval(RawOrigin::<AccountIdOf<T>>::Root, netuid, adjustment_interval)

  benchmark_sudo_set_target_registrations_per_interval {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let target_registrations_per_interval: u16 = 300;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_target_registrations_per_interval(RawOrigin::<AccountIdOf<T>>::Root, netuid, target_registrations_per_interval)

  benchmark_sudo_set_activity_cutoff {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let activity_cutoff: u16 = 300;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_activity_cutoff(RawOrigin::<AccountIdOf<T>>::Root, netuid, activity_cutoff)

  benchmark_sudo_set_rho {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let rho: u16 = 300;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_rho(RawOrigin::<AccountIdOf<T>>::Root, netuid, rho)

  benchmark_sudo_set_kappa {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let kappa: u16 = 3;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_kappa(RawOrigin::<AccountIdOf<T>>::Root, netuid, kappa)

  benchmark_sudo_set_max_allowed_uids {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let max_allowed_uids: u16 = 4097;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_allowed_uids(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_allowed_uids)

  benchmark_sudo_set_min_allowed_weights {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let min_allowed_weights: u16 = 10;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_min_allowed_weights(RawOrigin::<AccountIdOf<T>>::Root, netuid, min_allowed_weights)

  benchmark_sudo_set_validator_prune_len {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let validator_prune_len: u64 = 10;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_validator_prune_len(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_prune_len)

  benchmark_sudo_set_scaling_law_power {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let scaling_law_power: u16 = 100;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_scaling_law_power(RawOrigin::<AccountIdOf<T>>::Root, netuid, scaling_law_power)

  benchmark_sudo_set_immunity_period {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let immunity_period: u16 = 100;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_immunity_period(RawOrigin::<AccountIdOf<T>>::Root, netuid, immunity_period)

  benchmark_sudo_set_max_weight_limit {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let max_weight_limit: u16 = 100;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_weight_limit(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_weight_limit)

  benchmark_sudo_set_max_registrations_per_block {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let max_registrations_per_block: u16 = 100;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_registrations_per_block(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_registrations_per_block)
  */
  benchmark_burned_register {
	let netuid: u16 = 1;
	let seed : u32 = 1;
	let hotkey: T::AccountId = account("Alice", 0, seed);
	let coldkey: T::AccountId = account("Test", 0, seed);
	let modality: u16 = 0;
	let tempo: u16 = 1;

	Subtensor::<T>::init_new_network(netuid, tempo);
	Subtensor::<T>::set_burn(netuid, 1);

	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 1000000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

  }: burned_register(RawOrigin::Signed( coldkey.clone() ), netuid, hotkey)

  /*
  benchmark_sudo_set_max_burn {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let max_burn: u64 = 10;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_max_burn(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_burn)

  benchmark_sudo_set_min_burn {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;
	let min_burn: u64 = 10;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_min_burn(RawOrigin::<AccountIdOf<T>>::Root, netuid, min_burn)

  benchmark_sudo_set_registration_allowed {
	let netuid: u16 = 1;
	let tempo: u16 = 1;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);
  }: sudo_set_registration_allowed(RawOrigin::<AccountIdOf<T>>::Root, netuid, true)

  benchmark_sudo_set_tempo {
	let netuid: u16 = 1;
	let tempo_default: u16 = 1;
	let tempo: u16 = 15;
	let modality: u16 = 0;

	Subtensor::<T>::init_new_network(netuid, tempo);

  }: sudo_set_tempo(RawOrigin::<AccountIdOf<T>>::Root, netuid, tempo)
  	*/

  benchmark_root_register {
	let netuid: u16 = 1;
	let version_key: u64 = 1;
	let tempo: u16 = 1;
	let seed : u32 = 1;

	Subtensor::<T>::init_new_network(netuid, tempo);

	Subtensor::<T>::set_burn(netuid, 1);
	Subtensor::<T>::set_network_registration_allowed( netuid.try_into().unwrap(), true.into());

	Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
	assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

	let coldkey: T::AccountId = account("Test", 0, seed);
	let hotkey: T::AccountId = account("Alice", 0, seed);

	let amount: u64 = 1;
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance( 100_000_000_000_000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

	assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  }: root_register(RawOrigin::Signed(coldkey), hotkey)

  benchmark_register_network {
	let seed : u32 = 1;

	let coldkey: T::AccountId = account("Test", 0, seed);

	Subtensor::<T>::set_network_rate_limit(1);

	let amount: u64 = 1;
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance(100_000_000_000_000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());
  }: register_network(RawOrigin::Signed(coldkey))

  benchmark_dissolve_network {
	let seed : u32 = 1;

	let coldkey: T::AccountId = account("Test", 0, seed);

	Subtensor::<T>::set_network_rate_limit(0);

	let amount: u64 = 1;
	let amoun_to_be_staked = Subtensor::<T>::u64_to_balance(100_000_000_000_000);
	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());
	assert_ok!(Subtensor::<T>::register_network(RawOrigin::Signed(coldkey.clone()).into()));
  }: dissolve_network(RawOrigin::Signed(coldkey), 1)

  swap_hotkey {
	let seed: u32 = 1;
	let coldkey: T::AccountId = account("Alice", 0, seed);
	let old_hotkey: T::AccountId = account("Bob", 0, seed);
	let new_hotkey: T::AccountId = account("Charlie", 0, seed);

	let netuid = 1u16;
	Subtensor::<T>::init_new_network(netuid, 100);
	Subtensor::<T>::set_min_burn(netuid, 1);
	Subtensor::<T>::set_max_burn(netuid, 1);
	Subtensor::<T>::set_target_registrations_per_interval(netuid, 256);
	Subtensor::<T>::set_max_registrations_per_block(netuid, 256);

	Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(10_000_000_000).unwrap());
	assert_ok!(Subtensor::<T>::burned_register(RawOrigin::Signed(coldkey.clone()).into(), netuid, old_hotkey.clone()));
	assert_ok!(Subtensor::<T>::become_delegate(RawOrigin::Signed(coldkey.clone()).into(), old_hotkey.clone()));

	let max_uids = Subtensor::<T>::get_max_allowed_uids(netuid) as u32;
	for i in 0..max_uids - 1 {
		let coldkey: T::AccountId = account("Axon", 0, i);
		let hotkey: T::AccountId = account("Hotkey", 0, i);

		Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(10_000_000_000).unwrap());
		assert_ok!(Subtensor::<T>::burned_register(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey));
		assert_ok!(Subtensor::<T>::add_stake(RawOrigin::Signed(coldkey).into(), old_hotkey.clone(), 1_000_000_000));
	}
  }: _(RawOrigin::Signed(coldkey), old_hotkey, new_hotkey)
}