#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use frame_support::{assert_ok, weights::Weight};
use sp_core::U256;
use subtensor_runtime_common::TaoBalance;
use subtensor_swap_interface::SwapHandler;

fn setup_staked_subnet() -> (U256, U256, NetUid) {
    let owner_cold = U256::from(1001);
    let owner_hot = U256::from(1002);
    let netuid = add_dynamic_network(&owner_hot, &owner_cold);

    let stake_tao: u64 = 1000;
    setup_reserves(
        netuid,
        (stake_tao * 1_000_000).into(),
        (stake_tao * 10_000_000).into(),
    );
    let amount: TaoBalance = stake_tao.into();
    assert_ok!(SubtensorModule::create_account_if_non_existent(
        &owner_cold,
        &owner_hot
    ));
    add_balance_to_coldkey_account(&owner_cold, amount);
    assert_ok!(SubtensorModule::stake_into_subnet(
        &owner_hot,
        &owner_cold,
        netuid,
        amount,
        <Test as Config>::SwapInterface::max_price(),
        false,
    ));

    (owner_cold, owner_hot, netuid)
}

#[test]
fn test_destroy_alpha_in_out_stakes_get_total_alpha_value() {
    new_test_ext(0).execute_with(|| {
        let (_, _, netuid) = setup_staked_subnet();
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        assert!(
            run_resumable_netuid_cleanup_with_status(
                netuid,
                &mut weight_meter,
                &mut dissolve_cleanup_status(netuid),
                |netuid, weight_meter, last_key, status| {
                    SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                        netuid,
                        weight_meter,
                        last_key,
                        status,
                    )
                },
            ),
            "destroy_alpha_in_out_stakes_get_total_alpha_value should complete"
        );
        let mut status = dissolve_cleanup_status(netuid);
        run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        );
        assert!(status.subnet_total_alpha_value.is_some());
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_settle_stakes() {
    new_test_ext(0).execute_with(|| {
        let (_, _, netuid) = setup_staked_subnet();
        run_destroy_alpha_get_total_and_settle(netuid);
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clean_alpha() {
    new_test_ext(0).execute_with(|| {
        let (_, owner_hot, netuid) = setup_staked_subnet();
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        let mut status = dissolve_cleanup_status(netuid);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        status.subnet_distributed_tao = Some(0);
        let mut weight_meter2 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter2,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        let mut weight_meter3 = WeightMeter::with_limit(w);
        assert!(
            run_resumable_netuid_cleanup(
                netuid,
                &mut weight_meter3,
                SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha,
            ),
            "destroy_alpha_in_out_stakes_clean_alpha should complete"
        );
        assert_eq!(
            Alpha::<Test>::iter()
                .filter(|((_, _, nu), _)| *nu == netuid)
                .count(),
            0
        );
        assert!(TotalHotkeyAlpha::<Test>::contains_key(owner_hot, netuid));
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clear_hotkey_totals() {
    new_test_ext(0).execute_with(|| {
        let (_, owner_hot, netuid) = setup_staked_subnet();
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        let mut status = dissolve_cleanup_status(netuid);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        status.subnet_distributed_tao = Some(0);
        let mut weight_meter2 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter2,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        let mut weight_meter3 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup(
            netuid,
            &mut weight_meter3,
            SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha,
        ));
        let mut weight_meter4 = WeightMeter::with_limit(w);
        assert!(
            run_resumable_netuid_cleanup(
                netuid,
                &mut weight_meter4,
                SubtensorModule::destroy_alpha_in_out_stakes_clear_hotkey_totals,
            ),
            "destroy_alpha_in_out_stakes_clear_hotkey_totals should complete"
        );
        assert!(!TotalHotkeyAlpha::<Test>::contains_key(owner_hot, netuid));
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clear_locks() {
    new_test_ext(0).execute_with(|| {
        let (owner_cold, owner_hot, netuid) = setup_staked_subnet();
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        let mut status = dissolve_cleanup_status(netuid);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        status.subnet_distributed_tao = Some(0);
        let mut weight_meter2 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter2,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        let mut weight_meter3 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup(
            netuid,
            &mut weight_meter3,
            SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha,
        ));
        let mut weight_meter4 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup(
            netuid,
            &mut weight_meter4,
            SubtensorModule::destroy_alpha_in_out_stakes_clear_hotkey_totals,
        ));

        Lock::<Test>::insert(
            (owner_cold, netuid, owner_hot),
            crate::staking::lock::LockState {
                locked_mass: 10u64.into(),
                conviction: substrate_fixed::types::U64F64::from_num(1.5),
                last_update: 1,
            },
        );

        let mut weight_meter5 = WeightMeter::with_limit(w);
        assert!(
            run_resumable_netuid_cleanup(
                netuid,
                &mut weight_meter5,
                SubtensorModule::destroy_alpha_in_out_stakes_clear_locks,
            ),
            "destroy_alpha_in_out_stakes_clear_locks should complete"
        );
        assert!(!Lock::<Test>::contains_key((owner_cold, netuid, owner_hot)));
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes() {
    new_test_ext(0).execute_with(|| {
        let (_, _, netuid) = setup_staked_subnet();
        let mut status = run_destroy_alpha_get_total_and_settle(netuid);
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        assert!(
            SubtensorModule::destroy_alpha_in_out_stakes(netuid, &mut weight_meter, &mut status),
            "destroy_alpha_in_out_stakes should complete"
        );
    });
}

#[test]
fn test_destroy_alpha_clean_alpha_resumes_with_limited_weight() {
    new_test_ext(0).execute_with(|| {
        let (_, _, netuid) = setup_staked_subnet();
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = WeightMeter::with_limit(w);
        let mut status = dissolve_cleanup_status(netuid);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));
        status.subnet_distributed_tao = Some(0);
        let mut weight_meter2 = WeightMeter::with_limit(w);
        assert!(run_resumable_netuid_cleanup_with_status(
            netuid,
            &mut weight_meter2,
            &mut status,
            |netuid, weight_meter, last_key, status| {
                SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(
                    netuid,
                    weight_meter,
                    last_key,
                    status,
                )
            },
        ));

        let read_weight = <Test as frame_system::Config>::DbWeight::get().reads(1);
        let mut weight_meter3 = WeightMeter::with_limit(read_weight);
        let (done, mut last_key) = SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha(
            netuid,
            &mut weight_meter3,
            None,
        );
        assert!(!done);

        let mut iterations = 0;
        while Alpha::<Test>::iter().any(|((_, _, nu), _)| nu == netuid) {
            let mut weight_meter = WeightMeter::with_limit(Weight::from_parts(u64::MAX, u64::MAX));
            let (done, new_key) = SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha(
                netuid,
                &mut weight_meter,
                last_key,
            );
            last_key = new_key;
            assert!(
                done,
                "clean_alpha should finish once all alpha entries are removed"
            );
            iterations += 1;
            assert!(
                iterations < 10,
                "clean_alpha should complete within a few passes"
            );
        }
    });
}
