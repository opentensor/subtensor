#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::rpc_info::subnet_info::{HyperparamEntry, HyperparamValue, SubnetHyperparamsV3};
use codec::{Compact, Decode, Encode};
use std::collections::BTreeSet;
use substrate_fixed::types::I32F32;
use subtensor_runtime_common::{NetUid, TaoBalance};

/// Names that must always appear in V3. Adding a new hyperparam = add its
/// name here AND in the getter. Removing one = decide whether that's a
/// breaking change for clients.
const EXPECTED_V3_NAMES: &[&[u8]] = &[
    b"rho",
    b"kappa",
    b"immunity_period",
    b"min_allowed_weights",
    b"max_weights_limit",
    b"tempo",
    b"min_difficulty",
    b"max_difficulty",
    b"weights_version",
    b"weights_rate_limit",
    b"adjustment_interval",
    b"activity_cutoff",
    b"registration_allowed",
    b"target_regs_per_interval",
    b"min_burn",
    b"max_burn",
    b"bonds_moving_avg",
    b"max_regs_per_block",
    b"serving_rate_limit",
    b"max_validators",
    b"adjustment_alpha",
    b"difficulty",
    b"commit_reveal_period",
    b"commit_reveal_weights_enabled",
    b"alpha_high",
    b"alpha_low",
    b"liquid_alpha_enabled",
    b"alpha_sigmoid_steepness",
    b"yuma_version",
    b"subnet_is_active",
    b"transfers_enabled",
    b"bonds_reset_enabled",
    b"user_liquidity_enabled",
];

fn find<'a>(params: &'a [HyperparamEntry], name: &[u8]) -> &'a HyperparamValue {
    &params
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("missing hyperparam {:?}", std::str::from_utf8(name)))
        .value
}

#[test]
fn test_get_subnet_hyperparams_v3_returns_none_for_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(7);
        assert!(SubtensorModule::get_subnet_hyperparams_v3(netuid).is_none());
    });
}

#[test]
fn test_get_subnet_hyperparams_v3_contains_all_expected_names_exactly_once() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        let result = SubtensorModule::get_subnet_hyperparams_v3(netuid)
            .expect("subnet exists, V3 should be Some");

        let returned: Vec<&[u8]> = result.params.iter().map(|e| e.name.as_slice()).collect();
        let returned_set: BTreeSet<&[u8]> = returned.iter().copied().collect();

        // No duplicates.
        assert_eq!(
            returned.len(),
            returned_set.len(),
            "duplicate hyperparam name in V3 output"
        );

        // Exact-name match with the expected set — catches accidental drops
        // and accidental additions that weren't reviewed.
        let expected_set: BTreeSet<&[u8]> = EXPECTED_V3_NAMES.iter().copied().collect();
        assert_eq!(
            returned_set, expected_set,
            "V3 hyperparam name set drifted from EXPECTED_V3_NAMES"
        );
    });
}

#[test]
fn test_get_subnet_hyperparams_v3_values_reflect_storage() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // Distinct, easy-to-spot values for each storage we touch.
        SubtensorModule::set_rho(netuid, 11);
        SubtensorModule::set_kappa(netuid, 12);
        SubtensorModule::set_immunity_period(netuid, 13);
        SubtensorModule::set_min_allowed_weights(netuid, 14);
        SubtensorModule::set_tempo(netuid, 16);
        SubtensorModule::set_min_difficulty(netuid, 17);
        SubtensorModule::set_max_difficulty(netuid, 18);
        SubtensorModule::set_weights_version_key(netuid, 19);
        SubtensorModule::set_weights_set_rate_limit(netuid, 20);
        SubtensorModule::set_adjustment_interval(netuid, 21);
        SubtensorModule::set_activity_cutoff(netuid, 22);
        SubtensorModule::set_network_registration_allowed(netuid, false);
        SubtensorModule::set_target_registrations_per_interval(netuid, 24);
        SubtensorModule::set_min_burn(netuid, TaoBalance::from(25u64));
        SubtensorModule::set_max_burn(netuid, TaoBalance::from(26u64));
        SubtensorModule::set_bonds_moving_average(netuid, 27);
        SubtensorModule::set_max_registrations_per_block(netuid, 28);
        SubtensorModule::set_serving_rate_limit(netuid, 29);
        SubtensorModule::set_max_allowed_validators(netuid, 30);
        SubtensorModule::set_adjustment_alpha(netuid, 31);
        SubtensorModule::set_difficulty(netuid, 32);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_alpha_sigmoid_steepness(netuid, 5i16);
        SubtensorModule::set_yuma3_enabled(netuid, true);
        SubtensorModule::set_bonds_reset(netuid, true);

        let result = SubtensorModule::get_subnet_hyperparams_v3(netuid).unwrap();
        let p = &result.params;

        // Bool variants
        assert_eq!(
            find(p, b"registration_allowed"),
            &HyperparamValue::Bool(false)
        );
        assert_eq!(
            find(p, b"commit_reveal_weights_enabled"),
            &HyperparamValue::Bool(true)
        );
        assert_eq!(
            find(p, b"liquid_alpha_enabled"),
            &HyperparamValue::Bool(true)
        );
        assert_eq!(
            find(p, b"bonds_reset_enabled"),
            &HyperparamValue::Bool(true)
        );

        // U16 variants
        assert_eq!(find(p, b"rho"), &HyperparamValue::U16(Compact(11)));
        assert_eq!(find(p, b"kappa"), &HyperparamValue::U16(Compact(12)));
        assert_eq!(
            find(p, b"immunity_period"),
            &HyperparamValue::U16(Compact(13))
        );
        assert_eq!(
            find(p, b"min_allowed_weights"),
            &HyperparamValue::U16(Compact(14))
        );
        assert_eq!(find(p, b"tempo"), &HyperparamValue::U16(Compact(16)));
        assert_eq!(
            find(p, b"adjustment_interval"),
            &HyperparamValue::U16(Compact(21))
        );
        assert_eq!(
            find(p, b"activity_cutoff"),
            &HyperparamValue::U16(Compact(22))
        );
        assert_eq!(
            find(p, b"target_regs_per_interval"),
            &HyperparamValue::U16(Compact(24))
        );
        assert_eq!(
            find(p, b"max_regs_per_block"),
            &HyperparamValue::U16(Compact(28))
        );
        assert_eq!(
            find(p, b"max_validators"),
            &HyperparamValue::U16(Compact(30))
        );
        assert_eq!(find(p, b"yuma_version"), &HyperparamValue::U16(Compact(3)));

        // U64 variants
        assert_eq!(
            find(p, b"min_difficulty"),
            &HyperparamValue::U64(Compact(17))
        );
        assert_eq!(
            find(p, b"max_difficulty"),
            &HyperparamValue::U64(Compact(18))
        );
        assert_eq!(
            find(p, b"weights_version"),
            &HyperparamValue::U64(Compact(19))
        );
        assert_eq!(
            find(p, b"weights_rate_limit"),
            &HyperparamValue::U64(Compact(20))
        );
        assert_eq!(
            find(p, b"bonds_moving_avg"),
            &HyperparamValue::U64(Compact(27))
        );
        assert_eq!(
            find(p, b"serving_rate_limit"),
            &HyperparamValue::U64(Compact(29))
        );
        assert_eq!(
            find(p, b"adjustment_alpha"),
            &HyperparamValue::U64(Compact(31))
        );
        assert_eq!(find(p, b"difficulty"), &HyperparamValue::U64(Compact(32)));

        // TaoBalance variants
        assert_eq!(
            find(p, b"min_burn"),
            &HyperparamValue::TaoBalance(Compact(TaoBalance::from(25u64)))
        );
        assert_eq!(
            find(p, b"max_burn"),
            &HyperparamValue::TaoBalance(Compact(TaoBalance::from(26u64)))
        );

        // I32F32 variant
        assert_eq!(
            find(p, b"alpha_sigmoid_steepness"),
            &HyperparamValue::I32F32(I32F32::saturating_from_num(5))
        );
    });
}

#[test]
fn test_get_subnet_hyperparams_v3_yuma_version_reflects_flag() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        SubtensorModule::set_yuma3_enabled(netuid, false);
        assert_eq!(
            find(
                &SubtensorModule::get_subnet_hyperparams_v3(netuid)
                    .unwrap()
                    .params,
                b"yuma_version",
            ),
            &HyperparamValue::U16(Compact(2))
        );

        SubtensorModule::set_yuma3_enabled(netuid, true);
        assert_eq!(
            find(
                &SubtensorModule::get_subnet_hyperparams_v3(netuid)
                    .unwrap()
                    .params,
                b"yuma_version",
            ),
            &HyperparamValue::U16(Compact(3))
        );
    });
}

#[test]
fn test_get_subnet_hyperparams_v3_scale_round_trip() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        let original = SubtensorModule::get_subnet_hyperparams_v3(netuid).unwrap();
        let bytes = original.encode();
        let decoded =
            SubnetHyperparamsV3::decode(&mut &bytes[..]).expect("V3 must SCALE-round-trip cleanly");
        assert_eq!(original, decoded);
    });
}

#[test]
fn test_hyperparam_value_variants_round_trip() {
    let cases = [
        HyperparamValue::Bool(true),
        HyperparamValue::Bool(false),
        HyperparamValue::U16(Compact(u16::MAX)),
        HyperparamValue::U32(Compact(u32::MAX)),
        HyperparamValue::U64(Compact(u64::MAX)),
        HyperparamValue::U128(Compact(u128::MAX)),
        HyperparamValue::TaoBalance(Compact(TaoBalance::from(123_456_789u64))),
        HyperparamValue::I32F32(I32F32::saturating_from_num(-7)),
    ];
    for original in &cases {
        let bytes = original.encode();
        let decoded = HyperparamValue::decode(&mut &bytes[..])
            .expect("HyperparamValue variant must round-trip");
        assert_eq!(original, &decoded);
    }
}
