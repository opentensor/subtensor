use super::*;
use frame_support::weights::WeightMeter;
use sp_std::collections::btree_map::BTreeMap;
use subtensor_runtime_common::{NetUid, NetUidStorageIndex};

impl<T: Config> Pallet<T> {
    fn remove_account_entries_for_netuid<I>(
        weight_meter: &mut WeightMeter,
        netuid: NetUid,
        iter: I,
        raw_key_for: impl Fn(&T::AccountId, NetUid) -> Vec<u8>,
        remove: impl Fn(T::AccountId, NetUid),
    ) -> bool
    where
        I: Iterator<Item = (T::AccountId, NetUid)>,
    {
        true
    }

    /// Facilitates the removal of a user's subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    /// * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Event:
    /// * 'NetworkRemoved': Emitted when a network is successfully removed.
    ///
    /// # Raises:
    /// * 'MechanismDoesNotExist': If the specified network does not exist.
    /// * 'NotSubnetOwner': If the caller does not own the specified subnet.
    ///
    pub fn do_dissolve_network(netuid: NetUid) -> dispatch::DispatchResult {
        // --- The network exists?
        ensure!(
            Self::if_subnet_exist(netuid) && netuid != NetUid::ROOT,
            Error::<T>::SubnetNotExists
        );

        let mut dissolved_networks = DissolveCleanupQueue::<T>::get();
        ensure!(
            !dissolved_networks.contains(&netuid),
            Error::<T>::NetworkAlreadyDissolved
        );

        // TODO Most of data cleanup is done in the block hook, should we charge the user for this?

        // Just remove the network from the added networks, it is used to check if the network is existed.
        NetworksAdded::<T>::remove(netuid);
        // Reduce the total networks count.
        TotalNetworks::<T>::mutate(|n: &mut u16| *n = n.saturating_sub(1));

        TotalStake::<T>::mutate(|total| *total = total.saturating_sub(SubnetTAO::<T>::get(netuid)));

        dissolved_networks.push(netuid);
        DissolveCleanupQueue::<T>::set(dissolved_networks);

        log::debug!("NetworkRemoved( netuid:{netuid:?} )");

        // --- Emit the NetworkRemoved event
        Self::deposit_event(Event::NetworkRemoved(netuid));

        Ok(())
    }

    pub fn remove_network_map_parameters(netuid: NetUid, weight_meter: &mut WeightMeter) -> bool {
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            Keys<T>,
            netuid
        );

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            Uids<T>,
            netuid
        );

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            BlockAtRegistration<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            Axons<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            NeuronCertificates<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            Prometheus<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            AlphaDividendsPerSubnet<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            PendingChildKeys<T>,
            netuid
        );
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            AssociatedEvmAddress<T>,
            netuid
        );

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            HotkeyLock<T>,
            netuid
        );

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            DecayingHotkeyLock<T>,
            netuid
        );

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
        let mechanisms: u8 = MechanismCountCurrent::<T>::get(netuid).into();

        for subid in 0..mechanisms {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads_writes(1, 2));
            let netuid_index = Self::get_mechanism_storage_index(netuid, subid.into());

            LastUpdate::<T>::remove(netuid_index);
            Incentive::<T>::remove(netuid_index);

            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                WeightCommits<T>,
                netuid_index
            );
            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                TimelockedWeightCommits<T>,
                netuid_index
            );

            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                CRV3WeightCommits<T>,
                netuid_index
            );

            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                CRV3WeightCommitsV2<T>,
                netuid_index
            );

            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                Bonds<T>,
                netuid_index
            );

            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                Weights<T>,
                netuid_index
            );
        }

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(3));
        RevealPeriodEpochs::<T>::remove(netuid);
        MechanismCountCurrent::<T>::remove(netuid);
        MechanismEmissionSplit::<T>::remove(netuid);

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            LastHotkeySwapOnNetuid<T>,
            netuid
        );

        if let Some(lease_id) = SubnetUidToLeaseId::<T>::get(netuid) {
            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                SubnetLeaseShares<T>,
                lease_id
            );
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(3));
            SubnetLeases::<T>::remove(lease_id);
            AccumulatedLeaseDividends::<T>::remove(lease_id);
            SubnetUidToLeaseId::<T>::remove(netuid);
        }

        true
    }

    pub fn remove_network_parameters(netuid: NetUid, weight_meter: &mut WeightMeter) -> bool {
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(80));
        SubnetOwner::<T>::remove(netuid);
        SubnetworkN::<T>::remove(netuid);
        NetworkRegisteredAt::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);
        SubnetAlphaInEmission::<T>::remove(netuid);
        SubnetAlphaOutEmission::<T>::remove(netuid);
        SubnetTaoInEmission::<T>::remove(netuid);
        SubnetVolume::<T>::remove(netuid);
        SubnetMovingPrice::<T>::remove(netuid);
        SubnetTaoFlow::<T>::remove(netuid);
        SubnetEmaTaoFlow::<T>::remove(netuid);
        SubnetProtocolFlow::<T>::remove(netuid);
        SubnetEmaProtocolFlow::<T>::remove(netuid);
        SubnetExcessTao::<T>::remove(netuid);
        SubnetRootSellTao::<T>::remove(netuid);
        TokenSymbol::<T>::remove(netuid);
        SubnetMechanism::<T>::remove(netuid);
        SubnetOwnerHotkey::<T>::remove(netuid);
        NetworkRegistrationAllowed::<T>::remove(netuid);
        NetworkPowRegistrationAllowed::<T>::remove(netuid);
        TransferToggle::<T>::remove(netuid);
        SubnetLocked::<T>::remove(netuid);
        LargestLocked::<T>::remove(netuid);
        FirstEmissionBlockNumber::<T>::remove(netuid);
        PendingValidatorEmission::<T>::remove(netuid);
        PendingServerEmission::<T>::remove(netuid);
        PendingRootAlphaDivs::<T>::remove(netuid);
        PendingOwnerCut::<T>::remove(netuid);
        BlocksSinceLastStep::<T>::remove(netuid);
        LastMechansimStepBlock::<T>::remove(netuid);
        LastAdjustmentBlock::<T>::remove(netuid);
        ServingRateLimit::<T>::remove(netuid);
        Rho::<T>::remove(netuid);
        AlphaSigmoidSteepness::<T>::remove(netuid);
        MaxAllowedValidators::<T>::remove(netuid);
        BondsMovingAverage::<T>::remove(netuid);
        BondsPenalty::<T>::remove(netuid);
        BondsResetOn::<T>::remove(netuid);
        WeightsSetRateLimit::<T>::remove(netuid);
        ValidatorPruneLen::<T>::remove(netuid);
        ScalingLawPower::<T>::remove(netuid);
        TargetRegistrationsPerInterval::<T>::remove(netuid);
        CommitRevealWeightsEnabled::<T>::remove(netuid);
        BurnHalfLife::<T>::remove(netuid);
        BurnIncreaseMult::<T>::remove(netuid);
        Burn::<T>::remove(netuid);
        MinBurn::<T>::remove(netuid);
        MaxBurn::<T>::remove(netuid);
        MinDifficulty::<T>::remove(netuid);
        MaxDifficulty::<T>::remove(netuid);
        RegistrationsThisBlock::<T>::remove(netuid);
        EMAPriceHalvingBlocks::<T>::remove(netuid);
        RAORecycledForRegistration::<T>::remove(netuid);
        MaxRegistrationsPerBlock::<T>::remove(netuid);
        WeightsVersionKey::<T>::remove(netuid);
        LiquidAlphaOn::<T>::remove(netuid);
        Yuma3On::<T>::remove(netuid);
        AlphaValues::<T>::remove(netuid);
        SubtokenEnabled::<T>::remove(netuid);
        OwnerCutAutoLockEnabled::<T>::remove(netuid);
        ImmuneOwnerUidsLimit::<T>::remove(netuid);
        StakeWeight::<T>::remove(netuid);
        LoadedEmission::<T>::remove(netuid);
        OwnerLock::<T>::remove(netuid);
        DecayingOwnerLock::<T>::remove(netuid);

        if SubnetIdentitiesV3::<T>::contains_key(netuid) {
            SubnetIdentitiesV3::<T>::remove(netuid);
            Self::deposit_event(Event::SubnetIdentityRemoved(netuid));
        }
        true
    }

    pub fn remove_network_is_network_member(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => Keys::<T>::iter_from(raw_key),
            None => Keys::<T>::iter(),
        };
        for (nu, uid, hotkey) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(Keys::<T>::hashed_key_for(nu, uid)));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(Keys::<T>::hashed_key_for(nu, uid)));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hotkey);
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            IsNetworkMember::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_update_weights_on_root(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let mut map = BTreeMap::new();
        let mut read_all = true;
        let netuid_u16 = u16::from(netuid);

        let root = NetUidStorageIndex::ROOT;
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => Weights::<T>::iter_prefix_from(root, raw_key),
            None => Weights::<T>::iter_prefix(root),
        };

        // --- Iterate over stored weights and zero root weights pointing at this netuid.
        for (uid_i, weights_i) in iter {
            let can_consume = weight_meter.can_consume(T::DbWeight::get().reads(1));
            weight_meter.consume(T::DbWeight::get().reads(1));
            if !can_consume {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(Weights::<T>::hashed_key_for(root, uid_i)));
                break;
            }

            // Create a new vector to hold modified weights.
            let mut modified_weights = weights_i.clone();
            let mut need_update = false;
            for (subnet_id, weight) in modified_weights.iter_mut() {
                // If the root network had a weight pointing to this netuid, set it to 0
                if *subnet_id == netuid_u16 {
                    if *weight != 0 {
                        need_update = true;
                    }

                    *weight = 0;
                }
            }

            if need_update {
                let can_consume = weight_meter.can_consume(T::DbWeight::get().writes(1));
                if !can_consume {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(Weights::<T>::hashed_key_for(root, uid_i)));
                    break;
                }
                weight_meter.consume(T::DbWeight::get().writes(1));
                map.insert(uid_i, modified_weights);
            }
        }

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        read_all
    }

    pub fn remove_network_childkey_take(netuid: NetUid, weight_meter: &mut WeightMeter) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => ChildkeyTake::<T>::iter_from(raw_key),
            None => ChildkeyTake::<T>::iter(),
        };
        for (hot, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(ChildkeyTake::<T>::hashed_key_for(&hot, nu)));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(ChildkeyTake::<T>::hashed_key_for(&hot, nu)));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hot);
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            ChildkeyTake::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_childkeys(netuid: NetUid, weight_meter: &mut WeightMeter) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => ChildKeys::<T>::iter_from(raw_key),
            None => ChildKeys::<T>::iter(),
        };
        for (hot, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(ChildKeys::<T>::hashed_key_for(&hot, nu)));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(ChildKeys::<T>::hashed_key_for(&hot, nu)));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hot);
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            ChildKeys::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_parentkeys(netuid: NetUid, weight_meter: &mut WeightMeter) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => ParentKeys::<T>::iter_from(raw_key),
            None => ParentKeys::<T>::iter(),
        };
        for (hot, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(ParentKeys::<T>::hashed_key_for(&hot, nu)));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(ParentKeys::<T>::hashed_key_for(&hot, nu)));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hot);
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            ParentKeys::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_last_hotkey_emission_on_netuid(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => LastHotkeyEmissionOnNetuid::<T>::iter_from(raw_key),
            None => LastHotkeyEmissionOnNetuid::<T>::iter(),
        };
        for (hot, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(LastHotkeyEmissionOnNetuid::<T>::hashed_key_for(
                    &hot, nu,
                )));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(
                        LastHotkeyEmissionOnNetuid::<T>::hashed_key_for(&hot, nu),
                    ));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hot);
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            LastHotkeyEmissionOnNetuid::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_total_hotkey_alpha_last_epoch(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => TotalHotkeyAlphaLastEpoch::<T>::iter_from(raw_key),
            None => TotalHotkeyAlphaLastEpoch::<T>::iter(),
        };

        for (hot, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlphaLastEpoch::<T>::hashed_key_for(
                    &hot, nu,
                )));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(TotalHotkeyAlphaLastEpoch::<T>::hashed_key_for(
                        &hot, nu,
                    )));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push(hot);
            }
        }

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hot in to_rm {
            TotalHotkeyAlphaLastEpoch::<T>::remove(&hot, netuid);
        }
        read_all
    }

    pub fn remove_network_transaction_key_last_block(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<(T::AccountId, u16)> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => TransactionKeyLastBlock::<T>::iter_from(raw_key),
            None => TransactionKeyLastBlock::<T>::iter(),
        };
        for ((hot, nu, name), _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TransactionKeyLastBlock::<T>::hashed_key_for((
                    &hot, nu, name,
                ))));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(TransactionKeyLastBlock::<T>::hashed_key_for((
                        &hot, nu, name,
                    ))));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push((hot, name));
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for (hot, name) in to_rm {
            TransactionKeyLastBlock::<T>::remove((hot, netuid, name));
        }
        read_all
    }
}
