use frame_support::pallet_macros::pallet_section;
/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod genesis {

    fn init_network<T: Config>(netuid: u16, tempo: u16) {
        // The functions for initializing new networks/setting defaults cannot be run directly from genesis functions like extrinsics would
        // --- Set this network uid to alive.
        NetworksAdded::<T>::insert(netuid, true);

        // --- Fill tempo memory item.
        Tempo::<T>::insert(netuid, tempo);

        // --- Fill modality item.
        // Only modality 0 exists (text)
        NetworkModality::<T>::insert(netuid, 0);

        // Make network parameters explicit.
        if !Tempo::<T>::contains_key(netuid) {
            Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
        }
        if !Kappa::<T>::contains_key(netuid) {
            Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
        }
        if !Difficulty::<T>::contains_key(netuid) {
            Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
        }
        if !MaxAllowedUids::<T>::contains_key(netuid) {
            MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
        }
        if !ImmunityPeriod::<T>::contains_key(netuid) {
            ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
        }
        if !ActivityCutoff::<T>::contains_key(netuid) {
            ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
        }
        if !EmissionValues::<T>::contains_key(netuid) {
            EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));
        }
        if !MaxWeightsLimit::<T>::contains_key(netuid) {
            MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
        }
        if !MinAllowedWeights::<T>::contains_key(netuid) {
            MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
        }
        if !RegistrationsThisInterval::<T>::contains_key(netuid) {
            RegistrationsThisInterval::<T>::insert(
                netuid,
                RegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !POWRegistrationsThisInterval::<T>::contains_key(netuid) {
            POWRegistrationsThisInterval::<T>::insert(
                netuid,
                POWRegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) {
            BurnRegistrationsThisInterval::<T>::insert(
                netuid,
                BurnRegistrationsThisInterval::<T>::get(netuid),
            );
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial total issuance from balances
            TotalIssuance::<T>::put(self.balances_issuance);

            if self.initialize_network_3 {
                let tempo = 99;

                init_network::<T>(3, tempo);

                let netuid: u16 = 3;
                let max_uids = 4096;

                // Set max allowed uids
                MaxAllowedUids::<T>::insert(netuid, max_uids);

                let mut next_uid: u16 = 0;

                for (coldkey, hotkeys) in self.stakes.iter() {
                    for (hotkey, stake_uid) in hotkeys.iter() {
                        let (stake, uid) = stake_uid;

                        // Expand Yuma Consensus with new position.
                        Rank::<T>::mutate(netuid, |v| v.push(0));
                        Trust::<T>::mutate(netuid, |v| v.push(0));
                        Active::<T>::mutate(netuid, |v| v.push(true));
                        Emission::<T>::mutate(netuid, |v| v.push(0));
                        Consensus::<T>::mutate(netuid, |v| v.push(0));
                        Incentive::<T>::mutate(netuid, |v| v.push(0));
                        Dividends::<T>::mutate(netuid, |v| v.push(0));
                        LastUpdate::<T>::mutate(netuid, |v| v.push(0));
                        PruningScores::<T>::mutate(netuid, |v| v.push(0));
                        ValidatorTrust::<T>::mutate(netuid, |v| v.push(0));
                        ValidatorPermit::<T>::mutate(netuid, |v| v.push(false));

                        // Insert account information.
                        Keys::<T>::insert(netuid, uid, hotkey.clone()); // Make hotkey - uid association.
                        Uids::<T>::insert(netuid, hotkey.clone(), uid); // Make uid - hotkey association.
                        BlockAtRegistration::<T>::insert(netuid, uid, 0); // Fill block at registration.
                        IsNetworkMember::<T>::insert(hotkey.clone(), netuid, true); // Fill network is member.

                        // Fill stake information.
                        Owner::<T>::insert(hotkey.clone(), coldkey.clone());

                        TotalHotkeyStake::<T>::insert(hotkey.clone(), stake);
                        TotalColdkeyStake::<T>::insert(
                            coldkey.clone(),
                            TotalColdkeyStake::<T>::get(coldkey).saturating_add(*stake),
                        );

                        // Update total issuance value
                        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(*stake));

                        Stake::<T>::insert(hotkey.clone(), coldkey.clone(), stake);

                        next_uid = next_uid.saturating_add(1);
                    }
                }

                // Set correct length for Subnet neurons
                SubnetworkN::<T>::insert(netuid, next_uid);

                // --- Increase total network count.
                TotalNetworks::<T>::mutate(|n| *n = n.saturating_add(1));
            }

            // Get the root network uid.
            let root_netuid: u16 = 0;

            // Set the root network as added.
            NetworksAdded::<T>::insert(root_netuid, true);

            // Increment the number of total networks.
            TotalNetworks::<T>::mutate(|n| *n = n.saturating_add(1));

            // Set the number of validators to 0.
            SubnetworkN::<T>::insert(root_netuid, 0);

            // Set the maximum number to the number of senate members.
            MaxAllowedUids::<T>::insert(root_netuid, 64u16);

            // Set the maximum number to the number of validators to all members.
            MaxAllowedValidators::<T>::insert(root_netuid, 64u16);

            // Set the min allowed weights to zero, no weights restrictions.
            MinAllowedWeights::<T>::insert(root_netuid, 0);

            // Set the max weight limit to infitiy, no weight restrictions.
            MaxWeightsLimit::<T>::insert(root_netuid, u16::MAX);

            // Add default root tempo.
            Tempo::<T>::insert(root_netuid, 100);

            // Set the root network as open.
            NetworkRegistrationAllowed::<T>::insert(root_netuid, true);

            // Set target registrations for validators as 1 per block.
            TargetRegistrationsPerInterval::<T>::insert(root_netuid, 1);
        }
    }
}
