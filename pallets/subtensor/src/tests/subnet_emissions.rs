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
use subtensor_runtime_common::{AlphaCurrency, MechId, NetUid, TaoCurrency};

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

/// Normal (moderate, non-zero) EMA flows across 3 subnets.
/// Expect: shares sum to ~1 and are monotonic with flows.
#[test]
fn get_shares_normal_flows_three_subnets() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(10);
        let owner_coldkey = U256::from(20);

        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n3 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let block_num = FlowHalfLife::<Test>::get();
        System::set_block_number(block_num);

        // Set (block_number, flow) with reasonable positive flows
        SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1_000.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(3_000.0)));
        SubnetEmaTaoFlow::<Test>::insert(n3, (block_num, i64f64(6_000.0)));

        let subnets = vec![n1, n2, n3];
        let shares = SubtensorModule::get_shares(&subnets);

        // Sum ≈ 1
        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-9);

        // Each share in [0,1] and finite
        for (k, v) in &shares {
            let f = v.to_num::<f64>();
            assert!(f.is_finite(), "share for {k:?} not finite");
            assert!(
                (0.0..=1.0).contains(&f),
                "share for {k:?} out of [0,1]: {f}"
            );
        }

        // Monotonicity with the flows: share(n3) > share(n2) > share(n1)
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();
        let s3 = shares.get(&n3).unwrap().to_num::<f64>();
        assert!(
            s3 > s2 && s2 > s1,
            "expected s3 > s2 > s1; got {s1}, {s2}, {s3}"
        );
    });
}

/// Very low (but non-zero) EMA flows across 2 subnets.
/// Expect: shares sum to ~1 and higher-flow subnet gets higher share.
#[test]
fn get_shares_low_flows_sum_one_and_ordering() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(11);
        let owner_coldkey = U256::from(21);

        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let block_num = FlowHalfLife::<Test>::get();
        System::set_block_number(block_num);

        // Tiny flows to exercise precision/scaling path
        SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1e-9)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(2e-9)));

        let subnets = vec![n1, n2];
        let shares = SubtensorModule::get_shares(&subnets);

        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-8);

        for (k, v) in &shares {
            let f = v.to_num::<f64>();
            assert!(f.is_finite(), "share for {k:?} not finite");
            assert!(
                (0.0..=1.0).contains(&f),
                "share for {k:?} out of [0,1]: {f}"
            );
        }

        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();
        assert!(
            s2 > s1,
            "expected s2 > s1 with higher flow; got s1={s1}, s2={s2}"
        );
    });
}

/// High EMA flows across 2 subnets.
/// Expect: no overflow, shares sum to ~1, and ordering follows flows.
#[test]
fn get_shares_high_flows_sum_one_and_ordering() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(12);
        let owner_coldkey = U256::from(22);

        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let block_num = FlowHalfLife::<Test>::get();
        System::set_block_number(block_num);

        // Large but safe flows for I64F64
        SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(9.0e11)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(1.8e12)));

        let subnets = vec![n1, n2];
        let shares = SubtensorModule::get_shares(&subnets);

        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0_f64, epsilon = 1e-9);

        for (k, v) in &shares {
            let f = v.to_num::<f64>();
            assert!(f.is_finite(), "share for {k:?} not finite");
            assert!(
                (0.0..=1.0).contains(&f),
                "share for {k:?} out of [0,1]: {f}"
            );
        }

        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();
        assert!(
            s2 > s1,
            "expected s2 > s1 with higher flow; got s1={s1}, s2={s2}"
        );
    });
}

/// Single subnet should receive 100% of the share (1.0).
/// Expect: shares contain exactly 1 entry with value 1.0.
#[test]
fn test_get_shares_single_subnet() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(13);
        let owner_coldkey = U256::from(23);

        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let block_num = FlowHalfLife::<Test>::get();
        System::set_block_number(block_num);

        // Set (block_number, flow) with a positive flow
        SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1_000.0)));

        let subnets = vec![n1];
        let shares = SubtensorModule::get_shares(&subnets);

        // Should have exactly 1 entry
        assert_eq!(shares.len(), 1, "expected exactly 1 entry in shares");

        // The single subnet should get share of 1.0
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 1.0_f64, epsilon = 1e-9);
    });
}

/// Helper to (re)seed EMA price & flow at the *current* block.
fn seed_price_and_flow(n1: NetUid, n2: NetUid, price1: f64, price2: f64, flow1: f64, flow2: f64) {
    let now = frame_system::Pallet::<Test>::block_number();
    SubnetMovingPrice::<Test>::insert(n1, i96f32(price1));
    SubnetMovingPrice::<Test>::insert(n2, i96f32(price2));
    SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(flow1)));
    SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(flow2)));
}

/// If one subnet has a negative EMA flow and the other positive,
/// the negative one should contribute no weight (treated as zero),
/// so the positive-flow subnet gets the full share.
#[test]
fn get_shares_negative_vs_positive_flow() {
    new_test_ext(1).execute_with(|| {
        // 2 subnets
        let owner_hotkey = U256::from(60);
        let owner_coldkey = U256::from(61);
        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Configure blending window and current block
        let half_life: u64 = FlowHalfLife::<Test>::get();
        FlowNormExponent::<Test>::set(u64f64(1.0));
        frame_system::Pallet::<Test>::set_block_number(half_life);
        TaoFlowCutoff::<Test>::set(I64F64::from_num(0));

        // Set flows: n1 negative, n2 positive
        let now = frame_system::Pallet::<Test>::block_number();
        SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(500.0)));

        let shares = SubtensorModule::get_shares(&[n1, n2]);
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();

        // Sum ~ 1
        assert_abs_diff_eq!(s1 + s2, 1.0_f64, epsilon = 1e-9);
        // Negative flow subnet should not get weight from flow;
        // positive-flow subnet should get all the allocation.
        assert!(
            s2 > 0.999_999 && s1 < 1e-6,
            "expected s2≈1, s1≈0; got s1={s1}, s2={s2}"
        );
    });
}

/// If both subnets have negative EMA flows, flows should contribute zero weight
#[test]
fn get_shares_both_negative_flows_zero_emission() {
    new_test_ext(1).execute_with(|| {
        // 2 subnets
        let owner_hotkey = U256::from(60);
        let owner_coldkey = U256::from(61);
        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Configure blending window and current block
        let half_life: u64 = FlowHalfLife::<Test>::get();
        FlowNormExponent::<Test>::set(u64f64(1.0));
        frame_system::Pallet::<Test>::set_block_number(half_life);
        TaoFlowCutoff::<Test>::set(I64F64::from_num(0));

        // Set flows
        let now = frame_system::Pallet::<Test>::block_number();
        SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(-200.0)));

        let shares = SubtensorModule::get_shares(&[n1, n2]);
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();

        assert!(
            s1 < 1e-20 && s2 < 1e-20,
            "expected s2≈0, s1≈0; got s1={s1}, s2={s2}"
        );
    });
}

/// If both subnets have positive EMA flows lower than or equal to cutoff, flows should contribute zero weight
#[test]
fn get_shares_both_below_cutoff_zero_emission() {
    new_test_ext(1).execute_with(|| {
        // 2 subnets
        let owner_hotkey = U256::from(60);
        let owner_coldkey = U256::from(61);
        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Configure blending window and current block
        let half_life: u64 = FlowHalfLife::<Test>::get();
        FlowNormExponent::<Test>::set(u64f64(1.0));
        frame_system::Pallet::<Test>::set_block_number(half_life);
        TaoFlowCutoff::<Test>::set(I64F64::from_num(2_000));

        // Set flows
        let now = frame_system::Pallet::<Test>::block_number();
        SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(1000.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(2000.0)));

        let shares = SubtensorModule::get_shares(&[n1, n2]);
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();

        assert!(
            s1 < 1e-20 && s2 < 1e-20,
            "expected s2≈0, s1≈0; got s1={s1}, s2={s2}"
        );
    });
}

/// If one subnet has positive EMA flow lower than cutoff, the other gets full emission
#[test]
fn get_shares_one_below_cutoff_other_full_emission() {
    new_test_ext(1).execute_with(|| {
        [(1000.0, 2000.00001), (1000.0, 2000.001), (1000.0, 5000.0)]
            .into_iter()
            .for_each(|(flow1, flow2)| {
                // 2 subnets
                let owner_hotkey = U256::from(60);
                let owner_coldkey = U256::from(61);
                let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
                let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

                // Configure blending window and current block
                let half_life: u64 = FlowHalfLife::<Test>::get();
                FlowNormExponent::<Test>::set(u64f64(1.0));
                frame_system::Pallet::<Test>::set_block_number(half_life);
                TaoFlowCutoff::<Test>::set(I64F64::from_num(2_000));

                // Set flows
                let now = frame_system::Pallet::<Test>::block_number();
                SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(flow1)));
                SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(flow2)));

                let shares = SubtensorModule::get_shares(&[n1, n2]);
                let s1 = shares.get(&n1).unwrap().to_num::<f64>();
                let s2 = shares.get(&n2).unwrap().to_num::<f64>();

                // Sum ~ 1
                assert_abs_diff_eq!(s1 + s2, 1.0_f64, epsilon = 1e-9);
                assert!(
                    s2 > 0.999_999 && s1 < 1e-6,
                    "expected s2≈1, s1≈0; got s1={s1}, s2={s2}"
                );
            });
    });
}

/// If subnets have negative EMA flows, but they are above the cut-off, emissions are proportional
/// for all except the bottom one, which gets nothing
#[test]
fn get_shares_both_negative_above_cutoff() {
    new_test_ext(1).execute_with(|| {
        // 3 subnets
        let owner_hotkey = U256::from(60);
        let owner_coldkey = U256::from(61);
        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n3 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Configure blending window and current block
        let half_life: u64 = FlowHalfLife::<Test>::get();
        FlowNormExponent::<Test>::set(u64f64(1.0));
        frame_system::Pallet::<Test>::set_block_number(half_life);
        TaoFlowCutoff::<Test>::set(I64F64::from_num(-1000));

        // Set flows
        let now = frame_system::Pallet::<Test>::block_number();
        SubnetEmaTaoFlow::<Test>::insert(n1, (now, i64f64(-100.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (now, i64f64(-300.0)));
        SubnetEmaTaoFlow::<Test>::insert(n3, (now, i64f64(-400.0)));

        let shares = SubtensorModule::get_shares(&[n1, n2, n3]);
        let s1 = shares.get(&n1).unwrap().to_num::<f64>();
        let s2 = shares.get(&n2).unwrap().to_num::<f64>();
        let s3 = shares.get(&n3).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1, 0.75, epsilon = s1 / 100.0);
        assert_abs_diff_eq!(s2, 0.25, epsilon = s2 / 100.0);
        assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(s1 + s2 + s3, 1.0, epsilon = 1e-9);
    });
}

/// Test that get_shares is idempotent within the same block.
/// When called multiple times in the same block, it should return the same values.
#[test]
fn test_get_shares_idempotent_within_same_block() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(100);
        let owner_coldkey = U256::from(200);
        let n1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let n2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let block_num = FlowHalfLife::<Test>::get();
        System::set_block_number(block_num);

        SubnetEmaTaoFlow::<Test>::insert(n1, (block_num, i64f64(1000.0)));
        SubnetEmaTaoFlow::<Test>::insert(n2, (block_num, i64f64(3000.0)));

        // Call get_shares twice in the same block
        let shares1 = SubtensorModule::get_shares(&[n1, n2]);
        let shares2 = SubtensorModule::get_shares(&[n1, n2]);

        // Both calls should return the same values
        let s1a = shares1.get(&n1).unwrap().to_num::<f64>();
        let s1b = shares2.get(&n1).unwrap().to_num::<f64>();
        let s2a = shares1.get(&n2).unwrap().to_num::<f64>();
        let s2b = shares2.get(&n2).unwrap().to_num::<f64>();

        assert_abs_diff_eq!(s1a, s1b, epsilon = 1e-18);
        assert_abs_diff_eq!(s2a, s2b, epsilon = 1e-18);
    });
}

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
fn test_effective_root_prop_root_stake_but_no_root_dividends() {
    // When validators have root stake registered on the subnet but there are NO root dividends,
    // utilization should be 0 (the else if total_root_stake > zero branch).
    // This is different from test_effective_root_prop_no_root_dividends which has no registered
    // validators with root stake.
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(100);
        let coldkey1 = U256::from(200);
        let hotkey2 = U256::from(101);
        let coldkey2 = U256::from(201);

        // Register hotkeys on subnet
        Keys::<Test>::insert(netuid, 0u16, hotkey1);
        Keys::<Test>::insert(netuid, 1u16, hotkey2);
        SubnetworkN::<Test>::insert(netuid, 2u16);

        // Give the hotkeys root stake
        increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000u64.into(), NetUid::ROOT);
        increase_stake_on_coldkey_hotkey_account(&coldkey2, &hotkey2, 1000u64.into(), NetUid::ROOT);

        // Create non-empty alpha dividends (so raw_root_prop denominator is non-zero)
        let mut alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_dividends.insert(hotkey1, U96F32::from_num(1000));
        alpha_dividends.insert(hotkey2, U96F32::from_num(2000));

        // Create EMPTY root_alpha_dividends (so total_root_divs = 0)
        let root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let utilization = SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        // Assert utilization is 0 because no root dividends despite having root stake
        assert_abs_diff_eq!(utilization.to_num::<f64>(), 0.0, epsilon = 1e-12);

        // Assert EffectiveRootProp is also 0
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
fn test_effective_root_prop_single_validator_always_full_utilization() {
    // A single validator should always achieve 100% utilization because it gets
    // 100% of both expected and actual share (efficiency = 1.0).
    // With equal alpha and root dividends, raw_root_prop = 0.5.
    // EffectiveRootProp = raw_root_prop * utilization = 0.5 * 1.0 = 0.5
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(100);
        let coldkey = U256::from(200);

        // Register ONE hotkey on subnet
        Keys::<Test>::insert(netuid, 0u16, hotkey);
        SubnetworkN::<Test>::insert(netuid, 1u16);

        // Give it root stake (1M)
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1_000_000u64.into(), NetUid::ROOT);

        // Create alpha_dividends with hotkey: 5000
        let mut alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_dividends.insert(hotkey, U96F32::from_num(5000));

        // Create root_alpha_dividends with hotkey: 5000
        let mut root_alpha_dividends: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_alpha_dividends.insert(hotkey, U96F32::from_num(5000));

        let utilization = SubtensorModule::compute_and_store_effective_root_prop(
            netuid,
            &alpha_dividends,
            &root_alpha_dividends,
        );

        // Single validator always gets 100% utilization
        assert_abs_diff_eq!(utilization.to_num::<f64>(), 1.0, epsilon = 1e-12);

        let prop = EffectiveRootProp::<Test>::get(netuid);
        // raw_root_prop = 5000/(5000+5000) = 0.5
        // EffectiveRootProp = 0.5 * 1.0 = 0.5
        assert_abs_diff_eq!(prop.to_num::<f64>(), 0.5, epsilon = 1e-9);
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
// Tests for full filter chain composition (ERP scaling -> proportion -> absolute)
// ===========================================================================

#[test]
fn test_full_filter_chain_erp_zeroes_shares_then_proportion_sees_fewer_nonzero() {
    // Full filter chain: apply_effective_root_prop_scaling -> apply_top_subnet_proportion_filter
    // -> apply_top_subnet_absolute_limit compose correctly.
    //
    // Setup: 4 subnets. After ERP scaling, two subnets are effectively zeroed (ERP = 0),
    // leaving only 2 nonzero. The proportion filter at 50% of 4 would normally keep
    // ceil(4 * 0.5) = 2, but since only 2 are nonzero, both survive. The absolute limit
    // of 3 is not binding. Result: exactly 2 nonzero subnets.
    new_test_ext(1).execute_with(|| {
        EffectiveRootPropEmissionScaling::<Test>::set(true);
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(3));

        // Subnets 1 and 2 have zero ERP -> their shares will be zeroed by ERP scaling
        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0));
        EffectiveRootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0));
        EffectiveRootProp::<Test>::insert(NetUid::from(3), U96F32::from_num(0.5));
        EffectiveRootProp::<Test>::insert(NetUid::from(4), U96F32::from_num(0.8));

        RootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(3), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(4), U96F32::from_num(0.8));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.25));
        shares.insert(NetUid::from(2), u64f64(0.25));
        shares.insert(NetUid::from(3), u64f64(0.25));
        shares.insert(NetUid::from(4), u64f64(0.25));

        // Step 1: ERP scaling zeros subnets 1 and 2
        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        // Subnets 3 and 4 are the only nonzero ones
        let nonzero_after_erp = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_erp, 2);

        // Step 2: Proportion filter (50% of 4 = ceil(2) = 2)
        // The top 2 by share are subnets 3 and 4, and subnets 1,2 are already zero.
        // Threshold is set by 2nd-highest share. Subnets 1,2 are below it -> stay zero.
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        let nonzero_after_prop = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_prop, 2);

        // Step 3: Absolute limit of 3 is not binding since only 2 nonzero
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        let nonzero_final = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_final, 2);

        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        assert!(s3 > 0.0);
        assert!(s4 > 0.0);
        assert_abs_diff_eq!(s3 + s4, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_full_filter_chain_erp_reduces_then_absolute_limit_binds() {
    // After ERP scaling, 3 of 5 subnets remain nonzero.
    // Proportion filter at 100% does nothing.
    // Absolute limit = 2 then trims to top 2.
    new_test_ext(1).execute_with(|| {
        EffectiveRootPropEmissionScaling::<Test>::set(true);
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(1.0)); // 100%
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(2));

        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0));
        EffectiveRootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0));
        EffectiveRootProp::<Test>::insert(NetUid::from(3), U96F32::from_num(0.3));
        EffectiveRootProp::<Test>::insert(NetUid::from(4), U96F32::from_num(0.5));
        EffectiveRootProp::<Test>::insert(NetUid::from(5), U96F32::from_num(0.7));

        RootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(3), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(4), U96F32::from_num(0.5));
        RootProp::<Test>::insert(NetUid::from(5), U96F32::from_num(0.7));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.2));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.2));
        shares.insert(NetUid::from(4), u64f64(0.2));
        shares.insert(NetUid::from(5), u64f64(0.2));

        // Step 1: ERP scaling zeros subnets 1 and 2 (ERP=0), leaves 3,4,5
        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);
        let nonzero_after_erp = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_erp, 3);

        // Step 2: Proportion at 100% keeps all
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);
        let nonzero_after_prop = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_prop, 3);

        // Step 3: Absolute limit of 2 trims to top 2 nonzero by share
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);
        let nonzero_final = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_final, 2);

        // Subnets 1 and 2 were zeroed by ERP, subnet 3 zeroed by absolute limit
        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-12);

        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();
        assert!(s4 > 0.0);
        assert!(s5 > 0.0);
        assert_abs_diff_eq!(s4 + s5, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_full_filter_chain_all_three_filters_active_and_binding() {
    // ERP scaling differentiates shares, proportion filter trims further,
    // absolute limit trims even further. Each stage reduces nonzero count.
    new_test_ext(1).execute_with(|| {
        EffectiveRootPropEmissionScaling::<Test>::set(true);
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.5)); // 50%
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(2));

        // 6 subnets, all start equal. After ERP scaling, subnet 6 has the highest
        // effective share because it has the highest min(ERP, RP).
        EffectiveRootProp::<Test>::insert(NetUid::from(1), U96F32::from_num(0.1));
        EffectiveRootProp::<Test>::insert(NetUid::from(2), U96F32::from_num(0.2));
        EffectiveRootProp::<Test>::insert(NetUid::from(3), U96F32::from_num(0.3));
        EffectiveRootProp::<Test>::insert(NetUid::from(4), U96F32::from_num(0.4));
        EffectiveRootProp::<Test>::insert(NetUid::from(5), U96F32::from_num(0.5));
        EffectiveRootProp::<Test>::insert(NetUid::from(6), U96F32::from_num(0.6));

        // RootProp >= ERP for all, so min(ERP, RP) = ERP
        for i in 1u16..=6 {
            RootProp::<Test>::insert(NetUid::from(i), U96F32::from_num(1.0));
        }

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        for i in 1u16..=6 {
            shares.insert(NetUid::from(i), u64f64(1.0 / 6.0));
        }

        // Step 1: ERP scaling. Each share *= its ERP, then re-normalize.
        // After: shares proportional to [0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
        SubtensorModule::apply_effective_root_prop_scaling(&mut shares);
        let nonzero_after_erp = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_erp, 6);

        // Step 2: Proportion filter at 50% of 6 = ceil(3) = 3. Keep top 3 by share.
        // That's subnets 4, 5, 6.
        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);
        let nonzero_after_prop = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_after_prop, 3);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s1, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s2, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(s3, 0.0, epsilon = 1e-12);

        // Step 3: Absolute limit of 2 trims to top 2. Subnets 5 and 6.
        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);
        let nonzero_final = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(nonzero_final, 2);

        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();
        let s6 = shares.get(&NetUid::from(6)).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(s4, 0.0, epsilon = 1e-12);
        assert!(s5 > 0.0);
        assert!(s6 > 0.0);
        assert_abs_diff_eq!(s5 + s6, 1.0, epsilon = 1e-9);
    });
}

// ===========================================================================
// Tie-inclusion tests for zero_and_redistribute_bottom_shares
// ===========================================================================

#[test]
fn test_zero_and_redistribute_bottom_shares_multiple_ties_at_cutoff_all_kept() {
    // 5 subnets: A=0.4, B=0.2, C=0.2, D=0.2, E=0.0 with top_k=2.
    // Top 1 is A (0.4). The 2nd position threshold is 0.2.
    // B, C, D all tie at 0.2 (the cutoff), so all must be included.
    // Result: 4 nonzero subnets (exceeding top_k=2).
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.4));
    shares.insert(NetUid::from(2), u64f64(0.2));
    shares.insert(NetUid::from(3), u64f64(0.2));
    shares.insert(NetUid::from(4), u64f64(0.2));
    shares.insert(NetUid::from(5), u64f64(0.0));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 2);

    let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
    let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
    let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
    let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
    let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();

    // A and all three tied subnets should be kept (4 nonzero, exceeding top_k=2)
    assert!(s1 > 0.0, "Subnet 1 (highest) should be kept");
    assert!(s2 > 0.0, "Subnet 2 should be kept (tie at cutoff)");
    assert!(s3 > 0.0, "Subnet 3 should be kept (tie at cutoff)");
    assert!(s4 > 0.0, "Subnet 4 should be kept (tie at cutoff)");
    assert_abs_diff_eq!(s5, 0.0, epsilon = 1e-12); // Subnet 5 (zero) should stay zero

    let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
    assert_eq!(
        nonzero_count, 4,
        "Tie inclusion should allow more than top_k nonzero subnets"
    );
    assert_abs_diff_eq!(s1 + s2 + s3 + s4, 1.0, epsilon = 1e-9);
}

#[test]
fn test_zero_and_redistribute_bottom_shares_all_equal_top_k_less_than_total() {
    // When all subnets have equal shares and top_k < total, all should be kept
    // because they all tie at the cutoff value.
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.2));
    shares.insert(NetUid::from(2), u64f64(0.2));
    shares.insert(NetUid::from(3), u64f64(0.2));
    shares.insert(NetUid::from(4), u64f64(0.2));
    shares.insert(NetUid::from(5), u64f64(0.2));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 1);

    // All 5 subnets should be kept because they all tie at the threshold
    for i in 1u16..=5 {
        let s = shares.get(&NetUid::from(i)).unwrap().to_num::<f64>();
        assert!(
            s > 0.0,
            "Subnet {i} should be kept (all tied at cutoff with top_k=1)"
        );
    }

    let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
    assert_eq!(
        nonzero_count, 5,
        "All subnets should survive when they all tie"
    );

    let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
    assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
}

#[test]
fn test_zero_and_redistribute_bottom_shares_large_tie_group_exceeds_top_k() {
    // 6 subnets: top 1 distinct, then 5 tied at the cutoff. top_k=3.
    // Threshold = value at position 2 (0-indexed). Positions 0-4 have >= threshold.
    // So 6 nonzero (all tied subnets kept), exceeding top_k=3.
    let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
    shares.insert(NetUid::from(1), u64f64(0.5));
    shares.insert(NetUid::from(2), u64f64(0.1));
    shares.insert(NetUid::from(3), u64f64(0.1));
    shares.insert(NetUid::from(4), u64f64(0.1));
    shares.insert(NetUid::from(5), u64f64(0.1));
    shares.insert(NetUid::from(6), u64f64(0.1));

    SubtensorModule::zero_and_redistribute_bottom_shares(&mut shares, 3);

    // The threshold is set at position 2 (top_k-1=2), which has value 0.1.
    // All 5 subnets with 0.1 tie at the cutoff + subnet 1 at 0.5.
    // All 6 should be kept.
    let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
    assert_eq!(
        nonzero_count, 6,
        "All 6 subnets kept: 1 above threshold + 5 tied at threshold"
    );

    let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
    assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
}

// ===========================================================================
// Tie-inclusion test for apply_top_subnet_proportion_filter
// ===========================================================================

#[test]
fn test_apply_top_subnet_proportion_filter_ties_at_boundary_included() {
    // 5 subnets with shares: A=0.4, B=0.2, C=0.2, D=0.1, E=0.1
    // Proportion = 40% -> ceil(5 * 0.4) = 2 -> top_k=2.
    // Top by share: A=0.4 (1st), B=0.2 (2nd-tied), C=0.2 (2nd-tied).
    // Threshold = 0.2 (value at position 1). B and C tie at boundary,
    // both should be included -> 3 nonzero (exceeding top_k=2).
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.4)); // 40%

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.4));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.2));
        shares.insert(NetUid::from(4), u64f64(0.1));
        shares.insert(NetUid::from(5), u64f64(0.1));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();

        assert!(s1 > 0.0, "Subnet 1 (highest) should be kept");
        assert!(s2 > 0.0, "Subnet 2 should be kept (tie at proportion boundary)");
        assert!(s3 > 0.0, "Subnet 3 should be kept (tie at proportion boundary)");
        assert_abs_diff_eq!(s4, 0.0, epsilon = 1e-12); // Subnet 4 should be zeroed
        assert_abs_diff_eq!(s5, 0.0, epsilon = 1e-12); // Subnet 5 should be zeroed

        let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_count, 3,
            "Tie inclusion means 3 subnets kept, exceeding ceil(5*0.4)=2"
        );

        assert_abs_diff_eq!(s1 + s2 + s3, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_proportion_filter_all_equal_shares() {
    // When all subnets have equal shares and proportion < 1.0,
    // all tie at the cutoff -> all should be kept.
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetProportion::<Test>::set(U64F64::saturating_from_num(0.25)); // 25%

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.25));
        shares.insert(NetUid::from(2), u64f64(0.25));
        shares.insert(NetUid::from(3), u64f64(0.25));
        shares.insert(NetUid::from(4), u64f64(0.25));

        SubtensorModule::apply_top_subnet_proportion_filter(&mut shares);

        // All tie -> all should be kept
        let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_count, 4,
            "All 4 subnets kept because they all tie at the cutoff"
        );

        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
    });
}

// ===========================================================================
// Tie-inclusion test for apply_top_subnet_absolute_limit
// ===========================================================================

#[test]
fn test_apply_top_subnet_absolute_limit_ties_at_boundary_included() {
    // 5 subnets with shares: A=0.4, B=0.2, C=0.2, D=0.1, E=0.1
    // Absolute limit = 2. Top 2 by share: A=0.4 (1st), B=0.2 (2nd-tied), C=0.2 (2nd-tied).
    // Both B and C tie at boundary -> 3 nonzero (exceeding limit=2).
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(2));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.4));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.2));
        shares.insert(NetUid::from(4), u64f64(0.1));
        shares.insert(NetUid::from(5), u64f64(0.1));

        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        let s1 = shares.get(&NetUid::from(1)).unwrap().to_num::<f64>();
        let s2 = shares.get(&NetUid::from(2)).unwrap().to_num::<f64>();
        let s3 = shares.get(&NetUid::from(3)).unwrap().to_num::<f64>();
        let s4 = shares.get(&NetUid::from(4)).unwrap().to_num::<f64>();
        let s5 = shares.get(&NetUid::from(5)).unwrap().to_num::<f64>();

        assert!(s1 > 0.0, "Subnet 1 (highest) should be kept");
        assert!(s2 > 0.0, "Subnet 2 should be kept (tie at absolute limit boundary)");
        assert!(s3 > 0.0, "Subnet 3 should be kept (tie at absolute limit boundary)");
        assert_abs_diff_eq!(s4, 0.0, epsilon = 1e-12); // Subnet 4 should be zeroed
        assert_abs_diff_eq!(s5, 0.0, epsilon = 1e-12); // Subnet 5 should be zeroed

        let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_count, 3,
            "Tie inclusion means 3 subnets kept, exceeding limit=2"
        );

        assert_abs_diff_eq!(s1 + s2 + s3, 1.0, epsilon = 1e-9);
        // Verify normalization: 0.4/0.8 = 0.5, 0.2/0.8 = 0.25 each
        assert_abs_diff_eq!(s1, 0.4 / 0.8, epsilon = 1e-9);
        assert_abs_diff_eq!(s2, 0.2 / 0.8, epsilon = 1e-9);
        assert_abs_diff_eq!(s3, 0.2 / 0.8, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_absolute_limit_all_equal_shares() {
    // When all subnets have equal shares and limit < total nonzero,
    // all tie at the cutoff -> all should be kept.
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(1));

        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.2));
        shares.insert(NetUid::from(2), u64f64(0.2));
        shares.insert(NetUid::from(3), u64f64(0.2));
        shares.insert(NetUid::from(4), u64f64(0.2));
        shares.insert(NetUid::from(5), u64f64(0.2));

        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        // All tie -> all should be kept despite limit=1
        let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_count, 5,
            "All 5 subnets kept because they all tie at the cutoff (limit=1)"
        );

        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
    });
}

#[test]
fn test_apply_top_subnet_absolute_limit_ties_with_large_tie_group() {
    // 7 subnets: one at 0.3, six tied at ~0.116667. Limit=3.
    // Threshold at position 2 = ~0.116667. All 6 tied subnets >= threshold.
    // So all 7 should be kept.
    new_test_ext(1).execute_with(|| {
        EmissionTopSubnetAbsoluteLimit::<Test>::set(Some(3));

        let tied_share = 0.7 / 6.0; // ~0.116667
        let mut shares: BTreeMap<NetUid, U64F64> = BTreeMap::new();
        shares.insert(NetUid::from(1), u64f64(0.3));
        for i in 2u16..=7 {
            shares.insert(NetUid::from(i), u64f64(tied_share));
        }

        SubtensorModule::apply_top_subnet_absolute_limit(&mut shares);

        let nonzero_count = shares.values().filter(|v| v.to_num::<f64>() > 0.0).count();
        assert_eq!(
            nonzero_count, 7,
            "All 7 kept: 1 above threshold + 6 tied at threshold, exceeding limit=3"
        );

        let sum: f64 = shares.values().map(|v| v.to_num::<f64>()).sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-9);
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

#[test]
fn test_get_root_dividend_fraction_zero_tao_weight() {
    // With tao_weight = 0, root_alpha_weighted = root_stake * 0 = 0
    // fraction = 0 / (alpha_stake + 0) = 0
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, _hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0);

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey1, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_get_root_dividend_fraction_no_stake_at_all() {
    // Hotkey with no stake anywhere (no root, no alpha) → fraction = 0
    // Early return when root_stake_f <= 0
    new_test_ext(1).execute_with(|| {
        let (netuid, _hotkey1, _hotkey2) = setup_scaling_test();
        let hotkey_no_stake = U256::from(999);
        let tao_weight = U96F32::from_num(0.18);

        let frac = SubtensorModule::get_root_dividend_fraction(&hotkey_no_stake, netuid, tao_weight);
        assert_abs_diff_eq!(frac.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_root_proportion_computation() {
    // Test the root_proportion() function that computes:
    // root_proportion = (tao_weight * SubnetTAO(ROOT)) / ((tao_weight * SubnetTAO(ROOT)) + alpha_issuance)
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);

        // Scenario 1: Zero root TAO → root_proportion should be 0
        {
            // Set SubnetTAO for ROOT to 0
            SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(0u64));

            // Set alpha issuance components (only SubnetAlphaOut for simplicity)
            SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(1_000_000u64));

            // Set TaoWeight to 0.18 (18% of u64::MAX)
            TaoWeight::<Test>::set(u64::MAX / 100 * 18);

            let root_prop = SubtensorModule::root_proportion(netuid);
            assert_abs_diff_eq!(root_prop.to_num::<f64>(), 0.0, epsilon = 1e-12);
        }

        // Scenario 2: Zero alpha issuance, nonzero root → should return close to 1.0
        {
            // Set SubnetTAO for ROOT to a nonzero value
            SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000u64));

            // Set alpha issuance to 0 (clear all components)
            SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(0u64));
            SubnetAlphaInProvided::<Test>::insert(netuid, AlphaCurrency::from(0u64));
            SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(0u64));

            // Set TaoWeight to 0.18
            TaoWeight::<Test>::set(u64::MAX / 100 * 18);

            let root_prop = SubtensorModule::root_proportion(netuid);
            // With zero alpha issuance, denominator = tao_weight * root_tao + 0
            // So root_prop = tao_weight * root_tao / tao_weight * root_tao = 1.0
            assert_abs_diff_eq!(root_prop.to_num::<f64>(), 1.0, epsilon = 1e-9);
        }

        // Scenario 3: Balanced - equal weighted root and alpha → should be ~0.5
        {
            // Set SubnetTAO for ROOT
            SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000u64));

            // Set TaoWeight to 0.5 (50% of u64::MAX)
            TaoWeight::<Test>::set(u64::MAX / 2);

            // Set alpha issuance such that alpha_issuance = tao_weight * root_tao
            // tao_weight * root_tao = 0.5 * 1_000_000 = 500_000
            // So we need alpha_issuance = 500_000
            SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(500_000u64));
            SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(0u64));
            SubnetAlphaInProvided::<Test>::insert(netuid, AlphaCurrency::from(0u64));

            let root_prop = SubtensorModule::root_proportion(netuid);
            // root_prop = 500_000 / (500_000 + 500_000) = 0.5
            assert_abs_diff_eq!(root_prop.to_num::<f64>(), 0.5, epsilon = 1e-9);
        }
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

#[test]
fn test_apply_utilization_scaling_no_root_stake_at_all() {
    // Verify early exit when root_alpha_dividends is empty (pure alpha-only subnet)
    // Even with low utilization, nothing should happen since there are no root dividends
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0); // Low utilization - doesn't matter

        // Create alpha_dividends with entries
        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(10000));
        alpha_divs.insert(hotkey2, U96F32::from_num(5000));

        // Empty root_alpha_dividends (pure alpha-only subnet with no root stake)
        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();

        let alpha_divs_before = alpha_divs.clone();

        let recycled = SubtensorModule::apply_utilization_scaling(
            netuid,
            utilization,
            &mut alpha_divs,
            &mut root_divs,
            tao_weight,
        );

        // Should return 0 recycled (early exit on !has_root_dividends)
        assert_abs_diff_eq!(recycled.to_num::<f64>(), 0.0, epsilon = 1e-12);

        // alpha_dividends should be unchanged
        assert_eq!(alpha_divs, alpha_divs_before);
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

        // root_alpha_dividends should still be empty
        assert!(root_divs.is_empty(), "Root divs should remain empty");
    });
}

#[test]
fn test_apply_utilization_scaling_zero_utilization() {
    // utilization = 0 → hard cap: recycle ALL root dividends, set ERP = 0
    new_test_ext(1).execute_with(|| {
        let (netuid, hotkey1, hotkey2) = setup_scaling_test();
        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0);

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
fn test_apply_utilization_scaling_hotkey_only_root_stake() {
    // Test case: hotkey with ONLY root stake (no alpha stake on subnet)
    // Should have root_fraction = 1.0, causing all alpha dividends to be recycled
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(100);
        let coldkey1 = U256::from(200);
        let hotkey2 = U256::from(101);
        let coldkey2 = U256::from(201);

        // hotkey1: 0 alpha stake, 1M root stake (only root stake)
        increase_stake_on_coldkey_hotkey_account(
            &coldkey1,
            &hotkey1,
            1_000_000u64.into(),
            NetUid::ROOT,
        );

        // hotkey2: 500k alpha stake, 0 root stake (for comparison)
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey2,
            netuid,
            AlphaCurrency::from(500_000u64),
        );

        // Need SubnetAlphaOut for recycling to work
        SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(10_000_000u64));

        let tao_weight = U96F32::from_num(0.18);
        let utilization = U96F32::from_num(0.3); // Triggers hard cap

        // Set a non-zero ERP so we can verify it gets zeroed
        EffectiveRootProp::<Test>::insert(netuid, U96F32::from_num(0.5));

        let mut alpha_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        alpha_divs.insert(hotkey1, U96F32::from_num(8000));
        alpha_divs.insert(hotkey2, U96F32::from_num(5000));

        let mut root_divs: BTreeMap<U256, U96F32> = BTreeMap::new();
        root_divs.insert(hotkey1, U96F32::from_num(2000));

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

        // hotkey1 has only root stake (root_fraction = 1.0), so ALL alpha dividends should be recycled
        let alpha1 = alpha_divs.get(&hotkey1).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(alpha1, 0.0, epsilon = 1.0);

        // hotkey2 has no root stake (root_fraction = 0.0), so alpha dividends should be unchanged
        let alpha2 = alpha_divs.get(&hotkey2).unwrap().to_num::<f64>();
        assert_abs_diff_eq!(alpha2, 5000.0, epsilon = 1.0);

        // Total recycled should include all root divs (2000) + all of hotkey1's alpha divs (8000)
        let recycled_val = recycled.to_num::<f64>();
        assert_abs_diff_eq!(recycled_val, 10000.0, epsilon = 100.0);

        // EffectiveRootProp should be 0
        let erp = EffectiveRootProp::<Test>::get(netuid);
        assert_abs_diff_eq!(erp.to_num::<f64>(), 0.0, epsilon = 1e-12);
    });
}

#[test]
fn test_remove_network_cleans_root_alpha_dividends_per_subnet() {
    new_test_ext(1).execute_with(|| {
        // Setup: Create a subnet
        let owner_hotkey = U256::from(100);
        let owner_coldkey = U256::from(200);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Create some hotkeys to test cleanup
        let hotkey1 = U256::from(101);
        let hotkey2 = U256::from(102);

        // Insert entries into RootAlphaDividendsPerSubnet
        RootAlphaDividendsPerSubnet::<Test>::insert(netuid, hotkey1, AlphaCurrency::from(1000u64));
        RootAlphaDividendsPerSubnet::<Test>::insert(netuid, hotkey2, AlphaCurrency::from(2000u64));

        // Insert entries into EffectiveRootProp, RootProp, and RootClaimableThreshold
        EffectiveRootProp::<Test>::insert(netuid, U96F32::from_num(0.5));
        RootProp::<Test>::insert(netuid, U96F32::from_num(0.3));
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(100));

        // Verify that the data exists before removal
        assert_eq!(
            RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey1),
            AlphaCurrency::from(1000u64)
        );
        assert_eq!(
            RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey2),
            AlphaCurrency::from(2000u64)
        );
        assert_eq!(
            EffectiveRootProp::<Test>::get(netuid).to_num::<f64>(),
            0.5
        );
        assert_abs_diff_eq!(
            RootProp::<Test>::get(netuid).to_num::<f64>(),
            0.3,
            epsilon = 1e-6
        );
        assert_abs_diff_eq!(
            RootClaimableThreshold::<Test>::get(netuid).to_num::<f64>(),
            100.0,
            epsilon = 1e-6
        );

        // Action: Remove the network
        SubtensorModule::remove_network(netuid);

        // Assert: Verify that RootAlphaDividendsPerSubnet entries are cleaned up
        assert_eq!(
            RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey1),
            AlphaCurrency::from(0u64),
            "RootAlphaDividendsPerSubnet for hotkey1 should be zero after remove_network"
        );
        assert_eq!(
            RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey2),
            AlphaCurrency::from(0u64),
            "RootAlphaDividendsPerSubnet for hotkey2 should be zero after remove_network"
        );

        // Assert: Verify that EffectiveRootProp is cleaned up
        assert_eq!(
            EffectiveRootProp::<Test>::get(netuid).to_num::<f64>(),
            0.0,
            "EffectiveRootProp should be zero after remove_network"
        );

        // Assert: Verify that RootProp is cleaned up
        assert_eq!(
            RootProp::<Test>::get(netuid).to_num::<f64>(),
            0.0,
            "RootProp should be zero after remove_network"
        );

        // Assert: Verify that RootClaimableThreshold is cleaned up
        assert_eq!(
            RootClaimableThreshold::<Test>::get(netuid).to_num::<f64>(),
            0.0,
            "RootClaimableThreshold should be zero after remove_network"
        );
    });
}

#[test]
fn test_finalize_all_subnet_root_dividends_cleanup() {
    new_test_ext(1).execute_with(|| {
        // Setup: Create a subnet (netuid=1)
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        add_network(netuid1, 1, 0);

        // Create hotkeys
        let hotkey1 = U256::from(101);
        let hotkey2 = U256::from(102);
        let coldkey1 = U256::from(201);
        let coldkey2 = U256::from(202);

        // Set up RootClaimable for both hotkeys with entries for netuid=1 and netuid=2
        let mut claimable_map1 = BTreeMap::new();
        claimable_map1.insert(netuid1, I96F32::from_num(1000.0));
        claimable_map1.insert(netuid2, I96F32::from_num(2000.0));
        RootClaimable::<Test>::insert(hotkey1, claimable_map1);

        let mut claimable_map2 = BTreeMap::new();
        claimable_map2.insert(netuid1, I96F32::from_num(1500.0));
        claimable_map2.insert(netuid2, I96F32::from_num(2500.0));
        RootClaimable::<Test>::insert(hotkey2, claimable_map2);

        // Set up RootClaimed entries for (netuid, hotkey, coldkey) pairs
        // For netuid=1
        RootClaimed::<Test>::insert((netuid1, &hotkey1, &coldkey1), 500u128);
        RootClaimed::<Test>::insert((netuid1, &hotkey2, &coldkey2), 600u128);

        // For netuid=2 (these should NOT be cleaned)
        RootClaimed::<Test>::insert((netuid2, &hotkey1, &coldkey1), 700u128);
        RootClaimed::<Test>::insert((netuid2, &hotkey2, &coldkey2), 800u128);

        // Verify setup
        assert!(RootClaimable::<Test>::get(hotkey1).contains_key(&netuid1));
        assert!(RootClaimable::<Test>::get(hotkey1).contains_key(&netuid2));
        assert!(RootClaimable::<Test>::get(hotkey2).contains_key(&netuid1));
        assert!(RootClaimable::<Test>::get(hotkey2).contains_key(&netuid2));

        assert_eq!(RootClaimed::<Test>::get((netuid1, &hotkey1, &coldkey1)), 500u128);
        assert_eq!(RootClaimed::<Test>::get((netuid1, &hotkey2, &coldkey2)), 600u128);
        assert_eq!(RootClaimed::<Test>::get((netuid2, &hotkey1, &coldkey1)), 700u128);
        assert_eq!(RootClaimed::<Test>::get((netuid2, &hotkey2, &coldkey2)), 800u128);

        // Action: Call finalize_all_subnet_root_dividends for netuid=1
        SubtensorModule::finalize_all_subnet_root_dividends(NetUid::from(netuid1));

        // Assert: RootClaimable for hotkey1 no longer contains netuid=1 (but still contains netuid=2)
        assert!(
            !RootClaimable::<Test>::get(hotkey1).contains_key(&netuid1),
            "RootClaimable for hotkey1 should not contain netuid=1 after cleanup"
        );
        assert!(
            RootClaimable::<Test>::get(hotkey1).contains_key(&netuid2),
            "RootClaimable for hotkey1 should still contain netuid=2"
        );
        assert_eq!(
            RootClaimable::<Test>::get(hotkey1).get(&netuid2).unwrap().to_num::<f64>(),
            2000.0,
            "RootClaimable for hotkey1 netuid=2 should be unchanged"
        );

        // Assert: RootClaimable for hotkey2 no longer contains netuid=1 (but still contains netuid=2)
        assert!(
            !RootClaimable::<Test>::get(hotkey2).contains_key(&netuid1),
            "RootClaimable for hotkey2 should not contain netuid=1 after cleanup"
        );
        assert!(
            RootClaimable::<Test>::get(hotkey2).contains_key(&netuid2),
            "RootClaimable for hotkey2 should still contain netuid=2"
        );
        assert_eq!(
            RootClaimable::<Test>::get(hotkey2).get(&netuid2).unwrap().to_num::<f64>(),
            2500.0,
            "RootClaimable for hotkey2 netuid=2 should be unchanged"
        );

        // Assert: RootClaimed for (netuid=1, hotkey, coldkey) is gone
        assert_eq!(
            RootClaimed::<Test>::get((netuid1, &hotkey1, &coldkey1)),
            0u128,
            "RootClaimed for (netuid1, hotkey1, coldkey1) should be zero after cleanup"
        );
        assert_eq!(
            RootClaimed::<Test>::get((netuid1, &hotkey2, &coldkey2)),
            0u128,
            "RootClaimed for (netuid1, hotkey2, coldkey2) should be zero after cleanup"
        );

        // Assert: RootClaimed for (netuid=2, hotkey, coldkey) is still present
        assert_eq!(
            RootClaimed::<Test>::get((netuid2, &hotkey1, &coldkey1)),
            700u128,
            "RootClaimed for (netuid2, hotkey1, coldkey1) should remain unchanged"
        );
        assert_eq!(
            RootClaimed::<Test>::get((netuid2, &hotkey2, &coldkey2)),
            800u128,
            "RootClaimed for (netuid2, hotkey2, coldkey2) should remain unchanged"
        );
    });
}

#[test]
fn test_root_sell_flag_boundary() {
    new_test_ext(1).execute_with(|| {
        // Setup: Create owner for subnets
        let owner_hotkey = U256::from(100);
        let owner_coldkey = U256::from(200);

        // Create 2 subnets
        let netuid1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let netuid2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Scenario 1: Total exactly at 1.0 (both at 0.5)
        // Should return false (not strictly above 1.0)
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.5));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(0.5));

        let subnets = vec![netuid1, netuid2];
        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&subnets);
        assert!(
            !root_sell_flag,
            "Root sell flag should be false when total moving price equals 1.0"
        );

        // Scenario 2: Total just above 1.0 (e.g., 0.500001 and 0.500001)
        // Should return true (strictly above 1.0)
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.500001));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(0.500001));

        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&subnets);
        assert!(
            root_sell_flag,
            "Root sell flag should be true when total moving price is above 1.0"
        );

        // Scenario 3: Total below 1.0 (both at 0.4)
        // Should return false
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.4));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(0.4));

        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&subnets);
        assert!(
            !root_sell_flag,
            "Root sell flag should be false when total moving price is below 1.0"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::subnet_emissions::test_distribute_emission_zero_incentive_sum --exact --show-output --nocapture
#[test]
fn test_distribute_emission_zero_incentive_sum() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // Register 3 validator hotkeys
        let validator1 = U256::from(1);
        let validator2 = U256::from(2);
        let validator3 = U256::from(3);
        let coldkey1 = U256::from(11);
        let coldkey2 = U256::from(12);
        let coldkey3 = U256::from(13);

        // Register all validators
        register_ok_neuron(netuid, validator1, coldkey1, 0);
        register_ok_neuron(netuid, validator2, coldkey2, 0);
        register_ok_neuron(netuid, validator3, coldkey3, 0);

        // Give them some stake so they can receive dividends
        let stake_amount = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &validator1,
            &coldkey1,
            netuid,
            stake_amount,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &validator2,
            &coldkey2,
            netuid,
            stake_amount,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &validator3,
            &coldkey3,
            netuid,
            stake_amount,
        );

        // Mark them as validators so they can set weights
        ValidatorPermit::<Test>::insert(netuid, vec![true, true, true]);

        // Set up weights so validators are voting for each other (simulating all validators, no miners)
        let idx = SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(0));
        Weights::<Test>::insert(idx, 0, vec![(0u16, 0xFFFF / 3), (1u16, 0xFFFF / 3), (2u16, 0xFFFF / 3)]);
        Weights::<Test>::insert(idx, 1, vec![(0u16, 0xFFFF / 3), (1u16, 0xFFFF / 3), (2u16, 0xFFFF / 3)]);
        Weights::<Test>::insert(idx, 2, vec![(0u16, 0xFFFF / 3), (1u16, 0xFFFF / 3), (2u16, 0xFFFF / 3)]);

        // Manually set all incentives to ZERO (this simulates no miners getting any incentive)
        // This will cause incentive_sum to be zero in distribute_emission
        Incentive::<Test>::insert(idx, vec![0u16, 0u16, 0u16]);

        // Record initial stakes
        let initial_stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator1, &coldkey1, netuid);
        let initial_stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator2, &coldkey2, netuid);
        let initial_stake3 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator3, &coldkey3, netuid);

        // Set up emission amounts
        let pending_server_alpha = AlphaCurrency::from(500_000_000); // 500M for miners
        let pending_validator_alpha = AlphaCurrency::from(500_000_000); // 500M for validators
        let pending_root_alpha = AlphaCurrency::ZERO;
        let pending_owner_cut = AlphaCurrency::ZERO;

        // Call distribute_emission
        // When incentive_sum == 0, validators should get BOTH server and validator alpha
        SubtensorModule::distribute_emission(
            netuid,
            pending_server_alpha,
            pending_validator_alpha,
            pending_root_alpha,
            pending_owner_cut,
        );

        // Check final stakes
        let final_stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator1, &coldkey1, netuid);
        let final_stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator2, &coldkey2, netuid);
        let final_stake3 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&validator3, &coldkey3, netuid);

        // Calculate the increase in stake
        let increase1 = final_stake1.saturating_sub(initial_stake1);
        let increase2 = final_stake2.saturating_sub(initial_stake2);
        let increase3 = final_stake3.saturating_sub(initial_stake3);
        let total_increase = increase1.saturating_add(increase2).saturating_add(increase3);

        // The total increase should be close to server_alpha + validator_alpha = 1B
        // (minus any rounding or take amounts)
        let expected_total = pending_server_alpha.saturating_add(pending_validator_alpha);

        // Allow 10% margin for rounding, take, etc.
        let tolerance = expected_total.to_u64() / 10;

        assert!(
            (total_increase.to_u64() as i128 - expected_total.to_u64() as i128).abs() < tolerance as i128,
            "When incentive_sum == 0, validators should receive both server and validator alpha. \
             Expected: {}, Got: {}, Difference: {}",
            expected_total.to_u64(),
            total_increase.to_u64(),
            (total_increase.to_u64() as i128 - expected_total.to_u64() as i128).abs()
        );

        // Verify that each validator got some emission (roughly equal since they have equal stake)
        assert!(
            increase1 > AlphaCurrency::ZERO,
            "Validator 1 should receive emission"
        );
        assert!(
            increase2 > AlphaCurrency::ZERO,
            "Validator 2 should receive emission"
        );
        assert!(
            increase3 > AlphaCurrency::ZERO,
            "Validator 3 should receive emission"
        );
    });
}

#[test]
fn test_get_subnets_to_emit_to_excludes_registration_disabled() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(100);
        let owner_coldkey = U256::from(200);

        // Create 3 subnets with registration enabled by default
        let subnet1 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let subnet2 = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let subnet3 = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Subnet 1: keep registration enabled (default from add_dynamic_network)
        // Subnet 2: disable both NetworkRegistrationAllowed and NetworkPowRegistrationAllowed
        NetworkRegistrationAllowed::<Test>::insert(subnet2, false);
        NetworkPowRegistrationAllowed::<Test>::insert(subnet2, false);
        // Subnet 3: keep registration enabled (default from add_dynamic_network)

        // Get all subnets
        let all_subnets = vec![subnet1, subnet2, subnet3];

        // Call get_subnets_to_emit_to
        let subnets_to_emit = SubtensorModule::get_subnets_to_emit_to(&all_subnets);

        // Assert subnet2 is excluded (registration disabled)
        assert!(
            !subnets_to_emit.contains(&subnet2),
            "Subnet with disabled registration should be excluded from emission"
        );

        // Assert subnet1 and subnet3 are included (registration enabled)
        assert!(
            subnets_to_emit.contains(&subnet1),
            "Subnet 1 with enabled registration should be included in emission"
        );
        assert!(
            subnets_to_emit.contains(&subnet3),
            "Subnet 3 with enabled registration should be included in emission"
        );

        // Verify exactly 2 subnets are in the result
        assert_eq!(
            subnets_to_emit.len(),
            2,
            "Expected 2 subnets to be eligible for emission"
        );
    });
}
