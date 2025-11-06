use super::*;
use safe_math::*;
use substrate_fixed::types::{U96F32, U110F18};
use subtensor_runtime_common::{NetUid, TaoCurrency};

impl<T: Config + pallet_drand::Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        let last_block_hash: T::Hash = <frame_system::Pallet<T>>::parent_hash();

        // --- 1. Adjust difficulties.
        Self::adjust_registration_terms_for_networks();
        // --- 2. Get the current coinbase emission.
        let block_emission: U96F32 = U96F32::saturating_from_num(
            Self::get_block_emission()
                .unwrap_or(TaoCurrency::ZERO)
                .to_u64(),
        );
        log::debug!("Block emission: {block_emission:?}");
        // --- 3. Run emission through network.
        Self::run_coinbase(block_emission);
        // --- 4. Set pending children on the epoch; but only after the coinbase has been run.
        Self::try_set_pending_children(block_number);
        // --- 5. Run auto-claim root divs.
        Self::run_auto_claim_root_divs(last_block_hash);
        // --- 6. Populate root coldkey maps.
        Self::populate_root_coldkey_staking_maps();
        // --- 6. Cleanup deregistered network.
        Self::iterate_and_clean_root_claim_data_for_subnet();

        // Return ok.
        Ok(())
    }

    fn try_set_pending_children(block_number: u64) {
        for netuid in Self::get_all_subnet_netuids() {
            if Self::should_run_epoch(netuid, block_number) {
                // Set pending children on the epoch.
                Self::do_set_pending_children(netuid);
            }
        }
    }

    /// Adjusts the network difficulties/burns of every active network. Resetting state parameters.
    ///
    pub fn adjust_registration_terms_for_networks() {
        log::debug!("adjust_registration_terms_for_networks");

        // --- 1. Iterate through each network.
        for (netuid, _) in NetworksAdded::<T>::iter() {
            // --- 2. Pull counters for network difficulty.
            let last_adjustment_block: u64 = Self::get_last_adjustment_block(netuid);
            let adjustment_interval: u16 = Self::get_adjustment_interval(netuid);
            let current_block: u64 = Self::get_current_block_as_u64();
            log::debug!(
                "netuid: {netuid:?} last_adjustment_block: {last_adjustment_block:?} adjustment_interval: {adjustment_interval:?} current_block: {current_block:?}"
            );

            // --- 3. Check if we are at the adjustment interval for this network.
            // If so, we need to adjust the registration difficulty based on target and actual registrations.
            if current_block.saturating_sub(last_adjustment_block) >= adjustment_interval as u64 {
                log::debug!("interval reached.");

                // --- 4. Get the current counters for this network w.r.t burn and difficulty values.
                let current_burn = Self::get_burn(netuid);
                let current_difficulty: u64 = Self::get_difficulty_as_u64(netuid);
                let registrations_this_interval: u16 =
                    Self::get_registrations_this_interval(netuid);
                let pow_registrations_this_interval: u16 =
                    Self::get_pow_registrations_this_interval(netuid);
                let burn_registrations_this_interval: u16 =
                    Self::get_burn_registrations_this_interval(netuid);
                let target_registrations_this_interval: u16 =
                    Self::get_target_registrations_per_interval(netuid);
                // --- 5. Adjust burn + pow
                // There are six cases to consider. A, B, C, D, E, F
                if registrations_this_interval > target_registrations_this_interval {
                    #[allow(clippy::comparison_chain)]
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // A. There are too many registrations this interval and most of them are pow registrations
                        // this triggers an increase in the pow difficulty.
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else if pow_registrations_this_interval < burn_registrations_this_interval {
                        // B. There are too many registrations this interval and most of them are burn registrations
                        // this triggers an increase in the burn cost.
                        // burn_cost ++
                        Self::set_burn(
                            netuid,
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else {
                        // F. There are too many registrations this interval and the pow and burn registrations are equal
                        // this triggers an increase in the burn cost and pow difficulty
                        // burn_cost ++
                        Self::set_burn(
                            netuid,
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    }
                } else {
                    // Not enough registrations this interval.
                    #[allow(clippy::comparison_chain)]
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // C. There are not enough registrations this interval and most of them are pow registrations
                        // this triggers a decrease in the burn cost
                        // burn_cost --
                        Self::set_burn(
                            netuid,
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else if pow_registrations_this_interval < burn_registrations_this_interval {
                        // D. There are not enough registrations this interval and most of them are burn registrations
                        // this triggers a decrease in the pow difficulty
                        // pow_difficulty --
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else {
                        // E. There are not enough registrations this interval and the pow and burn registrations are equal
                        // this triggers a decrease in the burn cost and pow difficulty
                        // burn_cost --
                        Self::set_burn(
                            netuid,
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty --
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    }
                }

                // --- 6. Drain all counters for this network for this interval.
                Self::set_last_adjustment_block(netuid, current_block);
                Self::set_registrations_this_interval(netuid, 0);
                Self::set_pow_registrations_this_interval(netuid, 0);
                Self::set_burn_registrations_this_interval(netuid, 0);
            } else {
                log::debug!("interval not reached.");
            }

            // --- 7. Drain block registrations for each network. Needed for registration rate limits.
            Self::set_registrations_this_block(netuid, 0);
        }
    }

    /// Calculates the upgraded difficulty by multiplying the current difficulty by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use U110F18 to avoid any overflows on u64. Also min_difficulty and max_difficulty bound the range.
    ///
    pub fn upgraded_difficulty(
        netuid: NetUid,
        current_difficulty: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_difficulty: U110F18 = U110F18::saturating_from_num(current_difficulty)
            .saturating_mul(U110F18::saturating_from_num(
                registrations_this_interval.saturating_add(target_registrations_per_interval),
            ))
            .safe_div(U110F18::saturating_from_num(
                target_registrations_per_interval.saturating_add(target_registrations_per_interval),
            ));
        let alpha: U110F18 = U110F18::saturating_from_num(Self::get_adjustment_alpha(netuid))
            .safe_div(U110F18::saturating_from_num(u64::MAX));
        let next_value: U110F18 = alpha
            .saturating_mul(U110F18::saturating_from_num(current_difficulty))
            .saturating_add(
                U110F18::saturating_from_num(1.0)
                    .saturating_sub(alpha)
                    .saturating_mul(updated_difficulty),
            );
        if next_value >= U110F18::saturating_from_num(Self::get_max_difficulty(netuid)) {
            Self::get_max_difficulty(netuid)
        } else if next_value <= U110F18::saturating_from_num(Self::get_min_difficulty(netuid)) {
            return Self::get_min_difficulty(netuid);
        } else {
            return next_value.saturating_to_num::<u64>();
        }
    }

    /// Calculates the upgraded burn by multiplying the current burn by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use U110F18 to avoid any overflows on u64. Also min_burn and max_burn bound the range.
    ///
    pub fn upgraded_burn(
        netuid: NetUid,
        current_burn: TaoCurrency,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> TaoCurrency {
        let updated_burn: U110F18 = U110F18::saturating_from_num(current_burn)
            .saturating_mul(U110F18::saturating_from_num(
                registrations_this_interval.saturating_add(target_registrations_per_interval),
            ))
            .safe_div(U110F18::saturating_from_num(
                target_registrations_per_interval.saturating_add(target_registrations_per_interval),
            ));
        let alpha: U110F18 = U110F18::saturating_from_num(Self::get_adjustment_alpha(netuid))
            .safe_div(U110F18::saturating_from_num(u64::MAX));
        let next_value: U110F18 = alpha
            .saturating_mul(U110F18::saturating_from_num(current_burn))
            .saturating_add(
                U110F18::saturating_from_num(1.0)
                    .saturating_sub(alpha)
                    .saturating_mul(updated_burn),
            );
        if next_value >= U110F18::saturating_from_num(Self::get_max_burn(netuid)) {
            Self::get_max_burn(netuid)
        } else if next_value <= U110F18::saturating_from_num(Self::get_min_burn(netuid)) {
            return Self::get_min_burn(netuid);
        } else {
            return next_value.saturating_to_num::<u64>().into();
        }
    }
}
