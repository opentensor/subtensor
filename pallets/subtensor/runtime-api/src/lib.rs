#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use pallet_subtensor::types::TensorBytes;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
    pub trait DelegateInfoRuntimeApi {
        fn get_substake_for_hotkey( hotkey_bytes: Vec<u8>  ) -> Vec<u8>;
        fn get_substake_for_coldkey( coldkey_bytes: Vec<u8>  ) -> Vec<u8>;
        fn get_substake_for_netuid( netuid: u16 ) -> Vec<u8>;
        fn get_total_stake_for_hotkey( hotkey_bytes: Vec<u8>  ) -> u64;
        fn get_total_stake_for_coldkey( coldkey_bytes: Vec<u8>  ) -> u64;
        fn get_delegates() -> Vec<u8>;
        fn get_delegate( delegate_account_vec: Vec<u8> ) -> Vec<u8>;
        fn get_delegated( delegatee_account_vec: Vec<u8> ) -> Vec<u8>;
    }

    pub trait NeuronInfoRuntimeApi {
        fn get_neurons(netuid: u16) -> Vec<u8>;
        fn get_neuron(netuid: u16, uid: u16) -> Vec<u8>;
        fn get_neurons_lite(netuid: u16) -> Vec<u8>;
        fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8>;
    }

    pub trait SubnetInfoRuntimeApi {
        fn get_subnet_info(netuid: u16) -> Vec<u8>;
        fn get_subnets_info() -> Vec<u8>;
        fn get_subnet_hyperparams(netuid: u16) -> Vec<u8>;
    
        fn get_subnet_info_v2(netuid: u16) -> Vec<u8>;
        fn get_subnets_info_v2() -> Vec<u8>;
    }

    pub trait StakeInfoRuntimeApi {
        fn get_stake_info_for_coldkey( coldkey_account_vec: TensorBytes ) -> Vec<u8>;
        fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<TensorBytes> ) -> Vec<u8>;
        fn get_subnet_stake_info_for_coldkeys( coldkey_account_vecs: Vec<TensorBytes>, netuid: u16 ) -> Vec<u8>;
        fn get_subnet_stake_info_for_coldkey( coldkey_account_vec: TensorBytes , netuid: u16) -> Vec<u8>;
        fn get_total_subnet_stake( netuid: u16 ) -> Vec<u8>;
        fn get_all_stake_info_for_coldkey( coldkey_account_vec: TensorBytes ) -> Vec<u8>;
        fn get_all_subnet_stake_info_for_coldkey( coldkey_account_vec: TensorBytes ) -> Vec<u8>;
        fn get_total_stake_for_each_subnet() -> Vec<u8>;
    }

    pub trait SubnetRegistrationRuntimeApi {
        fn get_network_registration_cost() -> u64;
    }

    pub trait DynamicPoolInfoRuntimeApi {
        fn get_dynamic_pool_info(netuid: u16) -> Vec<u8>;
        fn get_all_dynamic_pool_infos() -> Vec<u8>;

        fn get_dynamic_pool_info_v2(netuid: u16) -> Vec<u8>;
        fn get_all_dynamic_pool_infos_v2() -> Vec<u8>;
    }
}
