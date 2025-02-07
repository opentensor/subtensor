use super::*;

/// Module containing deprecated storage format for Stake
pub mod deprecated_stake_format {
    use super::*;
    use frame_support::{pallet_prelude::ValueQuery, storage_alias, Blake2_128Concat, Identity};

    #[storage_alias]
    pub(super) type Stake<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        Identity,
        <T as frame_system::Config>::AccountId,
        u64,
        ValueQuery,
    >;
}

pub mod migrate_rao {
    use super::*;
    use codec::MaxEncodedLen;
    use frame_support::{
        migrations::{SteppedMigration, SteppedMigrationError},
        traits::Get,
        weights::{Weight, WeightMeter},
    };
    use substrate_fixed::types::U64F64;

    #[derive(Decode, Encode, MaxEncodedLen, Eq, PartialEq)]
    pub enum MigrationState<S, C> {
        DynamicBlockSet,
        Stake(S),
        FinishedStake,
        ConvertSubnet(C),
        Finished,
    }

    pub struct Migration<T: Config>(PhantomData<T>);

    impl<T: Config> SteppedMigration for Migration<T> {
        type Cursor = MigrationState<(T::AccountId, T::AccountId), u16>;
        type Identifier = [u8; 11];

        fn id() -> Self::Identifier {
            *b"migrate_rao"
        }

        fn max_steps() -> Option<u32> {
            Some(32) // TODO: Make sure to change this to something that makes sense
        }

        fn step(
            mut cursor: Option<Self::Cursor>,
            meter: &mut WeightMeter,
        ) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
            let migration_name = Self::id();
            if HasMigrationRun::<T>::get(migration_name.as_ref()) {
                log::info!(
                    "Migration '{:?}' has already run. Skipping.",
                    String::from_utf8_lossy(migration_name.as_ref())
                );
                return Ok(None);
            }

            let mut required_weight = Self::required_weight(cursor.as_ref());

            while meter.try_consume(required_weight).is_ok() {
                let next = match &cursor {
                    None => Self::dynamic_block_step(),
                    Some(MigrationState::DynamicBlockSet) => Self::stake_step(None),
                    Some(MigrationState::Stake(key)) => Self::stake_step(Some(key)),
                    Some(MigrationState::FinishedStake) => {
                        TaoWeight::<T>::set(332_041_393_326_771_929);
                        Self::convert_subnet_step(None)
                    }
                    Some(MigrationState::ConvertSubnet(key)) => {
                        Self::convert_subnet_step(Some(key))
                    }
                    Some(MigrationState::Finished) => {
                        HasMigrationRun::<T>::insert(migration_name.as_ref(), true);

                        log::info!(
                            "Migration '{:?}' completed.",
                            String::from_utf8_lossy(migration_name.as_ref())
                        );
                        return Ok(None);
                    }
                };

                cursor = Some(next);
                required_weight = Self::required_weight(cursor.as_ref());
            }

            Ok(cursor)
        }
    }

    impl<T: Config> Migration<T> {
        fn required_weight(
            step: Option<&MigrationState<(T::AccountId, T::AccountId), u16>>,
        ) -> Weight {
            match step {
                None => T::DbWeight::get().writes(1),
                Some(MigrationState::DynamicBlockSet | MigrationState::Stake(_)) => {
                    T::DbWeight::get().reads_writes(6, 6)
                }
                Some(MigrationState::FinishedStake) => T::DbWeight::get().reads_writes(1, 4),
                Some(MigrationState::ConvertSubnet(_)) => T::DbWeight::get().reads_writes(25, 33),
                Some(MigrationState::Finished) => T::DbWeight::get().writes(1),
            }
        }

        fn dynamic_block_step() -> MigrationState<(T::AccountId, T::AccountId), u16> {
            DynamicBlock::<T>::set(Pallet::<T>::get_current_block_as_u64());
            MigrationState::DynamicBlockSet
        }

        fn stake_step(
            maybe_last_key: Option<&(T::AccountId, T::AccountId)>,
        ) -> MigrationState<(T::AccountId, T::AccountId), u16> {
            let mut iter = if let Some((last_key1, last_key2)) = maybe_last_key {
                deprecated_stake_format::Stake::<T>::iter_from(
                    deprecated_stake_format::Stake::<T>::hashed_key_for(last_key1, last_key2),
                )
            } else {
                deprecated_stake_format::Stake::<T>::iter()
            };

            if let Some((hotkey, coldkey, stake)) = iter.next() {
                // Increase SubnetTAO on root.
                SubnetTAO::<T>::mutate(0, |total| {
                    *total = total.saturating_add(stake);
                });
                // Increase SubnetAlphaOut on root.
                SubnetAlphaOut::<T>::mutate(0, |total| {
                    *total = total.saturating_add(stake);
                });
                // Set all the stake on root 0 subnet.
                Alpha::<T>::mutate((hotkey.clone(), coldkey.clone(), 0), |total| {
                    *total = total.saturating_add(U64F64::from_num(stake))
                });
                TotalHotkeyShares::<T>::mutate(hotkey.clone(), 0, |total| {
                    *total = total.saturating_add(U64F64::from_num(stake))
                });
                // Set the total stake on the hotkey
                TotalHotkeyAlpha::<T>::mutate(hotkey.clone(), 0, |total| {
                    *total = total.saturating_add(stake)
                });

                MigrationState::Stake((hotkey, coldkey))
            } else {
                MigrationState::FinishedStake
            }
        }

        fn convert_subnet_step(
            maybe_last_key: Option<&u16>,
        ) -> MigrationState<(T::AccountId, T::AccountId), u16> {
            let mut iter = if let Some(last_key) = maybe_last_key {
                NetworksAdded::<T>::iter_from(NetworksAdded::<T>::hashed_key_for(last_key))
            } else {
                NetworksAdded::<T>::iter()
            };

            if let Some((netuid, _)) = iter.next() {
                if netuid == 0 {
                    // Give root a single RAO in pool to avoid any catestrophic division by zero.
                    SubnetAlphaIn::<T>::insert(netuid, 1);
                    SubnetMechanism::<T>::insert(netuid, 0); // Set to zero mechanism.
                    TokenSymbol::<T>::insert(netuid, Pallet::<T>::get_symbol_for_subnet(0));
                    return MigrationState::ConvertSubnet(0);
                }
                let owner = SubnetOwner::<T>::get(netuid);
                let lock = SubnetLocked::<T>::get(netuid);

                // Put initial TAO from lock into subnet TAO and produce numerically equal amount of Alpha
                // The initial TAO is the locked amount, with a minimum of 1 RAO and a cap of 100 TAO.
                let pool_initial_tao = Pallet::<T>::get_network_min_lock();
                if lock < pool_initial_tao {
                    let difference: u64 = pool_initial_tao.saturating_sub(lock);
                    TotalIssuance::<T>::mutate(|total| {
                        *total = total.saturating_add(difference);
                    });
                }

                let remaining_lock = lock.saturating_sub(pool_initial_tao);
                // Refund the owner for the remaining lock.
                Pallet::<T>::add_balance_to_coldkey_account(&owner, remaining_lock);
                SubnetLocked::<T>::insert(netuid, 0); // Clear lock amount.
                SubnetTAO::<T>::insert(netuid, pool_initial_tao);
                TotalStake::<T>::mutate(|total| {
                    *total = total.saturating_add(pool_initial_tao);
                }); // Increase total stake.
                SubnetAlphaIn::<T>::insert(netuid, pool_initial_tao); // Set initial alpha to pool initial tao.
                SubnetAlphaOut::<T>::insert(netuid, 0); // Set zero subnet alpha out.
                SubnetMechanism::<T>::insert(netuid, 1); // Convert to dynamic immediately with initialization.
                Tempo::<T>::insert(netuid, DefaultTempo::<T>::get());
                // Set the token symbol for this subnet using Self instead of Pallet::<T>
                TokenSymbol::<T>::insert(netuid, Pallet::<T>::get_symbol_for_subnet(netuid));
                TotalStakeAtDynamic::<T>::insert(netuid, 0);

                if let Ok(owner_coldkey) = SubnetOwner::<T>::try_get(netuid) {
                    // Set Owner as the coldkey.
                    SubnetOwnerHotkey::<T>::insert(netuid, owner_coldkey.clone());
                    // Associate the coldkey to coldkey.
                    Pallet::<T>::create_account_if_non_existent(&owner_coldkey, &owner_coldkey);

                    // Only register the owner coldkey if it's not already a hotkey on the subnet.
                    if !Uids::<T>::contains_key(netuid, &owner_coldkey) {
                        // Register the owner_coldkey as neuron to the network.
                        let _neuron_uid: u16 = Pallet::<T>::register_neuron(netuid, &owner_coldkey);
                    }
                    // Register the neuron immediately.
                    if !IdentitiesV2::<T>::contains_key(owner_coldkey.clone()) {
                        // Set the identitiy for the Owner coldkey if non existent.
                        let identity = ChainIdentityOfV2 {
                            name: format!("Owner{}", netuid).as_bytes().to_vec(),
                            url: Vec::new(),
                            image: Vec::new(),
                            github_repo: Vec::new(),
                            discord: Vec::new(),
                            description: Vec::new(),
                            additional: Vec::new(),
                        };
                        // Validate the created identity and set it.
                        if Pallet::<T>::is_valid_identity(&identity) {
                            IdentitiesV2::<T>::insert(owner_coldkey.clone(), identity.clone());
                        }
                    }
                }
                MigrationState::ConvertSubnet(netuid)
            } else {
                MigrationState::Finished
            }
        }
    }
}
