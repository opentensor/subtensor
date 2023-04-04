mod mock;
use mock::*;
use pallet_subtensor::Error;
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_system::Config;
use frame_support::sp_std::vec;
use frame_support::assert_ok;

#[test]
fn test_get_neuron_none() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let uid: u16 = 42;

        let neuron = SubtensorModule::get_neuron(netuid, uid);
        assert_eq!(neuron, None);
    });
}

#[test]
fn test_get_neuron_some() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        let uid: u16 = 0;
        let hotkey0: u64 = 0;
        let coldkey0: u64 = 0;

        add_network( netuid, tempo, modality );
        register_ok_neuron( netuid, hotkey0, coldkey0, 39420842 );

        let neuron = SubtensorModule::get_neuron(netuid, uid);
        assert_ne!(neuron, None);
    });
}

/* @TODO: Add more neurons to list */
#[test]
fn test_get_neurons_list() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network( netuid, tempo, modality );

        let uid: u16 = 42;

        let neuron_count = 1;
        for index in 0..neuron_count {
            let hotkey: u64 = 0 + index;
            let coldkey: u64 = 0 + index;
            let nonce: u64 = 39420842 + index;
            register_ok_neuron( netuid, hotkey, coldkey, nonce );
        }

        let neurons = SubtensorModule::get_neurons(netuid);
        assert_eq!(neurons.len(), neuron_count as usize);
    });
}

#[test]
fn test_get_neurons_empty() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;

        let neuron_count = 0;
        let neurons = SubtensorModule::get_neurons(netuid);
        assert_eq!(neurons.len(), neuron_count as usize);
    });
}
