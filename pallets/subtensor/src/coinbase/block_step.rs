use frame_support::storage::IterableStorageMap;
use substrate_fixed::types::I110F18;

use super::*;
use crate::AdjustmentAlpha;

impl<T: Config + pallet_drand::Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        log::debug!("block_step for block: {:?} ", block_number);
        // --- 1. Adjust difficulties.
        Self::adjust_registration_terms_for_networks();
        // --- 2. Run emission through network.
        Self::run_coinbase();
        // Return ok.
        Ok(())
    }

    /// Adjusts the network difficulties/burns of every active network. Resetting state parameters.
    ///
    pub fn adjust_registration_terms_for_networks() {
        log::debug!("adjust_registration_terms_for_networks");

        // --- 1. Iterate through each network.
        for (netuid, _) in <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter() {
            // --- 2. Pull counters for network difficulty.
            let last_adjustment_block: u64 = LastAdjustmentBlock::<T>::get(netuid);
            let adjustment_interval: u16 = AdjustmentInterval::<T>::get(netuid);
            let current_block: u64 = Self::get_current_block_as_u64();
            log::debug!("netuid: {:?} last_adjustment_block: {:?} adjustment_interval: {:?} current_block: {:?}",
                netuid,
                last_adjustment_block,
                adjustment_interval,
                current_block
            );

            // --- 3. Check if we are at the adjustment interval for this network.
            // If so, we need to adjust the registration difficulty based on target and actual registrations.
            if current_block.saturating_sub(last_adjustment_block) >= adjustment_interval as u64 {
                log::debug!("interval reached.");

                // --- 4. Get the current counters for this network w.r.t burn and difficulty values.
                let current_burn: u64 = Burn::<T>::get(netuid);
                let current_difficulty: u64 = Difficulty::<T>::get(netuid);
                let registrations_this_interval: u16 = RegistrationsThisInterval::<T>::get(netuid);
                let pow_registrations_this_interval: u16 =
                    POWRegistrationsThisInterval::<T>::get(netuid);
                let burn_registrations_this_interval: u16 =
                    BurnRegistrationsThisInterval::<T>::get(netuid);
                let target_registrations_this_interval: u16 =
                    TargetRegistrationsPerInterval::<T>::get(netuid);
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
                        Burn::<T>::insert(
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
                        Burn::<T>::insert(
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
                        Burn::<T>::insert(
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
                        Burn::<T>::insert(
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
                LastAdjustmentBlock::<T>::insert(netuid, current_block);
                RegistrationsThisInterval::<T>::insert(netuid, 0);
                POWRegistrationsThisInterval::<T>::insert(netuid, 0);
                BurnRegistrationsThisInterval::<T>::insert(netuid, 0);
            } else {
                log::debug!("interval not reached.");
            }

            // --- 7. Drain block registrations for each network. Needed for registration rate limits.
            RegistrationsThisBlock::<T>::insert(netuid, 0);
        }
    }

    /// Calculates the upgraded difficulty by multiplying the current difficulty by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use I110F18 to avoid any overflows on u64. Also min_difficulty and max_difficulty bound the range.
    ///
    pub fn upgraded_difficulty(
        netuid: u16,
        current_difficulty: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_difficulty: I110F18 = I110F18::from_num(current_difficulty)
            .saturating_mul(I110F18::from_num(
                registrations_this_interval.saturating_add(target_registrations_per_interval),
            ))
            .saturating_div(I110F18::from_num(
                target_registrations_per_interval.saturating_add(target_registrations_per_interval),
            ));
        let alpha: I110F18 = I110F18::from_num(AdjustmentAlpha::<T>::get(netuid))
            .saturating_div(I110F18::from_num(u64::MAX));
        let next_value: I110F18 = alpha
            .saturating_mul(I110F18::from_num(current_difficulty))
            .saturating_add(
                I110F18::from_num(1.0)
                    .saturating_sub(alpha)
                    .saturating_mul(updated_difficulty),
            );
        if next_value >= I110F18::from_num(MaxDifficulty::<T>::get(netuid)) {
            MaxDifficulty::<T>::get(netuid)
        } else if next_value <= I110F18::from_num(MinDifficulty::<T>::get(netuid)) {
            return MinDifficulty::<T>::get(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }

    /// Calculates the upgraded burn by multiplying the current burn by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use I110F18 to avoid any overflows on u64. Also min_burn and max_burn bound the range.
    ///
    pub fn upgraded_burn(
        netuid: u16,
        current_burn: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_burn: I110F18 = I110F18::from_num(current_burn)
            .saturating_mul(I110F18::from_num(
                registrations_this_interval.saturating_add(target_registrations_per_interval),
            ))
            .saturating_div(I110F18::from_num(
                target_registrations_per_interval.saturating_add(target_registrations_per_interval),
            ));
        let alpha: I110F18 = I110F18::from_num(AdjustmentAlpha::<T>::get(netuid))
            .saturating_div(I110F18::from_num(u64::MAX));
        let next_value: I110F18 = alpha
            .saturating_mul(I110F18::from_num(current_burn))
            .saturating_add(
                I110F18::from_num(1.0)
                    .saturating_sub(alpha)
                    .saturating_mul(updated_burn),
            );
        if next_value >= I110F18::from_num(MaxBurn::<T>::get(netuid)) {
            MaxBurn::<T>::get(netuid)
        } else if next_value <= I110F18::from_num(MinBurn::<T>::get(netuid)) {
            return MinBurn::<T>::get(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }
}
