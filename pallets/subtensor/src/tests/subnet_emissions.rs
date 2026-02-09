#![allow(
    unused,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]
use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use approx::assert_abs_diff_eq;
use sp_core::U256;
use substrate_fixed::types::{I64F64, I96F32, U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, NetUid};

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

#[test]
fn test_effective_root_prop_no_root_dividends() {
    // When there are no root alpha dividends, EffectiveRootProp should be 0
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(100);
        let hotkey2 = U256::from(101);

        let mut alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_dividends.insert(hotkey1, U96F32::from_num(1000));
        alpha_dividends.insert(hotkey2, U96F32::from_num(2000));

        let root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        let prop = EffectiveRootProp::<Test>::get(netuid);
        assert_abs_diff_eq!(prop.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_effective_root_prop_all_root_dividends() {
    // When there are only root alpha dividends with equal root stakes but unequal dividends,
    // efficiency-based utilization < 1.0 because the validator with less dividends than expected
    // has efficiency < 1.0.
    // hotkey1: expected_share=0.5, actual_share=1/3 → efficiency=2/3
    // hotkey2: expected_share=0.5, actual_share=2/3 → efficiency=1.0 (capped)
    // utilization = (1000*2/3 + 1000*1.0) / 2000 ≈ 0.8333
    // raw_root_prop = 1.0 (all root divs), so ERP ≈ 0.8333
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(100);
        let coldkey1 = U256::from(200);
        let hotkey2 = U256::from(101);
        let coldkey2 = U256::from(201);

        // Register hotkeys on subnet and give them root stake so utilization = 1.0
        Keys::<Test>::insert(netuid, 0u16, hotkey1);
        Keys::<Test>::insert(netuid, 1u16, hotkey2);
        SubnetworkN::<Test>::insert(netuid, 2u16);
        increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000u64.into(), NetUid::ROOT);
        increase_stake_on_coldkey_hotkey_account(&coldkey2, &hotkey2, 1000u64.into(), NetUid::ROOT);

        let alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let mut root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_alpha_dividends.insert(hotkey1, U96F32::from_num(1000));
        root_alpha_dividends.insert(hotkey2, U96F32::from_num(2000));

        let utilization = SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        assert_abs_diff_eq!(utilization.to_num::<f64>(), 0.8333, epsilon = 1e-3);

        let prop = EffectiveRootProp::<Test>::get(netuid);
        // raw_root_prop = 1.0, utilization ≈ 0.8333
        assert_abs_diff_eq!(prop.to_num::<f64>(), 0.8333, epsilon = 1e-3);
    });
}

#[test]
fn test_effective_root_prop_balanced() {
    // When root and alpha dividends are equal, EffectiveRootProp should be ~0.5
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(100);
        let coldkey1 = U256::from(200);

        // Register hotkey on subnet and give root stake so utilization = 1.0
        Keys::<Test>::insert(netuid, 0u16, hotkey1);
        SubnetworkN::<Test>::insert(netuid, 1u16);
        increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000u64.into(), NetUid::ROOT);

        let mut alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_dividends.insert(hotkey1, U96F32::from_num(5000));

        let mut root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_alpha_dividends.insert(hotkey1, U96F32::from_num(5000));

        SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        let prop = EffectiveRootProp::<Test>::get(netuid);
        assert_abs_diff_eq!(prop.to_num::<f64>(), 0.5, epsilon = 1e-9);
    });
}

#[test]
fn test_effective_root_prop_both_empty() {
    // When both are empty, EffectiveRootProp should be 0
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);

        let alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        let root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        let prop = EffectiveRootProp::<Test>::get(netuid);
        assert_abs_diff_eq!(prop.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_effective_root_prop_different_subnets() {
    // Test that different subnets get different EffectiveRootProp values
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let hotkey1 = U256::from(100);
        let coldkey1 = U256::from(200);

        // Register hotkey on both subnets and give root stake so utilization = 1.0
        Keys::<Test>::insert(netuid1, 0u16, hotkey1);
        SubnetworkN::<Test>::insert(netuid1, 1u16);
        Keys::<Test>::insert(netuid2, 0u16, hotkey1);
        SubnetworkN::<Test>::insert(netuid2, 1u16);
        increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000u64.into(), NetUid::ROOT);

        // Subnet 1: 25% root
        let mut alpha_divs1: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs1.insert(hotkey1, U96F32::from_num(3000));
        let mut root_divs1: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs1.insert(hotkey1, U96F32::from_num(1000));

        SubtensorModule::compute_and_store_effective_root_prop(netuid1, &alpha_divs1, &root_divs1);

        // Subnet 2: 75% root
        let mut alpha_divs2: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs2.insert(hotkey1, U96F32::from_num(1000));
        let mut root_divs2: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs2.insert(hotkey1, U96F32::from_num(3000));

        SubtensorModule::compute_and_store_effective_root_prop(netuid2, &alpha_divs2, &root_divs2);

        let prop1 = EffectiveRootProp::<Test>::get(netuid1);
        let prop2 = EffectiveRootProp::<Test>::get(netuid2);

        assert_abs_diff_eq!(prop1.to_num::<f64>(), 0.25, epsilon = 1e-9);
        assert_abs_diff_eq!(prop2.to_num::<f64>(), 0.75, epsilon = 1e-9);
    });
}

#[test]
fn test_normalize_shares_basic() {
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(2.0));
    shares.insert(NetUid::from(2), u64f64(3.0));
    shares.insert(NetUid::from(3), u64f64(5.0));

    SubtensorModule::normalize_shares(&mut shares);

    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();

    assert_abs_diff_eq!(s1, 0.2, epsilon = 1e-9);
    assert_abs_diff_eq!(s2, 0.3, epsilon = 1e-9);
    assert_abs_diff_eq!(s3, 0.5, epsilon = 1e-9);
}

#[test]
fn test_normalize_shares_all_zero() {
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.0));
    shares.insert(NetUid::from(2), u64f64(0.0));

    SubtensorModule::normalize_shares(&mut shares);

    // Should remain zero when all are zero
    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();

    assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
    assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
}

#[test]
fn test_apply_effective_root_prop_scaling_disabled() {
    new_test_ext(1).execute_with(|| {
        // Scaling is disabled by default
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.5));
        shares.insert(NetUid::from(2), u64f64(0.5));

        let shares_before = shares.clone();
        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        // Shares should be unchanged when scaling is disabled
        for (k, v) in shares_before {
            assert_abs_diff_eq!(
                shares.get(&k).unwrap().to_num::<f64>(),
                v.to_num::<f64>(),
                epsilon = 1e-12
            );
        }
    });
}

#[test]
fn test_apply_effective_root_prop_scaling_enabled() {
    new_test_ext(1).execute_with(|| {
        // Enable scaling
        EffectiveRootPropEmissionScaling::<Test>::set(true);

        // Set EffectiveRootProp and RootProp for subnets.
        // RootProp >= EffectiveRootProp, so min() uses EffectiveRootProp.
        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.8));
        EffectiveRootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.2));
        RootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.9));
        RootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.9));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.5));
        shares.insert(NetUid::from(2), u64f64(0.5));

        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        // After scaling: subnet1 = 0.5*0.8 = 0.4, subnet2 = 0.5*0.2 = 0.1
        // After normalization: subnet1 = 0.4/0.5 = 0.8, subnet2 = 0.1/0.5 = 0.2
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.8, epsilon = 1e-9);
        assert_abs_diff_eq!(s2, 0.2, epsilon = 1e-9);
        assert_abs_diff_eq!(s1 + s2, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_effective_root_prop_scaling_all_zero_props() {
    new_test_ext(1).execute_with(|| {
        // Enable scaling
        EffectiveRootPropEmissionScaling::<Test>::set(true);

        // EffectiveRootProp defaults to 0 for all subnets (ValueQuery default)
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.5));
        shares.insert(NetUid::from(2), u64f64(0.5));

        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        // All shares become 0 when all EffectiveRootProp are 0
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_apply_effective_root_prop_scaling_single_subnet() {
    new_test_ext(1).execute_with(|| {
        // Enable scaling
        EffectiveRootPropEmissionScaling::<Test>::set(true);

        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.3));
        RootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.5));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(1.0));

        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        // Single subnet should get normalized back to 1.0
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_effective_root_prop_scaling_capped_by_root_prop() {
    new_test_ext(1).execute_with(|| {
        // Enable scaling
        EffectiveRootPropEmissionScaling::<Test>::set(true);

        // Simulate exploitation: EffectiveRootProp inflated above RootProp
        // by disabling alpha validators. Scaling should use min(ERP, RP).
        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.9)); // inflated
        EffectiveRootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.2)); // normal
        RootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.3)); // actual root prop
        RootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.5)); // actual root prop

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.5));
        shares.insert(NetUid::from(2), u64f64(0.5));

        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        // min(0.9, 0.3) = 0.3 for subnet1, min(0.2, 0.5) = 0.2 for subnet2
        // After scaling: subnet1 = 0.5*0.3 = 0.15, subnet2 = 0.5*0.2 = 0.10
        // After normalization: subnet1 = 0.15/0.25 = 0.6, subnet2 = 0.10/0.25 = 0.4
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.6, epsilon = 1e-9);
        assert_abs_diff_eq!(s2, 0.4, epsilon = 1e-9);
        assert_abs_diff_eq!(s1 + s2, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_zero_and_redistribute_bottom_shares_basic() {
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.1));
    shares.insert(NetUid::from(2), u64f64(0.2));
    shares.insert(NetUid::from(3), u64f64(0.3));
    shares.insert(NetUid::from(4), u64f64(0.4));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 2);

    // Top 2 are netuid 4 (0.4) and netuid 3 (0.3)
    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
    let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();

    assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
    assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
    // s3 and s4 should be renormalized: 0.3/0.7 and 0.4/0.7
    assert_abs_diff_eq!(s3, 0.3 / 0.7, epsilon = 1e-9);
    assert_abs_diff_eq!(s4, 0.4 / 0.7, epsilon = 1e-9);
    assert_abs_diff_eq!(s3 + s4, 1.0, epsilon = 1e-9);
}

#[test]
fn test_zero_and_redistribute_bottom_shares_tie_at_boundary() {
    // A:0.4, B:0.3, C:0.3 with top_k=2 — B and C tie at the boundary.
    // Both should be kept (tie inclusion), so all 3 remain.
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.4));
    shares.insert(NetUid::from(2), u64f64(0.3));
    shares.insert(NetUid::from(3), u64f64(0.3));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 2);

    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();

    // All three should be nonzero since B and C tie at the k-th position
    assert!(s1 > 0.0, "A should be kept");
    assert!(s2 > 0.0, "B should be kept (tie at boundary)");
    assert!(s3 > 0.0, "C should be kept (tie at boundary)");
    assert_abs_diff_eq!(s1 + s2 + s3, 1.0, epsilon = 1e-9);
    // Normalized: 0.4/1.0, 0.3/1.0, 0.3/1.0
    assert_abs_diff_eq!(s1, 0.4, epsilon = 1e-9);
    assert_abs_diff_eq!(s2, 0.3, epsilon = 1e-9);
    assert_abs_diff_eq!(s3, 0.3, epsilon = 1e-9);
}

#[test]
fn test_zero_and_redistribute_bottom_shares_no_tie() {
    // A:0.5, B:0.3, C:0.2 with top_k=2 — no tie at boundary, C is strictly below.
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.5));
    shares.insert(NetUid::from(2), u64f64(0.3));
    shares.insert(NetUid::from(3), u64f64(0.2));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 2);

    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();

    assert!(s1 > 0.0);
    assert!(s2 > 0.0);
    assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-12);
    assert_abs_diff_eq!(s1 + s2, 1.0, epsilon = 1e-9);
}

#[test]
fn test_zero_and_redistribute_top_k_exceeds_count() {
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.5));
    shares.insert(NetUid::from(2), u64f64(0.5));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 10);

    // Nothing should change since top_k > len
    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    assert_abs_diff_eq!(s1, 0.5, epsilon = 1e-12);
    assert_abs_diff_eq!(s2, 0.5, epsilon = 1e-12);
}

#[test]
fn test_apply_top_subnet_proportion_filter_default_50_percent_4_subnets() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.1));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.3));
        shares.insert(NetUid::from(4), u64f64(0.4));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        // ceil(4 * 5000 / 10000) = ceil(2.0) = 2
        // Top 2: netuid 4 (0.4) and netuid 3 (0.3)
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        assert!(s3 > 0.0);
        assert!(s4 > 0.0);
        assert_abs_diff_eq!(s3 + s4, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_proportion_filter_default_50_percent_1_subnet() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        // 1 subnet -> ceil(1 * 0.5) = 1
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(1.0));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_proportion_filter_default_50_percent_3_subnets() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        // 3 subnets -> ceil(3 * 5000 / 10000) = ceil(1.5) = 2
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.2));
        shares.insert(NetUid::from(2), u64f64(0.3));
        shares.insert(NetUid::from(3), u64f64(0.5));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert!(s2 > 0.0);
        assert!(s3 > 0.0);
        assert_abs_diff_eq!(s2 + s3, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_proportion_filter_100_percent() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(1)); // 100%

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.25));
        shares.insert(NetUid::from(2), u64f64(0.75));

        let shares_before = shares.clone();
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        // All subnets should keep their shares
        for (k, v) in shares_before {
            assert_abs_diff_eq!(
                shares.get(&k).unwrap().to_num::<f64>(),
                v.to_num::<f64>(),
                epsilon = 1e-12
            );
        }
    });
}

#[test]
fn test_apply_top_subnet_proportion_filter_zeroed_get_no_emission() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.1));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.3));
        shares.insert(NetUid::from(4), u64f64(0.4));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        // Verify zeroed subnets produce zero emission
        let block_emission = U96F32::from_num(1_000_000);
        for (netuid, share) in &shares {
            let emission = U64F64::saturating_from_num(*share)
                .saturating_mul(U64F64::saturating_from_num(block_emission));
            if *netuid == NetUid::from(1) || *netuid == NetUid::from(2) {
                assert_abs_diff_eq!(emission.to_num::<f64>(), 0.0, epsilon = 1e-6);
            } else {
                assert!(emission.to_num::<f64>() > 0.0);
            }
        }
    });
}

#[test]
fn test_apply_top_subnet_absolute_limit_disabled() {
    new_test_ext(1).execute_with(|| {
        // Default limit is 0 (disabled)
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.25));
        shares.insert(NetUid::from(2), u64f64(0.25));
        shares.insert(NetUid::from(3), u64f64(0.25));
        shares.insert(NetUid::from(4), u64f64(0.25));

        let shares_before = shares.clone();
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // No change when disabled
        for (k, v) in shares_before {
            assert_abs_diff_eq!(
                shares.get(&k).unwrap().to_num::<f64>(),
                v.to_num::<f64>(),
                epsilon = 1e-12
            );
        }
    });
}

#[test]
fn test_apply_top_subnet_absolute_limit_two_of_five() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(2));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.05));
        shares.insert(NetUid::from(2), u64f64(0.10));
        shares.insert(NetUid::from(3), u64f64(0.15));
        shares.insert(NetUid::from(4), u64f64(0.30));
        shares.insert(NetUid::from(5), u64f64(0.40));

        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // Only top 2 (netuid 5 and 4) should have nonzero shares
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-12);
        assert!(s4 > 0.0);
        assert!(s5 > 0.0);
        assert_abs_diff_eq!(s4 + s5, 1.0, epsilon = 1e-9);
        // 0.30/0.70 and 0.40/0.70
        assert_abs_diff_eq!(s4, 0.30 / 0.70, epsilon = 1e-9);
        assert_abs_diff_eq!(s5, 0.40 / 0.70, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_absolute_limit_exceeds_count() {
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(10));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.5));
        shares.insert(NetUid::from(2), u64f64(0.3));
        shares.insert(NetUid::from(3), u64f64(0.2));

        let shares_before = shares.clone();
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // All keep their shares when limit > count
        for (k, v) in shares_before {
            assert_abs_diff_eq!(
                shares.get(&k).unwrap().to_num::<f64>(),
                v.to_num::<f64>(),
                epsilon = 1e-12
            );
        }
    });
}

#[test]
fn test_interaction_proportion_and_absolute_limit() {
    new_test_ext(1).execute_with(|| {
        // 50% proportion with 6 subnets -> ceil(6*0.5) = 3 subnets
        // Absolute limit = 2 -> further reduces to 2 subnets
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5));
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(2));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.05));
        shares.insert(NetUid::from(2), u64f64(0.10));
        shares.insert(NetUid::from(3), u64f64(0.15));
        shares.insert(NetUid::from(4), u64f64(0.20));
        shares.insert(NetUid::from(5), u64f64(0.25));
        shares.insert(NetUid::from(6), u64f64(0.25));

        // Apply proportion filter first (as in get_subnet_block_emissions)
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        // After 50% filter: top 3 subnets (6, 5, 4) keep their shares
        let nonzero_after_proportion = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_proportion, 3, "50% of 6 subnets = top 3");

        // Apply absolute limit
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // After absolute limit: only top 2 subnets
        let nonzero_after_limit = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_after_limit, 2,
            "Absolute limit of 2 should leave 2 subnets"
        );

        // Sum should be 1.0
        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_interaction_absolute_limit_stricter_than_proportion() {
    new_test_ext(1).execute_with(|| {
        // proportion = 100% (all subnets), absolute limit = 1
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(1));
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(1));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.3));
        shares.insert(NetUid::from(2), u64f64(0.3));
        shares.insert(NetUid::from(3), u64f64(0.4));

        // Apply both filters
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // Only subnet 3 should survive (highest share)
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s3, 1.0, epsilon = 1e-9);
    });
}

// ===========================================================================
// Tests for get_root_dividend_fraction
// ===========================================================================

#[test]
fn test_root_dividend_fraction_no_root_stake() {
    // Hotkey with 0 root stake → fraction = 0
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(100);
        let coldkey = U256::from(200);
        let tao_weight = U96F32::from_num(0.18);

        // Only alpha stake, no root stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            AlphaCurrency::from(1_000_000u64),
        );

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_root_dividend_fraction_no_alpha_stake() {
    // Hotkey with only root stake → fraction = 1.0
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(100);
        let coldkey = U256::from(200);
        let tao_weight = U96F32::from_num(0.18);

        // Only root stake, no alpha
        increase_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            1_000_000u64.into(),
            NetUid::ROOT,
        );

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_root_dividend_fraction_mixed_stake() {
    // Hotkey with both root and alpha stake
    // root_alpha_weighted = 1_000_000 * 0.18 = 180_000
    // alpha_stake = 820_000
    // fraction = 180_000 / (820_000 + 180_000) = 0.18
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(100);
        let coldkey = U256::from(200);
        let tao_weight = U96F32::from_num(0.18);

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            AlphaCurrency::from(820_000u64),
        );
        increase_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            1_000_000u64.into(),
            NetUid::ROOT,
        );

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 0.18, epsilon = 1e-6);
    });
}

#[test]
fn test_root_dividend_fraction_high_tao_weight() {
    // With high tao_weight, root fraction approaches 1.0
    // root_alpha_weighted = 100 * 10.0 = 1000
    // alpha_stake = 100
    // fraction = 1000 / (100 + 1000) ≈ 0.909
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(100);
        let coldkey = U256::from(200);
        let tao_weight = U96F32::from_num(10);

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            AlphaCurrency::from(100u64),
        );
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 100u64.into(), NetUid::ROOT);

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 10.0 / 11.0, epsilon = 1e-6);
    });
}

// ===========================================================================
// Tests for apply_utilization_scaling
// ===========================================================================

/// Helper: set up a subnet with hotkeys that have root + alpha stakes.
/// Returns (netuid, hotkey1, hotkey2).
fn setup_scaling_test() -> (NetUid, U256, U256) {
    let netuid = NetUid::from(1);
    let hotkey1 = U256::from(100);
    let coldkey1 = U256::from(200);
    let hotkey2 = U256::from(101);
    let coldkey2 = U256::from(201);

    // hotkey1: 900k alpha, 1M root
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &hotkey1,
        &coldkey1,
        netuid,
        AlphaCurrency::from(900_000u64),
    );
    increase_stake_on_coldkey_hotkey_account(
        &coldkey1,
        &hotkey1,
        1_000_000u64.into(),
        NetUid::ROOT,
    );

    // hotkey2: 500k alpha, 500k root
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &hotkey2,
        &coldkey2,
        netuid,
        AlphaCurrency::from(500_000u64),
    );
    increase_stake_on_coldkey_hotkey_account(&coldkey2, &hotkey2, 500_000u64.into(), NetUid::ROOT);

    // Need SubnetAlphaOut for recycling to work
    SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(10_000_000u64));

    (netuid, hotkey1, hotkey2)
}

#[test]
fn test_apply_utilization_scaling_full_utilization() {
    // utilization = 1.0 → no scaling, returns 0 recycled
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(1);

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));
        alpha_divs.insert(hotkey2, U96F32::from_num(5000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));
        root_divs.insert(hotkey2, U96F32::from_num(1000));

        let alpha_divs_before = alpha_divs.clone();
        let root_divs_before = root_divs.clone();

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        assert_abs_diff_eq!(recycled.to_num::<f64>(), 0.0, epsilon = 1e-12);
        // Maps unchanged
        assert_eq!(alpha_divs, alpha_divs_before);
        assert_eq!(root_divs, root_divs_before);
    });
}

#[test]
fn test_apply_utilization_scaling_no_root_dividends() {
    // Empty root dividends → no scaling regardless of utilization, returns 0
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, _hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0); // Would normally trigger hard cap

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new(); // empty

        let alpha_divs_before = alpha_divs.clone();

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        assert_abs_diff_eq!(recycled.to_num::<f64>(), 0.0, epsilon = 1e-12);
        // Alpha divs unchanged (no root dividends to trigger scaling)
        assert_eq!(alpha_divs, alpha_divs_before);
    });
}

#[test]
fn test_apply_utilization_scaling_partial() {
    // utilization = 0.7 >= 0.5 → full dividends, no scaling, no recycling
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0.7);

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));
        alpha_divs.insert(hotkey2, U96F32::from_num(5000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));
        root_divs.insert(hotkey2, U96F32::from_num(1000));

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        // Root dividends should be unchanged (no scaling when util >= 0.5)
        assert_abs_diff_eq!(
            root_divs.get(&hotkey1).unwrap().to_num::<f64>(),
            2000.0,
            epsilon = 1.0
        );
        assert_abs_diff_eq!(
            root_divs.get(&hotkey2).unwrap().to_num::<f64>(),
            1000.0,
            epsilon = 1.0
        );

        // Alpha divs should be unchanged
        assert_abs_diff_eq!(
            alpha_divs.get(&hotkey1).unwrap().to_num::<f64>(),
            10000.0,
            epsilon = 1.0
        );
        assert_abs_diff_eq!(
            alpha_divs.get(&hotkey2).unwrap().to_num::<f64>(),
            5000.0,
            epsilon = 1.0
        );

        // Nothing recycled
        assert_abs_diff_eq!(recycled.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_apply_utilization_scaling_hard_cap() {
    // utilization = 0.3 < 0.5 → hard cap: recycle ALL root dividends, set ERP = 0
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0.3);

        // Set a non-zero ERP so we can verify it gets zeroed
        EffectiveRootProp::<Test>::insert(netuid, U96F32::from_num(0.5));

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));
        alpha_divs.insert(hotkey2, U96F32::from_num(5000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));
        root_divs.insert(hotkey2, U96F32::from_num(1000));

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        // Root dividends should be completely cleared
        assert!(
            root_divs.is_empty(),
            "Root divs should be empty after hard cap"
        );

        // Alpha divs should be reduced by their root fraction
        let alpha1 = alpha_divs.get(&hotkey1).unwrap().to_num::<f64>();
        assert!(alpha1 < 10000.0, "Alpha divs should be reduced: {alpha1}");
        // hotkey1 root_fraction ≈ 0.1666, so alpha1 ≈ 10000 * (1 - 0.1666) ≈ 8334
        assert_abs_diff_eq!(alpha1, 8334.0, epsilon = 100.0);

        // Total recycled should account for all root divs + root fraction of alpha divs
        assert!(
            recycled.to_num::<f64>() > 3000.0,
            "Should recycle at least the 3000 root divs"
        );

        // EffectiveRootProp should be 0
        let erp = EffectiveRootProp::<Test>::get(netuid);
        assert_abs_diff_eq!(erp.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_apply_utilization_scaling_at_boundary() {
    // utilization = 0.5 exactly → full dividends, NOT hard cap
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, _hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0.5);

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        // Root dividends should be unchanged (full dividends at boundary)
        assert!(
            !root_divs.is_empty(),
            "Root divs should NOT be empty at boundary 0.5"
        );
        assert_abs_diff_eq!(
            root_divs.get(&hotkey1).unwrap().to_num::<f64>(),
            2000.0,
            epsilon = 1.0
        );

        // Alpha divs should be unchanged
        assert_abs_diff_eq!(
            alpha_divs.get(&hotkey1).unwrap().to_num::<f64>(),
            10000.0,
            epsilon = 1.0
        );

        // Nothing recycled
        assert_abs_diff_eq!(recycled.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_apply_utilization_scaling_just_below_boundary() {
    // utilization = 0.4999 → hard cap triggers
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, _hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0.4999);

        EffectiveRootProp::<Test>::insert(netuid, U96F32::from_num(0.5));

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));

        SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        // Hard cap: root divs cleared, ERP = 0
        assert!(root_divs.is_empty(), "Root divs should be empty below 0.5");
        assert_abs_diff_eq!(
            EffectiveRootProp::<Test>::get(netuid).to_num::<f64>(),
            0.0,
            epsilon = 1e-12
        );
    });
}
