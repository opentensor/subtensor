use super::*;
extern crate alloc;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::storage::IterableStorageDoubleMap;


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetState<T: Config> {
    // netuid: Compact<u16>,
    hotkeys: Vec<T::AccountId>,
    // coldkeys: Vec<T::AccountId>,
    // active: Vec<bool>,
    // axon_info: Vec<AxonInfo>,
    // local: Vec<Compact<u64>>,
    // global: Vec<Compact<u64>>,
    // alpha_stake: Compact<u64>,
    // tao_stake: Compact<u64>,
    // incentive: Compact<u16>,
    // consensus: Compact<u16>,
    // trust: Compact<u16>,
    // validator_trust: Compact<u16>,
    // dividends: Compact<u16>,
    // last_update: Compact<u64>,
    // validator_permit: bool,
    // // has no weights or bonds
    // pruning_score: Compact<u16>,
}

impl<T: Config> Pallet<T> {
    pub fn get_subnet_state(netuid: u16) -> Vec<NeuronInfo<T>> {
        if !Self::if_subnet_exist(netuid) { return Vec::new() }
        let mut neurons = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        let mut hotkeys: Vec<T::AccountId>;
        for hotkey in hotkeys.iter() {
            hotkeys.push(hotkey);
        }
        Some( SubnetState {
            hotkeys: Keys::<T>::get(netuid),
        } )
        neurons
    }
}
