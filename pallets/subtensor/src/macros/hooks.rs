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
                // Doesn't check storage version. TODO: Remove after upgrade
                // Storage version v5 -> v6
                .saturating_add(migrations::migrate_total_issuance::migrate_total_issuance::<T>(false))
                // Populate OwnedHotkeys map for coldkey swap. Doesn't update storage vesion.
                // Storage version v6 -> v7
                .saturating_add(migrations::migrate_populate_owned_hotkeys::migrate_populate_owned::<T>())
                // Migrate Delegate Ids on chain
                .saturating_add(migrations::migrate_chain_identity::migrate_set_hotkey_identities::<T>())
                // Migrate Commit-Reval 2.0
                .saturating_add(migrations::migrate_commit_reveal_v2::migrate_commit_reveal_2::<T>())
                // Migrate to RAO
                .saturating_add(migrations::migrate_rao::migrate_rao::<T>())
				// Fix the IsNetworkMember map to be consistent with other storage maps
				.saturating_add(migrations::migrate_fix_is_network_member::migrate_fix_is_network_member::<T>())
				.saturating_add(migrations::migrate_subnet_volume::migrate_subnet_volume::<T>())
                // Upgrade identities to V2
                .saturating_add(migrations::migrate_identities_v2::migrate_identities_to_v2::<T>())
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
                // Migrate Subnet Identities to V3
                .saturating_add(migrations::migrate_subnet_identities_to_v3::migrate_subnet_identities_to_v3::<T>())
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
                .saturating_add(migrations::migrate_auto_stake_destination::migrate_auto_stake_destination::<T>());
            weight
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            Self::check_total_issuance()?;
            // Disabled: https://github.com/opentensor/subtensor/pull/1166
            // Self::check_total_stake()?;
            Ok(())
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
    }
}
