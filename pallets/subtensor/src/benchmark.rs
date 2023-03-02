//! Paratensor pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

use crate::*;
use crate::Pallet as Paratensor;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
pub use pallet::*;
use frame_support::assert_ok;
//use mock::{Test, new_test_ext};

benchmarks! {
   
  // Add individual benchmarks here
  benchmark_epoch_without_weights { 

    // This is a whitelisted caller who can make transaction without weights.
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

    // Lets create a single network.
    let n: u16 = 4096;
    let netuid: u16 = 11; //11 is the benchmark network.
    let tempo: u16 = 1;
    let modality: u16 = 0;
    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, n ); 

    // Lets fill the network with 100 UIDS and no weights.
    let mut seed : u32 = 1;
    for uid in 0..n as u16 {
        let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
        let hotkey: T::AccountId = account("Alice", 0, seed);
        Paratensor::<T>::append_neuron( netuid, &hotkey, block_number );
        seed = seed + 1;
    }

  }: _( RawOrigin::Signed( caller.clone() ) )

  // Add individual benchmarks here
  /*benchmark_drain_emission { 

    // This is a whitelisted caller who can make transaction without weights.
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

    // Lets create a single network.
    let n: u16 = 4096;
    let netuid: u16 = 11; //11 is the benchmark network.
    let tempo: u16 = 1;
    let modality: u16 = 0;
    Paratensor::<T>::do_add_network( caller_origin.clone(), netuid.try_into().unwrap(), tempo.into(), modality.into());
    Paratensor::<T>::set_max_allowed_uids( netuid, n ); 
    Paratensor::<T>::set_tempo( netuid, tempo );

    // Lets fill the network with 100 UIDS and no weights.
    let mut seed : u32 = 1;
    let mut emission: Vec<(T::AccountId, u64)> = vec![];
    for uid in 0..n as u16 {
        let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
        let hotkey: T::AccountId = account("Alice", 0, SEED);
        Paratensor::<T>::append_neuron( netuid, &hotkey, block_number );
        SEED = SEED + 1;
        emission.push( ( hotkey, 1 ) );
    }
    Paratensor::<T>::sink_emission( netuid, emission );
 
  }: _( RawOrigin::Signed( caller.clone() ) )  */


  benchmark_register { 

    // This is a whitelisted caller who can make transaction without weights.
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 

    // Lets create a single network.
    let n: u16 = 10;
    let netuid: u16 = 1; //11 is the benchmark network.
    let tempo: u16 = 1;
    let modality: u16 = 0;

    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let start_nonce: u64 = (39420842u64 + 100u64*netuid as u64).into();
    let (nonce, work): (u64, Vec<u8>) = Paratensor::<T>::create_work_for_block_number( netuid, block_number, start_nonce );

    assert_ok!(Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    
    let mut seed : u32 = 1;
    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let hotkey: T::AccountId = account("Alice", 0, seed);
    let coldkey: T::AccountId = account("Test", 0, seed);
        
  }: register( RawOrigin::Signed( caller.clone() ), netuid, block_number, nonce, work, hotkey, coldkey )

 /* benchmark_epoch_with_weights { 
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    Paratensor::<T>::create_network_with_weights(
      caller_origin.clone(), 
      11u16.into(), // netuid
      4096u16.into(), // n
      1000u16.into(), // tempo
      100u16.into(), // n_vals
      1000u16.into() // nweights
    );
  }: _( RawOrigin::Signed( caller.clone() ) ) */

  benchmark_set_weights {
    
    // This is a whitelisted caller who can make transaction without weights.
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
   
    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, 4096 ); 

   assert_ok!(Paratensor::<T>::do_sudo_set_max_registrations_per_block(RawOrigin::Root.into(), netuid.try_into().unwrap(), 4096 ));
    
    let mut seed : u32 = 1; 
    let mut dests: Vec<u16> = vec![];
    let mut weights: Vec<u16> = vec![];
    let signer : T::AccountId = account("Alice", 0, seed);

    for id in 0..4096 as u16 {
      let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
      let start_nonce: u64 = (39420842u64 + 100u64*id as u64).into();
      let (nonce, work): (u64, Vec<u8>) = Paratensor::<T>::create_work_for_block_number( id, block_number, start_nonce );
      
        let hotkey: T::AccountId = account("Alice", 0, seed);
        let coldkey: T::AccountId = account("Test", 0, seed);
        seed = seed +1;
      
      
      let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
      
      assert_ok!( Paratensor::<T>::do_registration(caller_origin.clone(), netuid.try_into().unwrap(), block_number, nonce, work, hotkey.clone(), coldkey )); 

      let uid = Paratensor::<T>::get_uid_for_net_and_hotkey(netuid, &hotkey.clone()).unwrap();
      Paratensor::<T>::set_validator_permit_for_uid(netuid, uid.clone(), true);
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

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, 4096 ); 
    assert_eq!(Paratensor::<T>::get_max_allowed_uids(netuid), 4096);

    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let start_nonce: u64 = (39420842u64 + 100u64*netuid as u64).into();
    let (nonce, work): (u64, Vec<u8>) = Paratensor::<T>::create_work_for_block_number( netuid, block_number, start_nonce );
    let mut seed : u32 = 1;
    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    assert_ok!( Paratensor::<T>::do_registration(caller_origin.clone(), netuid.try_into().unwrap(), block_number, nonce, work, hotkey.clone(), coldkey.clone() ));

  }: become_delegate(RawOrigin::Signed( coldkey.clone() ), hotkey.clone())


  benchmark_add_stake {
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, 4096 ); 
    assert_eq!(Paratensor::<T>::get_max_allowed_uids(netuid), 4096);

    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let start_nonce: u64 = (39420842u64 + 100u64*netuid as u64).into();
    let (nonce, work): (u64, Vec<u8>) = Paratensor::<T>::create_work_for_block_number( netuid, block_number, start_nonce );
    let mut seed : u32 = 1;
    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    assert_ok!( Paratensor::<T>::do_registration(caller_origin.clone(), netuid.try_into().unwrap(), block_number, nonce, work, hotkey.clone(), coldkey.clone() ));

    let amount: u64 = 1;
    let amoun_to_be_staked = Paratensor::<T>::u64_to_balance( 1000000000);

    Paratensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

  }: add_stake(RawOrigin::Signed( coldkey.clone() ), hotkey, amount)

  benchmark_remove_stake{
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, 4096 ); 
    assert_eq!(Paratensor::<T>::get_max_allowed_uids(netuid), 4096);

    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let start_nonce: u64 = (39420842u64 + 100u64*netuid as u64).into();
    let (nonce, work): (u64, Vec<u8>) = Paratensor::<T>::create_work_for_block_number( netuid, block_number, start_nonce );
    let mut seed : u32 = 1;
    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    assert_ok!( Paratensor::<T>::do_registration(caller_origin.clone(), netuid.try_into().unwrap(), block_number, nonce, work, hotkey.clone(), coldkey.clone() ));

    let amoun_to_be_staked = Paratensor::<T>::u64_to_balance( 1000000000);
    Paratensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

    assert_ok!( Paratensor::<T>::add_stake(RawOrigin::Signed( coldkey.clone() ).into() , hotkey.clone(), 1000));

    let amount_unstaked: u64 = 1;

  }: remove_stake(RawOrigin::Signed( coldkey.clone() ), hotkey.clone(), amount_unstaked)

  benchmark_serve_axon{
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let version: u32 =  2;
    let ip: u128 = 1676056785;
    let port: u16 = 128;
    let ip_type: u8 = 4;
    let protocol: u8 = 0;
    let placeholder1: u8 = 0;
    let placeholder2: u8 = 0;

    Paratensor::<T>::set_serving_rate_limit(0);

  }: serve_axon(RawOrigin::Signed( caller.clone() ), version, ip, port, ip_type, protocol, placeholder1, placeholder2)

  benchmark_serve_prometheus {
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let version: u32 = 2;
    let ip: u128 = 1676056785;
    let port: u16 = 128;
    let ip_type: u8 = 4;

    Paratensor::<T>::set_serving_rate_limit(0);

  }: serve_prometheus(RawOrigin::Signed( caller.clone() ), version, ip, port, ip_type)

  benchmark_sudo_register {
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>(); 
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let tempo: u16 = 0;
    let modality: u16 = 0;
    let stake: u64 = 10;
    let balance: u64 = 1000000000;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    Paratensor::<T>::set_max_allowed_uids( netuid, 4096 ); 
    assert_eq!(Paratensor::<T>::get_max_allowed_uids(netuid), 4096);

    let mut seed : u32 = 1;
    let block_number: u64 = Paratensor::<T>::get_current_block_as_u64();
    let hotkey: T::AccountId = account("Alice", 0, seed);
    let coldkey: T::AccountId = account("Test", 0, seed);

    let amoun_to_be_staked = Paratensor::<T>::u64_to_balance( balance );
    Paratensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amoun_to_be_staked.unwrap());

  }: sudo_register(RawOrigin::<AccountIdOf<T>>::Root, netuid, hotkey, coldkey, stake, balance)

  benchmark_sudo_add_network {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;

  }: sudo_add_network(RawOrigin::<AccountIdOf<T>>::Root, netuid, tempo, modality)

  benchmark_sudo_remove_network {
    let netuid: u16 = 1;
    let tempo: u16 = 0;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_remove_network(RawOrigin::<AccountIdOf<T>>::Root, netuid)

  benchmark_sudo_set_emission_values{
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let netuids: Vec<u16> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let emission: Vec<u64> = vec![100000000, 100000000, 100000000, 100000000, 100000000, 100000000, 100000000, 100000000, 100000000, 100000000];

    for netuid in 0..10 as u16 {
      assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));
    }

  }: sudo_set_emission_values(RawOrigin::<AccountIdOf<T>>::Root, netuids, emission)

  benchmark_sudo_add_network_connection_requirement {
    let netuid_a: u16 = 1; 
    let netuid_b: u16 = 2; 
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let requirement: u16 = 1;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid_a.try_into().unwrap(), tempo.into(), modality.into()));
    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid_b.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_add_network_connection_requirement(RawOrigin::<AccountIdOf<T>>::Root, netuid_a, netuid_b, requirement)

  benchmark_sudo_remove_network_connection_requirement {
    let netuid_a: u16 = 1; 
    let netuid_b: u16 = 2; 
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let requirement: u16 = 1;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid_a.try_into().unwrap(), tempo.into(), modality.into()));
    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid_b.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_remove_network_connection_requirement(RawOrigin::<AccountIdOf<T>>::Root, netuid_a, netuid_b)

  benchmark_sudo_set_default_take {
    let default_take: u16 = 100; 

  }: sudo_set_default_take(RawOrigin::<AccountIdOf<T>>::Root, default_take)

  benchmark_sudo_set_serving_rate_limit {
    let serving_rate_limit: u64 = 100;

  }: sudo_set_serving_rate_limit(RawOrigin::<AccountIdOf<T>>::Root, serving_rate_limit)

  benchmark_sudo_set_max_difficulty {
    let netuid: u16 = 1;
    let max_difficulty: u64 = 100000;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_max_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_difficulty)

  benchmark_sudo_set_min_difficulty {
    let netuid: u16 = 1;
    let min_difficulty: u64 = 1000;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_min_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, min_difficulty)

  benchmark_sudo_set_weights_set_rate_limit {
    let netuid: u16 = 1; 
    let weights_set_rate_limit: u64 = 3;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_weights_set_rate_limit(RawOrigin::<AccountIdOf<T>>::Root, netuid, weights_set_rate_limit)

  benchmark_sudo_set_weights_version_key {
    let netuid: u16 = 1; 
    let weights_version_key: u64 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_weights_version_key(RawOrigin::<AccountIdOf<T>>::Root, netuid, weights_version_key)

  benchmark_sudo_set_bonds_moving_average {
    let netuid: u16 = 1;
    let bonds_moving_average: u64 = 100;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_bonds_moving_average(RawOrigin::<AccountIdOf<T>>::Root, netuid, bonds_moving_average)

  benchmark_sudo_set_max_allowed_validators {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let max_allowed_validators: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_max_allowed_validators(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_allowed_validators)

  benchmark_sudo_set_difficulty {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let difficulty: u64 = 1200000;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_difficulty(RawOrigin::<AccountIdOf<T>>::Root, netuid, difficulty)

  benchmark_sudo_set_adjustment_interval {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let adjustment_interval: u16 = 12;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_adjustment_interval(RawOrigin::<AccountIdOf<T>>::Root, netuid, adjustment_interval)

  benchmark_sudo_set_target_registrations_per_interval {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let target_registrations_per_interval: u16 = 300;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_target_registrations_per_interval(RawOrigin::<AccountIdOf<T>>::Root, netuid, target_registrations_per_interval)

  benchmark_sudo_set_activity_cutoff {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let activity_cutoff: u16 = 300;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_activity_cutoff(RawOrigin::<AccountIdOf<T>>::Root, netuid, activity_cutoff)

  benchmark_sudo_set_rho {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let rho: u16 = 300;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_rho(RawOrigin::<AccountIdOf<T>>::Root, netuid, rho)

  benchmark_sudo_set_kappa {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let kappa: u16 = 3;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_kappa(RawOrigin::<AccountIdOf<T>>::Root, netuid, kappa)

  benchmark_sudo_set_weight_cuts {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let weight_cuts: u16 = 3;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_weight_cuts(RawOrigin::<AccountIdOf<T>>::Root, netuid, weight_cuts)

  benchmark_sudo_set_max_allowed_uids {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let max_allowed_uids: u16 = 4096;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_max_allowed_uids(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_allowed_uids)

  benchmark_sudo_set_min_allowed_weights {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let min_allowed_weights: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_min_allowed_weights(RawOrigin::<AccountIdOf<T>>::Root, netuid, min_allowed_weights)

  benchmark_sudo_set_validator_batch_size{
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_batch_size: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_batch_size(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_batch_size)

  benchmark_sudo_set_validator_sequence_length{
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_sequence_length: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_sequence_length(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_sequence_length)

  benchmark_sudo_set_validator_epochs_per_reset {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_epochs_per_reset: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_epochs_per_reset(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_epochs_per_reset)

  benchmark_sudo_set_validator_exclude_quantile {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_exclude_quantile: u16 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_exclude_quantile(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_exclude_quantile)

  benchmark_sudo_set_validator_prune_len {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_prune_len: u64 = 10;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_prune_len(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_prune_len)

  benchmark_sudo_set_validator_logits_divergence {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let validator_logits_divergence: u64 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_validator_logits_divergence(RawOrigin::<AccountIdOf<T>>::Root, netuid, validator_logits_divergence)

  benchmark_sudo_set_scaling_law_power {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let scaling_law_power: u16 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_scaling_law_power(RawOrigin::<AccountIdOf<T>>::Root, netuid, scaling_law_power)

  benchmark_sudo_set_synergy_scaling_law_power {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let synergy_scaling_law_power: u16 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_synergy_scaling_law_power(RawOrigin::<AccountIdOf<T>>::Root, netuid, synergy_scaling_law_power)

  benchmark_sudo_set_immunity_period {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let immunity_period: u16 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_immunity_period(RawOrigin::<AccountIdOf<T>>::Root, netuid, immunity_period)

  benchmark_sudo_set_max_weight_limit {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let max_weight_limit: u16 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_max_weight_limit(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_weight_limit)

  benchmark_sudo_set_max_registrations_per_block {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;
    let max_registrations_per_block: u16 = 100;

    assert_ok!( Paratensor::<T>::do_add_network( RawOrigin::Root.into(), netuid.try_into().unwrap(), tempo.into(), modality.into()));

  }: sudo_set_max_registrations_per_block(RawOrigin::<AccountIdOf<T>>::Root, netuid, max_registrations_per_block)
}

