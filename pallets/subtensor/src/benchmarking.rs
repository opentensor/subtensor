//! Subtensor pallet benchmarking.
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as Subtensor;
use super::*;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::vec;

#[benchmarks]
mod benchmarks {
  use super::*;

  #[benchmark]
  fn register() {
    let netuid: u16 = 1; //11 is the benchmark network.
    let tempo: u16 = 1;
    let hotkey: T::AccountId = account("Alice", 0, 1);
    let coldkey: T::AccountId = account("Test", 0, 2);

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_network_registration_allowed(netuid, true);
    Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

    let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
        netuid,
        block_number,
        3,
        &hotkey,
    );

    #[extrinsic_call]
    _( RawOrigin::Signed( hotkey.clone() ), netuid, block_number, nonce, work, hotkey.clone(), coldkey.clone() );
  }

  #[benchmark]
  fn set_weights() {

    // This is a whitelisted caller who can make transaction without weights.
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );

    Subtensor::<T>::set_network_registration_allowed( netuid, true );
    Subtensor::<T>::set_max_registrations_per_block( netuid, 4096 );
    Subtensor::<T>::set_target_registrations_per_interval( netuid, 4096 );

    let mut seed : u32 = 1;
    let mut dests: Vec<u16> = vec![];
    let mut weights: Vec<u16> = vec![];
    let signer : T::AccountId = account("Alice", 0, seed);

    for id in 0..4096_u16 {
      let hotkey: T::AccountId = account("Alice", 0, seed);
      let coldkey: T::AccountId = account("Test", 0, seed);
      seed += 1;

      Subtensor::<T>::set_burn(netuid, 1);
      let amount_to_be_staked = 1000000u32.into();
      Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

      Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()).unwrap();

      let uid = Subtensor::<T>::get_uid_for_net_and_hotkey(netuid, &hotkey.clone()).unwrap();
      Subtensor::<T>::set_validator_permit_for_uid(netuid, uid, true);
      dests.push(id);
      weights.push(id);
    }

    #[extrinsic_call]
    _(RawOrigin::Signed( signer.clone() ), netuid, dests, weights, version_key);
  }

  #[benchmark]
  fn become_delegate() {
    // This is a whitelisted caller who can make transaction without weights.
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);
      Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );

    Subtensor::<T>::set_network_registration_allowed( netuid, true);
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount_to_be_staked = 1000000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));

    #[extrinsic_call]
    _(RawOrigin::Signed( coldkey.clone() ), hotkey.clone());
  }

  #[benchmark]
  fn add_stake() {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::set_target_stakes_per_interval(100);

    Subtensor::<T>::init_new_network(netuid, tempo);

    Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_network_registration_allowed( netuid, true );

    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount: u64 = 1;
    let amount_to_be_staked = 1000000000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));

    #[extrinsic_call]
    _(RawOrigin::Signed( coldkey.clone() ), hotkey, amount);
  }

  #[benchmark]
  fn remove_stake() {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::set_target_stakes_per_interval(100);

    // Set our total stake to 1000 TAO
    Subtensor::<T>::increase_total_stake(1_000_000_000_000);

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_network_registration_allowed( netuid, true );

    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);
      Subtensor::<T>::set_burn(netuid, 1);

    let wallet_bal = 1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), wallet_bal);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
    assert_ok!(Subtensor::<T>::do_become_delegate(RawOrigin::Signed(coldkey.clone()).into(), hotkey.clone(), Subtensor::<T>::get_default_delegate_take()));

      // Stake 10% of our current total staked TAO
      let u64_staked_amt = 100_000_000_000;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), u64_staked_amt);

    assert_ok!( Subtensor::<T>::add_stake(RawOrigin::Signed( coldkey.clone() ).into() , hotkey.clone(), u64_staked_amt));

    let amount_unstaked: u64 = u64_staked_amt - 1;

    #[extrinsic_call]
    _(RawOrigin::Signed( coldkey.clone() ), hotkey.clone(), amount_unstaked);
  }

  #[benchmark]
  fn serve_axon() {
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let tempo: u16 = 1;

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
    let amount_to_be_staked = 1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), netuid, caller.clone()));

    Subtensor::<T>::set_serving_rate_limit(netuid, 0);

    #[extrinsic_call]
    _(RawOrigin::Signed( caller.clone() ), netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2);
  }

  #[benchmark]
  fn serve_prometheus() {
    let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
    let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    let netuid: u16 = 1;
    let tempo: u16 = 1;

    let version: u32 = 2;
    let ip: u128 = 1676056785;
    let port: u16 = 128;
    let ip_type: u8 = 4;

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    Subtensor::<T>::set_burn(netuid, 1);
    let amount_to_be_staked = 1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), netuid, caller.clone()));
    Subtensor::<T>::set_serving_rate_limit(netuid, 0);

    #[extrinsic_call]
    _(RawOrigin::Signed( caller.clone() ), netuid, version, ip, port, ip_type);
  }

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

    let amount_to_be_staked = balance.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

  }: sudo_register(RawOrigin::<AccountIdOf<T>>::Root, netuid, hotkey, coldkey, stake, balance)
  */
  #[benchmark]
  fn burned_register() {
    let netuid: u16 = 1;
    let seed : u32 = 1;
    let hotkey: T::AccountId = account("Alice", 0, seed);
    let coldkey: T::AccountId = account("Test", 0, seed);
    let tempo: u16 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_burn(netuid, 1);

    let amount_to_be_staked =  1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    #[extrinsic_call]
    _(RawOrigin::Signed( coldkey.clone() ), netuid, hotkey);
  }

  #[benchmark]
  fn root_register() {
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);

    Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_network_registration_allowed( netuid, true);

    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount_to_be_staked =  100_000_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));

    #[extrinsic_call]
    _(RawOrigin::Signed(coldkey), hotkey);
  }

  #[benchmark]
  fn register_network() {
    let seed : u32 = 1;

    let coldkey: T::AccountId = account("Test", 0, seed);

    Subtensor::<T>::set_network_rate_limit(1);

    let amount_to_be_staked = 100_000_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    #[extrinsic_call]
    _(RawOrigin::Signed(coldkey));
  }

  #[benchmark]
  fn dissolve_network() {
    let seed : u32 = 1;

    let coldkey: T::AccountId = account("Test", 0, seed);

    Subtensor::<T>::set_network_rate_limit(0);

    let amount_to_be_staked = 100_000_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);
    assert_ok!(Subtensor::<T>::register_network(RawOrigin::Signed(coldkey.clone()).into()));

    #[extrinsic_call]
    _(RawOrigin::Root, coldkey.clone(), 1);
  }

  // swap_hotkey {
  //   let seed: u32 = 1;
  //   let coldkey: T::AccountId = account("Alice", 0, seed);
  //   let old_hotkey: T::AccountId = account("Bob", 0, seed);
  //   let new_hotkey: T::AccountId = account("Charlie", 0, seed);

  //   let netuid = 1u16;
  //   Subtensor::<T>::init_new_network(netuid, 100);
  //   Subtensor::<T>::set_min_burn(netuid, 1);
  //   Subtensor::<T>::set_max_burn(netuid, 1);
  //   Subtensor::<T>::set_target_registrations_per_interval(netuid, 256);
  //   Subtensor::<T>::set_max_registrations_per_block(netuid, 256);

  //   Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), 10_000_000_000u64);
  //   assert_ok!(Subtensor::<T>::burned_register(RawOrigin::Signed(coldkey.clone()).into(), netuid, old_hotkey.clone()));
  //   assert_ok!(Subtensor::<T>::become_delegate(RawOrigin::Signed(coldkey.clone()).into(), old_hotkey.clone()));

  //   let max_uids = Subtensor::<T>::get_max_allowed_uids(netuid) as u32;
  //   for i in 0..max_uids - 1 {
  //       let coldkey: T::AccountId = account("Axon", 0, i);
  //       let hotkey: T::AccountId = account("Hotkey", 0, i);

  //       Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), 10_000_000_000u64);
  //       assert_ok!(Subtensor::<T>::burned_register(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey));
  //       assert_ok!(Subtensor::<T>::add_stake(RawOrigin::Signed(coldkey).into(), old_hotkey.clone(), 1_000_000_000));
  //   }
  // }: _(RawOrigin::Signed(coldkey), old_hotkey, new_hotkey)

  #[benchmark]
  fn commit_weights() {
    let tempo: u16 = 1;
    let netuid: u16 = 1;
    let version_key: u64 = 0;
    let uids: Vec<u16> = vec![0];
    let weight_values: Vec<u16> = vec![10];
    let hotkey: T::AccountId = account("hot", 0, 1);
    let coldkey: T::AccountId = account("cold", 0, 2);
    let start_nonce = 300000;

    let commit_hash: H256 = BlakeTwo256::hash_of(&(
        hotkey.clone(),
        netuid,
        uids.clone(),
        weight_values.clone(),
        version_key,
    ));

    Subtensor::<T>::init_new_network(netuid, tempo);

    let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
        netuid,
        block_number,
        start_nonce,
        &hotkey,
    );
    Subtensor::<T>::register(
      <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())),
        netuid,
        block_number,
        nonce,
        work,
        hotkey.clone(),
        coldkey,
    )
    .unwrap();
    Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);

    #[extrinsic_call]
    _(RawOrigin::Signed(hotkey.clone()), netuid, commit_hash);
  }

  #[benchmark]
  fn reveal_weights() {
    let tempo: u16 = 0;
    let netuid: u16 = 1;
    let version_key: u64 = 0;
    let uids: Vec<u16> = vec![0];
    let weight_values: Vec<u16> = vec![10];
    let salt: Vec<u16> = vec![8];
    let hotkey: T::AccountId = account("hot", 0, 1);
    let coldkey: T::AccountId = account("cold", 1, 2);

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_network_registration_allowed(netuid, true);
    Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

    let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
        netuid,
        block_number,
        3,
        &hotkey,
    );

    let _ = Subtensor::<T>::register(
      <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())),
        netuid,
        block_number,
        nonce,
        work.clone(),
        hotkey.clone(),
        coldkey.clone(),
    );

    Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);

    let commit_hash: H256 = BlakeTwo256::hash_of(&(
      hotkey.clone(),
      netuid,
      uids.clone(),
      weight_values.clone(),
      salt.clone(),
      version_key,
    ));
    let _ = Subtensor::<T>::commit_weights(<T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())), netuid, commit_hash);

    #[extrinsic_call]
    _(RawOrigin::Signed(hotkey.clone()), netuid, uids, weight_values, salt, version_key);
  }

  #[benchmark]
  fn schedule_swap_coldkey() {
    let old_coldkey: T::AccountId = account("old_cold", 0, 1);
    let new_coldkey: T::AccountId = account("new_cold", 1, 2);

    #[extrinsic_call]
    _(RawOrigin::Signed(old_coldkey.clone()), new_coldkey.clone());
  }

  #[benchmark]
  fn schedule_dissolve_network() {
    let coldkey: T::AccountId = account("coldkey", 0, 1);
    let netuid = 1;

    #[extrinsic_call]
    _(RawOrigin::Signed(coldkey.clone()), netuid);
  }

  #[benchmark]
  fn sudo_set_tx_childkey_take_rate_limit() {
    // We don't need to set up any initial state for this benchmark
    // as it's a simple setter function that only requires root origin
    let new_rate_limit: u64 = 100;

    #[extrinsic_call]
    _(RawOrigin::Root, new_rate_limit);
  }

  #[benchmark]
  fn set_childkey_take() {
    // Setup
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed: u32 = 1;
    let coldkey: T::AccountId = account("Cold", 0, seed);
    let hotkey: T::AccountId = account("Hot", 0, seed);
    let take: u16 = 1000; // 10% in basis points

    // Initialize the network
    Subtensor::<T>::init_new_network(netuid, tempo);

    // Register the hotkey
    Subtensor::<T>::set_burn(netuid, 1);
    let amount_to_be_staked = 1_000_000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);
    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));

    #[extrinsic_call]
    _(RawOrigin::Signed(coldkey), hotkey, netuid, take);
  }

  #[benchmark]
  fn swap_coldkey() {
    // Set up initial state
    let old_coldkey: T::AccountId = account("old_coldkey", 0, 0);
    let new_coldkey: T::AccountId = account("new_coldkey", 0, 0);
    let hotkey1: T::AccountId = account("hotkey1", 0, 0);
    let netuid = 1u16;
    let stake_amount1 = 1000u64;
    let stake_amount2 = 2000u64;
    let swap_cost = Subtensor::<T>::get_key_swap_cost();
    let free_balance_old = 12345u64 + swap_cost;
    let tempo: u16 = 1;

    // Setup initial state
    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_network_registration_allowed(netuid, true);
    Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

    let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
        netuid,
        block_number,
        3,
        &hotkey1,
    );

    let _ = Subtensor::<T>::register(
      <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(old_coldkey.clone())),
        netuid,
        block_number,
        nonce,
        work.clone(),
        hotkey1.clone(),
        old_coldkey.clone(),
    );

    // Add balance to old coldkey
    Subtensor::<T>::add_balance_to_coldkey_account(
        &old_coldkey,
        stake_amount1 + stake_amount2 + free_balance_old,
    );

    // Insert an Identity
    let name: Vec<u8> = b"The fourth Coolest Identity".to_vec();
    let identity: ChainIdentity = ChainIdentity {
        name: name.clone(),
        url: vec![],
        image: vec![],
        discord: vec![],
        description: vec![],
        additional: vec![],
    };

    Identities::<T>::insert(&old_coldkey, identity);

    // Benchmark setup complete, now execute the extrinsic
    #[extrinsic_call]
    _(RawOrigin::Root, old_coldkey.clone(), new_coldkey.clone());
  }
}