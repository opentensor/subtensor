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
