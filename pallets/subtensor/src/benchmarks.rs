//! Subtensor pallet benchmarking.
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as Subtensor;
use crate::*;
use codec::Compact;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
pub use pallet::*;
use sp_core::H256;
use sp_runtime::{
    BoundedVec,
    traits::{BlakeTwo256, Hash},
};
use sp_std::vec;

benchmarks! {
  // Add individual benchmarks here
  benchmark_register {
    let netuid: u16 = 1; //11 is the benchmark network.
    let tempo: u16 = 1;
    let modality: u16 = 0;
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


  }: register( RawOrigin::Signed( hotkey.clone() ), netuid, block_number, nonce, work, hotkey.clone(), coldkey.clone() )

  benchmark_set_weights {

    // This is a whitelisted caller who can make transaction without weights.
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;
    let modality: u16 = 0;

    Subtensor::<T>::init_new_network(netuid, tempo);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    SubtokenEnabled::<T>::insert(netuid, true);
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

      Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone())?;

      let uid = Subtensor::<T>::get_uid_for_net_and_hotkey(netuid, &hotkey.clone()).unwrap();
      Subtensor::<T>::set_validator_permit_for_uid(netuid, uid, true);
      dests.push(id);
      weights.push(id);
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
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );

    Subtensor::<T>::set_network_registration_allowed( netuid, true);
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount_to_be_staked = 1000000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

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
    SubtokenEnabled::<T>::insert(netuid, true);

    Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_network_registration_allowed( netuid, true );

    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount: u64 = 60000000;
    let amount_to_be_staked = 1000000000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  }: add_stake(RawOrigin::Signed( coldkey.clone() ), hotkey, netuid, amount)

  benchmark_remove_stake{
    let caller: T::AccountId = whitelisted_caller::<T::AccountId>();
    let netuid: u16 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::increase_total_stake(1_000_000_000_000);
    Subtensor::<T>::init_new_network(netuid, tempo);
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_network_registration_allowed(netuid, true);
    Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    Subtensor::<T>::set_burn(netuid, 1);
    let wallet_bal = 9_999_999_999_999u64.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, wallet_bal);

    assert_ok!(Subtensor::<T>::do_burned_registration(
        RawOrigin::Signed(coldkey.clone()).into(),
        netuid,
        hotkey.clone()
    ));

    let tao_to_stake = 100_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, tao_to_stake);
    assert_ok!( Subtensor::<T>::add_stake(
        RawOrigin::Signed(coldkey.clone()).into(),
        hotkey.clone(),
        netuid,
        tao_to_stake
    ));

    let actual_alpha_minted: u64 = Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
    assert!(actual_alpha_minted > 0, "No alpha minted after add_stake");

    SubnetAlphaOut::<T>::insert(netuid, actual_alpha_minted * 2);

    let amount_unstaked: u64 = actual_alpha_minted / 2;
  }: remove_stake(RawOrigin::Signed( coldkey.clone() ), hotkey.clone(), netuid, amount_unstaked)

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
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    Subtensor::<T>::set_burn(netuid, 1);
    let amount_to_be_staked = 1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amount_to_be_staked);

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
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    Subtensor::<T>::set_burn(netuid, 1);
    let amount_to_be_staked = 1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), amount_to_be_staked);

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

    let amount_to_be_staked = balance.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

  }: sudo_register(RawOrigin::<AccountIdOf<T>>::Root, netuid, hotkey, coldkey, stake, balance)
  */
  benchmark_burned_register {
    let netuid: u16 = 1;
    let seed : u32 = 1;
    let hotkey: T::AccountId = account("Alice", 0, seed);
    let coldkey: T::AccountId = account("Test", 0, seed);
    let modality: u16 = 0;
    let tempo: u16 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_burn(netuid, 1);

    let amount_to_be_staked =  1000000u32.into();
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

  }: burned_register(RawOrigin::Signed( coldkey.clone() ), netuid, hotkey)


  benchmark_root_register {
    let netuid: u16 = 1;
    let version_key: u64 = 1;
    let tempo: u16 = 1;
    let seed : u32 = 1;

    Subtensor::<T>::init_new_network(netuid, tempo);
    SubtokenEnabled::<T>::insert(netuid, true);
    Subtensor::<T>::set_burn(netuid, 1);
    Subtensor::<T>::set_network_registration_allowed( netuid, true);

    Subtensor::<T>::set_max_allowed_uids( netuid, 4096 );
    assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("Alice", 0, seed);

    let amount_to_be_staked =  100_000_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

    assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  }: root_register(RawOrigin::Signed(coldkey), hotkey)

  benchmark_register_network {
    let seed : u32 = 1;

    let coldkey: T::AccountId = account("Test", 0, seed);
    let hotkey: T::AccountId = account("TestHotkey", 0, seed);

    Subtensor::<T>::set_network_rate_limit(1);

    let amount_to_be_staked = 100_000_000_000_000u64;
    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked.saturating_mul(2));
  }: register_network(RawOrigin::Signed(coldkey), hotkey.clone())

  // benchmark_dissolve_network {
  //   let seed : u32 = 1;

  //   let coldkey: T::AccountId = account("Test", 0, seed);
  //   let hotkey: T::AccountId = account("TestHotkey", 0, seed);

  //   Subtensor::<T>::set_network_rate_limit(0);

  //   let amount_to_be_staked = 100_000_000_000_000u64;
  //   Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);
  //   assert_ok!(Subtensor::<T>::register_network(RawOrigin::Root.into(), hotkey.clone()));
  // }: dissolve_network(RawOrigin::Root, coldkey.clone(), 1)


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

  commit_weights {
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
    Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

    let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
        netuid,
        block_number,
        start_nonce,
        &hotkey,
    );
    let result = Subtensor::<T>::register(
      <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())),
        netuid,
        block_number,
        nonce,
        work,
        hotkey.clone(),
        coldkey,
    );
    assert_ok!(result);
    Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);
    Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

}: commit_weights(RawOrigin::Signed(hotkey.clone()), netuid, commit_hash)

reveal_weights {
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
    Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

    let commit_hash: H256 = BlakeTwo256::hash_of(&(
      hotkey.clone(),
      netuid,
      uids.clone(),
      weight_values.clone(),
      salt.clone(),
      version_key,
  ));
    let _ = Subtensor::<T>::commit_weights(<T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())), netuid, commit_hash);

  }: reveal_weights(RawOrigin::Signed(hotkey.clone()), netuid, uids, weight_values, salt, version_key)

  schedule_swap_coldkey {
    let old_coldkey: T::AccountId = account("old_cold", 0, 1);
    let new_coldkey: T::AccountId = account("new_cold", 1, 2);
    Subtensor::<T>::add_balance_to_coldkey_account(&old_coldkey.clone(), 100_000_000_000_000u64);
    }: schedule_swap_coldkey(RawOrigin::Signed(old_coldkey.clone()), new_coldkey.clone())

// schedule_dissolve_network {
//     let coldkey: T::AccountId = account("coldkey", 0, 1);
//     let netuid = 1;
// }: schedule_dissolve_network(RawOrigin::Signed(coldkey.clone()), netuid)

  benchmark_sudo_set_tx_childkey_take_rate_limit {
    // We don't need to set up any initial state for this benchmark
    // as it's a simple setter function that only requires root origin
    let new_rate_limit: u64 = 100;
}: sudo_set_tx_childkey_take_rate_limit(RawOrigin::Root, new_rate_limit)

 benchmark_set_childkey_take {
  // Setup
  let netuid: u16 = 1;
  let tempo: u16 = 1;
  let seed: u32 = 1;
  let coldkey: T::AccountId = account("Cold", 0, seed);
  let hotkey: T::AccountId = account("Hot", 0, seed);
  let take: u16 = 1000; // 10% in basis points

  // Initialize the network
  Subtensor::<T>::init_new_network(netuid, tempo);
  SubtokenEnabled::<T>::insert(netuid, true);

  // Register the hotkey
  Subtensor::<T>::set_burn(netuid, 1);
  let amount_to_be_staked = 1_000_000u32.into();
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);
  assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
}: set_childkey_take(RawOrigin::Signed(coldkey), hotkey, netuid, take)

  swap_coldkey {
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
}: swap_coldkey(RawOrigin::Root, old_coldkey.clone(), new_coldkey.clone(), swap_cost)

batch_reveal_weights {
  let tempo: u16 = 0;
  let netuid: u16 = 1;
  let num_commits: usize = 10;

  let hotkey: T::AccountId = account("hot", 0, 1);
  let coldkey: T::AccountId = account("cold", 0, 2);

  Subtensor::<T>::init_new_network(netuid, tempo);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
  Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);
  Subtensor::<T>::set_weights_set_rate_limit(netuid, 0); // Disable rate limiting for benchmarking

  let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
  let (nonce, work): (u64, Vec<u8>) = Subtensor::<T>::create_work_for_block_number(
      netuid,
      block_number,
      3,
      &hotkey,
  );

  let origin = T::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone()));
  assert_ok!(Subtensor::<T>::register(
      origin.clone(),
      netuid,
      block_number,
      nonce,
      work.clone(),
      hotkey.clone(),
      coldkey.clone(),
  ));

  let uid: u16 = 0;

  Subtensor::<T>::set_validator_permit_for_uid(netuid, uid, true);

  let mut uids_list = Vec::new();
  let mut values_list = Vec::new();
  let mut salts_list = Vec::new();
  let mut version_keys = Vec::new();

  for i in 0..num_commits {
      let uids: Vec<u16> = vec![uid];
      let values: Vec<u16> = vec![i as u16];
      let salt: Vec<u16> = vec![i as u16];
      let version_key_i: u64 = i as u64;

      let commit_hash: H256 = BlakeTwo256::hash_of(&(
          hotkey.clone(),
          netuid,
          uids.clone(),
          values.clone(),
          salt.clone(),
          version_key_i,
      ));

      assert_ok!(Subtensor::<T>::commit_weights(
          T::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone())),
          netuid,
          commit_hash,
      ));

      uids_list.push(uids);
      values_list.push(values);
      salts_list.push(salt);
      version_keys.push(version_key_i);
  }
}: batch_reveal_weights(
  RawOrigin::Signed(hotkey.clone()),
  netuid,
  uids_list,
  values_list,
  salts_list,
  version_keys
)

benchmark_recycle_alpha {
  let caller: T::AccountId = whitelisted_caller::<T::AccountId>();
  let netuid: u16 = 1;
  let tempo: u16 = 1;
  let seed: u32 = 1;

  let coldkey: T::AccountId = account("Test", 0, seed);
  let hotkey: T::AccountId = account("Alice", 0, seed);

  Subtensor::<T>::init_new_network(netuid, tempo);
  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  Subtensor::<T>::set_burn(netuid, 1);

  let amount_to_be_staked = 1_000_000_000u64.into();
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);

  assert_ok!(Subtensor::<T>::do_burned_registration(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      hotkey.clone()
  ));

  let alpha_amount: u64 = 1_000_000;
  SubnetAlphaOut::<T>::insert(netuid, alpha_amount * 2);

  Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
      &hotkey,
      &coldkey,
      netuid,
      alpha_amount
  );

  assert_eq!(TotalHotkeyAlpha::<T>::get(&hotkey, netuid), alpha_amount);
}: recycle_alpha(RawOrigin::Signed(coldkey), hotkey, alpha_amount, netuid)

benchmark_burn_alpha {
  let caller: T::AccountId = whitelisted_caller::<T::AccountId>();
  let netuid: u16 = 1;
  let tempo: u16 = 1;
  let seed: u32 = 1;

  let coldkey: T::AccountId = account("Test", 0, seed);
  let hotkey: T::AccountId = account("Alice", 0, seed);

  Subtensor::<T>::init_new_network(netuid, tempo);
  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  Subtensor::<T>::set_burn(netuid, 1);

  let amount_to_be_staked = 1_000_000_000u64.into();
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);

  assert_ok!(Subtensor::<T>::do_burned_registration(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      hotkey.clone()
  ));

  let alpha_amount: u64 = 1_000_000;
  SubnetAlphaOut::<T>::insert(netuid, alpha_amount * 2);

  Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
      &hotkey,
      &coldkey,
      netuid,
      alpha_amount
  );

  assert_eq!(TotalHotkeyAlpha::<T>::get(&hotkey, netuid), alpha_amount);

}: burn_alpha(RawOrigin::Signed(coldkey), hotkey, alpha_amount, netuid)


benchmark_start_call {
  let caller: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let caller_origin = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
  let netuid: u16 = 1;
  let tempo: u16 = 1;
  let seed: u32 = 1;

  // Set up coldkey and hotkey
  let coldkey: T::AccountId = account("Test", 0, seed);
  let hotkey: T::AccountId = account("Alice", 0, seed);

  // Initialize network
  Subtensor::<T>::init_new_network(netuid, tempo);
  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);

  // Register the neuron
  Subtensor::<T>::set_burn(netuid, 1);
  let amount_to_be_staked = 1000000u32.into();
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);
  SubnetOwner::<T>::set(netuid, coldkey.clone());

  assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), netuid, hotkey.clone()));
  assert_eq!(SubnetOwner::<T>::get(netuid), coldkey.clone());
  assert_eq!(FirstEmissionBlockNumber::<T>::get(netuid), None);
  let current_block: u64 = Subtensor::<T>::get_current_block_as_u64();
  let duration = <T as Config>::DurationOfStartCall::get();
  let block: BlockNumberFor<T> = (current_block + duration).try_into().ok().expect("can't convert to block number");
  frame_system::Pallet::<T>::set_block_number(block);

}: start_call(RawOrigin::Signed(coldkey), netuid)

benchmark_adjust_senate {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey:  T::AccountId = account("Alice", 0, 1);
  let root: u16 = Subtensor::<T>::get_root_netuid();
  Subtensor::<T>::init_new_network(root, 1);
  Uids::<T>::insert(root, &hotkey, 0u16);
}: adjust_senate(RawOrigin::Signed(coldkey), hotkey.clone())

benchmark_add_stake_limit {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey : T::AccountId = account("Alice", 0, 1);
  let netuid : u16          = 1;
  let amount : u64          = 1_000_000;
  let limit  : u64          = 1_000_000;
  let allow  : bool         = true;
  Subtensor::<T>::init_new_network(netuid, 1);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  SubtokenEnabled::<T>::insert(netuid, true);

  let bond    = Subtensor::<T>::get_burn_as_u64(netuid);
  let deposit = (amount + bond + DefaultStakingFee::<T>::get()) * 10;
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);
  assert_ok!(
      Subtensor::<T>::burned_register(
          RawOrigin::Signed(coldkey.clone()).into(),
          netuid,
          hotkey.clone()
      )
  );
  SubnetTAO::<T>::insert(netuid, deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

}: add_stake_limit(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), netuid, amount, limit, allow)

benchmark_move_stake {
  let coldkey:     T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let origin:      T::AccountId = account("A", 0, 1);
  let destination: T::AccountId = account("B", 0, 2);
  let netuid:      u16          = 1;

  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::init_new_network(netuid, 1);
  let burn_fee  = Subtensor::<T>::get_burn_as_u64(netuid);
  let stake_tao = 1_000_000;
  let deposit   = burn_fee.saturating_mul(2).saturating_add(stake_tao);
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);

  assert_ok!(
    Subtensor::<T>::burned_register(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      origin.clone()
    )
  );

  SubnetTAO::<T>::insert(netuid,   deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

  assert_ok!(
    Subtensor::<T>::add_stake_limit(
      RawOrigin::Signed(coldkey.clone()).into(),
      origin.clone(),
      netuid,
      stake_tao,
      u64::MAX,
      false
    )
  );

  let alpha_to_move: u64 =
    Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
      &origin, &coldkey, netuid
    );

  Subtensor::<T>::create_account_if_non_existent(&coldkey, &destination);
}: move_stake(RawOrigin::Signed(coldkey.clone()),origin.clone(),destination.clone(),netuid,netuid,alpha_to_move)

benchmark_remove_stake_limit {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey:  T::AccountId = account("Alice", 0, 1);
  let netuid:  u16          = 1;

  Subtensor::<T>::init_new_network(netuid, 1);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  SubtokenEnabled::<T>::insert(netuid, true);

  let bond   = Subtensor::<T>::get_burn_as_u64(netuid);
  let fee    = DefaultStakingFee::<T>::get();
  let amount: u64 = 1_000_000;
  let deposit = (amount + bond + fee).saturating_mul(10);

  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);
  assert_ok!(
      Subtensor::<T>::burned_register(
          RawOrigin::Signed(coldkey.clone()).into(),
          netuid,
          hotkey.clone(),
      )
  );

  SubnetTAO::<T>::insert(netuid, deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  SubnetAlphaOut::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

  assert_ok!(
      Subtensor::<T>::add_stake_limit(
          RawOrigin::Signed(coldkey.clone()).into(),
          hotkey.clone(),
          netuid,
          amount,
          u64::MAX,
          false,
      )
  );

  let alpha: u64 = Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
      &hotkey, &coldkey, netuid
  );

  assert_ok!(
      Subtensor::<T>::remove_stake_limit(
          RawOrigin::Signed(coldkey.clone()).into(),
          hotkey.clone(),
          netuid,
          alpha,
          u64::MAX,
          true,
      )
  );
}: remove_stake_limit(RawOrigin::Signed(coldkey.clone()),hotkey.clone(),netuid,alpha,u64::MAX,true)

benchmark_swap_stake_limit {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hot:     T::AccountId = account("A", 0, 1);
  let netuid:  u16          = 1;
  let allow:   bool         = true;

  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::init_new_network(netuid, 1);

  let reg_fee   = Subtensor::<T>::get_burn_as_u64(netuid);
  let stake_tao = 1_000_000;
  let deposit   = reg_fee.saturating_mul(2).saturating_add(stake_tao);
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);

  assert_ok!(
    Subtensor::<T>::burned_register(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      hot.clone()
    )
  );

  SubnetTAO::<T>::insert(netuid,   deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

  assert_ok!(
    Subtensor::<T>::add_stake_limit(
      RawOrigin::Signed(coldkey.clone()).into(),
      hot.clone(),
      netuid,
      stake_tao,
      u64::MAX,
      allow
    )
  );

  let alpha_to_swap: u64 =
    Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
      &hot, &coldkey, netuid
    );
}: swap_stake_limit(RawOrigin::Signed(coldkey.clone()),hot.clone(),netuid,netuid,alpha_to_swap,u64::MAX,allow)

benchmark_transfer_stake {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let dest:    T::AccountId = account("B", 0, 2);
  let hot:     T::AccountId = account("A", 0, 1);
  let netuid:  u16          = 1;

  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::init_new_network(netuid, 1);

  let reg_fee = Subtensor::<T>::get_burn_as_u64(netuid);
  let stake_tao = 1_000_000;
  let deposit = reg_fee.saturating_mul(2).saturating_add(stake_tao);
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);

  assert_ok!(
    Subtensor::<T>::burned_register(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      hot.clone()
    )
  );

  SubnetTAO::<T>::insert(netuid, deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

  assert_ok!(
    Subtensor::<T>::add_stake_limit(
      RawOrigin::Signed(coldkey.clone()).into(),
      hot.clone(),
      netuid,
      stake_tao,
      u64::MAX,
      false
    )
  );

  let alpha_to_transfer: u64 =
    Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
      &hot, &coldkey, netuid
    );

  Subtensor::<T>::create_account_if_non_existent(&dest, &hot);
}: transfer_stake(RawOrigin::Signed(coldkey.clone()),dest.clone(),hot.clone(),netuid,netuid,alpha_to_transfer)

benchmark_swap_stake {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hot:     T::AccountId = account("A", 0, 9);
  let netuid:  u16          = 1;

  SubtokenEnabled::<T>::insert(netuid, true);
  Subtensor::<T>::init_new_network(netuid, 1);

  let reg_fee = Subtensor::<T>::get_burn_as_u64(netuid);
  let stake_tao = 1_000_000;
  let deposit = reg_fee.saturating_mul(2).saturating_add(stake_tao);
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);

  assert_ok!(
    Subtensor::<T>::burned_register(
      RawOrigin::Signed(coldkey.clone()).into(),
      netuid,
      hot.clone()
    )
  );

  SubnetTAO::<T>::insert(netuid, deposit);
  SubnetAlphaIn::<T>::insert(netuid, deposit);
  TotalStake::<T>::set(deposit);

  assert_ok!(
    Subtensor::<T>::add_stake_limit(
      RawOrigin::Signed(coldkey.clone()).into(),
      hot.clone(),
      netuid,
      stake_tao,
      u64::MAX,
      false
    )
  );

  let alpha_to_swap: u64 =
    Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
      &hot, &coldkey, netuid
    );
}: swap_stake(RawOrigin::Signed(coldkey.clone()),hot.clone(),netuid,netuid,alpha_to_swap)

benchmark_batch_commit_weights {
  let hotkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let netuid:  u16         = 1;
  let count:   usize       = 3;
  let mut netuids: Vec<Compact<u16>> = Vec::new();
  let mut hashes:  Vec<H256>         = Vec::new();

  Subtensor::<T>::init_new_network(netuid, 1);
  Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
  SubtokenEnabled::<T>::insert(netuid, true);
  let reg_fee = Subtensor::<T>::get_burn_as_u64(netuid);
  Subtensor::<T>::add_balance_to_coldkey_account(&hotkey, reg_fee.saturating_mul(2));

  assert_ok!(
      Subtensor::<T>::burned_register(
          RawOrigin::Signed(hotkey.clone()).into(),
          netuid,
          hotkey.clone()
      )
  );

  Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);
  Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

  for i in 0..count {
      netuids.push( Compact(netuid) );
      hashes.push( H256::repeat_byte(i as u8) );
  }
}: batch_commit_weights(RawOrigin::Signed(hotkey.clone()),netuids, hashes)

benchmark_batch_set_weights {
  let hotkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let netuid:  u16         = 1;
  let version: u64         = 1;
  let entries: Vec<(Compact<u16>, Compact<u16>)> = vec![
      (Compact(0u16), Compact(0u16))
  ];
  let netuids: Vec<Compact<u16>> =
      vec![ Compact(netuid) ];
  let weights: Vec<Vec<(Compact<u16>, Compact<u16>)>> =
      vec![ entries.clone() ];
  let keys: Vec<Compact<u64>> =
      vec![ Compact(version) ];

  Subtensor::<T>::init_new_network(netuid, 1);
  Subtensor::<T>::set_network_registration_allowed(netuid, true);
  SubtokenEnabled::<T>::insert(netuid, true);

  let reg_fee = Subtensor::<T>::get_burn_as_u64(netuid);
  Subtensor::<T>::add_balance_to_coldkey_account(&hotkey, reg_fee.saturating_mul(2));

  assert_ok!(
      Subtensor::<T>::burned_register(
          RawOrigin::Signed(hotkey.clone()).into(),
          netuid,
          hotkey.clone()
      )
  );
}: batch_set_weights(RawOrigin::Signed(hotkey.clone()),netuids, weights, keys)

benchmark_commit_crv3_weights {
  let hotkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let netuid:  u16         = 1;
  let vec_commit: Vec<u8>  = vec![0; MAX_CRV3_COMMIT_SIZE_BYTES as usize];
  let commit: BoundedVec<_, _> =
      vec_commit.try_into().unwrap();
  let round: u64           = 0;

  Subtensor::<T>::init_new_network(netuid, 1);
  Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
  SubtokenEnabled::<T>::insert(netuid, true);

  let reg_fee = Subtensor::<T>::get_burn_as_u64(netuid);
  Subtensor::<T>::add_balance_to_coldkey_account(&hotkey, reg_fee.saturating_mul(2));

  assert_ok!(
      Subtensor::<T>::burned_register(
          RawOrigin::Signed(hotkey.clone()).into(),
          netuid,
          hotkey.clone()
      )
  );

  Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);
}: commit_crv3_weights(RawOrigin::Signed(hotkey.clone()),netuid, commit, round)

benchmark_decrease_take {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey:  T::AccountId = account("Alice", 0, 1);
  let take:    u16          = 100;

  Delegates::<T>::insert(&hotkey, 200u16);
  Owner::<T>::insert(&hotkey, &coldkey);
}: decrease_take(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), take)

benchmark_increase_take {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey:  T::AccountId = account("Alice", 0, 2);
  let take:    u16          = 150;

  Delegates::<T>::insert(&hotkey, 100u16);
  Owner::<T>::insert(&hotkey, &coldkey);
}: increase_take(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), take)

benchmark_register_network_with_identity {
  let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
  let hotkey:  T::AccountId = account("Alice", 0, 1);
  let identity: Option<SubnetIdentityOfV2> = None;
  Subtensor::<T>::set_network_registration_allowed( 1, true );
  Subtensor::<T>::set_network_rate_limit(1);
  Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, 9_999_999_999_999u64);
}: register_network_with_identity(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), identity)
}
