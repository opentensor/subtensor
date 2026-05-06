use frame_support::pallet_macros::pallet_section;
// use subtensor_commitments_interface::CommitmentsHandler;
// use subtensor_swap_interface::SwapHandler;
/// A [`pallet_section`] that defines the events for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod hooks {
    // ================
    // ==== Hooks =====
    // ================
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are initializing.
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            let hotkey_swap_clean_up_weight = Self::clean_up_hotkey_swap_records(block_number);

            let block_step_result = Self::block_step();
            match block_step_result {
                Ok(_) => {
                    // --- If the block step was successful, return the weight.
                    log::debug!("Successfully ran block step.");
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                        .saturating_add(hotkey_swap_clean_up_weight)
                }
                Err(e) => {
                    // --- If the block step was unsuccessful, return the weight anyway.
                    log::error!("Error while stepping block: {:?}", e);
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                        .saturating_add(hotkey_swap_clean_up_weight)
                }
            }
        }

        // ---- Called on the finalization of this pallet. The code weight must be taken into account prior to the execution of this macro.
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are finalizing.
        fn on_finalize(_block_number: BlockNumberFor<T>) {
            for _ in StakingOperationRateLimiter::<T>::drain() {
                // Clear all entries each block
            }
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            // --- Migrate storage
            let mut weight = frame_support::weights::Weight::from_parts(0, 0);

            // Hex encoded foundation coldkey
            let hex = hex_literal::hex![
                "feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"
            ];
            weight = weight
                // Initializes storage version (to 1)
                .saturating_add(migrations::migrate_to_v1_separate_emission::migrate_to_v1_separate_emission::<T>())
                // Storage version v1 -> v2
                .saturating_add(migrations::migrate_to_v2_fixed_total_stake::migrate_to_v2_fixed_total_stake::<T>())
                // Doesn't check storage version. TODO: Remove after upgrade
                .saturating_add(migrations::migrate_create_root_network::migrate_create_root_network::<T>())
                // Storage version v2 -> v3
                .saturating_add(migrations::migrate_transfer_ownership_to_foundation::migrate_transfer_ownership_to_foundation::<T>(
                    hex,
                ))
                // Storage version v3 -> v4
                .saturating_add(migrations::migrate_delete_subnet_21::migrate_delete_subnet_21::<T>())
                // Storage version v4 -> v5
                .saturating_add(migrations::migrate_delete_subnet_3::migrate_delete_subnet_3::<T>())
                // Populate OwnedHotkeys map for coldkey swap. Doesn't update storage vesion.
                // Storage version v6 -> v7
                .saturating_add(migrations::migrate_populate_owned_hotkeys::migrate_populate_owned::<T>())
                // Migrate Commit-Reval 2.0
                .saturating_add(migrations::migrate_commit_reveal_v2::migrate_commit_reveal_2::<T>())
                // Migrate to RAO
                .saturating_add(migrations::migrate_rao::migrate_rao::<T>())
				// Fix the IsNetworkMember map to be consistent with other storage maps
				.saturating_add(migrations::migrate_fix_is_network_member::migrate_fix_is_network_member::<T>())
				.saturating_add(migrations::migrate_subnet_volume::migrate_subnet_volume::<T>())
				// Set the min burn across all subnets to a new minimum
				.saturating_add(migrations::migrate_set_min_burn::migrate_set_min_burn::<T>())
				// Set the min difficulty across all subnets to a new minimum
				.saturating_add(migrations::migrate_set_min_difficulty::migrate_set_min_difficulty::<T>())
                // Remove Stake map entries
				.saturating_add(migrations::migrate_remove_stake_map::migrate_remove_stake_map::<T>())
                // Remove unused maps entries
				.saturating_add(migrations::migrate_remove_unused_maps_and_values::migrate_remove_unused_maps_and_values::<T>())
                // Set last emission block number for all existed subnets before start call feature applied
                .saturating_add(migrations::migrate_set_first_emission_block_number::migrate_set_first_emission_block_number::<T>())
                // Remove all zero value entries in TotalHotkeyAlpha
                .saturating_add(migrations::migrate_remove_zero_total_hotkey_alpha::migrate_remove_zero_total_hotkey_alpha::<T>())
                // Wipe existing items to prevent bad decoding for new type
                .saturating_add(migrations::migrate_upgrade_revealed_commitments::migrate_upgrade_revealed_commitments::<T>())
                // Set subtoken enabled for all existed subnets
                .saturating_add(migrations::migrate_set_subtoken_enabled::migrate_set_subtoken_enabled::<T>())
                // Remove all entries in TotalHotkeyColdkeyStakesThisInterval
                .saturating_add(migrations::migrate_remove_total_hotkey_coldkey_stakes_this_interval::migrate_remove_total_hotkey_coldkey_stakes_this_interval::<T>())
                // Wipe the deprecated RateLimit storage item in the commitments pallet
                .saturating_add(migrations::migrate_remove_commitments_rate_limit::migrate_remove_commitments_rate_limit::<T>())
                // Remove all entries in orphaned storage items
                .saturating_add(
                    migrations::migrate_orphaned_storage_items::migrate_orphaned_storage_items::<T>(
                    ),
                )
                // Reset bonds moving average
                .saturating_add(migrations::migrate_reset_bonds_moving_average::migrate_reset_bonds_moving_average::<T>())
                // Reset max burn
                .saturating_add(migrations::migrate_reset_max_burn::migrate_reset_max_burn::<T>())
                // Migrate ColdkeySwapScheduled structure to new format
                .saturating_add(migrations::migrate_coldkey_swap_scheduled::migrate_coldkey_swap_scheduled::<T>())
                // Fix the root subnet TAO storage value
                .saturating_add(migrations::migrate_fix_root_subnet_tao::migrate_fix_root_subnet_tao::<T>())
                // Fix the owner disable the registration
                .saturating_add(migrations::migrate_set_registration_enable::migrate_set_registration_enable::<T>())
                // Migrate subnet symbols to fix the shift after subnet 81
                .saturating_add(migrations::migrate_subnet_symbols::migrate_subnet_symbols::<T>())
                // Migrate CRV3 add commit_block
                .saturating_add(migrations::migrate_crv3_commits_add_block::migrate_crv3_commits_add_block::<T>())
                // Migrate Commit-Reveal Settings
                .saturating_add(migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<T>())
                //Migrate CRV3 to TimelockedCommits
                .saturating_add(migrations::migrate_crv3_v2_to_timelocked::migrate_crv3_v2_to_timelocked::<T>())
                // Migrate to fix root counters
                .saturating_add(migrations::migrate_fix_root_tao_and_alpha_in::migrate_fix_root_tao_and_alpha_in::<T>())
                // Migrate last block rate limiting storage items
                .saturating_add(migrations::migrate_rate_limiting_last_blocks::migrate_obsolete_rate_limiting_last_blocks_storage::<T>())
                // Re-encode rate limit keys after introducing OwnerHyperparamUpdate variant
                .saturating_add(migrations::migrate_rate_limit_keys::migrate_rate_limit_keys::<T>())
                // Migrate remove network modality
                .saturating_add(migrations::migrate_remove_network_modality::migrate_remove_network_modality::<T>())
                // Migrate Immunity Period
                .saturating_add(migrations::migrate_network_immunity_period::migrate_network_immunity_period::<T>())
                // Migrate Subnet Limit
                .saturating_add(migrations::migrate_subnet_limit_to_default::migrate_subnet_limit_to_default::<T>())
                // Migrate Lock Reduction Interval
                .saturating_add(migrations::migrate_network_lock_reduction_interval::migrate_network_lock_reduction_interval::<T>())
                // Migrate subnet locked balances
                .saturating_add(migrations::migrate_subnet_locked::migrate_restore_subnet_locked::<T>())
                // Migrate subnet burn cost to 2500
                .saturating_add(migrations::migrate_network_lock_cost_2500::migrate_network_lock_cost_2500::<T>())
                // Cleanup child/parent keys
                .saturating_add(migrations::migrate_fix_childkeys::migrate_fix_childkeys::<T>())
                // Migrate AutoStakeDestinationColdkeys
                .saturating_add(migrations::migrate_auto_stake_destination::migrate_auto_stake_destination::<T>())
                // Migrate Kappa to default (0.5)
                .saturating_add(migrations::migrate_kappa_map_to_default::migrate_kappa_map_to_default::<T>())
                // Remove obsolete map entries
                .saturating_add(migrations::migrate_remove_tao_dividends::migrate_remove_tao_dividends::<T>())
                // Re-init tao flows
                .saturating_add(migrations::migrate_init_tao_flow::migrate_init_tao_flow::<T>())
                // Migrate pending emissions
                .saturating_add(migrations::migrate_pending_emissions::migrate_pending_emissions::<T>())
                // Reset unactive subnets
                .saturating_add(migrations::migrate_reset_unactive_sn::migrate_reset_unactive_sn::<T>())
                // Remove old identity map entries(Identities, SubnetIdentities, SubnetIdentitiesV2)
                .saturating_add(migrations::migrate_remove_old_identity_maps::migrate_remove_old_identity_maps::<T>())
                // Remove unknown neuron axon, certificate prom
                .saturating_add(migrations::migrate_remove_unknown_neuron_axon_cert_prom::migrate_remove_unknown_neuron_axon_cert_prom::<T>())
                // Fix staking hot keys
                .saturating_add(migrations::migrate_fix_staking_hot_keys::migrate_fix_staking_hot_keys::<T>())
                // Migrate coldkey swap scheduled to announcements
                .saturating_add(migrations::migrate_coldkey_swap_scheduled_to_announcements::migrate_coldkey_swap_scheduled_to_announcements::<T>())
                // Migration for new Neuron Registration
                .saturating_add(migrations::migrate_clear_deprecated_registration_maps::migrate_clear_deprecated_registration_maps::<T>())
                // Migrate fix bad hk swap
                .saturating_add(migrations::migrate_fix_bad_hk_swap::migrate_fix_bad_hk_swap::<T>())
                // Fix RootClaimed overclaim caused by single-subnet hotkey swap bug
                .saturating_add(migrations::migrate_fix_root_claimed_overclaim::migrate_fix_root_claimed_overclaim::<T>())
                // Mint missing SubnetTAO and SubnetLocked into subnet accounts to make TotalIssuance match in balances and subtensor
                .saturating_add(migrations::migrate_subnet_balances::migrate_subnet_balances::<T>());
            weight
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            Self::check_total_issuance()?;
            // Disabled: https://github.com/opentensor/subtensor/pull/1166
            // Self::check_total_stake()?;
            Ok(())
        }

        fn on_idle(_block: BlockNumberFor<T>, limit: Weight) -> Weight {
            let dissolved_networks = DissolvedNetworks::<T>::get();
            match dissolved_networks.get(0) {
                Some(netuid) => {
                    let used = limit
                        .saturating_sub(Self::remove_data_for_dissolved_networks(limit, netuid));
                    used
                }
                None => Weight::from_parts(0, 0),
            }
        }
    }

    impl<T: Config> Pallet<T> {
        // This function is to clean up the old hotkey swap records
        // It just clean up for one subnet at a time, according to the block number
        fn clean_up_hotkey_swap_records(block_number: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::from_parts(0, 0);
            let hotkey_swap_on_subnet_interval = T::HotkeySwapOnSubnetInterval::get();
            let block_number: u64 = TryInto::try_into(block_number)
                .ok()
                .expect("blockchain will not exceed 2^64 blocks; QED.");
            weight.saturating_accrue(T::DbWeight::get().reads(2_u64));

            let netuids = Self::get_all_subnet_netuids();
            weight.saturating_accrue(T::DbWeight::get().reads(netuids.len() as u64));

            if let Some(slot) = block_number.checked_rem(hotkey_swap_on_subnet_interval) {
                // only handle the subnet with the same residue as current block number by HotkeySwapOnSubnetInterval
                for netuid in netuids.iter().filter(|netuid| {
                    (u16::from(**netuid) as u64).checked_rem(hotkey_swap_on_subnet_interval)
                        == Some(slot)
                }) {
                    // Iterate over all the coldkeys in the subnet
                    for (coldkey, swap_block_number) in
                        LastHotkeySwapOnNetuid::<T>::iter_prefix(netuid)
                    {
                        // Clean up out of date swap records
                        if swap_block_number.saturating_add(hotkey_swap_on_subnet_interval)
                            < block_number
                        {
                            LastHotkeySwapOnNetuid::<T>::remove(netuid, coldkey);
                            weight.saturating_accrue(T::DbWeight::get().writes(1_u64));
                        }
                        weight.saturating_accrue(T::DbWeight::get().reads(1_u64));
                    }
                }
            }
            weight
        }

        // Cleans data for a dissolved network within the available block weight.
        //
        // The cleanup runs one stored phase at a time. `DissolvedNetworksCleanupPhase` is a
        // single `StorageValue` that tracks progress for the head of `DissolvedNetworks`
        // (the `netuid` passed here must be that head). If a phase completes, the next phase
        // is stored. Once all phases complete, the subnet is removed from `DissolvedNetworks`
        // and `DissolvedNetworkDataCleaned` is emitted.
        //
        // # Args:
        // 	* 'remaining_weight': (Weight):
        // 		- The weight available for this cleanup step.
        // 	* 'netuid': (&NetUid):
        // 		- The subnet to clean dissolved-network data for.
        //
        // # Returns:
        // 	* 'Weight': The weight remaining after the cleanup step.
        //
        fn remove_data_for_dissolved_networks(remaining_weight: Weight, netuid: &NetUid) -> Weight {
            let mut remaining_weight = remaining_weight;

            // if no phase is set, set the first phase
            if DissolvedNetworksCleanupPhase::<T>::get().is_none() {
                DissolvedNetworksCleanupPhase::<T>::set(Some(
                    DissolvedNetworksCleanupPhaseEnum::CleanSubnetRootDividendsRootClaimable,
                ));
            }

            // if one phase is done or exit because of weight limit
            let mut phase_done = true;
            // only reason for phase_done to be false is if the weight limit is reached
            while phase_done {
                if let Some(phase) = DissolvedNetworksCleanupPhase::<T>::get() {
                    log::error!("=== dissolved_networks phase: {:?}", phase);
                    let (weight_used, done) = match phase {
                        DissolvedNetworksCleanupPhaseEnum::CleanSubnetRootDividendsRootClaimable => {
                            let (weight_used, done) =
                                Self::clean_up_root_claimable_for_subnet(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::CleanSubnetRootDividendsRootClaimed,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::CleanSubnetRootDividendsRootClaimed => {
                            let (weight_used, done) =
                                Self::clean_up_root_claimed_for_subnet(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesGetTotalAlphaValue,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesGetTotalAlphaValue => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes_get_total_alpha_value(
                                *netuid,
                                remaining_weight,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesSettleStakes,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesSettleStakes => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes_settle_stakes(
                                *netuid,
                                remaining_weight,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesCleanAlpha,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesCleanAlpha => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes_clean_alpha(
                                *netuid,
                                remaining_weight,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesClearHotkeyTotals,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesClearHotkeyTotals => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes_clear_hotkey_totals(
                                *netuid,
                                remaining_weight,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesClearLocks,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakesClearLocks => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes_clear_locks(
                                *netuid,
                                remaining_weight,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakes,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::DestroyAlphaInOutStakes => {
                            let (weight_used, done) = Self::destroy_alpha_in_out_stakes(
                                *netuid,
                                remaining_weight,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::ClearProtocolLiquidity,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::ClearProtocolLiquidity => {
                            let (weight_used, done) =
                                T::SwapInterface::clear_protocol_liquidity(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::PurgeNetuid,
                                ));
                            }
                            (weight_used, done)
                        }

                        DissolvedNetworksCleanupPhaseEnum::PurgeNetuid => {
                            let (weight_used, done) =
                            T::CommitmentsInterface::purge_netuid(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkParameters,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkParameters => {
                            let (weight_used, done) =
                                Self::remove_network_parameters(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkMapParameters,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkMapParameters => {
                            let (weight_used, done) =
                                Self::remove_network_map_parameters(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkWeights,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkWeights => {
                            let (weight_used, done) =
                                Self::remove_network_weights(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkChildkeyTake,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkChildkeyTake => {
                            let (weight_used, done) =
                                Self::remove_network_childkey_take(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkChildkeys,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkChildkeys => {
                            let (weight_used, done) =
                                Self::remove_network_childkeys(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkParentkeys,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkParentkeys => {
                            let (weight_used, done) =
                                Self::remove_network_parentkeys(*netuid, remaining_weight);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkLastHotkeyEmissionOnNetuid,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkLastHotkeyEmissionOnNetuid => {
                            let (weight_used, done) =
                                Self::remove_network_last_hotkey_emission_on_netuid(
                                    *netuid,
                                    remaining_weight,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkTotalHotkeyAlphaLastEpoch,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkTotalHotkeyAlphaLastEpoch => {
                            let (weight_used, done) =
                                Self::remove_network_total_hotkey_alpha_last_epoch(
                                    *netuid,
                                    remaining_weight,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkTransactionKeyLastBlock,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkTransactionKeyLastBlock => {
                            let (weight_used, done) =
                                Self::remove_network_transaction_key_last_block(
                                    *netuid,
                                    remaining_weight,
                                );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolvedNetworksCleanupPhaseEnum::RemoveNetworkStakingOperationRateLimiter,
                                ));
                            }
                            (weight_used, done)
                        }
                        DissolvedNetworksCleanupPhaseEnum::RemoveNetworkStakingOperationRateLimiter => {
                            let (weight_used, done) =
                                Self::remove_network_staking_operation_rate_limiter(
                                    *netuid,
                                    remaining_weight,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(None);
                                DissolvedNetworks::<T>::mutate(|networks| {
                                    networks.retain(|n| *n != *netuid)
                                });
                                Self::deposit_event(Event::DissolvedNetworkDataCleaned { netuid: *netuid });
                            }
                            (weight_used, done)
                        }
                    };

                    phase_done = done;
                    remaining_weight = remaining_weight.saturating_sub(weight_used);

                    // if phase is cleared, break since all phases are done
                    if DissolvedNetworksCleanupPhase::<T>::get().is_none() {
                        break;
                    }
                }
            }

            remaining_weight
        }
    }
}
