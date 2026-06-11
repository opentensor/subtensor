use frame_support::pallet_macros::pallet_section;
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
                // Remove AddStakeBurn entries from LastRateLimitedBlock
                .saturating_add(migrations::migrate_remove_add_stake_burn_rate_limit::migrate_remove_add_stake_burn_rate_limit::<T>())
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
                .saturating_add(migrations::migrate_subnet_balances::migrate_subnet_balances::<T>())
                // Fix testnet Subtensor TotalIssuance after the EVM fees issue.
                .saturating_add(migrations::migrate_fix_total_issuance_evm_fees::migrate_fix_total_issuance_evm_fees::<T>())
                // Remove deprecated conviction lock storage.
                .saturating_add(migrations::migrate_remove_deprecated_conviction_maps::migrate_remove_deprecated_conviction_maps::<T>())
                // Reset testnet conviction lock storage before deploying the current design.
                .saturating_add(migrations::migrate_reset_tnet_conviction_locks::migrate_reset_tnet_conviction_locks::<T>())
                // Capture the runtime-upgrade block for TAO-in refund cutover.
                .saturating_add(migrations::migrate_tao_in_refund_deployment_block::migrate_tao_in_refund_deployment_block::<T>());
            weight
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            // Disabled: https://github.com/opentensor/subtensor/pull/1166
            // Self::check_total_stake()?;
            Ok(())
        }

        fn on_idle(_block: BlockNumberFor<T>, limit: Weight) -> Weight {
            let dissolved_networks = DissolveCleanupQueue::<T>::get();
            let weight = match dissolved_networks.get(0) {
                Some(netuid) => Self::remove_data_for_dissolved_networks(limit, netuid),
                None => Weight::from_parts(0, 0),
            };
            Self::process_network_registration_queue();

            weight
        }
    }

    impl<T: Config> Pallet<T> {
        // This function is to clean up the old hotkey swap records
        // It just clean up for one subnet at a time, according to the block number
        pub(crate) fn clean_up_hotkey_swap_records(block_number: BlockNumberFor<T>) -> Weight {
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
        // single `StorageValue` that tracks progress for the head of `DissolveCleanupQueue`
        // (the `netuid` passed here must be that head). If a phase completes, the next phase
        // is stored. Once all phases complete, the subnet is removed from `DissolveCleanupQueue`
        // and `NetworkDissolveCleanupCompleted` is emitted.
        //
        // # Args:
        // 	* 'remaining_weight': (Weight):
        // 		- The weight available for this cleanup step.
        // 	* 'netuid': (&NetUid):
        // 		- The subnet to clean dissolved-network data for.
        //
        // # Returns:
        // 	* 'Weight': The weight used for the cleanup step.
        //
        fn remove_data_for_dissolved_networks(remaining_weight: Weight, netuid: &NetUid) -> Weight {
            let mut weight_meter =
                frame_support::weights::WeightMeter::with_limit(remaining_weight);

            // if no phase is set, set the first phase
            if DissolvedNetworksCleanupPhase::<T>::get().is_none() {
                DissolvedNetworksCleanupPhase::<T>::set(Some(
                    DissolveCleanupPhase::CleanSubnetRootDividendsRootClaimable { last_key: None },
                ));
            }

            // if one phase is done or exit because of weight limit
            let mut phase_done = true;
            // only reason for phase_done to be false is if the weight limit is reached
            while phase_done {
                if let Some(phase) = DissolvedNetworksCleanupPhase::<T>::get() {
                    log::debug!(
                        "dissolved_networks phase: {:?} for netuid: {:?}",
                        phase,
                        netuid
                    );
                    let done = match phase {
                        DissolveCleanupPhase::CleanSubnetRootDividendsRootClaimable {
                            last_key,
                        } => {
                            let (done, new_key) = Self::clean_up_root_claimable_for_subnet(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::CleanSubnetRootDividendsRootClaimed,
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::CleanSubnetRootDividendsRootClaimable {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::CleanSubnetRootDividendsRootClaimed => {
                            let done =
                                Self::clean_up_root_claimed_for_subnet(*netuid, &mut weight_meter);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesGetTotalAlphaValue { last_key: None },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakesGetTotalAlphaValue {
                            last_key,
                        } => {
                            let (done, new_key) =
                                Self::destroy_alpha_in_out_stakes_get_total_alpha_value(
                                    *netuid,
                                    &mut weight_meter,
                                    last_key,
                                );
                            if done {
                                DissolvedSubnetDistributedTao::<T>::set(Some(0));
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesSettleStakes {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesGetTotalAlphaValue {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakesSettleStakes { last_key } => {
                            let (done, new_key) = Self::destroy_alpha_in_out_stakes_settle_stakes(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesCleanAlpha {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesSettleStakes {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakesCleanAlpha { last_key } => {
                            let (done, new_key) = Self::destroy_alpha_in_out_stakes_clean_alpha(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesClearHotkeyTotals { last_key: None },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesCleanAlpha {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakesClearHotkeyTotals {
                            last_key,
                        } => {
                            let (done, new_key) =
                                Self::destroy_alpha_in_out_stakes_clear_hotkey_totals(
                                    *netuid,
                                    &mut weight_meter,
                                    last_key,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesClearLocks {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesClearHotkeyTotals {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakesClearLocks { last_key } => {
                            let (done, new_key) = Self::destroy_alpha_in_out_stakes_clear_locks(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakes,
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::DestroyAlphaInOutStakesClearLocks {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::DestroyAlphaInOutStakes => {
                            let done =
                                Self::destroy_alpha_in_out_stakes(*netuid, &mut weight_meter);
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::ClearProtocolLiquidity,
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::ClearProtocolLiquidity => {
                            let done = T::SwapInterface::clear_protocol_liquidity(
                                *netuid,
                                &mut weight_meter,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::PurgeNetuid,
                                ));
                            }
                            done
                        }

                        DissolveCleanupPhase::PurgeNetuid => {
                            let done =
                                T::CommitmentsInterface::purge_netuid(*netuid, &mut weight_meter);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkIsNetworkMember {
                                        last_key: None,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkIsNetworkMember { last_key } => {
                            let (done, new_key) = Self::remove_network_is_network_member(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkParameters,
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkIsNetworkMember {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkParameters => {
                            let done = Self::remove_network_parameters(*netuid, &mut weight_meter);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkMapParameters,
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkMapParameters => {
                            let done =
                                Self::remove_network_map_parameters(*netuid, &mut weight_meter);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkUpdateWeightsOnRoot {
                                        last_key: None,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkUpdateWeightsOnRoot { last_key } => {
                            let (done, new_key) = Self::remove_network_update_weights_on_root(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkChildkeyTake {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkUpdateWeightsOnRoot {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkChildkeyTake { last_key } => {
                            let (done, new_key) = Self::remove_network_childkey_take(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkChildkeys { last_key: None },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkChildkeyTake {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkChildkeys { last_key } => {
                            let (done, new_key) = Self::remove_network_childkeys(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkParentkeys {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkChildkeys {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkParentkeys { last_key } => {
                            let (done, new_key) = Self::remove_network_parentkeys(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkLastHotkeyEmissionOnNetuid {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkParentkeys {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkLastHotkeyEmissionOnNetuid {
                            last_key,
                        } => {
                            let (done, new_key) =
                                Self::remove_network_last_hotkey_emission_on_netuid(
                                    *netuid,
                                    &mut weight_meter,
                                    last_key,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkTotalHotkeyAlphaLastEpoch {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkLastHotkeyEmissionOnNetuid {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkTotalHotkeyAlphaLastEpoch {
                            last_key,
                        } => {
                            let (done, new_key) =
                                Self::remove_network_total_hotkey_alpha_last_epoch(
                                    *netuid,
                                    &mut weight_meter,
                                    last_key,
                                );

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkTransactionKeyLastBlock {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkTotalHotkeyAlphaLastEpoch {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkTransactionKeyLastBlock { last_key } => {
                            let (done, new_key) = Self::remove_network_transaction_key_last_block(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkLock { last_key: None },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkTransactionKeyLastBlock {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkLock { last_key } => {
                            let (done, new_key) =
                                Self::remove_network_lock(*netuid, &mut weight_meter, last_key);

                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkDecayingLock {
                                        last_key: None,
                                    },
                                ));
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkLock { last_key: new_key },
                                ));
                            }
                            done
                        }
                        DissolveCleanupPhase::RemoveNetworkDecayingLock { last_key } => {
                            let (done, new_key) = Self::remove_network_decaying_lock(
                                *netuid,
                                &mut weight_meter,
                                last_key,
                            );

                            // if all phases are done, remove the network from the dissolved networks list and emit the event
                            if done {
                                DissolvedNetworksCleanupPhase::<T>::set(None);
                                DissolveCleanupQueue::<T>::mutate(|networks| {
                                    networks.retain(|n| *n != *netuid)
                                });
                                Self::deposit_event(Event::NetworkDissolveCleanupCompleted {
                                    netuid: *netuid,
                                });
                            } else {
                                DissolvedNetworksCleanupPhase::<T>::set(Some(
                                    DissolveCleanupPhase::RemoveNetworkDecayingLock {
                                        last_key: new_key,
                                    },
                                ));
                            }
                            done
                        }
                    };

                    phase_done = done;

                    // if phase is cleared, break since all phases are done
                    if DissolvedNetworksCleanupPhase::<T>::get().is_none() {
                        break;
                    }
                }
            }

            weight_meter.consumed()
        }

        pub(crate) fn process_network_registration_queue() {
            let queue = NetworkRegistrationQueue::<T>::get();
            if queue.is_empty() {
                return;
            }

            // Only release a queued registration once dissolve cleanup has actually
            // freed a slot; mirrors the queueing condition in `do_register_network`.
            let subnet_limit = u64::from(Self::get_max_subnets());
            let current_count = NetworksAdded::<T>::iter()
                .filter(|(netuid, added)| *added && *netuid != NetUid::ROOT)
                .count() as u64;
            let cleanup_queue_len = DissolveCleanupQueue::<T>::get().len() as u64;

            if current_count.saturating_add(cleanup_queue_len) >= subnet_limit {
                return;
            }

            for (index, info) in queue.iter().enumerate() {
                let result = Self::set_new_network_state(
                    &info.coldkey,
                    &info.hotkey,
                    info.mechid,
                    info.identity.clone(),
                    info.lock_amount,
                    info.median_subnet_alpha_price,
                    true,
                );
                if result.is_ok() {
                    NetworkRegistrationQueue::<T>::mutate(|queue| queue.remove(index));
                    break;
                }
            }
        }
    }
}
