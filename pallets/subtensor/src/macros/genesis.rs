use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod genesis {

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial total issuance from balances
            TotalIssuance::<T>::put(self.balances_issuance);
            
            // Get the root network uid.
            let root_netuid: u16 = 0;

            // Set the root network as added.
            NetworksAdded::<T>::insert(root_netuid, true);

            // Increment the number of total networks.
            TotalNetworks::<T>::mutate(|n| *n = n.saturating_add(1));

            // Set the number of validators to 1.
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
