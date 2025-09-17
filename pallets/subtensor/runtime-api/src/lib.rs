#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use codec::Compact;
use pallet_subtensor::rpc_info::{
    delegate_info::DelegateInfo,
    dynamic_info::DynamicInfo,
    metagraph::{Metagraph, SelectiveMetagraph},
    neuron_info::{NeuronInfo, NeuronInfoLite},
    show_subnet::SubnetState,
    stake_info::StakeInfo,
    subnet_info::{SubnetHyperparams, SubnetHyperparamsV2, SubnetInfo, SubnetInfov2},
};
use sp_runtime::AccountId32;
use subtensor_runtime_common::{AlphaCurrency, MechId, NetUid, TaoCurrency};

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
    pub trait DelegateInfoRuntimeApi {
        fn get_delegates() -> Vec<DelegateInfo<AccountId32>>;
        fn get_delegate( delegate_account: AccountId32 ) -> Option<DelegateInfo<AccountId32>>;
        fn get_delegated( delegatee_account: AccountId32 ) -> Vec<(DelegateInfo<AccountId32>, (Compact<NetUid>, Compact<AlphaCurrency>))>;
    }

    pub trait NeuronInfoRuntimeApi {
        fn get_neurons(netuid: NetUid) -> Vec<NeuronInfo<AccountId32>>;
        fn get_neuron(netuid: NetUid, uid: u16) -> Option<NeuronInfo<AccountId32>>;
        fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<AccountId32>>;
        fn get_neuron_lite(netuid: NetUid, uid: u16) -> Option<NeuronInfoLite<AccountId32>>;
    }

    pub trait SubnetInfoRuntimeApi {
        fn get_subnet_info(netuid: NetUid) -> Option<SubnetInfo<AccountId32>>;
        fn get_subnets_info() -> Vec<Option<SubnetInfo<AccountId32>>>;
        fn get_subnet_info_v2(netuid: NetUid) -> Option<SubnetInfov2<AccountId32>>;
        fn get_subnets_info_v2() -> Vec<Option<SubnetInfov2<AccountId32>>>;
        fn get_subnet_hyperparams(netuid: NetUid) -> Option<SubnetHyperparams>;
        fn get_subnet_hyperparams_v2(netuid: NetUid) -> Option<SubnetHyperparamsV2>;
        fn get_all_dynamic_info() -> Vec<Option<DynamicInfo<AccountId32>>>;
        fn get_all_metagraphs() -> Vec<Option<Metagraph<AccountId32>>>;
        fn get_metagraph(netuid: NetUid) -> Option<Metagraph<AccountId32>>;
        fn get_all_submetagraphs() -> Vec<Option<Metagraph<AccountId32>>>;
        fn get_submetagraph(netuid: NetUid, mecid: MechId) -> Option<Metagraph<AccountId32>>;
        fn get_dynamic_info(netuid: NetUid) -> Option<DynamicInfo<AccountId32>>;
        fn get_subnet_state(netuid: NetUid) -> Option<SubnetState<AccountId32>>;
        fn get_selective_metagraph(netuid: NetUid, metagraph_indexes: Vec<u16>) -> Option<SelectiveMetagraph<AccountId32>>;
        fn get_selective_submetagraph(netuid: NetUid, mecid: MechId, metagraph_indexes: Vec<u16>) -> Option<SelectiveMetagraph<AccountId32>>;
    }

    pub trait StakeInfoRuntimeApi {
        fn get_stake_info_for_coldkey( coldkey_account: AccountId32 ) -> Vec<StakeInfo<AccountId32>>;
        fn get_stake_info_for_coldkeys( coldkey_accounts: Vec<AccountId32> ) -> Vec<(AccountId32, Vec<StakeInfo<AccountId32>>)>;
        fn get_stake_info_for_hotkey_coldkey_netuid( hotkey_account: AccountId32, coldkey_account: AccountId32, netuid: NetUid ) -> Option<StakeInfo<AccountId32>>;
        fn get_stake_fee( origin: Option<(AccountId32, NetUid)>, origin_coldkey_account: AccountId32, destination: Option<(AccountId32, NetUid)>, destination_coldkey_account: AccountId32, amount: u64 ) -> u64;
    }

    pub trait SubnetRegistrationRuntimeApi {
        fn get_network_registration_cost() -> TaoCurrency;
    }
}
