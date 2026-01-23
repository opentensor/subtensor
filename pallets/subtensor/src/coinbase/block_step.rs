use super::*;
use safe_math::*;
use substrate_fixed::types::{U96F32, U110F18};
use subtensor_runtime_common::{NetUid, TaoCurrency};

impl<T: Config + pallet_drand::Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        let last_block_hash: T::Hash = <frame_system::Pallet<T>>::parent_hash();

        // --- 1. Update registration burn prices.
        Self::update_registration_prices_for_networks();

        // --- 2. Get the current coinbase emission.
        let block_emission: U96F32 = U96F32::saturating_from_num(
            Self::get_block_emission()
                .unwrap_or(TaoCurrency::ZERO)
                .to_u64(),
        );
        log::debug!("Block emission: {block_emission:?}");

        // --- 3. Reveal matured weights.
        Self::reveal_crv3_commits();
        // --- 4. Run emission through network.
        Self::run_coinbase(block_emission);
        // --- 5. Update moving prices AFTER using them for emissions.
        Self::update_moving_prices();
        // --- 6. Update roop prop AFTER using them for emissions.
        Self::update_root_prop();
        // --- 7. Set pending children on the epoch; but only after the coinbase has been run.
        Self::try_set_pending_children(block_number);
        // --- 8. Run auto-claim root divs.
        Self::run_auto_claim_root_divs(last_block_hash);
        // --- 9. Populate root coldkey maps.
        Self::populate_root_coldkey_staking_maps();

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

    /// Updates burn price and resets per-block counters.
    ///
    /// Behavior:
    /// - Every BurnHalfLife blocks: burn is halved and RegistrationsThisInterval is reset.
    /// - Each block: if there were registrations in the previous block, burn is multiplied by BurnIncreaseMult^regs_prev.
    /// - Each block: RegistrationsThisBlock is reset to 0 (for the new block).
    pub fn update_registration_prices_for_networks() {
        let current_block: u64 = Self::get_current_block_as_u64();

        for (netuid, _) in NetworksAdded::<T>::iter() {
            // 1) Apply halving + interval reset when half-life interval elapses.
            let half_life: u16 = BurnHalfLife::<T>::get(netuid);
            if half_life > 0 {
                let last_halving: u64 = BurnLastHalvingBlock::<T>::get(netuid);
                let delta: u64 = current_block.saturating_sub(last_halving);

                let intervals_passed: u64 = delta / half_life as u64;
                if intervals_passed > 0 {
                    // burn halves once per interval passed: burn /= 2^intervals_passed
                    let burn_u64: u64 = Self::get_burn(netuid).into();
                    let shift: u32 = core::cmp::min(intervals_passed, 64) as u32;

                    let new_burn_u64: u64 = if shift >= 64 { 0 } else { burn_u64 >> shift };
                    let mut new_burn: TaoCurrency = new_burn_u64.into();
                    new_burn = Self::clamp_burn(netuid, new_burn);

                    Self::set_burn(netuid, new_burn);

                    BurnLastHalvingBlock::<T>::insert(
                        netuid,
                        last_halving
                            .saturating_add(intervals_passed.saturating_mul(half_life as u64)),
                    );

                    // interval reset (MaxRegistrationsPerInterval == 1)
                    RegistrationsThisInterval::<T>::insert(netuid, 0);
                }
            }

            // 2) Apply post-registration bump (from previous block's registrations).
            // Note: at start of block N, RegistrationsThisBlock contains block N-1 counts.
            if !netuid.is_root() {
                let regs_prev_block: u16 = RegistrationsThisBlock::<T>::get(netuid);
                if regs_prev_block > 0 {
                    let mult: u64 = BurnIncreaseMult::<T>::get(netuid).max(1);
                    let bump: u64 = Self::saturating_pow_u64(mult, regs_prev_block);

                    let burn_u64: u64 = Self::get_burn(netuid).into();
                    let new_burn_u64: u64 = burn_u64.saturating_mul(bump);

                    let mut new_burn: TaoCurrency = new_burn_u64.into();
                    new_burn = Self::clamp_burn(netuid, new_burn);

                    Self::set_burn(netuid, new_burn);
                }
            }

            // 3) Reset per-block count for the new block
            Self::set_registrations_this_block(netuid, 0);
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
            Self::get_min_difficulty(netuid)
        } else {
            next_value.saturating_to_num::<u64>()
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
            Self::get_min_burn(netuid)
        } else {
            next_value.saturating_to_num::<u64>().into()
        }
    }

    pub fn update_moving_prices() {
        let subnets_to_emit_to: Vec<NetUid> =
            Self::get_subnets_to_emit_to(&Self::get_all_subnet_netuids());
        // Only update price EMA for subnets that we emit to.
        for netuid_i in subnets_to_emit_to.iter() {
            // Update moving prices after using them above.
            Self::update_moving_price(*netuid_i);
        }
    }

    pub fn update_root_prop() {
        let subnets_to_emit_to: Vec<NetUid> =
            Self::get_subnets_to_emit_to(&Self::get_all_subnet_netuids());
        // Only root_prop for subnets that we emit to.
        for netuid_i in subnets_to_emit_to.iter() {
            let root_prop = Self::root_proportion(*netuid_i);

            RootProp::<T>::insert(netuid_i, root_prop);
        }
    }

    pub fn root_proportion(netuid: NetUid) -> U96F32 {
        let alpha_issuance = U96F32::from_num(Self::get_alpha_issuance(netuid));
        let root_tao: U96F32 = U96F32::from_num(SubnetTAO::<T>::get(NetUid::ROOT));
        let tao_weight: U96F32 = root_tao.saturating_mul(Self::get_tao_weight());

        let root_proportion: U96F32 = tao_weight
            .checked_div(tao_weight.saturating_add(alpha_issuance))
            .unwrap_or(U96F32::from_num(0.0));

        root_proportion
    }

    pub fn reveal_crv3_commits() {
        let netuids: Vec<NetUid> = Self::get_all_subnet_netuids();
        for netuid in netuids.into_iter().filter(|netuid| *netuid != NetUid::ROOT) {
            // Reveal matured weights.
            if let Err(e) = Self::reveal_crv3_commits_for_subnet(netuid) {
                log::warn!("Failed to reveal commits for subnet {netuid} due to error: {e:?}");
            };
        }
    }
}
