#![cfg_attr(not(feature = "std"), no_std)]
use pallet_subtensor::delegate_info::DelegateInfo as DelegateInfoStruct;
use pallet_subtensor::neuron_info::NeuronInfo as NeuronInfoStruct;
use pallet_subtensor::subnet_info::SubnetInfo as SubnetInfoStruct;
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
	pub trait DelegateInfoRuntimeApi {
		fn get_delegates() -> Vec<DelegateInfoStruct>;
		fn get_delegate( delegate_account_vec: Vec<u8> ) -> Option<DelegateInfoStruct>;
	}

	pub trait NeuronInfoRuntimeApi {
		fn get_neurons(netuid: u16) -> Vec<NeuronInfoStruct>;
		fn get_neuron(netuid: u16, uid: u16) -> Option<NeuronInfoStruct>;
	}

	pub trait SubnetInfoRuntimeApi {
		fn get_subnet_info(netuid: u16) -> Option<SubnetInfoStruct>;
		fn get_subnets_info() -> Vec<Option<SubnetInfoStruct>>;
	}
}