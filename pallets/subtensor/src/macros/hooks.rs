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
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            let block_step_result = Self::block_step();
            match block_step_result {
                Ok(_) => {
                    // --- If the block step was successful, return the weight.
                    log::debug!("Successfully ran block step.");
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                }
                Err(e) => {
                    // --- If the block step was unsuccessful, return the weight anyway.
                    log::error!("Error while stepping block: {:?}", e);
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                }
            }
        }

        // ---- Called on the finalization of this pallet. The code weight must be taken into account prior to the execution of this macro.
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are finalizing.
        fn on_finalize(_block_number: BlockNumberFor<T>) {
            // Self::do_on_finalize(block_number);
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
                .saturating_add(migrations::migrate_coldkey_swap_scheduled::migrate_coldkey_swap_scheduled::<T>());

            // fix storage version for subtensor pallet
            use frame_support::traits::{GetStorageVersion, StorageVersion};
            let current_version = <Pallet<T> as GetStorageVersion>::on_chain_storage_version();
            let v7 = StorageVersion::new(7);
            if current_version == StorageVersion::new(6) {
                v7.put::<Pallet<T>>();
            } else if current_version < v7 {
                panic!(
                    "Storage version mismatch: expected at least {:?}, found {:?}",
                    v7, current_version
                );
            }

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
}
