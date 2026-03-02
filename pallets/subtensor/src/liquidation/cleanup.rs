use super::*;
use crate::CommitmentsInterface;
use crate::liquidation::types::{
    ChunkResult, CursorBytes, LiquidationPhase, LiquidationState, LiquidationWarning, MatrixMap,
    NeuronDataMap, TwoKeyMap,
};
use crate::pallet::*;
use crate::{Config, Event, Pallet};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::U256;
use subtensor_runtime_common::{Currency, NetUid};
use subtensor_swap_interface::SwapHandler;

/// Bounded scan-and-remove for `StorageDoubleMap<AccountId, NetUid, V>`.
/// Iterates the full map, collects keys matching `$netuid` up to `$limit`,
/// then removes them. Returns `(count_removed, is_complete)`.
macro_rules! bounded_remove_by_netuid {
    ($map:ty, $netuid:expr, $limit:expr) => {{
        let mut keys: sp_std::vec::Vec<_> = sp_std::vec::Vec::new();
        for (account, n, _) in <$map>::iter() {
            if n == $netuid {
                keys.push(account);
                if keys.len() >= $limit {
                    break;
                }
            }
        }
        let complete = keys.len() < $limit;
        for key in &keys {
            <$map>::remove(key, $netuid);
        }
        (keys.len(), complete)
    }};
}

impl<T: Config> Pallet<T> {
    /// Clear ~50 hyperparameter storage entries for a subnet. Instant (O(1)).
    pub fn clear_hyperparams(netuid: NetUid) -> Weight {
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
        SubnetTaoProvided::<T>::remove(netuid);
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
        AdjustmentInterval::<T>::remove(netuid);
        BondsMovingAverage::<T>::remove(netuid);
        BondsPenalty::<T>::remove(netuid);
        BondsResetOn::<T>::remove(netuid);
        WeightsSetRateLimit::<T>::remove(netuid);
        ValidatorPruneLen::<T>::remove(netuid);
        ScalingLawPower::<T>::remove(netuid);
        TargetRegistrationsPerInterval::<T>::remove(netuid);
        AdjustmentAlpha::<T>::remove(netuid);
        CommitRevealWeightsEnabled::<T>::remove(netuid);
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
        ImmuneOwnerUidsLimit::<T>::remove(netuid);
        StakeWeight::<T>::remove(netuid);
        LoadedEmission::<T>::remove(netuid);
        RevealPeriodEpochs::<T>::remove(netuid);

        Weight::from_parts(HYPERPARAM_COUNT.saturating_mul(WEIGHT_PER_HYPERPARAM), 0)
    }

    /// Clear neuron data prefix maps in chunks, one map at a time.
    ///
    /// Processes 10 prefix maps sequentially (map_idx 0..9), using the cursor
    /// from `clear_prefix` to resume if a map has more entries than the budget.
    /// Once all prefix maps are done, clears the vector storage items.
    pub fn clear_neuron_data_chunk(
        netuid: NetUid,
        map_idx: u8,
        cursor: Option<CursorBytes>,
        weight_budget: Weight,
    ) -> ChunkResult {
        let limit = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_NEURON_CLEAR)
            .unwrap_or(256)
            .max(1) as u32;
        let cursor_slice = cursor.as_ref().map(|c| c.as_slice());

        let result = match NeuronDataMap::from(map_idx) {
            NeuronDataMap::BlockAtRegistration => {
                BlockAtRegistration::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::Axons => Axons::<T>::clear_prefix(netuid, limit, cursor_slice),
            NeuronDataMap::NeuronCertificates => {
                NeuronCertificates::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::Prometheus => Prometheus::<T>::clear_prefix(netuid, limit, cursor_slice),
            NeuronDataMap::AlphaDividendsPerSubnet => {
                AlphaDividendsPerSubnet::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::PendingChildKeys => {
                PendingChildKeys::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::AssociatedEvmAddress => {
                AssociatedEvmAddress::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::Uids => Uids::<T>::clear_prefix(netuid, limit, cursor_slice),
            NeuronDataMap::Keys => Keys::<T>::clear_prefix(netuid, limit, cursor_slice),
            NeuronDataMap::LastHotkeySwapOnNetuid => {
                LastHotkeySwapOnNetuid::<T>::clear_prefix(netuid, limit, cursor_slice)
            }
            NeuronDataMap::VectorStorage => {
                Rank::<T>::remove(netuid);
                Trust::<T>::remove(netuid);
                Active::<T>::remove(netuid);
                Emission::<T>::remove(netuid);
                Consensus::<T>::remove(netuid);
                Dividends::<T>::remove(netuid);
                PruningScores::<T>::remove(netuid);
                ValidatorPermit::<T>::remove(netuid);
                ValidatorTrust::<T>::remove(netuid);
                return ChunkResult::Complete(Weight::from_parts(WEIGHT_PER_NEURON_CLEAR, 0));
            }
        };

        let used = Weight::from_parts(WEIGHT_PER_NEURON_CLEAR, 0);

        match result.maybe_cursor {
            Some(raw) => match Self::bound_cursor(raw, netuid) {
                Some(bounded) => ChunkResult::Incomplete {
                    weight_used: used,
                    phase: LiquidationPhase::ClearNeuronData {
                        map_idx,
                        cursor: Some(bounded),
                    },
                },
                None => ChunkResult::Incomplete {
                    weight_used: used,
                    phase: LiquidationPhase::ClearNeuronData {
                        map_idx: map_idx.saturating_add(1),
                        cursor: None,
                    },
                },
            },
            None => ChunkResult::Incomplete {
                weight_used: used,
                phase: LiquidationPhase::ClearNeuronData {
                    map_idx: map_idx.saturating_add(1),
                    cursor: None,
                },
            },
        }
    }

    /// Zero out root validator weights that reference this netuid.
    pub fn clear_root_weights_chunk(
        netuid: NetUid,
        uid_cursor: u16,
        weight_budget: Weight,
    ) -> ChunkResult {
        use subtensor_runtime_common::NetUidStorageIndex;

        let items_per_budget = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_MATRIX_ENTRY)
            .unwrap_or(0) as u16;
        if items_per_budget == 0 {
            return ChunkResult::Incomplete {
                weight_used: Weight::zero(),
                phase: LiquidationPhase::ClearRootWeights { uid_cursor },
            };
        }

        let mut count: u16 = 0;

        for (uid_i, weights_i) in Weights::<T>::iter_prefix(NetUidStorageIndex::ROOT) {
            if uid_i < uid_cursor {
                continue;
            }
            let mut modified = weights_i.clone();
            for (subnet_id, weight) in modified.iter_mut() {
                if *subnet_id == u16::from(netuid) {
                    *weight = 0;
                }
            }
            Weights::<T>::insert(NetUidStorageIndex::ROOT, uid_i, modified);
            count = count.saturating_add(1);
            if count >= items_per_budget {
                let used =
                    Weight::from_parts(u64::from(count).saturating_mul(WEIGHT_PER_MATRIX_ENTRY), 0);
                return ChunkResult::Incomplete {
                    weight_used: used,
                    phase: LiquidationPhase::ClearRootWeights {
                        uid_cursor: uid_i.saturating_add(1),
                    },
                };
            }
        }

        ChunkResult::Complete(Weight::from_parts(
            u64::from(count).saturating_mul(WEIGHT_PER_MATRIX_ENTRY),
            0,
        ))
    }

    /// Process root claims for this subnet.
    pub fn finalize_root_dividends(netuid: NetUid) -> Weight {
        Self::finalize_all_subnet_root_dividends(netuid);
        Weight::from_parts(FIXED_OVERHEAD, 0)
    }

    /// Distribute TAO to stakers in chunks using U256 arithmetic.
    pub fn distribute_alpha_chunk(
        netuid: NetUid,
        cursor_idx: u32,
        state: &mut LiquidationState<BlockNumberFor<T>>,
        weight_budget: Weight,
    ) -> ChunkResult {
        let snapshot_len = state.snapshot_count;
        let items_per_budget = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_DISTRIBUTION)
            .unwrap_or(0) as u32;
        if items_per_budget == 0 {
            return ChunkResult::Incomplete {
                weight_used: Weight::zero(),
                phase: LiquidationPhase::DistributeAlpha { cursor_idx },
            };
        }

        let end_idx = cursor_idx
            .saturating_add(items_per_budget)
            .min(snapshot_len);
        let mut weight_used = Weight::zero();

        for i in cursor_idx..end_idx {
            let Some((hot, cold, alpha_val)) = LiquidationStakerSnapshot::<T>::get(netuid, i)
            else {
                continue;
            };

            let share = Self::calculate_share(state.tao_pot, alpha_val, state.total_alpha_value);

            if share > 0 {
                Self::add_balance_to_coldkey_account(&cold, share);
            }

            state.tao_distributed = state.tao_distributed.saturating_add(share);

            // Handle dust on final staker
            let is_final = i.saturating_add(1) >= snapshot_len;
            if is_final {
                let dust = state.tao_pot.saturating_sub(state.tao_distributed);
                if dust > 0 {
                    // Burn dust — no treasury in subtensor pallet
                    TotalIssuance::<T>::mutate(|total| {
                        let dust_currency: subtensor_runtime_common::TaoCurrency = dust.into();
                        *total = (*total).saturating_sub(dust_currency);
                    });
                    state.tao_distributed = state.tao_pot;
                    Self::deposit_event(Event::LiquidationWarning {
                        netuid,
                        warning: LiquidationWarning::DistributionDust(dust),
                    });
                }
            }

            // Clean up alpha entry
            Alpha::<T>::remove((&hot, &cold, netuid));

            weight_used =
                weight_used.saturating_add(Weight::from_parts(WEIGHT_PER_DISTRIBUTION, 0));
        }

        if end_idx >= snapshot_len {
            ChunkResult::Complete(weight_used)
        } else {
            ChunkResult::Incomplete {
                weight_used,
                phase: LiquidationPhase::DistributeAlpha {
                    cursor_idx: end_idx,
                },
            }
        }
    }

    /// Dissolve user LP positions during liquidation.
    /// Runs BEFORE snapshot so that LP-derived alpha is captured in the snapshot
    /// and included in TAO distribution.
    pub fn dissolve_user_lps(netuid: NetUid) -> Weight {
        if let Err(e) = T::SwapInterface::dissolve_all_liquidity_providers(netuid) {
            log::error!(
                "dissolve_all_liquidity_providers failed for netuid {:?}: {:?}",
                netuid,
                e
            );
            Self::deposit_event(Event::LiquidationWarning {
                netuid,
                warning: LiquidationWarning::LpDissolutionFailed,
            });
        }
        Weight::from_parts(FIXED_OVERHEAD, 0)
    }

    /// Clear protocol LP positions.
    pub fn clear_protocol_lps(netuid: NetUid) -> Weight {
        if let Err(e) = T::SwapInterface::clear_protocol_liquidity(netuid) {
            log::error!(
                "clear_protocol_liquidity failed for netuid {:?}: {:?}",
                netuid,
                e
            );
            Self::deposit_event(Event::LiquidationWarning {
                netuid,
                warning: LiquidationWarning::ProtocolLpClearFailed,
            });
        }
        Weight::from_parts(FIXED_OVERHEAD, 0)
    }

    /// Clear Bonds/Weights matrices for a subnet in chunks, one prefix map at a time.
    ///
    /// For each mechanism, processes 6 prefix maps (map_idx 0..5):
    ///   0: WeightCommits, 1: TimelockedWeightCommits, 2: CRV3WeightCommits,
    ///   3: CRV3WeightCommitsV2, 4: Bonds, 5: Weights
    /// When entering a new mechanism (map_idx == 0, cursor is None), also removes
    /// the single-key items LastUpdate and Incentive.
    /// After all mechanisms are done, removes MechanismEmissionSplit and returns complete.
    pub fn clear_matrices_chunk(
        netuid: NetUid,
        mechanism_idx: u8,
        map_idx: u8,
        cursor: Option<CursorBytes>,
        weight_budget: Weight,
    ) -> ChunkResult {
        let mechanism_count: u8 = MechanismCountCurrent::<T>::get(netuid).into();

        if mechanism_idx >= mechanism_count {
            MechanismEmissionSplit::<T>::remove(netuid);
            return ChunkResult::Complete(Weight::from_parts(WEIGHT_PER_MATRIX_ENTRY, 0));
        }

        let limit = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_MATRIX_ENTRY)
            .unwrap_or(256)
            .max(1) as u32;

        let netuid_index = Self::get_mechanism_storage_index(netuid, mechanism_idx.into());
        let cursor_slice = cursor.as_ref().map(|c| c.as_slice());

        // When starting a new mechanism (first map, no resume cursor), clear single-key items
        if map_idx == 0 && cursor.is_none() {
            LastUpdate::<T>::remove(netuid_index);
            Incentive::<T>::remove(netuid_index);
        }

        let result = match MatrixMap::from(map_idx) {
            MatrixMap::WeightCommits => {
                WeightCommits::<T>::clear_prefix(netuid_index, limit, cursor_slice)
            }
            MatrixMap::TimelockedWeightCommits => {
                TimelockedWeightCommits::<T>::clear_prefix(netuid_index, limit, cursor_slice)
            }
            MatrixMap::CRV3WeightCommits => {
                CRV3WeightCommits::<T>::clear_prefix(netuid_index, limit, cursor_slice)
            }
            MatrixMap::CRV3WeightCommitsV2 => {
                CRV3WeightCommitsV2::<T>::clear_prefix(netuid_index, limit, cursor_slice)
            }
            MatrixMap::Bonds => Bonds::<T>::clear_prefix(netuid_index, limit, cursor_slice),
            MatrixMap::Weights => Weights::<T>::clear_prefix(netuid_index, limit, cursor_slice),
            MatrixMap::NextMechanism => {
                return ChunkResult::Incomplete {
                    weight_used: Weight::from_parts(WEIGHT_PER_MATRIX_ENTRY, 0),
                    phase: LiquidationPhase::ClearMatrices {
                        mechanism_idx: mechanism_idx.saturating_add(1),
                        map_idx: 0,
                        cursor: None,
                    },
                };
            }
        };

        let used = Weight::from_parts(WEIGHT_PER_MATRIX_ENTRY, 0);

        match result.maybe_cursor {
            Some(raw) => {
                let next_cursor = Self::bound_cursor(raw, netuid);
                if next_cursor.is_some() {
                    ChunkResult::Incomplete {
                        weight_used: used,
                        phase: LiquidationPhase::ClearMatrices {
                            mechanism_idx,
                            map_idx,
                            cursor: next_cursor,
                        },
                    }
                } else {
                    // Cursor overflow — skip to next map
                    ChunkResult::Incomplete {
                        weight_used: used,
                        phase: LiquidationPhase::ClearMatrices {
                            mechanism_idx,
                            map_idx: map_idx.saturating_add(1),
                            cursor: None,
                        },
                    }
                }
            }
            None => {
                let next_map = map_idx.saturating_add(1);
                if next_map > MatrixMap::LAST_IDX {
                    ChunkResult::Incomplete {
                        weight_used: used,
                        phase: LiquidationPhase::ClearMatrices {
                            mechanism_idx: mechanism_idx.saturating_add(1),
                            map_idx: 0,
                            cursor: None,
                        },
                    }
                } else {
                    ChunkResult::Incomplete {
                        weight_used: used,
                        phase: LiquidationPhase::ClearMatrices {
                            mechanism_idx,
                            map_idx: next_map,
                            cursor: None,
                        },
                    }
                }
            }
        }
    }

    /// Clear two-key maps where netuid is NOT the first key, in bounded chunks.
    /// Uses budget-limited iteration to avoid unbounded memory/weight usage.
    pub fn clear_two_key_maps_chunk(
        netuid: NetUid,
        map_idx: u8,
        weight_budget: Weight,
    ) -> ChunkResult {
        let limit = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_NEURON_CLEAR)
            .unwrap_or(100)
            .max(1) as usize;

        let (count, map_complete) = match TwoKeyMap::from(map_idx) {
            TwoKeyMap::ChildkeyTake => bounded_remove_by_netuid!(ChildkeyTake::<T>, netuid, limit),
            TwoKeyMap::ChildKeys => bounded_remove_by_netuid!(ChildKeys::<T>, netuid, limit),
            TwoKeyMap::ParentKeys => bounded_remove_by_netuid!(ParentKeys::<T>, netuid, limit),
            TwoKeyMap::LastHotkeyEmissionOnNetuid => {
                bounded_remove_by_netuid!(LastHotkeyEmissionOnNetuid::<T>, netuid, limit)
            }
            TwoKeyMap::TotalHotkeyAlphaLastEpoch => {
                bounded_remove_by_netuid!(TotalHotkeyAlphaLastEpoch::<T>, netuid, limit)
            }
            TwoKeyMap::IsNetworkMember => {
                bounded_remove_by_netuid!(IsNetworkMember::<T>, netuid, limit)
            }
            TwoKeyMap::HotkeyAlphaAndShares => Self::clear_hotkey_alpha_and_shares(netuid, limit),
            TwoKeyMap::LeasesAndIdentity => Self::clear_leases_and_identity(netuid),
            TwoKeyMap::Done => return ChunkResult::Complete(Weight::from_parts(FIXED_OVERHEAD, 0)),
        };

        let weight_used = Weight::from_parts(
            (count as u64)
                .max(1)
                .saturating_mul(WEIGHT_PER_NEURON_CLEAR),
            0,
        );

        if map_complete {
            let next_idx = map_idx.saturating_add(1);
            if next_idx > TwoKeyMap::LAST_IDX {
                ChunkResult::Complete(weight_used)
            } else {
                ChunkResult::Incomplete {
                    weight_used,
                    phase: LiquidationPhase::ClearTwoKeyMaps {
                        map_idx: next_idx,
                        cursor: None,
                    },
                }
            }
        } else {
            ChunkResult::Incomplete {
                weight_used,
                phase: LiquidationPhase::ClearTwoKeyMaps {
                    map_idx,
                    cursor: None,
                },
            }
        }
    }

    /// Remove TotalHotkeyAlpha + TotalHotkeyShares for this subnet, bounded by `limit`.
    /// When fully drained, also removes SubnetAlphaIn/Out.
    fn clear_hotkey_alpha_and_shares(netuid: NetUid, limit: usize) -> (usize, bool) {
        let mut keys: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();
        for (hot, n, _) in TotalHotkeyAlpha::<T>::iter() {
            if n == netuid {
                keys.push(hot);
                if keys.len() >= limit {
                    break;
                }
            }
        }
        let complete = keys.len() < limit;
        for hot in &keys {
            TotalHotkeyAlpha::<T>::remove(hot, netuid);
            TotalHotkeyShares::<T>::remove(hot, netuid);
        }
        if complete {
            SubnetAlphaIn::<T>::remove(netuid);
            SubnetAlphaOut::<T>::remove(netuid);
        }
        (keys.len(), complete)
    }

    /// Remove subnet lease data and identity.
    /// Uses a bounded limit for `clear_prefix` to avoid unbounded weight.
    fn clear_leases_and_identity(netuid: NetUid) -> (usize, bool) {
        if let Some(lease_id) = SubnetUidToLeaseId::<T>::take(netuid) {
            SubnetLeases::<T>::remove(lease_id);
            let result = SubnetLeaseShares::<T>::clear_prefix(lease_id, 1000, None);
            if result.maybe_cursor.is_some() {
                log::warn!(
                    "SubnetLeaseShares clear_prefix incomplete for lease {:?}",
                    lease_id,
                );
            }
            AccumulatedLeaseDividends::<T>::remove(lease_id);
        }
        SubnetIdentitiesV3::<T>::remove(netuid);
        (1, true)
    }

    /// Final cleanup: remove remaining entries, free subnet slot.
    pub fn final_cleanup(netuid: NetUid) {
        SubnetOwner::<T>::remove(netuid);
        SubnetworkN::<T>::remove(netuid);
        NetworksAdded::<T>::remove(netuid);
        TotalNetworks::<T>::mutate(|n: &mut u16| *n = n.saturating_sub(1));
        NetworkRegisteredAt::<T>::remove(netuid);
        MechanismCountCurrent::<T>::remove(netuid);
        SubnetTAO::<T>::remove(netuid);
        T::CommitmentsInterface::purge_netuid(netuid);
    }

    /// Calculate pro-rata share of TAO pot for a staker using U256 to prevent overflow.
    pub fn calculate_share(tao_pot: u64, alpha_val: u128, total_alpha: u128) -> u64 {
        if total_alpha == 0 || alpha_val == 0 || tao_pot == 0 {
            return 0;
        }

        let numerator = U256::from(tao_pot).saturating_mul(U256::from(alpha_val));
        let result = numerator
            .checked_div(U256::from(total_alpha))
            .unwrap_or(U256::zero());

        result.min(U256::from(u64::MAX)).as_u64()
    }

    /// Try to convert a raw cursor into a bounded cursor.
    /// Returns `None` and emits a warning on overflow.
    pub fn bound_cursor(raw: sp_std::vec::Vec<u8>, netuid: NetUid) -> Option<CursorBytes> {
        raw.try_into()
            .map_err(|_| {
                Self::deposit_event(Event::LiquidationWarning {
                    netuid,
                    warning: LiquidationWarning::CursorOverflow,
                });
            })
            .ok()
    }
}
