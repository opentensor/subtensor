mod mock;
use mock::*;
use pallet_subtensor::Error;
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_system::Config;
use frame_support::sp_std::vec;
use frame_support::assert_ok;

#[test]
fn test_get_neuron_ok() {
    println!("WOOT");

    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let uid: u16 = 42;
        let hotkey0: u64 = 0;
        let coldkey0: u64 = 0;
        register_ok_neuron( netuid, hotkey0, coldkey0, 39420842 );
        assert_eq!(SubtensorModule::get_neuron(netuid, uid), None);
        // assert_eq!(SubtensorModule::get_neuron(neuron_uid, uid), "WAT");
    });
}

