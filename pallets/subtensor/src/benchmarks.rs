#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use
{
    crate::
    {
        Pallet as Subtensor
    },
    frame_benchmarking::
    {
        v1::
        {
            account
        },
        v2::
        {
            *
        }
    },
    frame_system::
    {
        RawOrigin
    },
    frame_support::
    {
        assert_ok,
        traits::
        {
            Get
        }
    },
    sp_runtime::
    {
        traits::
        {
            StaticLookup,
            Bounded
        }
    },
    sp_std::
    {
        mem::
        {
            size_of
        }
    }
};

#[benchmarks]
mod benchmark
{
    use super::*;

    /*#[benchmark]
    fn register()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_network_registration_allowed(1, true);
        
        let block_number:   u64             = Subtensor::<T>::get_current_block_as_u64();
        let start_nonce:    u64             = 39420842 + 100;
        let hotkey:         T::AccountId    = account("Alice", 0, 1);
        let (nonce, work):  (u64, Vec<u8>)  = Subtensor::<T>::create_work_for_block_number(1, block_number, start_nonce, &hotkey);
        let block_number:   u64             = Subtensor::<T>::get_current_block_as_u64();
        let coldkey:        T::AccountId    = account("Test", 0, 1);

        #[extrinsic_call]
        _(RawOrigin::Signed(hotkey.clone()), 1, block_number, nonce, work, hotkey.clone(), coldkey.clone());
    }*/

    #[benchmark]
    fn set_weights()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
        Subtensor::<T>::set_network_registration_allowed(1, true);
        Subtensor::<T>::set_max_registrations_per_block(1, 4096);
        Subtensor::<T>::set_target_registrations_per_interval(1, 4096);

        let mut seed:     u32           = 1;
        let mut dests:    Vec<u16>      = vec![];
        let mut weights:  Vec<u16>      = vec![];
        let signer:       T::AccountId  = account("Alice", 0, seed);

        for id in 0..4096 as u16 
        {
            let hotkey:   T::AccountId = account("Alice", 0, seed);
            let coldkey:  T::AccountId = account("Test", 0, seed);
            seed = seed + 1;

            Subtensor::<T>::set_burn(1, 1);
            Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(1_000_000).unwrap());

            assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), 1, hotkey.clone()));

            let uid = Subtensor::<T>::get_uid_for_net_and_hotkey(1, &hotkey.clone()).unwrap();
            Subtensor::<T>::set_validator_permit_for_uid(1, uid.clone(), true);

            dests.push(id.clone());
            weights.push(id.clone());
        }

        #[extrinsic_call  ]
        _(RawOrigin::Signed(signer.clone()), 1, dests, weights, 1);
    }

    #[benchmark]
    fn become_delegate()
    {
        let caller:         T::AccountId  = whitelisted_caller::<AccountIdOf<T>>();
        let caller_origin                 = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));
    
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_burn(1, 1);
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
    
        Subtensor::<T>::set_network_registration_allowed(1, true);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);
    
        let coldkey:  T::AccountId = account("Test", 0, 1);
        let hotkey:   T::AccountId = account("Alice", 0, 1);
    
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(1_000_000_000).unwrap());
        assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), 1, hotkey.clone()));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone());
    }

    #[benchmark]
    fn add_stake()
    {
        let caller:       T::AccountId  = whitelisted_caller::<AccountIdOf<T>>();
        let caller_origin               = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

        Subtensor::<T>::init_new_network(1, 1);
    
        Subtensor::<T>::set_burn(1, 1);
        Subtensor::<T>::set_network_registration_allowed(1, true);
    
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);
    
        let coldkey:  T::AccountId = account("Test", 0, 1);
        let hotkey:   T::AccountId = account("Alice", 0, 1);
    
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(1_000_000_000).unwrap());
        assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), 1, hotkey.clone()));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey, 1);
    }

    #[benchmark]
    fn serve_axon()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_serving_rate_limit(1, 0);
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);

        let caller:         T::AccountId    = whitelisted_caller::<AccountIdOf<T>>();
        let caller_origin                   = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

        Subtensor::<T>::set_burn(1, 1);
        Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), Subtensor::<T>::u64_to_balance(1_000_000).unwrap());
        assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), 1, caller.clone()));
        
        let version:        u32             = 2;
        let ip:             u128            = 1676056785;
        let port:           u16             = 128;
        let ip_type:        u8              = 4;
        let protocol:       u8              = 0;
        let placeholder1:   u8              = 0;
        let placeholder2:   u8              = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), 1, version, ip, port, ip_type, protocol, placeholder1, placeholder2);
    }

    #[benchmark]
    fn remove_stake()
    {
        let caller:       T::AccountId  = whitelisted_caller::<AccountIdOf<T>>();
        let caller_origin               = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

        // Set our total stake to 1000 TAO
	      Subtensor::<T>::increase_total_stake(1_000_000_000_000);

	      Subtensor::<T>::init_new_network(1, 1);
	      Subtensor::<T>::set_network_registration_allowed(1, true);
        Subtensor::<T>::set_burn(1, 1);
	      Subtensor::<T>::set_max_allowed_uids(1, 4096 );
  	    assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);

	      let coldkey:  T::AccountId  = account("Test", 0, 1);
	      let hotkey:   T::AccountId  = account("Alice", 0, 1);
	      let wallet_bal              = Subtensor::<T>::u64_to_balance(1_000_000).unwrap();
  	    Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), wallet_bal);

	      assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), 1, hotkey.clone()));
	      assert_ok!(Subtensor::<T>::do_become_delegate(RawOrigin::Signed(coldkey.clone()).into(), hotkey.clone(), Subtensor::<T>::get_default_take()));

	      // Stake 10% of our current total staked TAO
	      let u64_staked_amt = 100_000_000_000;
	      let amount_to_be_staked = Subtensor::<T>::u64_to_balance(u64_staked_amt);
	      Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked.unwrap());

	      assert_ok!( Subtensor::<T>::add_stake(RawOrigin::Signed( coldkey.clone() ).into() , hotkey.clone(), u64_staked_amt));
	      let amount_unstaked: u64 = u64_staked_amt - 1;
    
        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), amount_unstaked);
    }

    #[benchmark]
    fn serve_prometheus()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);

        let caller:         T::AccountId    = whitelisted_caller::<AccountIdOf<T>>();
        let caller_origin                   = <T as frame_system::Config>::RuntimeOrigin::from(RawOrigin::Signed(caller.clone()));

        Subtensor::<T>::set_burn(1, 1);
        Subtensor::<T>::add_balance_to_coldkey_account(&caller.clone(), Subtensor::<T>::u64_to_balance(1_000_000).unwrap());
        assert_ok!(Subtensor::<T>::do_burned_registration(caller_origin.clone(), 1, caller.clone()));
        Subtensor::<T>::set_serving_rate_limit(1, 0);

        let version:    u32     = 2;
        let ip:         u128    = 1676056785;
        let port:       u16     = 128;
        let ip_type:    u8      = 4;
    
        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), 1, version, ip, port, ip_type);
    }

    #[benchmark]
    fn burned_register()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_burn(1, 1);

        let hotkey:     T::AccountId = account("Alice", 0, 1);
        let coldkey:    T::AccountId = account("Test", 0, 1);

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(100_000_000_000_000).unwrap());    

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), 1, hotkey);
    }

    #[benchmark]
    fn root_register()
    {
        Subtensor::<T>::init_new_network(1, 1);
        Subtensor::<T>::set_burn(1, 1);
        Subtensor::<T>::set_network_registration_allowed(1, true);
        Subtensor::<T>::set_max_allowed_uids(1, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(1), 4096);

        let coldkey:    T::AccountId = account("Test", 0, 1);
        let hotkey:     T::AccountId = account("Alice", 0, 1);

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(100_000_000_000_000).unwrap());

        assert_ok!(Subtensor::<T>::do_burned_registration(RawOrigin::Signed(coldkey.clone()).into(), 1, hotkey.clone()));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), hotkey);
    }

    #[benchmark]
    fn register_network()
    {
        Subtensor::<T>::set_network_rate_limit(0);

        let seed:                   u32                 = 1;
        let coldkey:                T::AccountId        = account("Test", 0, seed);
        let amount:                 u64                 = 1;
        let amount_to_be_staked                         = Subtensor::<T>::u64_to_balance(100_000_000_000_000).unwrap();

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey));
    }

    #[benchmark]
    fn dissolve_network()
    {
        Subtensor::<T>::set_network_rate_limit(0);

        let seed:                   u32                 = 1;
        let coldkey:                T::AccountId        = account("Test", 0, seed);
        let amount:                 u64                 = 1;
        let amount_to_be_staked                         = Subtensor::<T>::u64_to_balance(100_000_000_000_000).unwrap();

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_to_be_staked);
        assert_ok!(Subtensor::<T>::register_network(RawOrigin::Signed(coldkey.clone()).into()));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), 1);
    }

    #[benchmark]
    fn swap_hotkey()
    {	
        Subtensor::<T>::init_new_network(1, 100);
        Subtensor::<T>::set_min_burn(1, 1);
        Subtensor::<T>::set_max_burn(1, 1);
        Subtensor::<T>::set_target_registrations_per_interval(1, 256);
        Subtensor::<T>::set_max_registrations_per_block(1, 256);

        let seed:       u32             = 1;
	      let coldkey:    T::AccountId    = account("Alice", 0, seed);
	      let old_hotkey: T::AccountId    = account("Bob", 0, seed);
	      let new_hotkey: T::AccountId    = account("Charlie", 0, seed);

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(10_000_000_000).unwrap());
        assert_ok!(
            Subtensor::<T>::burned_register(
                RawOrigin::Signed(coldkey.clone()).into(), 
                1, 
                old_hotkey.clone()
            )
        );

        assert_ok!(
            Subtensor::<T>::become_delegate(
                RawOrigin::Signed(coldkey.clone()).into(), 
                old_hotkey.clone()
            )
        );

        for i in 0..(Subtensor::<T>::get_max_allowed_uids(1) as u32 - 1) 
        {
            let coldkey:    T::AccountId = account("Axon", 0, i);
            let hotkey:     T::AccountId = account("Hotkey", 0, i);
    
            Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), Subtensor::<T>::u64_to_balance(10_000_000_000).unwrap());
            assert_ok!(
                Subtensor::<T>::burned_register(
                    RawOrigin::Signed(coldkey.clone()).into(), 
                    1, 
                    hotkey
                )
            );

            assert_ok!(
                Subtensor::<T>::add_stake(
                    RawOrigin::Signed(coldkey).into(), 
                    old_hotkey.clone(), 
                    1_000_000_000
                )
            );
        }

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), old_hotkey, new_hotkey);
    }
}

/*
//! Subtensor pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as Subtensor;
use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use sp_std::vec::{Vec};
use frame_system::RawOrigin;
pub use pallet::*;

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
 */