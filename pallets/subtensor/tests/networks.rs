mod mock;
use mock::*;
use pallet_subtensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_system::Config;
use frame_support::{sp_std::vec};
use frame_support::{assert_ok};

/*TO DO SAM: write test for LatuUpdate after it is set */

// --- add network tests ----
// #[test]
fn test_add_network_dispatch_info_ok() { 
        new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let modality = 0;
        let tempo: u16 = 13;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::sudo_add_network{netuid, tempo, modality});
        assert_eq!(call.get_dispatch_info(), 
                DispatchInfo {
                        weight: frame_support::weights::Weight::from_ref_time(0),
                        class: DispatchClass::Operational,
                        pays_fee: Pays::No
                });
});}

#[test]
fn test_add_network() { 
        new_test_ext().execute_with(|| {
        let modality = 0;
        let tempo: u16 = 13;
        add_network(10, tempo, modality);
        assert_eq!(SubtensorModule::get_number_of_subnets(), 1);
        add_network( 20, tempo, modality);
        assert_eq!(SubtensorModule::get_number_of_subnets(), 2); 
});}

#[test]
fn test_add_network_check_tempo() { 
        new_test_ext().execute_with(|| {
        let modality = 0; 
        let tempo: u16 = 13;
        assert_eq!(SubtensorModule::get_tempo(1), 0);
	add_network(1, tempo, modality);
        assert_eq!(SubtensorModule::get_tempo(1), 13);
});}

#[test]
fn test_clear_min_allowed_weight_for_network() { 
        new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let min_allowed_weight = 2;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
	register_ok_neuron( 1, 55, 66, 0);
        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weight);
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), 2);
        assert_ok!(SubtensorModule::do_remove_network(<<Test as Config>::RuntimeOrigin>::root(), netuid));
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), 0);
});}

#[test]
fn test_remove_uid_for_network() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
	register_ok_neuron( 1, 55, 66, 0);
        let neuron_id ;
        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &55) {
            Ok(k) => neuron_id = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &55).is_ok());
        assert_eq!(neuron_id, 0);
        register_ok_neuron( 1, 56, 67, 300000);
        let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &56).unwrap();
        assert_eq!(neuron_uid, 1);
        assert_ok!(SubtensorModule::do_remove_network(<<Test as Config>::RuntimeOrigin>::root(), netuid));
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &55).is_err());
});}

#[test]
fn test_remove_difficulty_for_network() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let difficulty: u64 = 10;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
	register_ok_neuron( 1, 55, 66, 0);
        assert_ok!(SubtensorModule::sudo_set_difficulty(<<Test as Config>::RuntimeOrigin>::root(), netuid, difficulty));
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), difficulty);
        assert_ok!(SubtensorModule::do_remove_network(<<Test as Config>::RuntimeOrigin>::root(), netuid));
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), 10000);
});}


#[test]
fn test_remove_network_for_all_hotkeys() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        register_ok_neuron( 1, 55, 66, 0);
        register_ok_neuron( 1, 77, 88, 65536);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 2);
        assert_ok!(SubtensorModule::do_remove_network(<<Test as Config>::RuntimeOrigin>::root(), netuid));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);
});}

#[test]
fn test_network_set_default_value_for_other_parameters() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        add_network(netuid, tempo, 0);
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid), 0);
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), u16::MAX);
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), 10000);
	assert_eq!(SubtensorModule::get_immunity_period(netuid), 2);
        
});}

// --- Set Emission Ratios Tests
// #[test]
fn test_network_set_emission_ratios_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
        let netuids: Vec<u16> = vec![ 1,2 ]; 
        let emission: Vec<u64> = vec![ 100000000, 900000000 ]; 
        let call = RuntimeCall::SubtensorModule(SubtensorCall::sudo_set_emission_values{ netuids, emission });
        assert_eq!(call.get_dispatch_info(), DispatchInfo {
                weight: frame_support::weights::Weight::from_ref_time(0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
        });
});}

#[test]
fn test_network_set_emission_ratios_ok() {
	new_test_ext().execute_with(|| {
        let netuids: Vec<u16> = vec![ 1,2 ]; 
        let emission: Vec<u64> = vec![ 100000000, 900000000 ]; 
        add_network(1, 0, 0);
        add_network(2, 0, 0);
        assert_ok!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission ) );
});}

#[test]
fn test_network_set_emission_ratios_fail_summation() {
	new_test_ext().execute_with(|| {
        let netuids: Vec<u16> = vec![ 1, 2 ]; 
        let emission: Vec<u64> = vec![ 100000000, 910000000 ]; 
        add_network(1, 0, 0);
        add_network(2, 0, 0);
        assert_eq!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission ), Err(Error::<Test>::InvalidEmissionValues.into()) );
});}

#[test]
fn test_network_set_emission_invalid_netuids() {
	new_test_ext().execute_with(|| {
        let netuids: Vec<u16> = vec![ 1, 2 ]; 
        let emission: Vec<u64> = vec![ 100000000, 900000000 ]; 
        add_network(1, 0, 0);
        assert_eq!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission ), Err(Error::<Test>::NotSettingEnoughWeights.into()) );
});}

#[test]
fn test_network_set_emission_ratios_fail_net() {
	new_test_ext().execute_with(|| {
        let netuids: Vec<u16> = vec![ 1, 2 ]; 
        let emission: Vec<u64> = vec![ 100000000, 900000000 ]; 
        add_network(1, 0, 0);
        add_network(3, 0, 0);
        assert_eq!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission ), Err(Error::<Test>::InvalidUid.into()) );
});}

#[test]
fn test_add_difficulty_fail(){
new_test_ext().execute_with(|| { 
        let netuid: u16 = 1;
        assert_eq!(SubtensorModule::sudo_set_difficulty(<<Test as Config>::RuntimeOrigin>::root(), netuid, 120000) , Err(Error::<Test>::NetworkDoesNotExist.into()) );
});}



#[test]
fn test_multi_tempo_with_emission(){
new_test_ext().execute_with(|| { 
        let netuid: u16 = 1;
        assert_eq!(SubtensorModule::sudo_set_difficulty(<<Test as Config>::RuntimeOrigin>::root(), netuid, 120000) , Err(Error::<Test>::NetworkDoesNotExist.into()) );
});}

