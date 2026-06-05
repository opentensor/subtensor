use super::*;
use alloc::collections::BTreeMap;
use safe_math::FixedExt;
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
            .filter(|&netuid| Self::get_network_registration_allowed(*netuid))
            .copied()
            .collect()
    }

    pub fn get_subnet_block_emissions(
        subnets_to_emit_to: &[NetUid],
        block_emission: U96F32,
    ) -> BTreeMap<NetUid, U96F32> {
        // Disabled subnets get zero TAO-side emission, redistributed to enabled subnets.
        // They stay in the map so the normal alpha_out/root-prop path still runs.
        let shares = Self::get_shares(subnets_to_emit_to);
        log::debug!("Subnet emission shares = {shares:?}");

        let zero = U64F64::saturating_from_num(0.0);
        let mut shares_with_emission_enabled = Vec::with_capacity(shares.len());
        let mut has_disabled_subnets = false;
        let mut enabled_share_sum = zero;

        for (netuid, share) in shares {
            let emission_enabled = SubnetEmissionEnabled::<T>::get(netuid);

            if emission_enabled {
                enabled_share_sum = enabled_share_sum.saturating_add(share);
            } else {
                has_disabled_subnets = true;
            }

            shares_with_emission_enabled.push((netuid, share, emission_enabled));
        }

        shares_with_emission_enabled
            .into_iter()
            .map(|(netuid, share, emission_enabled)| {
                let share = if has_disabled_subnets {
                    if emission_enabled && enabled_share_sum > zero {
                        share.safe_div(enabled_share_sum)
                    } else {
                        zero
                    }
                } else {
                    share
                };
                let emission = U64F64::saturating_from_num(block_emission).saturating_mul(share);
                (netuid, U96F32::saturating_from_num(emission))
            })
            .collect::<BTreeMap<NetUid, U96F32>>()
    }
    pub fn record_tao_inflow(netuid: NetUid, tao: TaoBalance) {
        SubnetTaoFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_add(u64::from(tao) as i64);
        });
    }

    pub fn record_tao_outflow(netuid: NetUid, tao: TaoBalance) {
        SubnetTaoFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_sub(u64::from(tao) as i64)
        });
    }

    pub fn reset_tao_outflow(netuid: NetUid) {
        SubnetTaoFlow::<T>::remove(netuid);
    }

    pub fn record_protocol_inflow(netuid: NetUid, tao: TaoBalance) {
        SubnetProtocolFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_add(u64::from(tao) as i64);
        });
    }

    pub fn record_protocol_outflow(netuid: NetUid, tao: TaoBalance) {
        SubnetProtocolFlow::<T>::mutate(netuid, |flow| {
            *flow = flow.saturating_sub(u64::from(tao) as i64);
        });
    }

    pub fn reset_protocol_flow(netuid: NetUid) {
        SubnetProtocolFlow::<T>::remove(netuid);
    }

    fn update_ema_protocol_flow(netuid: NetUid) -> I64F64 {
        let current_block: u64 = Self::get_current_block_as_u64();

        let block_flow = I64F64::saturating_from_num(SubnetProtocolFlow::<T>::get(netuid));
        let (last_block, last_block_ema) =
            SubnetEmaProtocolFlow::<T>::get(netuid).unwrap_or((0, I64F64::saturating_from_num(0)));

        if last_block != current_block {
            let flow_alpha = I64F64::saturating_from_num(FlowEmaSmoothingFactor::<T>::get())
                .safe_div(I64F64::saturating_from_num(i64::MAX));
            let one = I64F64::saturating_from_num(1);
            let ema_flow = (one.saturating_sub(flow_alpha))
                .saturating_mul(last_block_ema)
                .saturating_add(flow_alpha.saturating_mul(block_flow));
            SubnetEmaProtocolFlow::<T>::insert(netuid, (current_block, ema_flow));

            Self::reset_protocol_flow(netuid);
            ema_flow
        } else {
            last_block_ema
        }
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

    /// Compute the slow EMA of the raw user-flow EMA (second smoothing layer).
    ///
    /// Reuses the main `FlowEmaSmoothingFactor` rather than introducing a separate
    /// maturity factor. A parameter sweep of the maturity half-life against both
    /// manipulation resistance and honest-subnet bootstrap time found the best
    /// balance at the same half-life as the main flow EMA: it sits at the knee of
    /// the trade-off (shorter weakens the clamp; longer barely improves resistance
    /// while slowing new-subnet onboarding). Equal factors also make the slow layer
    /// a clean double-EMA of the flow, which is the simplest behaviour to reason
    /// about and govern.
    ///
    /// This stores EMA(raw), NOT the clamped min(raw, slow). The clamp is applied
    /// at read time in `get_shares_flow`. Storing the unclamped slow EMA ensures it
    /// tracks the true long-run raw signal rather than the clamped value.
    ///
    /// On first access for a subnet, the slow EMA initializes to the current raw EMA,
    /// so existing subnets do not face an emission cliff at deployment.
    fn get_slow_ema_flow(netuid: NetUid, raw_ema: I64F64) -> I64F64 {
        let current_block: u64 = Self::get_current_block_as_u64();

        // First access: seed the slow EMA at the current raw EMA (so no emission
        // cliff at deployment) with last_block = 0. On any normal block
        // (current_block != 0) the update branch then runs and persists the value;
        // the first update is a no-op (slow = raw) and subsequent blocks smooth
        // normally. (At genuine block 0 this would skip persistence, but subnets do
        // not emit at genesis.)
        let (last_block, last_slow_ema) =
            SubnetEmaSlowTaoFlow::<T>::get(netuid).unwrap_or((0, raw_ema));

        if last_block != current_block {
            let flow_alpha = I64F64::saturating_from_num(FlowEmaSmoothingFactor::<T>::get())
                .safe_div(I64F64::saturating_from_num(i64::MAX));
            let one = I64F64::saturating_from_num(1);
            let slow_ema = (one.saturating_sub(flow_alpha))
                .saturating_mul(last_slow_ema)
                .saturating_add(flow_alpha.saturating_mul(raw_ema));
            SubnetEmaSlowTaoFlow::<T>::insert(netuid, (current_block, slow_ema));
            slow_ema
        } else {
            last_slow_ema
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
        let net_flow_enabled = NetTaoFlowEnabled::<T>::get();
        let zero = I64F64::saturating_from_num(0);

        // Always update all EMAs (keeps protocol/slow EMAs warm for when toggled on).
        // Fixes #2667: protocol EMA accumulator was only drained when enabled,
        // causing a shock on toggle.
        //
        // matured = min(raw, slow): a second EMA smoothing layer (slow EMA of the raw
        // flow EMA) that delays emission credit from inflow spikes (raw rises before
        // slow) while applying outflows immediately (raw falls below slow). This makes
        // emission share track durable demand rather than transient flow.
        let subnet_emas: Vec<(NetUid, I64F64, I64F64)> = subnets_to_emit_to
            .iter()
            .map(|netuid| {
                let raw_user_ema = Self::get_ema_flow(*netuid);
                let slow_user_ema = Self::get_slow_ema_flow(*netuid, raw_user_ema);
                let matured_user_ema = raw_user_ema.min(slow_user_ema);
                let protocol_ema = Self::update_ema_protocol_flow(*netuid);
                (*netuid, matured_user_ema, protocol_ema)
            })
            .collect();

        // When net flow is enabled, normalize protocol EMA so that its
        // positive total matches the matured user EMA positive total. This prevents
        // subsidy concentration: as emissions concentrate on fewer subnets,
        // their protocol EMA grows, but the normalization factor shrinks to
        // compensate, keeping the deduction proportional to user demand.
        let norm_factor = if net_flow_enabled {
            let (user_positive_ema_sum, protocol_positive_ema_sum) =
                subnet_emas
                    .iter()
                    .fold((zero, zero), |(su, sp), (_, u, p)| {
                        (
                            su.saturating_add((*u).max(zero)),
                            sp.saturating_add((*p).max(zero)),
                        )
                    });
            let one = I64F64::saturating_from_num(1);
            if protocol_positive_ema_sum > zero {
                user_positive_ema_sum
                    .safe_div(protocol_positive_ema_sum)
                    .min(one)
            } else {
                zero
            }
        } else {
            zero
        };
        log::debug!("Protocol normalization factor: {norm_factor:?}");

        let ema_flows: BTreeMap<NetUid, I64F64> = subnet_emas
            .into_iter()
            .map(|(netuid, matured_user_ema, protocol_ema)| {
                let net = if net_flow_enabled {
                    // Only scale positive protocol cost by norm_factor. Negative
                    // protocol cost (root drain > emissions) is a benefit, kept as-is.
                    let scaled_protocol = if protocol_ema > zero {
                        norm_factor.saturating_mul(protocol_ema)
                    } else {
                        protocol_ema
                    };
                    matured_user_ema.saturating_sub(scaled_protocol)
                } else {
                    matured_user_ema
                };
                (netuid, net)
            })
            .collect();
        log::debug!("EMA flows (net_flow_enabled={net_flow_enabled}): {ema_flows:?}");

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
