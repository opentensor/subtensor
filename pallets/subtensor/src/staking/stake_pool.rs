use super::*;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    pub fn add_to_stake_pool_for_all_coldkeys(
        hotkey: &T::AccountId,
        netuid: u16,
        alpha_to_add: u64,
    ) {
        AlphaNP::<T>::mutate(hotkey, netuid, |alpha| {
            *alpha = alpha.saturating_add(alpha_to_add);
        });
        // Share proportions stay the same
    }

    pub fn add_to_stake_pool(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        alpha_to_add: u64,
    ) {
        // Add to total pool's Alpha
        let mut old_alpha: u64 = 0;
        AlphaNP::<T>::mutate(hotkey, netuid, |alpha| {
            old_alpha = *alpha;
            *alpha = alpha.saturating_add(alpha_to_add)
        });

        // Increase this coldkey's share and total shares to reflect added proportion
        let shares_to_add: u64;
        if old_alpha > 0 {
            let old_total_shares = TotalNominationPoolShares::<T>::get(hotkey, netuid);
            let shares_per_alpha: I96F32 = I96F32::from_num(old_total_shares).saturating_div(
                I96F32::from_num(old_alpha)
            );
            shares_to_add = shares_per_alpha.saturating_mul(
                I96F32::from_num(alpha_to_add)
            ).to_num();
        } else {
            shares_to_add = alpha_to_add;
        }

        NominationPoolShares::<T>::mutate((hotkey, coldkey, netuid), |share| 
            *share = share.saturating_add(shares_to_add)
        );
        TotalNominationPoolShares::<T>::mutate(hotkey, netuid, |share| 
            *share = share.saturating_add(shares_to_add)
        );
    }

    pub fn remove_from_stake_pool(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        alpha_to_remove: u64,
    ) {
        // In case if alpha_to_remove is greater than total existing alpha, we cap at total existing
        // with no errors. Staking error checks are to be done in the higher level functions, here we 
        // only care about math precision.
        let old_alpha: u64 = AlphaNP::<T>::get(hotkey, netuid);
        let alpha_to_remove = old_alpha.min(alpha_to_remove);

        // Remove from total pool's Alpha
        AlphaNP::<T>::mutate(hotkey, netuid, |alpha| {
            *alpha = alpha.saturating_sub(alpha_to_remove)
        });

        // Decrease this coldkey's share and total shares to reflect removed proportion
        if old_alpha > 0 {
            let old_total_shares = TotalNominationPoolShares::<T>::get(hotkey, netuid);
            let shares_per_alpha: I96F32 = I96F32::from_num(old_total_shares).saturating_div(
                I96F32::from_num(old_alpha)
            );
            let shares_to_remove = shares_per_alpha.saturating_mul(
                I96F32::from_num(alpha_to_remove)
            ).to_num();

            let shares = NominationPoolShares::<T>::get((hotkey, coldkey, netuid));
            let new_shares = shares.saturating_add(shares_to_remove);
            if new_shares > 0 {
                NominationPoolShares::<T>::insert((hotkey, coldkey, netuid), new_shares);
            } else {
                NominationPoolShares::<T>::remove((hotkey, coldkey, netuid));
            }

            TotalNominationPoolShares::<T>::mutate(hotkey, netuid, |share| {
                *share = share.saturating_add(shares_to_remove)
            });
        }
    }

    pub fn get_total_coldkey_alpha(
        coldkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        StakingHotkeys::<T>::get(coldkey).iter().map(|hotkey| {
            Self::get_hotkey_coldkey_alpha(hotkey, coldkey, netuid)
        }).sum::<u64>()
    }

    pub fn get_total_hotkey_alpha(
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        AlphaNP::<T>::get(hotkey, netuid)
    }

    pub fn get_hotkey_coldkey_alpha(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        let total_shares = TotalNominationPoolShares::<T>::get(hotkey, netuid);
        if total_shares > 0 {
            let total_alpha: u64 = AlphaNP::<T>::get(hotkey, netuid);
            let coldkey_shares = NominationPoolShares::<T>::get((hotkey, coldkey, netuid));
            let alpha_per_share: I96F32 = I96F32::from_num(total_alpha).saturating_div(
                I96F32::from_num(total_shares)
            );
            alpha_per_share.saturating_mul(
                I96F32::from_num(coldkey_shares)
            ).to_num()
        } else {
            0
        }
    }

    // pub fn get_hotkey_coldkey_tao(
    //     hotkey: &T::AccountId,
    //     coldkey: &T::AccountId,
    //     netuid: u16,
    // ) -> u64 {

    // }

    // pub fn get_total_coldkey_tao(
    //     coldkey: &T::AccountId,
    //     netuid: u16,
    // ) -> u64 {

    // }

    // pub fn get_total_hotkey_tao(
    //     hotkey: &T::AccountId,
    //     netuid: u16,
    // ) -> u64 {

    // }

}
