use super::*;
extern crate alloc;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetState<T: Config> {
    netuid: Compact<u16>,
    hotkeys: Vec<T::AccountId>,
    coldkeys: Vec<T::AccountId>,
    active: Vec<bool>,
    validator_permit: Vec<bool>,
    pruning_score: Vec<Compact<u16>>,
    last_update: Vec<Compact<u64>>,
    emission: Vec<Compact<u64>>,
    dividends: Vec<Compact<u16>>,
    incentives: Vec<Compact<u16>>,
    consensus: Vec<Compact<u16>>,
    trust: Vec<Compact<u16>>,
    rank: Vec<Compact<u16>>,
    block_at_registration: Vec<Compact<u64>>,
    local_stake: Vec<Compact<u64>>,
    global_stake: Vec<Compact<u64>>,
    stake_weight: Vec<Compact<u16>>,
    // identities: Vec<ChainIdentityOf>,
    // tao_stake: Compact<u64>,
    // incentive: Compact<u16>,
    // consensus: Compact<u16>,
    // trust: Compact<u16>,
    // validator_trust: Compact<u16>,
    // dividends: Compact<u16>,
    // // has no weights or bonds
}

impl<T: Config> Pallet<T> {
    pub fn get_subnet_state(netuid: u16) -> Option<SubnetState<T>> {
        if !Self::if_subnet_exist(netuid) { return None; }
        let n: u16 = Self::get_subnetwork_n(netuid);
        let mut hotkeys: Vec<T::AccountId> = vec![];
        let mut coldkeys: Vec<T::AccountId> = vec![];
        let mut block_at_registration: Vec<Compact<u64>> = vec![];
        // let mut identities: Vec<ChainIdentityOf> = vec![];
        for uid in 0..n {
            let hotkey = Keys::<T>::get(netuid, uid);
            let coldkey = Owner::<T>::get( hotkey.clone() );
            hotkeys.push( hotkey );
            coldkeys.push( coldkey );
            block_at_registration.push( BlockAtRegistration::<T>::get( netuid, uid ).into() );
            // identities.push( Identities::<T>::get( coldkey.clone() ) );
        }
        let active: Vec<bool> = Active::<T>::get( netuid );
        let validator_permit: Vec<bool> = ValidatorPermit::<T>::get( netuid );
        let pruning_score: Vec<Compact<u16>> = PruningScores::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let last_update: Vec<Compact<u64>> = LastUpdate::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let emission: Vec<Compact<u64>> = Emission::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let dividends: Vec<Compact<u16>> = Dividends::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let incentives: Vec<Compact<u16>> = Incentive::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let consensus: Vec<Compact<u16>> = Consensus::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let trust: Vec<Compact<u16>> = Trust::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let rank: Vec<Compact<u16>> = Rank::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let local_stake: Vec<Compact<u64>> = LocalStake::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let global_stake: Vec<Compact<u64>> = GlobalStake::<T>::get(netuid).into_iter().map(Compact::from).collect();
        let stake_weight: Vec<Compact<u16>> = StakeWeight::<T>::get(netuid).into_iter().map(Compact::from).collect();
        Some( SubnetState {
            netuid: netuid.into(),
            hotkeys: hotkeys.into(),
            coldkeys: coldkeys.into(),
            active: active.into(),
            validator_permit: validator_permit.into(),
            pruning_score: pruning_score.into(),
            last_update: last_update.into(),
            emission: emission.into(),
            dividends: dividends.into(),
            incentives: incentives.into(),
            consensus: consensus.into(),
            trust: trust.into(),
            rank: rank.into(),
            block_at_registration: block_at_registration.into(),
            local_stake: local_stake.into(),
            global_stake: global_stake.into(),
            stake_weight: stake_weight.into(),
        } )
    }
}
