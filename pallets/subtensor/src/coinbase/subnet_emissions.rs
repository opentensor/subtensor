use super::*;
use crate::alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use safe_math::FixedExt;
use substrate_fixed::transcendental::{exp, ln};
use substrate_fixed::types::{I32F32, I64F64, U64F64, U96F32};
use subtensor_swap_interface::SwapHandler;

impl<T: Config> Pallet<T> {
    pub fn get_subnet_block_emissions(
        subnets: &[NetUid],
        block_emission: U96F32,
    ) -> BTreeMap<NetUid, U96F32> {
        // Filter out subnets with no first emission block number.
        let subnets_to_emit_to: Vec<NetUid> = subnets
            .to_owned()
            .clone()
            .into_iter()
            .filter(|netuid| FirstEmissionBlockNumber::<T>::get(*netuid).is_some())
            .collect();
        log::debug!("Subnets to emit to: {subnets_to_emit_to:?}");

        // Get subnet TAO emissions.
        let shares = Self::get_shares(&subnets_to_emit_to);
        log::debug!("Subnet emission shares = {shares:?}");

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
        if let Some((last_block, last_block_ema)) = SubnetEmaTaoFlow::<T>::get(netuid) {
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
        } else {
            // Initialize EMA flow, set S(current_block) = min(price, ema_price) * init_factor
            let init_factor = I64F64::saturating_from_num(1_000_000_000);
            let moving_price = I64F64::saturating_from_num(Self::get_moving_alpha_price(netuid));
            let current_price =
                I64F64::saturating_from_num(T::SwapInterface::current_alpha_price(netuid));
            let ema_flow = init_factor.saturating_mul(moving_price.min(current_price));
            SubnetEmaTaoFlow::<T>::insert(netuid, (current_block, ema_flow));
            ema_flow
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
        // Self::get_shares_flow(subnets_to_emit_to)
        Self::get_shares_price_ema(subnets_to_emit_to)
    }

    // DEPRECATED: Implementation of shares that uses EMA prices will be gradually deprecated
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
