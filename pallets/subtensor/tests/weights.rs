mod mock;
use mock::*;
use pallet_subtensor::{Error};
use frame_system::Config;
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;
use substrate_fixed::types::I32F32;

/***************************
  pub fn set_weights() tests
*****************************/

// Test the call passes through the subtensor module.
#[test]
fn test_set_weights_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let dests = vec![1, 1];
		let weights = vec![1, 1];
        let netuid: u16 = 1;
		let version_key: u64 = 0;
		let call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights{netuid, dests, weights, version_key});
		let dispatch_info = call.get_dispatch_info();
		
		assert_eq!(dispatch_info.class, DispatchClass::Normal);
		assert_eq!(dispatch_info.pays_fee, Pays::No);
	});
}

// Test ensures that uid has validator permit to set non-self weights.
#[test]
fn test_weights_err_no_validator_permit() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id: u64 = 55;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids(netuid, 3);
    	register_ok_neuron( netuid, hotkey_account_id, 66, 0);
		register_ok_neuron( netuid, 1, 1, 65555 );
		register_ok_neuron( netuid, 2, 2, 75555 );
		
		let weights_keys: Vec<u16> = vec![1, 2];
		let weight_values: Vec<u16> = vec![1, 2];
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), netuid, weights_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::NoValidatorPermit.into()));

		let weights_keys: Vec<u16> = vec![1, 2];
		let weight_values: Vec<u16> = vec![1, 2];
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), netuid, weights_keys, weight_values, 0);
		assert_ok!(result);
	});
}

// Test ensures that a uid can only set weights if it has the valid weights set version key.
#[test]
fn test_weights_version_key() {
	new_test_ext().execute_with(|| {
        let hotkey: u64= 55;
		let coldkey: u64= 66;
		let netuid0: u16 = 0;
		let netuid1: u16 = 2;
		add_network(netuid0, 0, 0);
		add_network(netuid1, 0, 0);
		register_ok_neuron( netuid0, hotkey, coldkey, 2143124 );
		register_ok_neuron( netuid1, hotkey, coldkey, 3124124 );

		let weights_keys: Vec<u16> = vec![0];
		let weight_values: Vec<u16> = vec![1];
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid0, weights_keys.clone(), weight_values.clone(), 0) );
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid1, weights_keys.clone(), weight_values.clone(), 0) );

		// Set version keys.
		let key0: u64 = 12312;
		let key1: u64 = 20313;
		SubtensorModule::set_weights_version_key( netuid0, key0 );
		SubtensorModule::set_weights_version_key( netuid1, key1 );

		// Setting works with version key.
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid0, weights_keys.clone(), weight_values.clone(), key0) );
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid1, weights_keys.clone(), weight_values.clone(), key1) );

		// Setting fails with incorrect keys.
		assert_eq!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid0, weights_keys.clone(), weight_values.clone(), key1), Err(Error::<Test>::IncorrectNetworkVersionKey.into()) );
		assert_eq!( SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey), netuid1, weights_keys.clone(), weight_values.clone(), key0), Err(Error::<Test>::IncorrectNetworkVersionKey.into()) );
	});
}

// Test ensures that uid has validator permit to set non-self weights.
#[test]
fn test_weights_err_setting_weights_too_fast() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id: u64 = 55;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids(netuid, 3);
    	register_ok_neuron( netuid, hotkey_account_id, 66, 0);
		register_ok_neuron( netuid, 1, 1, 65555 );
		register_ok_neuron( netuid, 2, 2, 75555 );

		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
		assert_ok!( SubtensorModule::sudo_set_weights_set_rate_limit(<<Test as Config>::RuntimeOrigin>::root(), netuid, 10) );
		assert_eq!( SubtensorModule::get_weights_set_rate_limit(netuid), 10);
		
		let weights_keys: Vec<u16> = vec![1, 2];
		let weight_values: Vec<u16> = vec![1, 2];

		// Note that LastUpdate has default 0 for new uids, but if they have actually set weights on block 0
		// then they are allowed to set weights again once more without a wait restriction, to accommodate the default.
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), netuid, weights_keys.clone(), weight_values.clone(), 0);
		assert_ok!(result);
		run_to_block(1);

		for i in 1..100 {
			let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), netuid, weights_keys.clone(), weight_values.clone(), 0);
			if i % 10 == 1 {
				assert_ok!(result);
			}
			else {
				assert_eq!(result, Err(Error::<Test>::SettingWeightsTooFast.into()));
			}
			run_to_block(i + 1);
		}
	});
}

// Test ensures that uids -- weights must have the same size.
#[test]
fn test_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id: u64 = 55;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
    	register_ok_neuron(1, hotkey_account_id, 66, 0);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5]; // Uneven sizes
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), 1, weights_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

// Test ensures that uids can have not duplicates
#[test]
fn test_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id: u64 = 666;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);

		SubtensorModule::set_max_allowed_uids(netuid, 100); // Allow many registrations per block.
		SubtensorModule::set_max_registrations_per_block(netuid, 100); // Allow many registrations per block.
		
		// uid 0
		register_ok_neuron( netuid, hotkey_account_id, 77, 0);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

		// uid 1
		register_ok_neuron( netuid, 1, 1, 100_000);
		SubtensorModule::get_uid_for_net_and_hotkey( netuid, &1 ).expect("Not registered.");

		// uid 2
		register_ok_neuron( netuid, 2, 1, 200_000);
		SubtensorModule::get_uid_for_net_and_hotkey( netuid, &2 ).expect("Not registered.");

		// uid 3
		register_ok_neuron( netuid, 3, 1, 300_000);
		SubtensorModule::get_uid_for_net_and_hotkey( netuid, &3 ).expect("Not registered.");
		
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 4);

		let weights_keys: Vec<u16> = vec![1, 1, 1]; // Contains duplicates
		let weight_values: Vec<u16> = vec![1, 2, 3];
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(hotkey_account_id), netuid, weights_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

// Test ensures weights cannot exceed max weight limit.
#[test]
fn test_weights_err_max_weight_limit() { //TO DO SAM: uncomment when we implement run_to_block fn
	new_test_ext().execute_with(|| { 
		// Add network.
		let netuid: u16 = 1;
		let tempo: u16 = 100;
		add_network(netuid, tempo, 0);

		// Set params.
		SubtensorModule::set_max_allowed_uids(netuid, 5);
		SubtensorModule::set_max_weight_limit( netuid, u16::MAX/5 );

		// Add 5 accounts.
		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 0, 0 );
		register_ok_neuron( netuid, 0, 0, 55555 );
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &0 ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
		assert_eq!( SubtensorModule::get_subnetwork_n(netuid), 1 );
		assert!( SubtensorModule::is_hotkey_registered_on_network( netuid, &0 ) );
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 1, 1 );
		register_ok_neuron( netuid, 1, 1, 65555 );
		assert!( SubtensorModule::is_hotkey_registered_on_network( netuid, &1 ) );
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 2);
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 2, 2 );
		register_ok_neuron( netuid, 2, 2, 75555 );
		assert!( SubtensorModule::is_hotkey_registered_on_network( netuid, &2 ) );
		assert_eq!( SubtensorModule::get_subnetwork_n(netuid), 3 );
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 3, 3 );
		register_ok_neuron( netuid, 3, 3, 95555 );
		assert!( SubtensorModule::is_hotkey_registered_on_network( netuid, &3 ) );
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 4);
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 4, 4 );
		register_ok_neuron( netuid, 4, 4, 35555 );
		assert!( SubtensorModule::is_hotkey_registered_on_network( netuid, &4 ) );
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 5);
		step_block(1);

		// Non self-weight fails.
		let uids: Vec<u16> = vec![ 1, 2, 3, 4 ]; 
		let values: Vec<u16> = vec![ u16::MAX/4, u16::MAX/4, u16::MAX/54, u16::MAX/4];
		let result = SubtensorModule::set_weights( RuntimeOrigin::signed(0), 1, uids, values, 0 );
		assert_eq!(result, Err(Error::<Test>::MaxWeightExceeded.into()));

		// Self-weight is a success.
		let uids: Vec<u16> = vec![ 0 ];  // Self.
		let values: Vec<u16> = vec![ u16::MAX ]; // normalizes to u32::MAX
		assert_ok!(SubtensorModule::set_weights( RuntimeOrigin::signed(0), 1, uids, values, 0));
	});
}

// Tests the call requires a valid origin.
#[test]
fn test_no_signature() {
	new_test_ext().execute_with(|| {
		let uids: Vec<u16> = vec![];
		let values: Vec<u16> = vec![];
		let result = SubtensorModule::set_weights(RuntimeOrigin::none(), 1, uids, values, 0);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

// Tests that weights cannot be set BY non-registered hotkeys.
#[test]
fn test_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);

		// Register one neuron. Should have uid 0
		register_ok_neuron(1, 666, 2, 100000);
		SubtensorModule::get_uid_for_net_and_hotkey( netuid, &666 ).expect("Not registered.");

		let weights_keys: Vec<u16> = vec![0]; // Uid 0 is valid.
		let weight_values: Vec<u16> = vec![1];
		// This hotkey is NOT registered.
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1, weights_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::NotRegistered.into()));
	});
}

// Tests that set weights fails if you pass invalid uids.
#[test]
fn test_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id: u64 = 55;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		register_ok_neuron( 1, hotkey_account_id, 66, 0);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
		let weight_keys : Vec<u16> = vec![9999]; // Does not exist
		let weight_values : Vec<u16> = vec![88]; // random value
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(55), 1, weight_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));
	});
}

// Tests that set weights fails if you dont pass enough values.
#[test]
fn test_set_weight_not_enough_values() {
	new_test_ext().execute_with(|| {
        
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		
		register_ok_neuron(1, 1, 2, 100000);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &1 ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

		register_ok_neuron(1, 3, 4, 300000);
		SubtensorModule::set_min_allowed_weights(1, 2);

		// Should fail because we are only setting a single value and its not the self weight.
		let weight_keys : Vec<u16> = vec![1]; // not weight. 
		let weight_values : Vec<u16> = vec![88]; // random value.
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1, weight_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::NotSettingEnoughWeights.into()));

		// Shouldnt fail because we setting a single value but it is the self weight.
		let weight_keys : Vec<u16> = vec![0]; // self weight.
		let weight_values : Vec<u16> = vec![88]; // random value.
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1 , weight_keys, weight_values, 0)) ;

		// Should pass because we are setting enough values.
		let weight_keys : Vec<u16> = vec![0, 1]; // self weight. 
		let weight_values : Vec<u16> = vec![10, 10]; // random value.
		SubtensorModule::set_min_allowed_weights(1, 1);
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1,  weight_keys, weight_values, 0)) ;
	});
}

// Tests that the weights set fails if you pass too many uids for the subnet
#[test]
fn test_set_weight_too_many_uids() {
	new_test_ext().execute_with(|| {
        
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		
		register_ok_neuron(1, 1, 2, 100_000);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &1 ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

		register_ok_neuron(1, 3, 4, 300_000);
		SubtensorModule::set_min_allowed_weights(1, 2);

		// Should fail because we are setting more weights than there are neurons.
		let weight_keys : Vec<u16> = vec![0, 1, 2, 3, 4]; // more uids than neurons in subnet.
		let weight_values : Vec<u16> = vec![88, 102, 303, 1212, 11]; // random value.
		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1, weight_keys, weight_values, 0);
		assert_eq!(result, Err(Error::<Test>::TooManyUids.into()));

		// Shouldnt fail because we are setting less weights than there are neurons.
		let weight_keys : Vec<u16> = vec![0, 1]; // Only on neurons that exist.
		let weight_values : Vec<u16> = vec![10, 10]; // random value.
		assert_ok!( SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1 , weight_keys, weight_values, 0)) ;
	});
}

// Tests that the weights set doesn't panic if you pass weights that sum to larger than u16 max.
#[test]
fn test_set_weights_sum_larger_than_u16_max() {
	new_test_ext().execute_with(|| {
        
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		
		register_ok_neuron(1, 1, 2, 100_000);
		let neuron_uid: u16 = SubtensorModule::get_uid_for_net_and_hotkey( netuid, &1 ).expect("Not registered.");
		SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);

		register_ok_neuron(1, 3, 4, 300_000);
		SubtensorModule::set_min_allowed_weights(1, 2);
	

		// Shouldn't fail because we are setting the right number of weights.
		let weight_keys : Vec<u16> = vec![0, 1];
		let weight_values : Vec<u16> = vec![u16::MAX, u16::MAX];
		// sum of weights is larger than u16 max.
		assert!( weight_values.iter().map(|x| *x as u64).sum::<u64>() > (u16::MAX as u64) );

		let result = SubtensorModule::set_weights(RuntimeOrigin::signed(1), 1, weight_keys, weight_values, 0);
		assert_ok!(result);

		let all_weights: Vec<Vec<I32F32>> = SubtensorModule::get_weights(netuid);
		let weights_set: Vec<u16> = all_weights[neuron_uid as usize].iter().map(|x| x.to_bits() as u16).collect();

		// Should sum less than u16 max.
		assert!(weights_set.iter().map(|x| *x as u64).sum::<u64>() <= (u16::MAX as u64) );

		// Should be normalized to 50% each.
		assert_eq!(weights_set, vec![u16::MAX/2, u16::MAX/2]);
	});
}