#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use super::mock::*;
use crate::migrations::migrate_seed_subnet_emission_weights::{
    MIGRATION_NAME, migrate_seed_subnet_emission_weights,
};
use crate::*;
use alloc::collections::BTreeMap;
use approx::assert_abs_diff_eq;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};
use subtensor_swap_interface::SwapHandler;

const ONE: u64 = EMISSION_WEIGHT_ONE;
const BURN: u16 = EmissionIndex::Burn as u16;
const RECYCLE: u16 = EmissionIndex::Recycle as u16;
const OWNER: u16 = EmissionIndex::Owner as u16;
const VALIDATOR: u16 = EmissionIndex::Validator as u16;
const MINER: u16 = EmissionIndex::Miner as u16;

/// `pct` percent of emission in u64 parts.
fn pct(p: u64) -> u64 {
    emission_fraction(p, 100)
}

fn sum(weights: &EmissionWeights) -> u128 {
    weights.iter().map(|w| *w as u128).sum()
}

fn full_vector(owner: u64, validator: u64) -> (Vec<u16>, Vec<u64>) {
    let miner = ONE - owner - validator;
    (
        vec![BURN, RECYCLE, OWNER, VALIDATOR, MINER],
        vec![0, 0, owner, validator, miner],
    )
}

fn new_subnet() -> NetUid {
    add_dynamic_network(&U256::from(1), &U256::from(2))
}

// ---------------------------------------------------------------------------
// Defaults and encoding
// ---------------------------------------------------------------------------

#[test]
fn test_constants_are_consistent() {
    assert_eq!(MAX_OWNER_EMISSION, pct(33));
    assert_eq!(MAX_VALIDATOR_EMISSION, pct(50));
    assert_eq!(MIN_MINER_EMISSION, pct(33));
    assert_eq!(emission_fraction(1, 1), ONE);
    assert_eq!(emission_fraction(0, 1), 0);
    assert_eq!(EmissionIndex::ALL.len(), EMISSION_WEIGHT_SLOTS as usize);
    for (position, index) in EmissionIndex::ALL.iter().enumerate() {
        assert_eq!(index.position(), position);
        assert_eq!(EmissionIndex::from_u16(position as u16), Some(*index));
    }
    assert_eq!(EmissionIndex::from_u16(EMISSION_WEIGHT_SLOTS as u16), None);
}

#[test]
fn test_new_subnet_is_seeded_with_default_vector() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let weights = SubtensorModule::get_subnet_emission_weights(netuid);
        let default = SubtensorModule::default_emission_weights();
        assert_eq!(weights, default);
        assert_eq!(sum(&weights), ONE as u128);
        assert_eq!(weights[0], 0);
        assert_eq!(weights[1], 0);
        // Mock owner cut is 0%: the remainder is split 50/50.
        assert_eq!(weights[2], 0);
        assert_eq!(weights[3], ONE / 2);
        assert_eq!(weights[4], ONE - ONE / 2);
        // No per-subnet TAO weight yet: the subnet follows the global knob.
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight(netuid),
            SubtensorModule::get_tao_weight()
        );
        assert!(!SubtensorModule::get_owner_cut_enabled(netuid));
    });
}

#[test]
fn test_default_vector_tracks_initial_owner_cut() {
    new_test_ext(1).execute_with(|| {
        InitialSubnetOwnerCut::set(11_796); // mainnet 18%
        let owner = SubtensorModule::default_owner_emission();
        assert_eq!(
            owner,
            SubtensorModule::scale_u16_fraction_to_emission_weight(11_796)
        );
        let weights = SubtensorModule::default_emission_weights();
        assert_eq!(weights[2], owner);
        assert_eq!(weights[3], (ONE - owner) / 2);
        assert_eq!(sum(&weights), ONE as u128);
        assert_eq!(SubtensorModule::leased_subnet_min_owner_emission(), owner);

        let netuid = new_subnet();
        assert_eq!(
            SubtensorModule::get_subnet_emission_weights(netuid),
            weights
        );
        assert!(SubtensorModule::get_owner_cut_enabled(netuid));
        InitialSubnetOwnerCut::set(0);
    });
}

#[test]
fn test_named_getters_mirror_slots() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let weights = [0, 0, pct(10), pct(45), ONE - pct(10) - pct(45)];
        SubtensorModule::set_subnet_emission_weights_unchecked(netuid, weights);
        assert_eq!(SubtensorModule::get_subnet_burn_emission(netuid), 0);
        assert_eq!(SubtensorModule::get_subnet_recycle_emission(netuid), 0);
        assert_eq!(
            SubtensorModule::get_subnet_owner_emission_cut(netuid),
            pct(10)
        );
        assert_eq!(
            SubtensorModule::get_subnet_validator_emission_cut(netuid),
            pct(45)
        );
        assert_eq!(
            SubtensorModule::get_subnet_miner_emission_cut(netuid),
            weights[4]
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_float_subnet_owner_emission_cut(netuid).to_num::<f64>(),
            0.10,
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_float_subnet_validator_emission_cut(netuid).to_num::<f64>(),
            0.45,
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_float_subnet_miner_emission_cut(netuid).to_num::<f64>(),
            0.45,
            epsilon = 1e-9
        );
        assert!(SubtensorModule::get_owner_cut_enabled(netuid));
    });
}

// ---------------------------------------------------------------------------
// Sparse merge semantics
// ---------------------------------------------------------------------------

#[test]
fn test_merge_keeps_omitted_slots_and_zeroes_reserved_ones() {
    let current = [7, 9, pct(10), pct(45), pct(45)];
    let merged =
        SubtensorModule::merge_emission_weight_updates(current, &[VALIDATOR], &[pct(40)]).unwrap();
    assert_eq!(merged, [0, 0, pct(10), pct(40), pct(45)]);
}

#[test]
fn test_merge_rejects_malformed_updates() {
    let current = SubtensorModule::default_emission_weights();
    assert_eq!(
        SubtensorModule::merge_emission_weight_updates(current, &[OWNER], &[]),
        Err(Error::<Test>::EmissionWeightsLengthMismatch)
    );
    assert_eq!(
        SubtensorModule::merge_emission_weight_updates(current, &[5], &[1]),
        Err(Error::<Test>::EmissionWeightIndexOutOfRange)
    );
    assert_eq!(
        SubtensorModule::merge_emission_weight_updates(current, &[OWNER, OWNER], &[1, 2]),
        Err(Error::<Test>::EmissionWeightIndexDuplicated)
    );
}

// ---------------------------------------------------------------------------
// Rules: one error per constraint
// ---------------------------------------------------------------------------

#[test]
fn test_set_rejects_unknown_or_root_subnet() {
    new_test_ext(1).execute_with(|| {
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(NetUid::from(42), vec![OWNER], vec![0]),
            Error::<Test>::SubnetNotExists
        );
        add_network(NetUid::ROOT, 1, 0);
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(NetUid::ROOT, vec![OWNER], vec![0]),
            Error::<Test>::EmissionWeightsNotSettableOnRoot
        );
    });
}

#[test]
fn test_set_rejects_burn_and_recycle() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, vec![BURN], vec![1]),
            Error::<Test>::BurnEmissionMustBeZero
        );
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, vec![RECYCLE], vec![1]),
            Error::<Test>::RecycleEmissionMustBeZero
        );
        // Explicit zeros for the reserved slots are accepted.
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            vec![BURN, RECYCLE],
            vec![0, 0]
        ));
    });
}

#[test]
fn test_set_rejects_owner_above_maximum() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let (uids, values) = full_vector(MAX_OWNER_EMISSION + 1, pct(34));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::OwnerEmissionAboveMaximum
        );
        // The bound is inclusive.
        let (uids, values) = full_vector(MAX_OWNER_EMISSION, pct(34));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert_eq!(
            SubtensorModule::get_subnet_owner_emission_cut(netuid),
            MAX_OWNER_EMISSION
        );
    });
}

#[test]
fn test_set_rejects_validator_above_maximum() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let (uids, values) = full_vector(0, MAX_VALIDATOR_EMISSION + 1);
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionAboveMaximum
        );
        let (uids, values) = full_vector(0, MAX_VALIDATOR_EMISSION);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
    });
}

#[test]
fn test_set_rejects_miner_below_minimum() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        // owner 33% + validator 34.1% leaves the miner below 33%.
        let (uids, values) = full_vector(pct(33), pct(34) + pct(1));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::MinerEmissionBelowMinimum
        );
        let (uids, values) = full_vector(pct(33), pct(34));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(SubtensorModule::get_subnet_miner_emission_cut(netuid) >= MIN_MINER_EMISSION);
    });
}

#[test]
fn test_set_rejects_vector_not_summing_to_one() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        // Each slot individually valid, but the total is one ulp short.
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(
                netuid,
                vec![OWNER, VALIDATOR, MINER],
                vec![pct(10), pct(45), ONE - pct(10) - pct(45) - 1]
            ),
            Error::<Test>::EmissionWeightsDoNotSumToOne
        );
        // ...and one ulp over.
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(
                netuid,
                vec![OWNER, VALIDATOR, MINER],
                vec![pct(10), pct(45), ONE - pct(10) - pct(45) + 1]
            ),
            Error::<Test>::EmissionWeightsDoNotSumToOne
        );
        // A sparse update that only touches the validator slot must also renormalise.
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, vec![VALIDATOR], vec![pct(40)]),
            Error::<Test>::EmissionWeightsDoNotSumToOne
        );
    });
}

#[test]
fn test_set_enforces_leased_subnet_owner_floor() {
    new_test_ext(1).execute_with(|| {
        InitialSubnetOwnerCut::set(11_796); // mainnet 18%
        let netuid = new_subnet();
        let floor = SubtensorModule::leased_subnet_min_owner_emission();
        assert!(floor > 0);

        // Not lease-created: owner may drop to zero.
        let (uids, values) = full_vector(0, pct(50));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            uids.clone(),
            values.clone()
        ));

        // Lease-created: the owner slot may not fall below the launch default.
        SubnetCreatedViaLease::<Test>::insert(netuid, true);
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::LeasedSubnetOwnerCutBelowMinimum
        );
        let (uids, values) = full_vector(floor - 1, pct(45));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::LeasedSubnetOwnerCutBelowMinimum
        );
        let (uids, values) = full_vector(floor, pct(45));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        InitialSubnetOwnerCut::set(0);
    });
}

// ---------------------------------------------------------------------------
// Effective TAO weight: solved once at set time so root's share is preserved
// ---------------------------------------------------------------------------

const TAO: u64 = 1_000_000_000;

/// Put `root_tao` rao of TAO on root and make `netuid`'s alpha issuance about
/// `alpha_issuance`; returns the issuance actually read back by the chain.
fn fund_root_and_issuance(netuid: NetUid, root_tao: u64, alpha_issuance: u64) -> f64 {
    SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(root_tao));
    SubnetAlphaIn::<Test>::insert(netuid, AlphaBalance::ZERO);
    SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(alpha_issuance));
    SubtensorModule::get_alpha_issuance(netuid).to_u64() as f64
}

/// Root's alpha out of one block's `alpha_out` on `netuid` under the live
/// state: `consensus_root_proportion · V · alpha_out`, exactly how
/// `emit_to_subnets` carves it out of the validator slot.
fn root_alpha_for(netuid: NetUid, alpha_out: f64) -> f64 {
    let proportion = SubtensorModule::consensus_root_proportion(netuid).to_num::<f64>();
    let validator =
        SubtensorModule::get_float_subnet_validator_emission_cut(netuid).to_num::<f64>();
    proportion * validator * alpha_out
}

/// `r · (1 + A / root_tao)`: the lowest validator share that still admits a
/// TAO weight `≤ 1`, given root's current share `r`.
fn validator_floor_for(netuid: NetUid, root_tao: f64, alpha_issuance: f64) -> f64 {
    let root_share = SubtensorModule::get_root_emission_share(netuid).to_num::<f64>();
    root_share * (1.0 + alpha_issuance / root_tao)
}

#[test]
fn test_set_emits_full_vector_event() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let (uids, values) = full_vector(pct(10), pct(40));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            uids,
            values.clone()
        ));
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::SubnetEmissionWeightsSet {
                netuid,
                weights: values,
            })
        );
    });
}

#[test]
fn test_subnet_follows_global_tao_weight_until_validator_share_changes() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let other = add_dynamic_network(&U256::from(3), &U256::from(4));
        SubtensorModule::set_tao_weight(pct(18));
        for subnet in [netuid, other] {
            assert_eq!(
                SubtensorModule::get_subnet_tao_weight_parts(subnet),
                pct(18)
            );
        }
        SubtensorModule::set_tao_weight(pct(30));
        for subnet in [netuid, other] {
            assert_eq!(
                SubtensorModule::get_subnet_tao_weight_parts(subnet),
                pct(30)
            );
        }

        // Touching owner/miner only never writes a per-subnet weight.
        let (uids, values) = full_vector(pct(10), ONE / 2);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));

        // Changing the validator share with root stake present solves and freezes it.
        fund_root_and_issuance(netuid, 1_000_000 * TAO, 1_000_000 * TAO);
        let (uids, values) = full_vector(pct(10), pct(40));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(SubnetTaoWeight::<Test>::contains_key(netuid));
        let frozen = SubtensorModule::get_subnet_tao_weight_parts(netuid);
        assert!(
            frozen > pct(30),
            "lower V must raise the subnet's TAO weight"
        );

        // A later global change moves the untouched subnet but not the frozen one...
        SubtensorModule::set_tao_weight(pct(5));
        assert_eq!(SubtensorModule::get_subnet_tao_weight_parts(netuid), frozen);
        assert_eq!(SubtensorModule::get_subnet_tao_weight_parts(other), pct(5));

        // ...and a set that leaves V alone keeps the frozen value too.
        let (uids, values) = full_vector(pct(20), pct(40));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert_eq!(SubtensorModule::get_subnet_tao_weight_parts(netuid), frozen);
    });
}

#[test]
fn test_set_without_root_stake_keeps_tao_weight_and_accepts_any_share() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(18));
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::ZERO);
        SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(1_000_000 * TAO));

        // Nothing to preserve: even a zero validator slot is accepted and w_s is untouched.
        let (uids, values) = full_vector(0, 0);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight_parts(netuid),
            pct(18)
        );

        // Root stake arrives later; coming back from V = 0 there is still nothing to
        // preserve, so the subnet keeps following the global weight.
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(1_000_000 * TAO));
        let (uids, values) = full_vector(0, pct(40));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));

        // No alpha issuance at all: the root proportion is weight-independent, keep w_s.
        let issuance = fund_root_and_issuance(netuid, 1_000_000 * TAO, 0);
        assert_eq!(issuance, 0.0);
        let (uids, values) = full_vector(0, pct(30));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
    });
}

#[test]
fn test_solve_tao_weight_for_root_share_edge_cases() {
    let f = U96F32::saturating_from_num::<f64>;
    let root_tao = f(1_000_000.0);
    let alpha = f(2_000_000.0);
    // V ≤ r: nothing is left for validators.
    assert_eq!(
        solve_tao_weight_for_root_share(f(0.2), f(0.2), root_tao, alpha),
        None
    );
    assert_eq!(
        solve_tao_weight_for_root_share(f(0.2), f(0.1), root_tao, alpha),
        None
    );
    // Would need root TAO to count more than 1:1: r = 0.25, V = 0.3125 →
    // x = 0.5M / 0.0625 = 8M → w = 8.
    assert_eq!(
        solve_tao_weight_for_root_share(f(0.25), f(0.3125), root_tao, alpha),
        None
    );
    // Exactly at the cap (binary-exact inputs): r = 0.25, V = 0.75 → x = 0.5M / 0.5 = 1M
    // → w = 1.
    let at_cap = solve_tao_weight_for_root_share(f(0.25), f(0.75), root_tao, alpha).unwrap();
    assert_eq!(at_cap, f(1.0));
    // A solution reproduces the requested share: r = 0.1, V = 0.5 → x = 0.2M / 0.4 = 0.5M
    // → w = 0.5.
    let w = solve_tao_weight_for_root_share(f(0.1), f(0.5), root_tao, alpha).unwrap();
    assert_abs_diff_eq!(w.to_num::<f64>(), 0.5, epsilon = 1e-9);
    assert_abs_diff_eq!(
        root_emission_share(f(0.5), w, root_tao, alpha).to_num::<f64>(),
        0.1,
        epsilon = 1e-9
    );
    // No root TAO: the division has no answer.
    assert_eq!(
        solve_tao_weight_for_root_share(f(0.1), f(0.3), f(0.0), alpha),
        None
    );
}

/// (a) Root's alpha the block after a set equals root's alpha the block
/// before, across global TaoWeight × issuance/root-TAO ratios × validator
/// shares, including the case where weighted root TAO is about the alpha
/// issuance and V sits just above its new floor.
#[test]
fn test_set_preserves_root_alpha_across_tao_weights_ratios_and_shares() {
    new_test_ext(1).execute_with(|| {
        let alpha_out = 1e9_f64;
        let root_tao_rao = 1_000_000 * TAO;
        let mut checked = 0u32;
        for tao_weight in [pct(5), pct(18), pct(50), ONE] {
            // A / root_tao: issuance far below, about equal to, and far above root TAO,
            // plus the skeptic's case A ≈ TaoWeight · root_tao for TaoWeight = 5%.
            for ratio in [0.05_f64, 0.1, 1.0, 10.0] {
                let netuid = new_subnet();
                SubtensorModule::set_tao_weight(tao_weight);
                let alpha_issuance = fund_root_and_issuance(
                    netuid,
                    root_tao_rao,
                    (root_tao_rao as f64 * ratio) as u64,
                );
                let root_tao = root_tao_rao as f64;
                let floor = validator_floor_for(netuid, root_tao, alpha_issuance);
                let share = |fraction: f64| (fraction * ONE as f64) as u64;
                for validator in [
                    pct(50),
                    pct(45),
                    pct(30),
                    pct(10),
                    share(floor * 1.001),
                    share(floor * 1.01),
                ] {
                    if validator > MAX_VALIDATOR_EMISSION || (validator as f64) < floor * ONE as f64
                    {
                        continue;
                    }
                    let before = root_alpha_for(netuid, alpha_out);
                    let share_before = SubtensorModule::get_root_emission_share(netuid);
                    let (uids, values) = full_vector(0, validator);
                    assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
                        netuid, uids, values
                    ));
                    let after = root_alpha_for(netuid, alpha_out);
                    assert!(before > 0.0);
                    assert_abs_diff_eq!(after, before, epsilon = before * 1e-6);
                    assert_abs_diff_eq!(
                        SubtensorModule::get_root_emission_share(netuid).to_num::<f64>(),
                        share_before.to_num::<f64>(),
                        epsilon = 1e-9
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 40, "only {checked} cases exercised");
    });
}

/// (b) Rejection when `V_new ≤ r` or the solved weight would exceed one, and
/// the rejected call leaves the vector and weight untouched.
#[test]
fn test_set_rejects_validator_share_that_cannot_preserve_root() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(50));
        let root_tao = 100_000_000 * TAO;
        let alpha_issuance = fund_root_and_issuance(netuid, root_tao, TAO * 1_000_000);
        // Weighted root TAO dwarfs issuance: root takes almost all of V = 0.5.
        let root_share = SubtensorModule::get_root_emission_share(netuid).to_num::<f64>();
        assert!(root_share > 0.49 && root_share < 0.5, "{root_share}");
        let floor = validator_floor_for(netuid, root_tao as f64, alpha_issuance);
        let share = |fraction: f64| (fraction * ONE as f64) as u64;

        // V_new ≤ r.
        let (uids, values) = full_vector(0, share(root_share * 0.99));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionTooLowToPreserveRoot
        );
        // r < V_new < floor: the weight would have to exceed one.
        let (uids, values) = full_vector(0, share((root_share + floor) / 2.0));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionTooLowToPreserveRoot
        );
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
        assert_eq!(
            SubtensorModule::get_subnet_emission_weights(netuid),
            SubtensorModule::default_emission_weights()
        );
        // Just above the floor is accepted with a weight a hair under one.
        let (uids, values) = full_vector(0, share(floor * 1.0001));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        let weight = SubtensorModule::get_subnet_tao_weight(netuid).to_num::<f64>();
        assert!(weight > 0.98 && weight <= 1.0, "{weight}");
    });
}

/// (c) Mainnet SN4-like numbers: 5.443M TAO on root, 6.457M alpha issued,
/// TaoWeight 18%, owner 18% / validator 41%. The validator share may go down
/// to about 11.8%.
#[test]
fn test_sn4_like_subnet_accepts_11_9_percent_and_rejects_11_7_percent() {
    new_test_ext(1).execute_with(|| {
        InitialSubnetOwnerCut::set(11_796); // mainnet 18%
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(18));
        let root_tao = 5_443_000 * TAO;
        let alpha_issuance = fund_root_and_issuance(netuid, root_tao, 6_457_000 * TAO);
        let owner = SubtensorModule::default_owner_emission();
        assert_abs_diff_eq!(
            SubtensorModule::get_float_subnet_validator_emission_cut(netuid).to_num::<f64>(),
            0.41,
            epsilon = 1e-4
        );
        let floor = validator_floor_for(netuid, root_tao as f64, alpha_issuance);
        assert_abs_diff_eq!(floor, 0.118, epsilon = 1e-3);

        let before = root_alpha_for(netuid, 1e9);
        let (uids, values) = full_vector(owner, emission_fraction(117, 1000));
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionTooLowToPreserveRoot
        );
        let (uids, values) = full_vector(owner, emission_fraction(119, 1000));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert_abs_diff_eq!(root_alpha_for(netuid, 1e9), before, epsilon = before * 1e-6);
        let weight = SubtensorModule::get_subnet_tao_weight(netuid).to_num::<f64>();
        assert!(weight > 0.98 && weight <= 1.0, "{weight}");
        InitialSubnetOwnerCut::set(0);
    });
}

#[test]
fn test_emission_weights_info_reports_vector_and_tao_weight() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::get_subnet_emission_weights_info(NetUid::from(42)).is_none());
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(18));
        fund_root_and_issuance(netuid, 1_000_000 * TAO, 1_000_000 * TAO);
        let (uids, values) = full_vector(pct(10), pct(25));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            uids,
            values.clone()
        ));
        let info = SubtensorModule::get_subnet_emission_weights_info(netuid).unwrap();
        assert_eq!(info.netuid, netuid.into());
        assert_eq!(
            info.weights.iter().map(|w| w.0).collect::<Vec<u64>>(),
            values
        );
        assert_eq!(info.burn.0, 0);
        assert_eq!(info.recycle.0, 0);
        assert_eq!(info.owner.0, pct(10));
        assert_eq!(info.validator.0, pct(25));
        assert_eq!(info.miner.0, ONE - pct(10) - pct(25));
        assert_eq!(
            info.effective_tao_weight.0,
            SubtensorModule::get_subnet_tao_weight_parts(netuid)
        );
        assert!(info.effective_tao_weight.0 > pct(18));
        assert!(!info.created_via_lease);
    });
}

/// The read-once view (what the EVM getter charges `EMISSION_WEIGHTS_VIEW_READS`
/// reads for) must agree with the per-netuid helpers.
#[test]
fn test_emission_weights_view_matches_per_netuid_helpers() {
    new_test_ext(1).execute_with(|| {
        assert_eq!(EMISSION_WEIGHTS_VIEW_READS, 2);
        let netuid = new_subnet();
        let (uids, values) = full_vector(pct(10), pct(40));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        let view = SubtensorModule::read_subnet_emission_weights_view(netuid);
        assert_eq!(
            view.weights,
            SubtensorModule::get_subnet_emission_weights(netuid)
        );
        assert!(!view.created_via_lease);
        SubnetCreatedViaLease::<Test>::insert(netuid, true);
        assert!(SubtensorModule::read_subnet_emission_weights_view(netuid).created_via_lease);
    });
}

// ---------------------------------------------------------------------------
// Coinbase: slots are honoured exactly and root payout is preserved
// ---------------------------------------------------------------------------

/// Run one `emit_to_subnets` on a fresh subnet with a funded root and return
/// `(alpha_emission, owner, server, validator, root_divs)`.
fn emit_once(validator_share: Option<u64>) -> (u64, u64, u64, u64, u64) {
    let netuid = new_subnet();
    SubtensorModule::set_tao_weight(pct(18));
    SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(100_000_000_000_000_u64));
    setup_reserves(
        netuid,
        TaoBalance::from(1_000_000_000_000_000_u64),
        AlphaBalance::from(1_000_000_000_000_000_u64),
    );
    assert_ok!(Swap::maybe_initialize_palswap(netuid, None));
    if let Some(validator) = validator_share {
        let (uids, values) = full_vector(pct(10), validator);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
    }

    let tao_emission = U96F32::from_num(12_345_678);
    let subnet_emissions = BTreeMap::from([(netuid, tao_emission)]);
    let alpha_emission = SubtensorModule::get_block_emission_for_issuance(
        SubtensorModule::get_alpha_issuance(netuid).into(),
    )
    .unwrap_or(0);
    let credit = SubtensorModule::mint_tao(12_345_678.into());
    SubtensorModule::emit_to_subnets(&[netuid], &subnet_emissions, credit, true);
    (
        alpha_emission,
        PendingOwnerCut::<Test>::get(netuid).to_u64(),
        PendingServerEmission::<Test>::get(netuid).to_u64(),
        PendingValidatorEmission::<Test>::get(netuid).to_u64(),
        PendingRootAlphaDivs::<Test>::get(netuid).to_u64(),
    )
}

#[test]
fn test_coinbase_honours_emission_weight_slots_exactly() {
    new_test_ext(1).execute_with(|| {
        let validator = pct(40);
        let (alpha_emission, owner, server, validator_divs, root_divs) = emit_once(Some(validator));
        let alpha = alpha_emission as f64;
        assert_abs_diff_eq!(owner as f64, alpha * 0.10, epsilon = 200.0);
        // Miner slot = 1 - 0.10 - 0.40 = 0.50 of emission, untouched by root.
        assert_abs_diff_eq!(server as f64, alpha * 0.50, epsilon = 200.0);
        // Validator slot = 0.40 of emission, root carved out of it.
        assert_abs_diff_eq!(
            (validator_divs + root_divs) as f64,
            alpha * 0.40,
            epsilon = 200.0
        );
        assert!(root_divs > 0);
        // Nothing is created or lost.
        assert_abs_diff_eq!(
            (owner + server + validator_divs + root_divs) as f64,
            alpha,
            epsilon = 400.0
        );
    });
}

#[test]
fn test_root_payout_is_invariant_to_validator_share() {
    new_test_ext(1).execute_with(|| {
        // Baseline: default vector, V = V₀ = 0.5 (mock owner cut is 0).
        let (alpha_a, _, _, _, root_a) = emit_once(None);
        assert!(root_a > 0);

        // Halve V to 0.25 (owner 10%, miner 65%) under the same pre-state: the solved
        // TAO weight keeps root's alpha exactly where it was.
        let (alpha_b, _, server_b, validator_b, root_b) = emit_once(Some(ONE / 4));
        assert_eq!(alpha_a, alpha_b);
        // Root's share of the smaller validator slot is larger...
        let root_share_b = root_b as f64 / (validator_b + root_b) as f64;
        let root_share_a = root_a as f64 / alpha_a as f64 * 2.0;
        assert!(root_share_b > root_share_a);
        // ...and root's absolute alpha is preserved.
        assert_abs_diff_eq!(root_b as f64, root_a as f64, epsilon = root_a as f64 * 1e-6);
        // Validators absorbed the cut, miners gained.
        assert_abs_diff_eq!(server_b as f64, alpha_b as f64 * 0.65, epsilon = 200.0);
    });
}

// ---------------------------------------------------------------------------
// Pool side (tao_in / alpha_in / injection cap) ignores the vector
// ---------------------------------------------------------------------------

/// A fresh subnet with a funded root and a TAO emission large enough that the
/// alpha-injection cap (`root_proportion × alpha_emission`) binds.
fn pool_subnet() -> (NetUid, BTreeMap<NetUid, U96F32>) {
    let netuid = new_subnet();
    SubtensorModule::set_tao_weight(pct(18));
    SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(100_000_000_000_000_u64));
    setup_reserves(
        netuid,
        TaoBalance::from(1_000_000_000_000_000_u64),
        AlphaBalance::from(1_000_000_000_000_000_u64),
    );
    assert_ok!(Swap::maybe_initialize_palswap(netuid, None));
    let emissions = BTreeMap::from([(netuid, U96F32::from_num(POOL_TAO_EMISSION))]);
    (netuid, emissions)
}

const POOL_TAO_EMISSION: u64 = 5_000_000_000;

/// `(tao_in, alpha_in, alpha_out, excess)` for the single subnet in `emissions`.
fn subnet_terms(
    netuid: NetUid,
    emissions: &BTreeMap<NetUid, U96F32>,
) -> (U96F32, U96F32, U96F32, U96F32) {
    let (tao_in, alpha_in, alpha_out, excess) = SubtensorModule::get_subnet_terms(emissions);
    (
        tao_in[&netuid],
        alpha_in[&netuid],
        alpha_out[&netuid],
        excess[&netuid],
    )
}

#[test]
fn test_pool_injection_terms_ignore_validator_share() {
    new_test_ext(1).execute_with(|| {
        let (netuid, emissions) = pool_subnet();
        let root_prop_before = SubtensorModule::root_proportion(netuid);
        let terms_before = subnet_terms(netuid, &emissions);
        // The cap binds: some TAO is excess rather than injected.
        assert!(terms_before.3 > U96F32::from_num(0));
        assert_eq!(
            SubtensorModule::consensus_root_proportion(netuid),
            root_prop_before
        );

        // Halve V: the consensus-side proportion doubles so root's alpha is unchanged...
        let (uids, values) = full_vector(pct(10), ONE / 4);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        let consensus_after = SubtensorModule::consensus_root_proportion(netuid);
        assert!(consensus_after > root_prop_before);
        assert_abs_diff_eq!(
            consensus_after.to_num::<f64>(),
            root_prop_before.to_num::<f64>() * 2.0,
            epsilon = root_prop_before.to_num::<f64>() * 1e-6
        );

        // ...while the pool side is bit-for-bit unchanged.
        assert_eq!(SubtensorModule::root_proportion(netuid), root_prop_before);
        assert_eq!(subnet_terms(netuid, &emissions), terms_before);
        SubtensorModule::update_root_prop();
        assert_eq!(RootProp::<Test>::get(netuid), root_prop_before);
    });
}

#[test]
fn test_pool_reserves_move_identically_whatever_the_validator_share() {
    new_test_ext(1).execute_with(|| {
        let mut deltas = Vec::new();
        for validator_share in [None, Some(ONE / 4), Some(pct(50))] {
            let (netuid, emissions) = pool_subnet();
            if let Some(validator) = validator_share {
                let (uids, values) = full_vector(pct(10), validator);
                assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
                    netuid, uids, values
                ));
            }
            let tao_before = SubnetTAO::<Test>::get(netuid).to_u64();
            let alpha_in_before = SubnetAlphaIn::<Test>::get(netuid).to_u64();
            let alpha_out_before = SubnetAlphaOut::<Test>::get(netuid).to_u64();
            let credit = SubtensorModule::mint_tao(POOL_TAO_EMISSION.into());
            SubtensorModule::emit_to_subnets(&[netuid], &emissions, credit, true);
            deltas.push((
                SubnetTAO::<Test>::get(netuid).to_u64() as i128 - tao_before as i128,
                SubnetAlphaIn::<Test>::get(netuid).to_u64() as i128 - alpha_in_before as i128,
                SubnetAlphaOut::<Test>::get(netuid).to_u64() as i128 - alpha_out_before as i128,
                SubnetTaoInEmission::<Test>::get(netuid).to_u64(),
                SubnetAlphaInEmission::<Test>::get(netuid).to_u64(),
                SubnetAlphaOutEmission::<Test>::get(netuid).to_u64(),
            ));
        }
        assert!(
            deltas[0].0 != 0 || deltas[0].1 != 0,
            "pool did not move at all"
        );
        assert!(deltas.iter().all(|d| *d == deltas[0]), "{deltas:?}");
    });
}

// ---------------------------------------------------------------------------
// Consensus stake weighting uses the subnet TAO weight
// ---------------------------------------------------------------------------

#[test]
fn test_stake_weights_scale_with_subnet_tao_weight() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        let hotkey = U256::from(10);
        let coldkey = U256::from(11);
        SubtensorModule::set_tao_weight(pct(18));
        add_network(NetUid::ROOT, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            AlphaBalance::from(1_000_000_000),
        );

        let (total_before, alpha_before, tao_before) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&hotkey, netuid);
        assert_eq!(alpha_before, 0);
        assert!(tao_before > 0);

        // Double the subnet's effective TAO weight: TAO counts twice as much in
        // consensus on this subnet.
        SubnetTaoWeight::<Test>::insert(netuid, pct(36));
        let (total_after, _, tao_after) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&hotkey, netuid);
        assert_eq!(tao_after, tao_before);
        assert_abs_diff_eq!(
            total_after.to_num::<f64>(),
            total_before.to_num::<f64>() * 2.0,
            epsilon = 1.0
        );
    });
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

#[test]
fn test_migration_seeds_from_legacy_owner_cut_state() {
    new_test_ext(1).execute_with(|| {
        let enabled = NetUid::from(1);
        let disabled = NetUid::from(2);
        let leased = NetUid::from(3);
        for netuid in [enabled, disabled, leased] {
            add_network(netuid, 1, 0);
            // Wipe whatever registration seeded so the migration is observable, and
            // leave a stale per-subnet TAO weight behind.
            SubnetEmissionWeights::<Test>::remove(netuid);
            SubnetTaoWeight::<Test>::insert(netuid, pct(99));
        }
        add_network(NetUid::ROOT, 1, 0);
        SubtensorModule::set_tao_weight(pct(18));
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(1_000_000 * TAO));
        SubnetAlphaOut::<Test>::insert(enabled, AlphaBalance::from(1_000_000_u64));
        SubnetAlphaOut::<Test>::insert(disabled, AlphaBalance::from(2_000_000_u64));
        SubnetOwnerCut::<Test>::put(11_796u16);
        OwnerCutEnabled::<Test>::insert(disabled, false);
        SubnetUidToLeaseId::<Test>::insert(leased, 0u32);
        assert!(!HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()));

        let weight = migrate_seed_subnet_emission_weights::<Test>();
        assert!(weight.ref_time() > 0);
        assert!(HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()));

        let owner = SubtensorModule::scale_u16_fraction_to_emission_weight(11_796);
        let on = SubtensorModule::get_subnet_emission_weights(enabled);
        assert_eq!(
            on,
            [
                0,
                0,
                owner,
                (ONE - owner) / 2,
                ONE - owner - (ONE - owner) / 2
            ]
        );
        assert_eq!(sum(&on), ONE as u128);

        let off = SubtensorModule::get_subnet_emission_weights(disabled);
        assert_eq!(off, [0, 0, 0, ONE / 2, ONE - ONE / 2]);

        // Parity with main: every subnet follows the global TaoWeight, so the
        // consensus-side root proportion is the pool-side one.
        for netuid in [enabled, disabled, leased] {
            assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
            assert_eq!(
                SubtensorModule::get_subnet_tao_weight_parts(netuid),
                pct(18)
            );
            assert_eq!(
                SubtensorModule::consensus_root_proportion(netuid),
                SubtensorModule::root_proportion(netuid)
            );
            assert!(SubtensorModule::root_proportion(netuid) > U96F32::saturating_from_num(0));
        }
        assert!(SubtensorModule::get_subnet_created_via_lease(leased));
        assert!(!SubtensorModule::get_subnet_created_via_lease(enabled));
        assert!(!SubnetEmissionWeights::<Test>::contains_key(NetUid::ROOT));

        // The accrued owner cut is seeded to the legacy refund deduction
        // (issuance × global cut), independent of OwnerCutEnabled.
        for netuid in [enabled, disabled, leased] {
            let issuance = SubtensorModule::get_alpha_issuance(netuid).to_u64();
            let expected = (issuance as u128 * 11_796 / u16::MAX as u128) as u64;
            assert_eq!(
                SubnetOwnerCutAccrued::<Test>::get(netuid).to_u64(),
                expected,
                "netuid {netuid:?}"
            );
        }

        // A re-run is a stamped no-op.
        SubtensorModule::set_subnet_emission_weights_unchecked(
            enabled,
            [0, 0, 0, ONE / 2, ONE / 2 + 1],
        );
        let rerun = migrate_seed_subnet_emission_weights::<Test>();
        assert_eq!(
            rerun,
            <Test as frame_system::Config>::DbWeight::get().reads(1)
        );
        assert_eq!(
            SubtensorModule::get_subnet_emission_weights(enabled),
            [0, 0, 0, ONE / 2, ONE / 2 + 1]
        );
    });
}

// ---------------------------------------------------------------------------
// Owner-cut history and netuid reuse
// ---------------------------------------------------------------------------

#[test]
fn test_coinbase_accrues_owner_cut_history() {
    new_test_ext(1).execute_with(|| {
        let (_, owner, _, _, _) = emit_once(Some(pct(40)));
        assert!(owner > 0);
        let netuid = NetUid::from(1);
        assert_eq!(SubnetOwnerCutAccrued::<Test>::get(netuid).to_u64(), owner);

        // Draining the pending cut at epoch does not touch the history.
        PendingOwnerCut::<Test>::insert(netuid, AlphaBalance::ZERO);
        assert_eq!(SubnetOwnerCutAccrued::<Test>::get(netuid).to_u64(), owner);
    });
}

#[test]
fn test_dissolve_refund_deducts_accrued_owner_cut_not_current_share() {
    new_test_ext(1).execute_with(|| {
        let owner_cold = U256::from(70);
        let owner_hot = U256::from(71);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        remove_owner_registration_stake(netuid);

        // Legacy subnet: eligible for the lock refund.
        let reg_at = NetworkRegisteredAt::<Test>::get(netuid);
        NetworkRegistrationStartBlock::<Test>::put(reg_at.saturating_add(1));
        let lock = TaoBalance::from(2_000_000_u64);
        SubtensorModule::set_subnet_locked_balance(netuid, lock);
        // A small pot (well below the refund) so no pot leftover is folded into the
        // owner's payout, with a 1:1 reserve ratio so α prices at ~1 τ.
        SubnetTAO::<Test>::insert(netuid, TaoBalance::from(1_000_u64));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaBalance::from(1_000_u64));

        // The owner has already been paid 500_000 α of owner cut ...
        let accrued = AlphaBalance::from(500_000_u64);
        SubnetOwnerCutAccrued::<Test>::insert(netuid, accrued);
        // ... and now zeroes the owner slot right before dissolving.
        let (uids, values) = full_vector(0, pct(50));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert_eq!(SubtensorModule::get_subnet_owner_emission_cut(netuid), 0);

        let price = U96F32::saturating_from_num(
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into()),
        );
        let deduction: u64 = U96F32::from_num(accrued.to_u64())
            .saturating_mul(price)
            .floor()
            .saturating_to_num::<u64>();
        assert!(deduction > 1_000, "test needs a deduction above the pot");
        let expected_refund = lock.to_u64() - deduction;

        let before = SubtensorModule::get_coldkey_balance(&owner_cold);
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        run_block_idle();
        let after = SubtensorModule::get_coldkey_balance(&owner_cold);
        let gain = after.to_u64() - before.to_u64();

        assert!(
            gain < lock.to_u64(),
            "zeroing the owner slot must not restore the full lock: gain {gain}"
        );
        assert_abs_diff_eq!(gain as f64, expected_refund as f64, epsilon = 2.0);
    });
}

/// A legacy (refund-eligible) subnet with a 2_000_000 τ lock, 2_000_000 α issued
/// and a tiny 1:1 pot so α prices at ~1 τ and no pot leftover reaches the owner.
fn refundable_legacy_subnet(owner_hot: U256, owner_cold: U256) -> (NetUid, TaoBalance) {
    let netuid = add_dynamic_network(&owner_hot, &owner_cold);
    remove_owner_registration_stake(netuid);
    let reg_at = NetworkRegisteredAt::<Test>::get(netuid);
    NetworkRegistrationStartBlock::<Test>::put(reg_at.saturating_add(1));
    let lock = TaoBalance::from(2_000_000_u64);
    SubtensorModule::set_subnet_locked_balance(netuid, lock);
    SubnetTAO::<Test>::insert(netuid, TaoBalance::from(1_000_u64));
    SubnetAlphaIn::<Test>::insert(netuid, AlphaBalance::from(1_000_u64));
    SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(2_000_000_u64));
    SubnetOwnerCut::<Test>::put(11_796u16);
    (netuid, lock)
}

/// `issuance × 11_796 / u16::MAX`: the legacy owner-cut deduction in α.
fn legacy_deduction_alpha(netuid: NetUid) -> u64 {
    let issuance = SubtensorModule::get_alpha_issuance(netuid).to_u64() as u128;
    (issuance * 11_796 / u16::MAX as u128) as u64
}

/// `owner_alpha` priced at the subnet's current α price, floored, as the refund does.
fn priced(netuid: NetUid, owner_alpha: u64) -> u64 {
    let price = U96F32::from_num(
        <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into()),
    );
    (U96F32::from_num(owner_alpha) * price)
        .floor()
        .to_num::<u64>()
}

#[test]
fn test_owner_cut_alpha_for_lock_refund_falls_back_to_legacy_formula() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(2_000_000_u64));
        SubnetOwnerCut::<Test>::put(11_796u16);
        let legacy = SubtensorModule::legacy_owner_cut_accrued(netuid, 11_796);
        // Same U96F32 arithmetic as the pre-upgrade refund: a few parts of
        // fixed-point error against the exact integer at 1.8e10 α.
        assert_abs_diff_eq!(
            legacy.to_u64() as f64,
            legacy_deduction_alpha(netuid) as f64,
            epsilon = 16.0
        );

        // Registration wrote an explicit zero: a seeded zero is honoured, not replaced.
        assert!(SubnetOwnerCutAccrued::<Test>::contains_key(netuid));
        assert_eq!(
            SubtensorModule::owner_cut_alpha_for_lock_refund(netuid),
            AlphaBalance::ZERO
        );
        // A real history is used as is.
        SubnetOwnerCutAccrued::<Test>::insert(netuid, AlphaBalance::from(123_u64));
        assert_eq!(
            SubtensorModule::owner_cut_alpha_for_lock_refund(netuid).to_u64(),
            123
        );
        // No entry at all (pre-migration subnet that was missed): legacy formula, never 0.
        SubnetOwnerCutAccrued::<Test>::remove(netuid);
        assert_eq!(
            SubtensorModule::owner_cut_alpha_for_lock_refund(netuid),
            legacy
        );
        assert!(legacy.to_u64() > 0);
    });
}

#[test]
fn test_dissolve_refund_without_accrued_entry_still_deducts_legacy_owner_cut() {
    new_test_ext(1).execute_with(|| {
        let owner_cold = U256::from(80);
        let owner_hot = U256::from(81);
        let (netuid, lock) = refundable_legacy_subnet(owner_hot, owner_cold);
        // Simulate a subnet the seed migration never touched.
        SubnetOwnerCutAccrued::<Test>::remove(netuid);

        let deduction = priced(netuid, legacy_deduction_alpha(netuid));
        assert!(deduction > 1_000, "test needs a deduction above the pot");

        let before = SubtensorModule::get_coldkey_balance(&owner_cold);
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        run_block_idle();
        let gain = SubtensorModule::get_coldkey_balance(&owner_cold).to_u64() - before.to_u64();

        assert!(
            gain < lock.to_u64(),
            "full lock must never be refunded: {gain}"
        );
        assert_abs_diff_eq!(
            gain as f64,
            (lock.to_u64() - deduction) as f64,
            epsilon = 2.0
        );
    });
}

#[test]
fn test_migration_seeds_accrued_owner_cut_for_subnets_mid_dissolution() {
    new_test_ext(1).execute_with(|| {
        let live = NetUid::from(1);
        let queued = NetUid::from(2);
        let current = NetUid::from(3);
        for netuid in [live, queued, current] {
            add_network(netuid, 1, 0);
            SubnetAlphaOut::<Test>::insert(netuid, AlphaBalance::from(1_000_000_u64));
        }
        add_network(NetUid::ROOT, 1, 0);
        SubnetOwnerCut::<Test>::put(11_796u16);
        // Pre-upgrade state: nothing seeded, two subnets already dissolving.
        for netuid in [queued, current] {
            NetworksAdded::<Test>::remove(netuid);
            SubnetEmissionWeights::<Test>::remove(netuid);
        }
        DissolveCleanupQueue::<Test>::put(vec![current, queued]);
        let expected = legacy_deduction_alpha(live);
        assert_eq!(legacy_deduction_alpha(queued), expected);
        assert_eq!(legacy_deduction_alpha(current), expected);

        let pending =
            crate::subnets::dissolution::DissolveCleanupPhase::AlphaInOutStakesSettleStakes;
        let paid = crate::subnets::dissolution::DissolveCleanupPhase::NetworkMapParameters;
        assert!(pending.lock_refund_pending());
        assert!(!paid.lock_refund_pending());

        for (phase, current_seeded) in [(pending, true), (paid, false)] {
            for netuid in [live, queued, current] {
                SubnetOwnerCutAccrued::<Test>::remove(netuid);
            }
            HasMigrationRun::<Test>::remove(MIGRATION_NAME.to_vec());
            let mut status = crate::subnets::dissolution::DissolveCleanupStatus::new(current);
            status.phase = phase.clone();
            CurrentDissolveCleanupStatus::<Test>::put(status);

            migrate_seed_subnet_emission_weights::<Test>();

            assert_eq!(SubnetOwnerCutAccrued::<Test>::get(live).to_u64(), expected);
            // Queued, cleanup not started: refund pending, seeded.
            assert_eq!(
                SubnetOwnerCutAccrued::<Test>::get(queued).to_u64(),
                expected
            );
            assert_eq!(
                SubnetOwnerCutAccrued::<Test>::contains_key(current),
                current_seeded,
                "{phase:?}"
            );
            if current_seeded {
                assert_eq!(
                    SubnetOwnerCutAccrued::<Test>::get(current).to_u64(),
                    expected
                );
            }
            // Dissolving subnets get nothing else: cleanup would only orphan it.
            for netuid in [queued, current] {
                assert!(!SubnetEmissionWeights::<Test>::contains_key(netuid));
                assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
            }
        }
    });
}

#[test]
fn test_subnet_dissolving_at_upgrade_gets_reduced_refund_after_migration() {
    new_test_ext(1).execute_with(|| {
        let owner_cold = U256::from(90);
        let owner_hot = U256::from(91);
        let (netuid, lock) = refundable_legacy_subnet(owner_hot, owner_cold);
        let deduction = priced(netuid, legacy_deduction_alpha(netuid));
        assert!(deduction > 1_000);

        // Dissolve first (out of NetworksAdded, refund still owed) ...
        let before = SubtensorModule::get_coldkey_balance(&owner_cold);
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        assert!(DissolveCleanupQueue::<Test>::get().contains(&netuid));
        assert!(!NetworksAdded::<Test>::contains_key(netuid));
        // ... as a pre-upgrade subnet: no accrued entry, migration not yet run.
        SubnetOwnerCutAccrued::<Test>::remove(netuid);
        HasMigrationRun::<Test>::remove(MIGRATION_NAME.to_vec());

        migrate_seed_subnet_emission_weights::<Test>();
        assert_eq!(
            SubnetOwnerCutAccrued::<Test>::get(netuid).to_u64(),
            legacy_deduction_alpha(netuid)
        );

        run_block_idle();
        let gain = SubtensorModule::get_coldkey_balance(&owner_cold).to_u64() - before.to_u64();
        assert!(
            gain < lock.to_u64(),
            "full lock must never be refunded: {gain}"
        );
        assert_abs_diff_eq!(
            gain as f64,
            (lock.to_u64() - deduction) as f64,
            epsilon = 2.0
        );
    });
}

#[test]
fn test_dissolution_clears_emission_weight_state_and_reuse_starts_clean() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        remove_owner_registration_stake(netuid);
        let (uids, values) = full_vector(pct(20), pct(30));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        SubnetCreatedViaLease::<Test>::insert(netuid, true);
        SubnetOwnerCutAccrued::<Test>::insert(netuid, AlphaBalance::from(123_u64));
        SubnetTaoWeight::<Test>::insert(netuid, pct(77));

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        run_block_idle();

        assert!(!SubnetEmissionWeights::<Test>::contains_key(netuid));
        assert!(!SubnetTaoWeight::<Test>::contains_key(netuid));
        assert!(!SubnetCreatedViaLease::<Test>::contains_key(netuid));
        assert!(!SubnetOwnerCutAccrued::<Test>::contains_key(netuid));

        // A plain registration reusing the netuid is not bound by the old lease.
        let reused = add_dynamic_network(&U256::from(5), &U256::from(6));
        assert_eq!(reused, netuid, "test expects the freed netuid to be reused");
        assert!(!SubtensorModule::get_subnet_created_via_lease(reused));
        assert_eq!(SubnetOwnerCutAccrued::<Test>::get(reused).to_u64(), 0);
        assert_eq!(
            SubtensorModule::get_subnet_emission_weights(reused),
            SubtensorModule::default_emission_weights()
        );
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight(reused),
            SubtensorModule::get_tao_weight()
        );
        // ... so it may lower its owner cut below the lease floor.
        let (uids, values) = full_vector(pct(5), pct(45));
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            reused, uids, values
        ));
    });
}

#[test]
fn test_registration_seed_clears_stale_lease_marker_even_without_cleanup() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(7);
        SubnetCreatedViaLease::<Test>::insert(netuid, true);
        SubnetOwnerCutAccrued::<Test>::insert(netuid, AlphaBalance::from(9_u64));
        SubtensorModule::seed_subnet_emission_weights(netuid);
        assert!(!SubtensorModule::get_subnet_created_via_lease(netuid));
        assert_eq!(SubnetOwnerCutAccrued::<Test>::get(netuid).to_u64(), 0);
    });
}
