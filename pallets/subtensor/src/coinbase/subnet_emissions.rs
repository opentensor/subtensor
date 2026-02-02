use super::*;
use alloc::collections::BTreeMap;
use safe_math::*;
use substrate_fixed::transcendental::{exp, ln};
use substrate_fixed::types::{I32F32, I64F64, U64F64, U96F32};

impl<T: Config> Pallet<T> {
    pub fn get_subnets_to_emit_to(subnets: &[NetUid]) -> Vec<NetUid> {
        // Filter out root subnet.
        // Filter out subnets with no first emission block number.
        subnets
            .iter()
            .filter(|netuid| !netuid.is_root())
            .filter(|netuid| FirstEmissionBlockNumber::<T>::get(*netuid).is_some())
            .filter(|netuid| SubtokenEnabled::<T>::get(*netuid))
            .filter(|&netuid| {
                // Only emit TAO if the subnetwork allows registration.
                Self::get_network_registration_allowed(*netuid)
                    || Self::get_network_pow_registration_allowed(*netuid)
            })
            .copied()
            .collect()
    }

    /// Normalizes shares so they sum to 1.0. If all shares are zero, leaves them unchanged.
    pub(crate) fn normalize_shares(shares: &mut BTreeMap<NetUid, U64F64>) {
        let sum: U64F64 = shares.values().copied().sum();
        if sum > U64F64::saturating_from_num(0) {
            for share in shares.values_mut() {
                *share = share.safe_div(sum);
            }
        }
    }

    /// When EffectiveRootPropEmissionScaling is enabled, multiplies each subnet's share
    /// by min(EffectiveRootProp, RootProp) and re-normalizes shares to sum to 1.0.
    /// Using the minimum of the two prevents exploitation by disabling alpha validators
    /// to artificially inflate EffectiveRootProp above the configured RootProp.
    pub(crate) fn apply_effective_root_prop_scaling(shares: &mut BTreeMap<NetUid, U64F64>) {
        if !EffectiveRootPropEmissionScaling::<T>::get() {
            return;
        }

        for (netuid, share) in shares.iter_mut() {
            let effective_root_prop = EffectiveRootProp::<T>::get(netuid);
            let root_prop = U64F64::saturating_from_num(RootProp::<T>::get(netuid));
            *share = share.saturating_mul(effective_root_prop.min(root_prop));
        }

        Self::normalize_shares(shares);
    }

    /// Zeros shares outside top_k (by descending share value) and re-normalizes the rest.
    /// Subnets with equal shares at the boundary are included if they tie with the k-th position.
    pub(crate) fn zero_and_redistribute_bottom_shares(
        shares: &mut BTreeMap<NetUid, U64F64>,
        top_k: usize,
    ) {
        let zero = U64F64::saturating_from_num(0);
        if top_k == 0 || shares.is_empty() {
            // Zero everything
            for share in shares.values_mut() {
                *share = zero;
            }
            return;
        }
        if top_k >= shares.len() {
            return; // Nothing to filter
        }

        // Sort netuids by share descending
        let mut sorted: Vec<(NetUid, U64F64)> = shares.iter().map(|(k, v)| (*k, *v)).collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // The threshold is the share value at the k-th position (0-indexed: top_k - 1).
        // All entries with share >= threshold are kept (ties at the boundary are included).
        let threshold = sorted
            .get(top_k.saturating_sub(1))
            .map(|(_, v)| *v)
            .unwrap_or(zero);

        for share in shares.values_mut() {
            if *share < threshold {
                *share = zero;
            }
        }

        Self::normalize_shares(shares);
    }

    /// Filters subnets so only the top proportion (by share) receive emission.
    /// Uses ceil(count * proportion / 10000) to determine how many subnets to keep.
    /// A single subnet always counts as in top 50%.
    pub(crate) fn apply_top_subnet_proportion_filter(shares: &mut BTreeMap<NetUid, U64F64>) {
        let proportion = EmissionTopSubnetProportion::<T>::get();
        if proportion >= 10000 {
            return; // 100% means all subnets get emission
        }

        let total = shares.len() as u32;
        if total == 0 {
            return;
        }

        // ceil(total * proportion / 10000) using saturating arithmetic
        let top_k = (total as u64)
            .saturating_mul(proportion as u64)
            .div_ceil(10000);
        let top_k = top_k.max(1) as usize; // At least 1 subnet

        log::debug!(
            "EmissionTopSubnetProportion: keeping top {top_k} of {total} subnets (proportion: {proportion}/10000)"
        );

        Self::zero_and_redistribute_bottom_shares(shares, top_k);
    }

    /// Limits the number of subnets receiving emission to an absolute number.
    /// When limit is 0, no filtering occurs (disabled).
    /// When limit > 0 and less than the number of subnets with nonzero shares,
    /// zeros shares beyond the top `limit` subnets and re-normalizes.
    pub(crate) fn apply_top_subnet_absolute_limit(shares: &mut BTreeMap<NetUid, U64F64>) {
        let limit = EmissionTopSubnetAbsoluteLimit::<T>::get();
        if limit == 0 {
            return; // Disabled
        }

        let nonzero_count = shares
            .values()
            .filter(|v| **v > U64F64::saturating_from_num(0))
            .count();

        if nonzero_count <= limit as usize {
            return; // Already within limit
        }

        log::debug!(
            "EmissionTopSubnetAbsoluteLimit: limiting to top {limit} subnets (had {nonzero_count} nonzero)"
        );

        Self::zero_and_redistribute_bottom_shares(shares, limit as usize);
    }

    pub fn get_subnet_block_emissions(
        subnets_to_emit_to: &[NetUid],
        block_emission: U96F32,
    ) -> BTreeMap<NetUid, U96F32> {
        // Get subnet TAO emissions.
        let mut shares = Self::get_shares(subnets_to_emit_to);
        log::debug!("Subnet emission shares = {shares:?}");

        // Apply EffectiveRootProp scaling if enabled.
        Self::apply_effective_root_prop_scaling(&mut shares);

        // Apply top subnet proportion filter.
        Self::apply_top_subnet_proportion_filter(&mut shares);

        // Apply absolute subnet limit.
        Self::apply_top_subnet_absolute_limit(&mut shares);

        shares
            .into_iter()
            .map(|(netuid, share)| {
                let emission = U64F64::saturating_from_num(block_emission).saturating_mul(share);
                (netuid, U96F32::saturating_from_num(emission))
            })
            .collect::<BTreeMap<NetUid, U96F32>>()
    }

    pub fn record_tao_inflow(netuid: NetUid, tao: TaoCurrency) {
        SubnetTaoFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_add(u64::from(tao) as i64);
        });
    }

    pub fn record_tao_outflow(netuid: NetUid, tao: TaoCurrency) {
        SubnetTaoFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_sub(u64::from(tao) as i64)
        });
    }

    pub fn reset_tao_outflow(netuid: NetUid) {
        SubnetTaoFlow::<T>::remove(netuid);
    }

    // Update SubnetEmaTaoFlow if needed and return its value for
    // the current block
    #[allow(dead_code)]
    fn get_ema_flow(netuid: NetUid) -> I64F64 {
        let current_block: u64 = Self::get_current_block_as_u64();

        // Calculate net ema flow for the next block
        let block_flow = I64F64::saturating_from_num(SubnetTaoFlow::<T>::get(netuid));
        let (last_block, last_block_ema) =
            SubnetEmaTaoFlow::<T>::get(netuid).unwrap_or((0, I64F64::saturating_from_num(0)));

        // EMA flow already initialized
        if last_block != current_block {
            let flow_alpha = I64F64::saturating_from_num(FlowEmaSmoothingFactor::<T>::get())
                .safe_div(I64F64::saturating_from_num(i64::MAX));
            let one = I64F64::saturating_from_num(1);
            let ema_flow = (one.saturating_sub(flow_alpha))
                .saturating_mul(last_block_ema)
                .saturating_add(flow_alpha.saturating_mul(block_flow));
            SubnetEmaTaoFlow::<T>::insert(netuid, (current_block, ema_flow));

            // Drop the accumulated flow in the last block
            Self::reset_tao_outflow(netuid);
            ema_flow
        } else {
            last_block_ema
        }
    }

    // Either the minimal EMA flow L = min{Si}, or an artificial
    // cut off at some higher value A (TaoFlowCutoff)
    // L = max {A, min{min{S[i], 0}}}
    #[allow(dead_code)]
    fn get_lower_limit(ema_flows: &BTreeMap<NetUid, I64F64>) -> I64F64 {
        let zero = I64F64::saturating_from_num(0);
        let min_flow = ema_flows
            .values()
            .map(|flow| flow.min(&zero))
            .min()
            .unwrap_or(&zero);
        let flow_cutoff = TaoFlowCutoff::<T>::get();
        flow_cutoff.max(*min_flow)
    }

    // Estimate the upper value of pow with hardcoded p = 2
    fn pow_estimate(val: U64F64) -> U64F64 {
        val.saturating_mul(val)
    }

    fn safe_pow(val: U64F64, p: U64F64) -> U64F64 {
        // If val is too low so that ln(val) doesn't fit I32F32::MIN,
        // return 0 from the function
        let zero = U64F64::saturating_from_num(0);
        let i32f32_max = I32F32::saturating_from_num(i32::MAX);
        if let Ok(val_ln) = ln(I32F32::saturating_from_num(val)) {
            // If exp doesn't fit, do the best we can - max out on I32F32::MAX
            U64F64::saturating_from_num(I32F32::saturating_from_num(
                exp(I32F32::saturating_from_num(p).saturating_mul(val_ln)).unwrap_or(i32f32_max),
            ))
        } else {
            zero
        }
    }

    fn inplace_scale(offset_flows: &mut BTreeMap<NetUid, U64F64>) {
        let zero = U64F64::saturating_from_num(0);
        let flow_max = offset_flows.values().copied().max().unwrap_or(zero);

        // Calculate scale factor so that max becomes 1.0
        let flow_factor = U64F64::saturating_from_num(1).safe_div(flow_max);

        // Upscale/downscale in-place
        for flow in offset_flows.values_mut() {
            *flow = flow_factor.saturating_mul(*flow);
        }
    }

    pub(crate) fn inplace_pow_normalize(offset_flows: &mut BTreeMap<NetUid, U64F64>, p: U64F64) {
        // Scale offset flows so that that are no overflows and underflows when we use safe_pow:
        //  flow_factor * subnet_count * (flow_max ^ p) <= I32F32::MAX
        let zero = U64F64::saturating_from_num(0);
        let subnet_count = offset_flows.len();

        // Pre-scale to max 1.0
        Self::inplace_scale(offset_flows);

        // Scale to maximize precision
        let flow_max = offset_flows.values().copied().max().unwrap_or(zero);
        log::debug!("Offset flow max: {flow_max:?}");
        let flow_max_pow_est = Self::pow_estimate(flow_max);
        log::debug!("flow_max_pow_est: {flow_max_pow_est:?}");

        let max_times_count =
            U64F64::saturating_from_num(subnet_count).saturating_mul(flow_max_pow_est);
        let i32f32_max = U64F64::saturating_from_num(i32::MAX);
        let precision_min = i32f32_max.safe_div(U64F64::saturating_from_num(u64::MAX));

        // If max_times_count < precision_min, all flow values are too low to fit I32F32.
        if max_times_count >= precision_min {
            let epsilon =
                U64F64::saturating_from_num(1).safe_div(U64F64::saturating_from_num(1_000));
            let flow_factor = i32f32_max
                .safe_div(max_times_count)
                .checked_sqrt(epsilon)
                .unwrap_or(zero);

            // Calculate sum
            let sum = offset_flows
                .clone()
                .into_values()
                .map(|flow| flow_factor.saturating_mul(flow))
                .map(|scaled_flow| Self::safe_pow(scaled_flow, p))
                .sum();
            log::debug!("Scaled offset flow sum: {sum:?}");

            // Normalize in-place
            for flow in offset_flows.values_mut() {
                let scaled_flow = flow_factor.saturating_mul(*flow);
                *flow = Self::safe_pow(scaled_flow, p).safe_div(sum);
            }
        }
    }

    // Implementation of shares that uses TAO flow
    #[allow(dead_code)]
    fn get_shares_flow(subnets_to_emit_to: &[NetUid]) -> BTreeMap<NetUid, U64F64> {
        // Get raw flows
        let ema_flows = subnets_to_emit_to
            .iter()
            .map(|netuid| (*netuid, Self::get_ema_flow(*netuid)))
            .collect();
        log::debug!("EMA flows: {ema_flows:?}");

        // Clip the EMA flow with lower limit L
        // z[i] = max{S[i] − L, 0}
        let lower_limit = Self::get_lower_limit(&ema_flows);
        log::debug!("Lower flow limit: {lower_limit:?}");
        let mut offset_flows = ema_flows
            .iter()
            .map(|(netuid, flow)| {
                (
                    *netuid,
                    if *flow > lower_limit {
                        U64F64::saturating_from_num(flow.saturating_sub(lower_limit))
                    } else {
                        U64F64::saturating_from_num(0)
                    },
                )
            })
            .collect::<BTreeMap<NetUid, U64F64>>();

        // Normalize the set {z[i]}, using an exponent parameter (p ≥ 1)
        let p = FlowNormExponent::<T>::get();
        Self::inplace_pow_normalize(&mut offset_flows, p);
        offset_flows
    }

    // Combines ema price method and tao flow method linearly over FlowHalfLife blocks
    pub(crate) fn get_shares(subnets_to_emit_to: &[NetUid]) -> BTreeMap<NetUid, U64F64> {
        Self::get_shares_flow(subnets_to_emit_to)
        // Self::get_shares_price_ema(subnets_to_emit_to)
    }

    // DEPRECATED: Implementation of shares that uses EMA prices will be gradually deprecated
    #[allow(dead_code)]
    fn get_shares_price_ema(subnets_to_emit_to: &[NetUid]) -> BTreeMap<NetUid, U64F64> {
        // Get sum of alpha moving prices
        let total_moving_prices = subnets_to_emit_to
            .iter()
            .map(|netuid| U64F64::saturating_from_num(Self::get_moving_alpha_price(*netuid)))
            .fold(U64F64::saturating_from_num(0.0), |acc, ema| {
                acc.saturating_add(ema)
            });
        log::debug!("total_moving_prices: {total_moving_prices:?}");

        // Calculate shares.
        subnets_to_emit_to
            .iter()
            .map(|netuid| {
                let moving_price =
                    U64F64::saturating_from_num(Self::get_moving_alpha_price(*netuid));
                log::debug!("moving_price_i: {moving_price:?}");

                let share = moving_price
                    .checked_div(total_moving_prices)
                    .unwrap_or(U64F64::saturating_from_num(0));

                (*netuid, share)
            })
            .collect::<BTreeMap<NetUid, U64F64>>()
    }
}
