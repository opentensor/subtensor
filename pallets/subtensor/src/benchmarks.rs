//! Subtensor pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

use crate::Pallet as Subtensor;
use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::sp_std::vec;
use frame_system::RawOrigin;
pub use pallet::*;
use sp_std::vec::Vec;
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
