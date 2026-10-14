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
        assert_eq!(
            SubtensorModule::get_base_validator_emission_share(netuid),
            weights[3]
        );
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight_multiplier(netuid),
            U96F32::saturating_from_num(1)
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
fn test_set_rejects_validator_zero_even_with_zero_tao_weight() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        assert_eq!(TaoWeight::<Test>::get(), 0);
        assert_eq!(
            SubtensorModule::min_validator_emission_for_tao_weight(netuid),
            1
        );
        let (uids, values) = full_vector(0, 0);
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionBelowTaoWeightFloor
        );
    });
}

#[test]
fn test_set_rejects_validator_below_tao_weight_floor() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        // Global TAO weight 50%: V₀ = 0.5 → the floor is 0.25.
        SubtensorModule::set_tao_weight(pct(50));
        let floor = SubtensorModule::min_validator_emission_for_tao_weight(netuid);
        assert_eq!(
            floor,
            ((ONE / 2) as u128 * pct(50) as u128 / ONE as u128) as u64
        );

        let (uids, values) = full_vector(0, floor - 1);
        assert_noop!(
            SubtensorModule::do_set_subnet_emission_weights(netuid, uids, values),
            Error::<Test>::ValidatorEmissionBelowTaoWeightFloor
        );
        let (uids, values) = full_vector(0, floor);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        // At the floor the effective TAO weight is one (up to the floor's rounding).
        let effective = SubtensorModule::get_subnet_tao_weight(netuid).to_num::<f64>();
        assert!(effective <= 1.0);
        assert_abs_diff_eq!(effective, 1.0, epsilon = 1e-6);
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
// Multiplier and event
// ---------------------------------------------------------------------------

#[test]
fn test_set_updates_multiplier_and_emits_event() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(18));
        let base = SubtensorModule::get_base_validator_emission_share(netuid);
        assert_eq!(base, ONE / 2);

        // Halve the validator slot: the multiplier doubles.
        let validator = base / 2;
        let (uids, values) = full_vector(0, validator);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            uids,
            values.clone()
        ));
        let multiplier = SubtensorModule::get_subnet_tao_weight_multiplier(netuid);
        assert_abs_diff_eq!(multiplier.to_num::<f64>(), 2.0, epsilon = 1e-9);
        assert_abs_diff_eq!(
            SubtensorModule::get_subnet_tao_weight(netuid).to_num::<f64>(),
            0.36,
            epsilon = 1e-6
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::SubnetEmissionWeightsSet {
                netuid,
                weights: values,
                tao_weight_multiplier: multiplier,
                effective_tao_weight: SubtensorModule::get_subnet_tao_weight_parts(netuid),
            })
        );

        // The base share never moves, so restoring V restores the multiplier.
        assert_eq!(
            SubtensorModule::get_base_validator_emission_share(netuid),
            base
        );
        let (uids, values) = full_vector(0, base);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight_multiplier(netuid),
            U96F32::saturating_from_num(1)
        );
    });
}

#[test]
fn test_emission_weights_info_reports_scaled_multiplier() {
    new_test_ext(1).execute_with(|| {
        assert!(SubtensorModule::get_subnet_emission_weights_info(NetUid::from(42)).is_none());
        let netuid = new_subnet();
        let base = SubtensorModule::get_base_validator_emission_share(netuid);

        // Halve V: the multiplier is 2.0, which the u64::MAX weight scale cannot hold.
        let (uids, values) = full_vector(pct(10), base / 2);
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
        assert_eq!(info.owner.0, pct(10));
        assert_eq!(info.validator.0, base / 2);
        assert_eq!(info.miner.0, ONE - pct(10) - base / 2);
        assert_eq!(info.base_validator.0, base);
        assert_eq!(
            info.min_validator.0,
            SubtensorModule::min_validator_emission_for_tao_weight(netuid)
        );
        assert_eq!(
            info.tao_weight_multiplier.0,
            2 * TAO_WEIGHT_MULTIPLIER_SCALE
        );
        assert_eq!(
            info.effective_tao_weight.0,
            SubtensorModule::get_subnet_tao_weight_parts(netuid)
        );
        assert!(!info.created_via_lease);
    });
}

#[test]
fn test_multiplier_is_recomputed_even_when_validator_slot_is_omitted() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        // Lower V through a full vector first.
        let (uids, values) = full_vector(0, ONE / 4);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
        // Simulate a stale multiplier, then touch only owner/miner.
        SubnetTaoWeightMultiplier::<Test>::insert(netuid, U96F32::saturating_from_num(1));
        let miner = ONE - ONE / 4 - pct(5);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid,
            vec![OWNER, MINER],
            vec![pct(5), miner]
        ));
        assert_abs_diff_eq!(
            SubtensorModule::get_subnet_tao_weight_multiplier(netuid).to_num::<f64>(),
            2.0,
            epsilon = 1e-9
        );
    });
}

#[test]
fn test_subnet_tao_weight_saturates_at_one() {
    new_test_ext(1).execute_with(|| {
        let netuid = new_subnet();
        SubtensorModule::set_tao_weight(pct(60));
        SubnetTaoWeightMultiplier::<Test>::insert(netuid, U96F32::saturating_from_num(5));
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight(netuid),
            U96F32::saturating_from_num(1)
        );
        assert_eq!(SubtensorModule::get_subnet_tao_weight_parts(netuid), ONE);
        // Other subnets keep the global weight.
        let other = add_dynamic_network(&U256::from(3), &U256::from(4));
        assert_eq!(
            SubtensorModule::get_subnet_tao_weight(other),
            SubtensorModule::get_tao_weight()
        );
    });
}

#[test]
fn test_compute_tao_weight_multiplier_edge_cases() {
    assert_eq!(
        SubtensorModule::compute_tao_weight_multiplier(ONE / 2, ONE / 2),
        U96F32::saturating_from_num(1)
    );
    assert_eq!(
        SubtensorModule::compute_tao_weight_multiplier(ONE / 2, 0),
        U96F32::max_value()
    );
    assert_abs_diff_eq!(
        SubtensorModule::compute_tao_weight_multiplier(ONE / 2, ONE / 8).to_num::<f64>(),
        4.0,
        epsilon = 1e-9
    );
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

        // Halve V to 0.25 (owner 10%, miner 65%): the multiplier doubles the TAO
        // weight, so root alpha is unchanged to first order.
        let (alpha_b, _, server_b, validator_b, root_b) = emit_once(Some(ONE / 4));
        assert_eq!(alpha_a, alpha_b);
        // Root's share of the smaller validator slot is larger...
        let root_share_b = root_b as f64 / (validator_b + root_b) as f64;
        let root_share_a = root_a as f64 / alpha_a as f64 * 2.0;
        assert!(root_share_b > root_share_a);
        // ...and root's absolute alpha is preserved (small second-order drift from the
        // non-linear root proportion).
        let drift = (root_a as f64 - root_b as f64).abs() / root_a as f64;
        assert!(drift < 0.03, "root payout drifted by {drift}");
        // Validators absorbed the cut, miners gained.
        assert_abs_diff_eq!(server_b as f64, alpha_b as f64 * 0.65, epsilon = 200.0);
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

        // Halve the validator slot: TAO counts twice as much in consensus.
        let (uids, values) = full_vector(0, ONE / 4);
        assert_ok!(SubtensorModule::do_set_subnet_emission_weights(
            netuid, uids, values
        ));
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
            // Wipe whatever registration seeded so the migration is observable.
            SubnetEmissionWeights::<Test>::remove(netuid);
            BaseValidatorEmissionShare::<Test>::remove(netuid);
            SubnetTaoWeightMultiplier::<Test>::insert(netuid, U96F32::saturating_from_num(3));
        }
        add_network(NetUid::ROOT, 1, 0);
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
        assert_eq!(
            SubtensorModule::get_base_validator_emission_share(enabled),
            (ONE - owner) / 2
        );

        let off = SubtensorModule::get_subnet_emission_weights(disabled);
        assert_eq!(off, [0, 0, 0, ONE / 2, ONE - ONE / 2]);
        assert_eq!(
            SubtensorModule::get_base_validator_emission_share(disabled),
            ONE / 2
        );

        for netuid in [enabled, disabled, leased] {
            assert_eq!(
                SubtensorModule::get_subnet_tao_weight_multiplier(netuid),
                U96F32::saturating_from_num(1)
            );
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
        assert_ne!(
            SubtensorModule::get_subnet_tao_weight_multiplier(netuid),
            U96F32::saturating_from_num(1)
        );

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        run_block_idle();

        assert!(!SubnetEmissionWeights::<Test>::contains_key(netuid));
        assert!(!BaseValidatorEmissionShare::<Test>::contains_key(netuid));
        assert!(!SubnetTaoWeightMultiplier::<Test>::contains_key(netuid));
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
            SubtensorModule::get_subnet_tao_weight_multiplier(reused),
            U96F32::saturating_from_num(1)
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
