#![cfg_attr(not(feature = "std"), no_std)]
use pallet_subtensor::InfoResponse;
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
	pub trait DelegateInfoRuntimeApi {
		fn get_delegates() -> InfoResponse;
		fn get_delegate( delegate_account_vec: Vec<u8> ) -> InfoResponse;
	}

	pub trait NeuronInfoRuntimeApi {
		fn get_neurons(netuid: u16) -> InfoResponse;
		fn get_neuron(netuid: u16, uid: u16) -> InfoResponse;
	}

	pub trait SubnetInfoRuntimeApi {
		fn get_subnet_info(netuid: u16) -> InfoResponse;
		fn get_subnets_info() -> InfoResponse;
	}
}