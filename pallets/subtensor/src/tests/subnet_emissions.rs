#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;
use crate::*;
use alloc::{collections::BTreeMap, vec::Vec};
use approx::assert_abs_diff_eq;
use sp_core::U256;
use substrate_fixed::types::{I64F64, I96F32, U64F64, U96F32};
use subtensor_runtime_common::{NetUid, TaoBalance};

fn u64f64(x: f64) -> U64F64 {
    U64F64::from_num(x)
}

fn i64f64(x: f64) -> I64F64 {
    I64F64::from_num(x)
}

fn i96f32(x: f64) -> I96F32 {
    I96F32::from_num(x)
}

#[test]
fn inplace_pow_normalize_all_zero_inputs_no_panic_and_unchanged() {
    let mut m: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    m.insert(NetUid::from(1), u64f64(0.0));
    m.insert(NetUid::from(2), u64f64(0.0));
    m.insert(NetUid::from(3), u64f64(0.0));

    let before = m.clone();
    // p = 1.0 (doesn't matter here)
    SubtensorModule::inplace_pow_normalize(&mut m, u64f64(1.0));

    // Expect unchanged (sum becomes 0 → safe_div handles, or branch skips)
    for (k, v_before) in before {
        let v_after = m.get(&k).copied().unwrap();
        assert_abs_diff_eq!(
            v_after.to_num::<f64>(),
            v_before.to_num::<f64>(),
            epsilon = 1e-18
        );
    }
}

#[test]
fn inplace_pow_normalize_tiny_values_no_panic() {
    use alloc::collections::BTreeMap;

    // Very small inputs so that scaling branch is skipped in inplace_pow_normalize
    let mut m: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    m.insert(NetUid::from(10), u64f64(1e-9));
    m.insert(NetUid::from(11), u64f64(2e-9));
    m.insert(NetUid::from(12), u64f64(3e-9));

    let before = m.clone();
    SubtensorModule::inplace_pow_normalize(&mut m, u64f64(2.0)); // p = 2

    let sum = (1 + 4 + 9) as f64;
    for (k, v_before) in before {
        let v_after = m.get(&k).copied().unwrap();
        let mut expected = v_before.to_num::<f64>();
        expected *= 1e18 * expected / sum;
        assert_abs_diff_eq!(
            v_after.to_num::<f64>(),
            expected,
            epsilon = expected / 100.0
        );
    }
}

#[test]
fn inplace_pow_normalize_large_values_no_overflow_and_sum_to_one() {
    use alloc::collections::BTreeMap;

    let mut m: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    m.insert(NetUid::from(1), u64f64(1e9));
    m.insert(NetUid::from(2), u64f64(5e9));
    m.insert(NetUid::from(3), u64f64(1e10));

    SubtensorModule::inplace_pow_normalize(&mut m, u64f64(2.0)); // p = 2

    // Sum ≈ 1
    let sum: f64 = m.values().map(|v| v.to_num::<f64>()).sum();
    assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-9);

    // Each value is finite and within [0, 1]
    for (k, v) in &m {
        let f = v.to_num::<f64>();
        assert!(f.is_finite(), "value for {k:?} not finite");
        assert!(
            (0.0..=1.0).contains(&f),
            "value for {k:?} out of [0,1]: {f}"
        );
    }
}

#[test]
fn inplace_pow_normalize_regular_case_relative_proportions_preserved() {
    use alloc::collections::BTreeMap;

    // With p = 1, normalization should yield roughly same proportions
    let mut m: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    m.insert(NetUid::from(7), u64f64(2.0));
    m.insert(NetUid::from(8), u64f64(3.0));
    m.insert(NetUid::from(9), u64f64(5.0));

    SubtensorModule::inplace_pow_normalize(&mut m, u64f64(1.0)); // p = 1

    let a = m.get(&NetUid::from(7)).copied().unwrap().to_num::<f64>();
    let b = m.get(&NetUid::from(8)).copied().unwrap().to_num::<f64>();
    let c = m.get(&NetUid::from(9)).copied().unwrap().to_num::<f64>();

    assert_abs_diff_eq!(a, 0.2_f64, epsilon = 0.001);
    assert_abs_diff_eq!(b, 0.3_f64, epsilon = 0.001);
    assert_abs_diff_eq!(c, 0.5_f64, epsilon = 0.001);

    // The sum of shares is 1.0 with good precision
    let sum = a + b + c;
    assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-12);
}

#[test]
fn inplace_pow_normalize_fractional_exponent() {
    use alloc::collections::BTreeMap;

    [1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0]
        .into_iter()
        .for_each(|p| {
            let mut m: BTreeMap<NetUid, U64F64> = BTreeMap::new();
            m.insert(NetUid::from(7), u64f64(2.0));
            m.insert(NetUid::from(8), u64f64(3.0));
            m.insert(NetUid::from(9), u64f64(5.0));

            SubtensorModule::inplace_pow_normalize(&mut m, u64f64(p));

            let a = m.get(&NetUid::from(7)).copied().unwrap().to_num::<f64>();
            let b = m.get(&NetUid::from(8)).copied().unwrap().to_num::<f64>();
            let c = m.get(&NetUid::from(9)).copied().unwrap().to_num::<f64>();

            let sum = (2.0_f64).powf(p) + (3.0_f64).powf(p) + (5.0_f64).powf(p);
            let expected_a = (2.0_f64).powf(p) / sum;
            let expected_b = (3.0_f64).powf(p) / sum;
            let expected_c = (5.0_f64).powf(p) / sum;

            assert_abs_diff_eq!(a, expected_a, epsilon = expected_a / 100.0);
            assert_abs_diff_eq!(b, expected_b, epsilon = expected_b / 100.0);
            assert_abs_diff_eq!(c, expected_c, epsilon = expected_c / 100.0);

            // The sum of shares is 1.0 with good precision
            let sum = a + b + c;
            assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-12);
        })
}

#[test]
fn protocol_normalization_keeps_eligible_subnet_count_from_collapsing() {
    new_test_ext(1).execute_with(|| {
        let subnet_count = 70usize;
        let user_flow = 100u64;
        let protocol_flow_start = 40u64;
        let protocol_flow_step = 4u64;

        NetTaoFlowEnabled::<Test>::set(true);
        FlowNormExponent::<Test>::set(u64f64(1.0));
        TaoFlowCutoff::<Test>::set(i64f64(0.0));
        FlowEmaSmoothingFactor::<Test>::set(i64::MAX as u64);

        let subnets = (0..subnet_count)
            .map(|i| {
                let netuid = NetUid::from((i + 1) as u16);
                add_network(netuid, 360, 0);
                SubnetEmissionEnabled::<Test>::insert(netuid, true);

                let protocol_flow = protocol_flow_start + protocol_flow_step.saturating_mul(i as u64);
                SubtensorModule::record_tao_inflow(netuid, TaoBalance::from(user_flow));
                SubtensorModule::record_protocol_inflow(netuid, TaoBalance::from(protocol_flow));

                netuid
            })
            .collect::<Vec<_>>();

        let subnets_to_emit_to = SubtensorModule::get_subnets_to_emit_to(&subnets);
        assert_eq!(
            subnets_to_emit_to.len(),
            subnets.len(),
            "test setup should make every subnet structurally eligible before flow scoring"
        );

        let emissions = SubtensorModule::get_subnet_block_emissions(
            &subnets_to_emit_to,
            U96F32::saturating_from_num(1_000_000_000u64),
        );

        let ema_rows = subnets_to_emit_to
            .iter()
            .map(|netuid| {
                let (_, user_ema) = SubnetEmaTaoFlow::<Test>::get(*netuid)
                    .expect("user EMA should be initialized by get_subnet_block_emissions");
                let (_, protocol_ema) = SubnetEmaProtocolFlow::<Test>::get(*netuid)
                    .expect("protocol EMA should be initialized by get_subnet_block_emissions");

                (*netuid, user_ema.to_num::<f64>(), protocol_ema.to_num::<f64>())
            })
            .collect::<Vec<_>>();

        let positive_user_ema_count = ema_rows
            .iter()
            .filter(|(_, user_ema, _)| *user_ema > 0.0)
            .count();
        let dynamic_eligibility_floor = positive_user_ema_count / 2;

        let sum_positive_user_ema: f64 = ema_rows
            .iter()
            .map(|(_, user_ema, _)| (*user_ema).max(0.0))
            .sum();
        let sum_positive_protocol_ema: f64 = ema_rows
            .iter()
            .map(|(_, _, protocol_ema)| (*protocol_ema).max(0.0))
            .sum();
        let protocol_norm_factor = if sum_positive_protocol_ema > 0.0 {
            (sum_positive_user_ema / sum_positive_protocol_ema).min(1.0)
        } else {
            0.0
        };

        let unnormalized_eligible = ema_rows
            .iter()
            .filter(|(_, user_ema, protocol_ema)| *user_ema > *protocol_ema)
            .count();
        let expected_normalized_eligible = ema_rows
            .iter()
            .filter(|(_, user_ema, protocol_ema)| {
                let scaled_protocol_ema = if *protocol_ema > 0.0 {
                    protocol_norm_factor * *protocol_ema
                } else {
                    *protocol_ema
                };
                *user_ema > scaled_protocol_ema
            })
            .count();
        let actual_eligible = emissions
            .values()
            .filter(|emission| emission.to_num::<f64>() > 0.0)
            .count();
        let total_emission: f64 = emissions
            .values()
            .map(|emission| emission.to_num::<f64>())
            .sum();

        assert_abs_diff_eq!(total_emission, 1_000_000_000.0_f64, epsilon = 1.0);
        assert!(
            unnormalized_eligible < dynamic_eligibility_floor,
            "test setup should reproduce the old unnormalized collapse: unnormalized_eligible={unnormalized_eligible}, dynamic_eligibility_floor={dynamic_eligibility_floor}"
        );
        assert!(
            expected_normalized_eligible >= dynamic_eligibility_floor,
            "test setup should keep enough subnets eligible after protocol normalization: expected_normalized_eligible={expected_normalized_eligible}, dynamic_eligibility_floor={dynamic_eligibility_floor}"
        );
        assert_eq!(
            actual_eligible, expected_normalized_eligible,
            "eligible subnet count should be derived from the normalized protocol-cost calculation"
        );
        assert!(
            actual_eligible >= dynamic_eligibility_floor,
            "eligible subnet count collapsed below the dynamic floor: actual_eligible={actual_eligible}, dynamic_eligibility_floor={dynamic_eligibility_floor}, unnormalized_eligible={unnormalized_eligible}"
        );
        assert!(
            actual_eligible > unnormalized_eligible,
            "normalization should preserve more eligible subnets than the old unnormalized path: actual_eligible={actual_eligible}, unnormalized_eligible={unnormalized_eligible}"
        );
    });
}

// /// Normal (moderate, non-zero) EMA flows across 3 subnets.
// /// Expect: shares sum to ~1 and are monotonic with flows.
// #[test]
// fn get_shares_normal_flows_three_subnets() {
//     new_test_ext(1).execute_with(|| {
//         let owner_hotkey = U256::from(10);
//         let owner_coldkey = U256::from(20);

//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n3 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         let block_num = FlowHalfLife::<Test>::get();
//         System::set_block_number(block_num);

//         // Set (block_number, flow) with reasonable positive flows
//         SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1_000.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(3_000.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n3, (block_num, i64f64(6_000.0)));

//         let subnets = vec![n1, n2, n3];
//         let shares = SubtensorModule::get_shares(&subnets);

//         // Sum ≈ 1
//         let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
//         assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-9);

//         // Each share in [0,1] and finite
//         for (k, v) in &shares {
//             let f = v.to_num::<f64>();
//             assert!(f.is_finite(), "share for {k:?} not finite");
//             assert!(
//                 (0.0..=1.0).contains(&f),
//                 "share for {k:?} out of [0,1]: {f}"
//             );
//         }

//         // Monotonicity with the flows: share(n3) > share(n2) > share(n1)
//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();
//         let s3 = shares.get(&n3).unwrap().to_num::<f64>();
//         assert!(
//             s3 > s2 && s2 > s1,
//             "expected s3 > s2 > s1; got {s1}, {s2}, {s3}"
//         );
//     });
// }

// /// Very low (but non-zero) EMA flows across 2 subnets.
// /// Expect: shares sum to ~1 and higher-flow subnet gets higher share.
// #[test]
// fn get_shares_low_flows_sum_one_and_ordering() {
//     new_test_ext(1).execute_with(|| {
//         let owner_hotkey = U256::from(11);
//         let owner_coldkey = U256::from(21);

//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         let block_num = FlowHalfLife::<Test>::get();
//         System::set_block_number(block_num);

//         // Tiny flows to exercise precision/scaling path
//         SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1e-9)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(2e-9)));

//         let subnets = vec![n1, n2];
//         let shares = SubtensorModule::get_shares(&subnets);

//         let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
//         assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-8);

//         for (k, v) in &shares {
//             let f = v.to_num::<f64>();
//             assert!(f.is_finite(), "share for {k:?} not finite");
//             assert!(
//                 (0.0..=1.0).contains(&f),
//                 "share for {k:?} out of [0,1]: {f}"
//             );
//         }

//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();
//         assert!(
//             s2 > s1,
//             "expected s2 > s1 with higher flow; got s1={s1}, s2={s2}"
//         );
//     });
// }

// /// High EMA flows across 2 subnets.
// /// Expect: no overflow, shares sum to ~1, and ordering follows flows.
// #[test]
// fn get_shares_high_flows_sum_one_and_ordering() {
//     new_test_ext(1).execute_with(|| {
//         let owner_hotkey = U256::from(12);
//         let owner_coldkey = U256::from(22);

//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         let block_num = FlowHalfLife::<Test>::get();
//         System::set_block_number(block_num);

//         // Large but safe flows for I64F64
//         SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(9.0e11)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(1.8e12)));

//         let subnets = vec![n1, n2];
//         let shares = SubtensorModule::get_shares(&subnets);

//         let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
//         assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-9);

//         for (k, v) in &shares {
//             let f = v.to_num::<f64>();
//             assert!(f.is_finite(), "share for {k:?} not finite");
//             assert!(
//                 (0.0..=1.0).contains(&f),
//                 "share for {k:?} out of [0,1]: {f}"
//             );
//         }

//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();
//         assert!(
//             s2 > s1,
//             "expected s2 > s1 with higher flow; got s1={s1}, s2={s2}"
//         );
//     });
// }

/// Helper to (re)seed EMA price & flow at the *current* block.
fn seed_price_and_flow(n1: NetUid, n2: NetUid, price1: f64, price2: f64, flow1: f64, flow2: f64) {
    let now = frame_system::Pallet::<Test>::block_number();
    SubnetMovingPrice::<Test>::insert(n1, i96f32(price1));
    SubnetMovingPrice::<Test>::insert(n2, i96f32(price2));
    SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(flow1)));
    SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(flow2)));
}

// /// If one subnet has a negative EMA flow and the other positive,
// /// the negative one should contribute no weight (treated as zero),
// /// so the positive-flow subnet gets the full share.
// #[test]
// fn get_shares_negative_vs_positive_flow() {
//     new_test_ext(1).execute_with(|| {
//         // 2 subnets
//         let owner_hotkey = U256::from(60);
//         let owner_coldkey = U256::from(61);
//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         // Configure blending window and current block
//         let half_life: u64 = FlowHalfLife::<Test>::get();
//         FlowNormExponent::<Test>::set(u64f64(1.0));
//         frame_system::Pallet::<Test>::set_block_number(half_life);
//         TaoFlowCutoff::<Test>::set(I64F64::from_num(0));

//         // Equal EMA prices so price side doesn't bias
//         SubnetMovingPrice::<Test>::insert(n1, i96f32(1.0));
//         SubnetMovingPrice::<Test>::insert(n2, i96f32(1.0));

//         // Set flows: n1 negative, n2 positive
//         let now = frame_system::Pallet::<Test>::block_number();
//         SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(500.0)));

//         let shares = SubtensorModule::get_shares(&[n1, n2]);
//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();

//         // Sum ~ 1
//         assert_abs_diff_eq!(s1 + s2, 1.0_f64, epsilon = 1e-9);
//         // Negative flow subnet should not get weight from flow; with equal prices mid-window,
//         // positive-flow subnet should dominate and get all the allocation.
//         assert!(
//             s2 > 0.999_999 && s1 < 1e-6,
//             "expected s2≈1, s1≈0; got s1={s1}, s2={s2}"
//         );
//     });
// }

// /// If both subnets have negative EMA flows, flows should contribute zero weight
// #[test]
// fn get_shares_both_negative_flows_zero_emission() {
//     new_test_ext(1).execute_with(|| {
//         // 2 subnets
//         let owner_hotkey = U256::from(60);
//         let owner_coldkey = U256::from(61);
//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         // Configure blending window and current block
//         let half_life: u64 = FlowHalfLife::<Test>::get();
//         FlowNormExponent::<Test>::set(u64f64(1.0));
//         frame_system::Pallet::<Test>::set_block_number(half_life);
//         TaoFlowCutoff::<Test>::set(I64F64::from_num(0));

//         // Equal EMA prices so price side doesn't bias
//         SubnetMovingPrice::<Test>::insert(n1, i96f32(1.0));
//         SubnetMovingPrice::<Test>::insert(n2, i96f32(1.0));

//         // Set flows
//         let now = frame_system::Pallet::<Test>::block_number();
//         SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(-200.0)));

//         let shares = SubtensorModule::get_shares(&[n1, n2]);
//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();

//         assert!(
//             s1 < 1e-20 && s2 < 1e-20,
//             "expected s2≈0, s1≈0; got s1={s1}, s2={s2}"
//         );
//     });
// }

// /// If both subnets have positive EMA flows lower than or equal to cutoff, flows should contribute zero weight
// #[test]
// fn get_shares_both_below_cutoff_zero_emission() {
//     new_test_ext(1).execute_with(|| {
//         // 2 subnets
//         let owner_hotkey = U256::from(60);
//         let owner_coldkey = U256::from(61);
//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         // Configure blending window and current block
//         let half_life: u64 = FlowHalfLife::<Test>::get();
//         FlowNormExponent::<Test>::set(u64f64(1.0));
//         frame_system::Pallet::<Test>::set_block_number(half_life);
//         TaoFlowCutoff::<Test>::set(I64F64::from_num(2_000));

//         // Equal EMA prices so price side doesn't bias
//         SubnetMovingPrice::<Test>::insert(n1, i96f32(1.0));
//         SubnetMovingPrice::<Test>::insert(n2, i96f32(1.0));

//         // Set flows
//         let now = frame_system::Pallet::<Test>::block_number();
//         SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(1000.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(2000.0)));

//         let shares = SubtensorModule::get_shares(&[n1, n2]);
//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();

//         assert!(
//             s1 < 1e-20 && s2 < 1e-20,
//             "expected s2≈0, s1≈0; got s1={s1}, s2={s2}"
//         );
//     });
// }

// /// If one subnet has positive EMA flow lower than cutoff, the other gets full emission
// #[test]
// fn get_shares_one_below_cutoff_other_full_emission() {
//     new_test_ext(1).execute_with(|| {
//         [(1000.0, 2000.00001), (1000.0, 2000.001), (1000.0, 5000.0)]
//             .into_iter()
//             .for_each(|(flow1, flow2)| {
//                 // 2 subnets
//                 let owner_hotkey = U256::from(60);
//                 let owner_coldkey = U256::from(61);
//                 let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//                 let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//                 // Configure blending window and current block
//                 let half_life: u64 = FlowHalfLife::<Test>::get();
//                 FlowNormExponent::<Test>::set(u64f64(1.0));
//                 frame_system::Pallet::<Test>::set_block_number(half_life);
//                 TaoFlowCutoff::<Test>::set(I64F64::from_num(2_000));

//                 // Equal EMA prices (price side doesn't bias)
//                 SubnetMovingPrice::<Test>::insert(n1, i96f32(1.0));
//                 SubnetMovingPrice::<Test>::insert(n2, i96f32(1.0));

//                 // Set flows
//                 let now = frame_system::Pallet::<Test>::block_number();
//                 SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(flow1)));
//                 SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(flow2)));

//                 let shares = SubtensorModule::get_shares(&[n1, n2]);
//                 let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//                 let s2 = shares.get(&n2).unwrap().to_num::<f64>();

//                 // Sum ~ 1
//                 assert_abs_diff_eq!(s1 + s2, 1.0_f64, epsilon = 1e-9);
//                 assert!(
//                     s2 > 0.999_999 && s1 < 1e-6,
//                     "expected s2≈1, s1≈0; got s1={s1}, s2={s2}"
//                 );
//             });
//     });
// }

// /// If subnets have negative EMA flows, but they are above the cut-off, emissions are proportional
// /// for all except the bottom one, which gets nothing
// #[test]
// fn get_shares_both_negative_above_cutoff() {
//     new_test_ext(1).execute_with(|| {
//         // 2 subnets
//         let owner_hotkey = U256::from(60);
//         let owner_coldkey = U256::from(61);
//         let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
//         let n3 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         // Configure blending window and current block
//         let half_life: u64 = FlowHalfLife::<Test>::get();
//         FlowNormExponent::<Test>::set(u64f64(1.0));
//         frame_system::Pallet::<Test>::set_block_number(half_life);
//         TaoFlowCutoff::<Test>::set(I64F64::from_num(-1000.0));

//         // Equal EMA prices so price side doesn't bias
//         SubnetMovingPrice::<Test>::insert(n1, i96f32(1.0));
//         SubnetMovingPrice::<Test>::insert(n2, i96f32(1.0));
//         SubnetMovingPrice::<Test>::insert(n3, i96f32(1.0));

//         // Set flows
//         let now = frame_system::Pallet::<Test>::block_number();
//         SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(-300.0)));
//         SubnetEmaTaoFlow::<Test>::insert(n3, (now, i64f64(-400.0)));

//         let shares = SubtensorModule::get_shares(&[n1, n2, n3]);
//         let s1 = shares.get(&n1).unwrap().to_num::<f64>();
//         let s2 = shares.get(&n2).unwrap().to_num::<f64>();
//         let s3 = shares.get(&n3).unwrap().to_num::<f64>();

//         assert_abs_diff_eq!(s1, 0.75, epsilon = s1 / 100.0);
//         assert_abs_diff_eq!(s2, 0.25, epsilon = s2 / 100.0);
//         assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-9);
//         assert_abs_diff_eq!(s1 + s2 + s3, 1.0, epsilon = 1e-9);
//     });
// }
