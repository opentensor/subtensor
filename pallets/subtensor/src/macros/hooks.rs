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
            let mut stake_jobs = StakeJobs::<T>::drain().collect::<Vec<_>>();

            // Sort jobs by job type
            stake_jobs.sort_by_key(|(_, job)| match job {
                StakeJob::AddStakeLimit { .. } => 0,
                StakeJob::AddStake { .. } => 1,
                StakeJob::RemoveStakeLimit { .. } => 2,
                StakeJob::RemoveStake { .. } => 3,
            });

            for (_, job) in stake_jobs.into_iter() {
                match job {
                    StakeJob::AddStake {
                        hotkey,
                        coldkey,
                        netuid,
                        stake_to_be_added,
                    } => {
                        let result = Self::do_add_stake(
                            dispatch::RawOrigin::Signed(coldkey.clone()).into(),
                            hotkey.clone(),
                            netuid,
                            stake_to_be_added,
                        );

                        if let Err(err) = result {
                            log::debug!(
                                "Failed to add aggregated stake: {:?}, {:?}, {:?}, {:?}, {:?}",
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                                err
                            );
                            Self::deposit_event(Event::FailedToAddAggregatedStake(
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                            ));
                        } else {
                            Self::deposit_event(Event::AggregatedStakeAdded(
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                            ));
                        }
                    }
                    StakeJob::AddStakeLimit {
                        hotkey,
                        coldkey,
                        netuid,
                        stake_to_be_added,
                        limit_price,
                        allow_partial,
                    } => {
                        let result = Self::do_add_stake_limit(
                            dispatch::RawOrigin::Signed(coldkey.clone()).into(),
                            hotkey.clone(),
                            netuid,
                            stake_to_be_added,
                            limit_price,
                            allow_partial,
                        );

                        if let Err(err) = result {
                            log::debug!(
                                "Failed to add aggregated limited stake: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                                limit_price,
                                allow_partial,
                                err
                            );
                            Self::deposit_event(Event::FailedToAddAggregatedLimitedStake(
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                                limit_price,
                                allow_partial,
                            ));
                        } else {
                            Self::deposit_event(Event::AggregatedLimitedStakeAdded(
                                coldkey,
                                hotkey,
                                netuid,
                                stake_to_be_added,
                                limit_price,
                                allow_partial,
                            ));
                        }
                    }
                    StakeJob::RemoveStake {
                        coldkey,
                        hotkey,
                        netuid,
                        alpha,
                        fee,
                    } => {
                        let tao_unstaked = Self::unstake_from_subnet(
                            &hotkey,
                            &coldkey,
                            netuid,
                            0,
                            fee,
                            Some(alpha),
                        );

                        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);
                        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);

                        if Self::get_total_stake_for_hotkey(&hotkey) < StakeThreshold::<T>::get() {
                            Self::get_all_subnet_netuids().iter().for_each(|netuid| {
                                PendingChildKeys::<T>::remove(netuid, &hotkey);
                            })
                        }

                        Self::deposit_event(Event::AggregatedStakeRemoved(
                            coldkey.clone(),
                            hotkey.clone(),
                            tao_unstaked,
                            alpha,
                            netuid,
                            fee,
                        ));
                    }
                    StakeJob::RemoveStakeLimit {
                        hotkey,
                        coldkey,
                        netuid,
                        alpha_unstaked,
                        limit_price,
                        allow_partial,
                    } => {
                        let result = Self::do_remove_stake_limit(
                            dispatch::RawOrigin::Signed(coldkey.clone()).into(),
                            hotkey.clone(),
                            netuid,
                            alpha_unstaked,
                            limit_price,
                            allow_partial,
                        );

                        if let Err(err) = result {
                            log::debug!(
                                "Failed to remove aggregated limited stake: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                                coldkey,
                                hotkey,
                                netuid,
                                alpha_unstaked,
                                limit_price,
                                allow_partial,
                                err
                            );
                            Self::deposit_event(Event::FailedToRemoveAggregatedLimitedStake(
                                coldkey,
                                hotkey,
                                netuid,
                                alpha_unstaked,
                                limit_price,
                                allow_partial,
                            ));
                        } else {
                            Self::deposit_event(Event::AggregatedLimitedStakeRemoved(
                                coldkey,
                                hotkey,
                                netuid,
                                alpha_unstaked,
                                limit_price,
                                allow_partial,
                            ));
                        }
                    }
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
                ;
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
