// The MIT License (MIT)
// Copyright © 2023 Yuma Rao

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
// documentation files (the “Software”), to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or substantial portions of
// the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use super::*;
use frame_support::weights::{Weight, WeightMeter};
use safe_math::*;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use substrate_fixed::types::{I64F64, U96F32};
use subtensor_runtime_common::{AlphaBalance, NetUid, NetUidStorageIndex, TaoBalance, Token};
impl<T: Config> Pallet<T> {
    /// Fetches the total count of root network validators
    ///
    /// This function retrieves the total number of root network validators.
    ///
    /// # Returns:
    /// * 'u16': The total number of root network validators
    ///
    pub fn get_num_root_validators() -> u16 {
        Self::get_subnetwork_n(NetUid::ROOT)
    }

    /// Fetches the max validators count of root network.
    ///
    /// This function retrieves the max validators count of root network.
    ///
    /// # Returns:
    /// * 'u16': The max validators count of root network.
    ///
    pub fn get_max_root_validators() -> u16 {
        Self::get_max_allowed_uids(NetUid::ROOT)
    }

    /// Checks for any UIDs in the given list that are either equal to the root netuid or exceed the total number of subnets.
    ///
    /// It's important to check for invalid UIDs to ensure data integrity and avoid referencing nonexistent subnets.
    ///
    /// # Arguments:
    /// * 'uids': A reference to a vector of UIDs to check.
    ///
    /// # Returns:
    /// * 'bool': 'true' if any of the UIDs are invalid, 'false' otherwise.
    ///
    pub fn contains_invalid_root_uids(netuids: &[NetUid]) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist(*netuid) {
                log::debug!("contains_invalid_root_uids: netuid {netuid:?} does not exist");
                return true;
            }
        }
        false
    }

    /// Registers a user's hotkey to the root network.
    ///
    /// This function is responsible for registering the hotkey of a user.
    /// The root key with the least stake if pruned in the event of a filled network.
    ///
    /// # Arguments:
    /// * 'origin': Represents the origin of the call.
    /// * 'hotkey': The hotkey that the user wants to register to the root network.
    ///
    /// # Returns:
    /// * 'DispatchResult': A result type indicating success or failure of the registration.
    ///
    pub fn do_root_register(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
        // --- 0. Get the unique identifier (UID) for the root network.
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::if_subnet_exist(NetUid::ROOT),
            Error::<T>::RootNetworkDoesNotExist
        );

        // --- 1. Ensure that the call originates from a signed source and retrieve the caller's account ID (coldkey).
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_root_register( coldkey: {coldkey:?}, hotkey: {hotkey:?} )");

        // --- 2. Ensure that the number of registrations in this block doesn't exceed the allowed limit.
        ensure!(
            Self::get_registrations_this_block(NetUid::ROOT)
                < Self::get_max_registrations_per_block(NetUid::ROOT),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 3. Ensure that the number of registrations in this interval doesn't exceed thrice the target limit.
        ensure!(
            Self::get_registrations_this_interval(NetUid::ROOT)
                < Self::get_target_registrations_per_interval(NetUid::ROOT).saturating_mul(3),
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 4. Check if the hotkey is already registered. If so, error out.
        ensure!(
            !Uids::<T>::contains_key(NetUid::ROOT, &hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 6. Create a network account for the user if it doesn't exist.
        Self::create_account_if_non_existent(&coldkey, &hotkey)?;

        // --- 7. Fetch the current size of the subnetwork.
        let current_num_root_validators: u16 = Self::get_num_root_validators();

        // Declare a variable to hold the root UID.
        let subnetwork_uid: u16;

        // --- 8. Check if the root net is below its allowed size.
        // max allowed is senate size.
        if current_num_root_validators < Self::get_max_root_validators() {
            // --- 12.1.1 We can append to the subnetwork as it's not full.
            subnetwork_uid = current_num_root_validators;

            // --- 12.1.2 Add the new account and make them a member of the Senate.
            Self::append_neuron(NetUid::ROOT, &hotkey, current_block_number);
            log::debug!("add new neuron: {hotkey:?} on uid {subnetwork_uid:?}");
        } else {
            // --- 13.1.1 The network is full. Perform replacement.
            // Find the neuron with the lowest stake value to replace.
            let mut lowest_stake = AlphaBalance::MAX;
            let mut lowest_uid: u16 = 0;

            // Iterate over all keys in the root network to find the neuron with the lowest stake.
            for (uid_i, hotkey_i) in Keys::<T>::iter_prefix(NetUid::ROOT) {
                let stake_i = Self::get_stake_for_hotkey_on_subnet(&hotkey_i, NetUid::ROOT);
                if stake_i < lowest_stake {
                    lowest_stake = stake_i;
                    lowest_uid = uid_i;
                }
            }
            subnetwork_uid = lowest_uid;
            let replaced_hotkey: T::AccountId =
                Self::get_hotkey_for_net_and_uid(NetUid::ROOT, subnetwork_uid)?;

            // --- 13.1.2 The new account has a higher stake than the one being replaced.
            ensure!(
                lowest_stake < Self::get_stake_for_hotkey_on_subnet(&hotkey, NetUid::ROOT),
                Error::<T>::StakeTooLowForRoot
            );

            // --- 13.1.3 The new account has a higher stake than the one being replaced.
            // Replace the neuron account with new information.
            Self::replace_neuron(NetUid::ROOT, lowest_uid, &hotkey, current_block_number);

            log::debug!(
                "replace neuron: {replaced_hotkey:?} with {hotkey:?} on uid {subnetwork_uid:?}"
            );
        }

        // --- 13. Force all members on root to become a delegate.
        if !Self::hotkey_is_delegate(&hotkey) {
            Self::delegate_hotkey(&hotkey, 11_796); // 18% cut defaulted.
        }

        // --- 14. Update the registration counters for both the block and interval.
        #[allow(clippy::arithmetic_side_effects)]
        // note this RA + clippy false positive is a known substrate issue
        RegistrationsThisInterval::<T>::mutate(NetUid::ROOT, |val| *val += 1);
        #[allow(clippy::arithmetic_side_effects)]
        // note this RA + clippy false positive is a known substrate issue
        RegistrationsThisBlock::<T>::mutate(NetUid::ROOT, |val| *val += 1);

        // --- 15. Log and announce the successful registration.
        log::debug!(
            "RootRegistered(netuid:{:?} uid:{:?} hotkey:{:?})",
            NetUid::ROOT,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(
            NetUid::ROOT,
            subnetwork_uid,
            hotkey,
        ));

        // --- 16. Finish and return success.
        Ok(())
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

        let mut dissolved_networks = DissolvedNetworks::<T>::get();
        ensure!(
            !dissolved_networks.contains(&netuid),
            Error::<T>::NetworkAlreadyDissolved
        );

        // TODO Most of data cleanup is done in the block hook, should we charge the user for this?

        // Just remove the network from the added networks, it is used to check if the network is existed.
        NetworksAdded::<T>::remove(netuid);
        // Reduce the total networks count.
        TotalNetworks::<T>::mutate(|n: &mut u16| *n = n.saturating_sub(1));

        dissolved_networks.push(netuid);
        DissolvedNetworks::<T>::set(dissolved_networks);

        DissolvedNetworksCleanupPhase::<T>::insert(
            netuid,
            DissolvedNetworksCleanupPhaseEnum::CleanSubnetRootDividendsRootClaimable,
        );

        log::info!("NetworkRemoved( netuid:{netuid:?} )");

        // --- Emit the NetworkRemoved event
        Self::deposit_event(Event::NetworkRemoved(netuid));

        Ok(())
    }

    pub fn remove_network_map_parameters(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);

        // IsNetworkMember depends on Keys
        let mut keys_set = BTreeSet::new();
        for (_uid, key) in Keys::<T>::iter_prefix(netuid) {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
            if !keys_set.contains(&key) {
                WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
                IsNetworkMember::<T>::remove(&key, netuid);
                keys_set.insert(key);
            }
        }

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

        // Commit-reveal / weights commits (all per-net prefixes):
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
        let mechanisms: u8 = MechanismCountCurrent::<T>::get(netuid).into();

        for subid in 0..mechanisms {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
            let netuid_index = Self::get_mechanism_storage_index(netuid, subid.into());

            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
            LastUpdate::<T>::remove(netuid_index);

            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
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

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        RevealPeriodEpochs::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MechanismCountCurrent::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MechanismEmissionSplit::<T>::remove(netuid);

        // Last hotkey swap (DMAP where netuid is FIRST key → easy)
        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            LastHotkeySwapOnNetuid<T>,
            netuid
        );

        // --- 22. Subnet leasing: remove mapping and any lease-scoped state linked to this netuid.
        if let Some(lease_id) = SubnetUidToLeaseId::<T>::get(netuid) {
            // Fixed: Import the macro type to resolve the error
            LoopRemovePrefixWithWeightMeter!(
                weight_meter,
                T::DbWeight::get().writes(1),
                SubnetLeaseShares<T>,
                lease_id
            );
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
            SubnetLeases::<T>::remove(lease_id);
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
            AccumulatedLeaseDividends::<T>::remove(lease_id);
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
            SubnetUidToLeaseId::<T>::remove(netuid);
        }

        // --- Final removal logging.
        (weight_meter.consumed(), true)
    }

    pub fn remove_network_parameters(netuid: NetUid, remaining_weight: Weight) -> (Weight, bool) {
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetOwner::<T>::remove(netuid);

        // --- 2. Remove network count.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetworkN::<T>::remove(netuid);

        // --- 5. Remove various network-related storages.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        NetworkRegisteredAt::<T>::remove(netuid);

        // --- 9. Remove various network-related parameters.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Active::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Emission::<T>::remove(netuid);

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Consensus::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Dividends::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ValidatorPermit::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ValidatorTrust::<T>::remove(netuid);

        // --- 10. Erase network parameters.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Tempo::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Kappa::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Difficulty::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MaxAllowedUids::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ImmunityPeriod::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ActivityCutoff::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MinAllowedWeights::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        RegistrationsThisInterval::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        POWRegistrationsThisInterval::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        // --- 11. AMM / price / accounting.
        // SubnetTAO, SubnetAlpha{In,InProvided,Out} are already cleared during dissolve/destroy.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetAlphaInEmission::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetAlphaOutEmission::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetTaoInEmission::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetVolume::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetMovingPrice::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetTaoFlow::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetEmaTaoFlow::<T>::remove(netuid);
        SubnetTaoProvided::<T>::remove(netuid);

        // --- 13. Token / mechanism / registration toggles.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        TokenSymbol::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetMechanism::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetOwnerHotkey::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        NetworkRegistrationAllowed::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        NetworkPowRegistrationAllowed::<T>::remove(netuid);

        // --- 14. Locks & toggles.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        TransferToggle::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetLocked::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        LargestLocked::<T>::remove(netuid);

        // --- 15. Mechanism step / emissions bookkeeping.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        FirstEmissionBlockNumber::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        PendingValidatorEmission::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        PendingServerEmission::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        PendingRootAlphaDivs::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        PendingOwnerCut::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BlocksSinceLastStep::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        LastMechansimStepBlock::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        LastAdjustmentBlock::<T>::remove(netuid);

        // --- 16. Serving / rho / curves, and other per-net controls.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ServingRateLimit::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Rho::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        AlphaSigmoidSteepness::<T>::remove(netuid);

        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MaxAllowedValidators::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BondsMovingAverage::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BondsPenalty::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BondsResetOn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        WeightsSetRateLimit::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ValidatorPruneLen::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ScalingLawPower::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        TargetRegistrationsPerInterval::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        CommitRevealWeightsEnabled::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BurnHalfLife::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        BurnIncreaseMult::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Burn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MinBurn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MaxBurn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MinDifficulty::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MaxDifficulty::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        RegistrationsThisBlock::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        EMAPriceHalvingBlocks::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        RAORecycledForRegistration::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        MaxRegistrationsPerBlock::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        WeightsVersionKey::<T>::remove(netuid);

        // --- 17. Subtoken / feature flags.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        LiquidAlphaOn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Yuma3On::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        AlphaValues::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubtokenEnabled::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        ImmuneOwnerUidsLimit::<T>::remove(netuid);

        // --- 18. Consensus aux vectors.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        StakeWeight::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        LoadedEmission::<T>::remove(netuid);

        // --- 20. Identity maps across versions (netuid-scoped).
        if SubnetIdentitiesV3::<T>::contains_key(netuid) {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
            SubnetIdentitiesV3::<T>::remove(netuid);
            Self::deposit_event(Event::SubnetIdentityRemoved(netuid));
        }

        (weight_meter.consumed(), true)
    }

    pub fn remove_network_weights(netuid: NetUid, remaining_weight: Weight) -> (Weight, bool) {
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);

        let mut map = BTreeMap::new();
        let mut read_all = true;

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
                if subnet_id == &u16::from(netuid) {
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

        for (uid_i, weights_i) in map.iter() {
            Weights::<T>::insert(NetUidStorageIndex::ROOT, uid_i, weights_i.clone());
        }
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_childkey_take(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_childkeys(netuid: NetUid, remaining_weight: Weight) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_parentkeys(netuid: NetUid, remaining_weight: Weight) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_last_hotkey_emission_on_netuid(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_total_hotkey_alpha_last_epoch(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_transaction_key_last_block(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
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
        (weight_meter.consumed(), read_all)
    }

    pub fn remove_network_staking_operation_rate_limiter(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;

        let mut to_rm: sp_std::vec::Vec<(T::AccountId, T::AccountId)> = sp_std::vec::Vec::new();
        let iter = match LastKeptRawKey::<T>::get() {
            Some(raw_key) => StakingOperationRateLimiter::<T>::iter_from(raw_key),
            None => StakingOperationRateLimiter::<T>::iter(),
        };
        for ((hot, cold, nu), _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(StakingOperationRateLimiter::<T>::hashed_key_for((
                    &hot, &cold, nu,
                ))));
                break;
            }
            weight_meter.consume(r);
            if nu == netuid {
                if !weight_meter.can_consume(w) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(
                        StakingOperationRateLimiter::<T>::hashed_key_for((&hot, &cold, nu)),
                    ));
                    break;
                }
                weight_meter.consume(w);
                to_rm.push((hot, cold));
            }
        }
        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for (hot, cold) in to_rm {
            StakingOperationRateLimiter::<T>::remove((hot, cold, netuid));
        }
        (weight_meter.consumed(), read_all)
    }

    #[allow(clippy::arithmetic_side_effects)]
    /// This function calculates the lock cost for a network based on the last lock amount, minimum lock cost, last lock block, and current block.
    /// The lock cost is calculated using the formula:
    /// lock_cost = (last_lock * mult) - (last_lock / lock_reduction_interval) * (current_block - last_lock_block)
    /// where:
    /// - last_lock is the last lock amount for the network
    /// - mult is the multiplier which increases lock cost each time a registration occurs
    /// - last_lock_block is the block number at which the last lock occurred
    /// - lock_reduction_interval the number of blocks before the lock returns to previous value.
    /// - current_block is the current block number
    /// - DAYS is the number of blocks in a day
    /// - min_lock is the minimum lock cost for the network
    ///
    /// If the calculated lock cost is less than the minimum lock cost, the minimum lock cost is returned.
    ///
    /// # Returns:
    ///  * 'u64':
    ///     - The lock cost for the network.
    ///
    pub fn get_network_lock_cost() -> TaoBalance {
        let last_lock = Self::get_network_last_lock();
        let min_lock = Self::get_network_min_lock();
        let last_lock_block = Self::get_network_last_lock_block();
        let current_block = Self::get_current_block_as_u64();
        let lock_reduction_interval = Self::get_lock_reduction_interval();
        let mult: TaoBalance = if last_lock_block == 0 { 1 } else { 2 }.into();

        let mut lock_cost = last_lock.saturating_mul(mult).saturating_sub(
            last_lock
                .to_u64()
                .safe_div(lock_reduction_interval)
                .saturating_mul(current_block.saturating_sub(last_lock_block))
                .into(),
        );

        if lock_cost < min_lock {
            lock_cost = min_lock;
        }

        log::debug!(
            "last_lock: {last_lock:?}, min_lock: {min_lock:?}, last_lock_block: {last_lock_block:?}, lock_reduction_interval: {lock_reduction_interval:?}, current_block: {current_block:?}, mult: {mult:?} lock_cost: {lock_cost:?}"
        );

        lock_cost
    }

    pub fn get_network_registered_block(netuid: NetUid) -> u64 {
        NetworkRegisteredAt::<T>::get(netuid)
    }
    pub fn get_registered_subnet_counter(netuid: NetUid) -> u64 {
        RegisteredSubnetCounter::<T>::get(netuid)
    }
    pub fn get_network_immunity_period() -> u64 {
        NetworkImmunityPeriod::<T>::get()
    }
    pub fn set_network_immunity_period(net_immunity_period: u64) {
        NetworkImmunityPeriod::<T>::set(net_immunity_period);
        Self::deposit_event(Event::NetworkImmunityPeriodSet(net_immunity_period));
    }
    pub fn set_start_call_delay(delay: u64) {
        StartCallDelay::<T>::set(delay);
        Self::deposit_event(Event::StartCallDelaySet(delay));
    }
    pub fn set_network_min_lock(net_min_lock: TaoBalance) {
        NetworkMinLockCost::<T>::set(net_min_lock);
        Self::deposit_event(Event::NetworkMinLockCostSet(net_min_lock));
    }
    pub fn get_network_min_lock() -> TaoBalance {
        NetworkMinLockCost::<T>::get()
    }
    pub fn set_network_last_lock(net_last_lock: TaoBalance) {
        NetworkLastLockCost::<T>::set(net_last_lock);
    }
    pub fn get_network_last_lock() -> TaoBalance {
        NetworkLastLockCost::<T>::get()
    }
    pub fn get_network_last_lock_block() -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::NetworkLastRegistered)
    }
    pub fn set_network_last_lock_block(block: u64) {
        Self::set_rate_limited_last_block(&RateLimitKey::NetworkLastRegistered, block);
    }
    pub fn set_lock_reduction_interval(interval: u64) {
        NetworkLockReductionInterval::<T>::set(interval);
        Self::deposit_event(Event::NetworkLockCostReductionIntervalSet(interval));
    }
    pub fn get_lock_reduction_interval() -> u64 {
        let interval: I64F64 =
            I64F64::saturating_from_num(NetworkLockReductionInterval::<T>::get());
        let block_emission: I64F64 = I64F64::saturating_from_num(
            Self::calculate_block_emission()
                .unwrap_or(1_000_000_000.into())
                .to_u64(),
        );
        let halving: I64F64 = block_emission
            .checked_div(I64F64::saturating_from_num(1_000_000_000))
            .unwrap_or(I64F64::saturating_from_num(0.0));
        let halved_interval: I64F64 = interval.saturating_mul(halving);
        halved_interval.saturating_to_num::<u64>()
    }
    pub fn get_rate_limited_last_block(rate_limit_key: &RateLimitKey<T::AccountId>) -> u64 {
        LastRateLimitedBlock::<T>::get(rate_limit_key)
    }
    pub fn set_rate_limited_last_block(rate_limit_key: &RateLimitKey<T::AccountId>, block: u64) {
        LastRateLimitedBlock::<T>::insert(rate_limit_key, block);
    }
    pub fn remove_rate_limited_last_block(rate_limit_key: &RateLimitKey<T::AccountId>) {
        LastRateLimitedBlock::<T>::remove(rate_limit_key);
    }

    pub fn get_network_to_prune() -> Option<NetUid> {
        let current_block: u64 = Self::get_current_block_as_u64();

        let mut candidate_netuid: Option<NetUid> = None;
        let mut candidate_price: U96F32 = U96F32::saturating_from_num(u128::MAX);
        let mut candidate_timestamp: u64 = u64::MAX;

        for (netuid, added) in NetworksAdded::<T>::iter() {
            if !added || netuid == NetUid::ROOT {
                continue;
            }

            let registered_at = NetworkRegisteredAt::<T>::get(netuid);

            // Skip immune networks.
            if current_block < registered_at.saturating_add(Self::get_network_immunity_period()) {
                continue;
            }

            let price: U96F32 = Self::get_moving_alpha_price(netuid);

            // If tie on price, earliest registration wins.
            if price < candidate_price
                || (price == candidate_price && registered_at < candidate_timestamp)
            {
                candidate_netuid = Some(netuid);
                candidate_price = price;
                candidate_timestamp = registered_at;
            }
        }

        candidate_netuid
    }
}
