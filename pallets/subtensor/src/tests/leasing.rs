#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]
use super::mock::*;
use crate::*;
use frame_support::{StorageDoubleMap, assert_noop, assert_ok};
use sp_core::U256;
use sp_runtime::{Percent, traits::Hash};
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::AlphaCurrency;

#[test]
fn test_register_leased_network_works() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Register the leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        assert_ok!(SubtensorModule::register_leased_network(
            RuntimeOrigin::signed(beneficiary),
            emissions_share,
            Some(end_block),
        ));

        // Ensure the lease was created
        let lease_id = 0;
        let lease = SubnetLeases::<Test>::get(lease_id).unwrap();
        assert_eq!(lease.beneficiary, beneficiary);
        assert_eq!(lease.emissions_share, emissions_share);
        assert_eq!(lease.end_block, Some(end_block));

        // Ensure the lease owns the subnet
        assert_eq!(SubnetOwner::<Test>::get(lease.netuid), lease.coldkey);

        // Ensure the subnet exists
        assert!(SubnetMechanism::<Test>::contains_key(lease.netuid));

        // Ensure the subnet uid to lease id mapping exists
        assert_eq!(
            SubnetUidToLeaseId::<Test>::get(lease.netuid),
            Some(lease_id)
        );

        // Ensure the beneficiary has been added as a proxy
        assert!(PROXIES.with_borrow(|proxies| proxies.0 == vec![(lease.coldkey, beneficiary)]));

        // Ensure the lease shares have been created for each contributor

        let contributor1_share = U64F64::from(contributions[0].1).saturating_div(U64F64::from(cap));
        assert_eq!(
            SubnetLeaseShares::<Test>::get(lease_id, contributions[0].0),
            contributor1_share
        );
        let contributor2_share = U64F64::from(contributions[1].1).saturating_div(U64F64::from(cap));
        assert_eq!(
            SubnetLeaseShares::<Test>::get(lease_id, contributions[1].0),
            contributor2_share
        );
        let shares_count = SubnetLeaseShares::<Test>::iter_prefix(lease_id).count();
        assert_eq!(shares_count, 2);

        // Ensure the beneficiary has no lease shares because computed dynamically
        assert!(!SubnetLeaseShares::<Test>::contains_key(
            lease_id,
            beneficiary
        ));

        // Ensure the lease hotkey has 0 take from staking
        assert_eq!(SubtensorModule::get_hotkey_take(&lease.hotkey), 0);

        // Ensure each contributor and beneficiary has been refunded their share of the leftover cap
        let leftover_cap = cap.saturating_sub(lease.cost);

        let expected_contributor1_refund = U64F64::from(leftover_cap)
            .saturating_mul(contributor1_share)
            .floor()
            .to_num::<u64>();
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&contributions[0].0),
            expected_contributor1_refund
        );

        let expected_contributor2_refund = U64F64::from(leftover_cap)
            .saturating_mul(contributor2_share)
            .floor()
            .to_num::<u64>();
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&contributions[1].0),
            expected_contributor2_refund
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&beneficiary),
            leftover_cap - (expected_contributor1_refund + expected_contributor2_refund)
        );

        // Ensure the event is emitted
        assert_last_event::<Test>(RuntimeEvent::SubtensorModule(Event::SubnetLeaseCreated {
            beneficiary,
            lease_id,
            netuid: lease.netuid,
            end_block: Some(end_block),
        }));
    });
}

#[test]
fn test_register_leased_network_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::none(),
                emissions_share,
                Some(end_block),
            ),
            DispatchError::BadOrigin,
        );

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::root(),
                emissions_share,
                Some(end_block),
            ),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn test_register_leased_network_fails_if_crowdloan_does_not_exists() {
    new_test_ext(1).execute_with(|| {
        let beneficiary = U256::from(1);
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);

        pallet_crowdloan::CurrentCrowdloanId::<Test>::set(Some(0));

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::signed(beneficiary),
                emissions_share,
                Some(end_block),
            ),
            pallet_crowdloan::Error::<Test>::InvalidCrowdloanId,
        );
    });
}

#[test]
fn test_register_lease_network_fails_if_current_crowdloan_id_is_not_set() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Mark as if the current crowdloan id is not set
        pallet_crowdloan::CurrentCrowdloanId::<Test>::set(None);

        let end_block = 500;
        let emissions_share = Percent::from_percent(30);

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::signed(beneficiary),
                emissions_share,
                Some(end_block),
            ),
            pallet_crowdloan::Error::<Test>::CurrentCrowdloanIdNotSet,
        );
    });
}

#[test]
fn test_register_leased_network_fails_if_origin_is_not_crowdloan_creator() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        let end_block = 500;
        let emissions_share = Percent::from_percent(30);

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::signed(U256::from(2)),
                emissions_share,
                Some(end_block),
            ),
            Error::<Test>::ExpectedBeneficiaryOrigin,
        );
    });
}

#[test]
fn test_register_lease_network_fails_if_end_block_is_in_the_past() {
    new_test_ext(501).execute_with(|| {
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        let end_block = 500;
        let emissions_share = Percent::from_percent(30);

        assert_noop!(
            SubtensorModule::register_leased_network(
                RuntimeOrigin::signed(beneficiary),
                emissions_share,
                Some(end_block),
            ),
            Error::<Test>::LeaseCannotEndInThePast,
        );
    });
}

#[test]
fn test_terminate_lease_works() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(2), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Run to the end of the lease
        run_to_block(end_block);

        // Create a hotkey for the beneficiary
        let hotkey = U256::from(3);
        SubtensorModule::create_account_if_non_existent(&beneficiary, &hotkey);

        // Terminate the lease
        assert_ok!(SubtensorModule::terminate_lease(
            RuntimeOrigin::signed(beneficiary),
            lease_id,
            hotkey,
        ));

        // Ensure the beneficiary is now the owner of the subnet
        assert_eq!(SubnetOwner::<Test>::get(lease.netuid), beneficiary);
        assert_eq!(SubnetOwnerHotkey::<Test>::get(lease.netuid), hotkey);

        // Ensure everything has been cleaned up
        assert_eq!(SubnetLeases::<Test>::get(lease_id), None);
        assert!(!SubnetLeaseShares::<Test>::contains_prefix(lease_id));
        assert!(!AccumulatedLeaseDividends::<Test>::contains_key(lease_id));

        // Ensure the beneficiary has been removed as a proxy
        assert!(PROXIES.with_borrow(|proxies| proxies.0.is_empty()));

        // Ensure the event is emitted
        assert_last_event::<Test>(RuntimeEvent::SubtensorModule(
            Event::<Test>::SubnetLeaseTerminated {
                beneficiary: lease.beneficiary,
                netuid: lease.netuid,
            },
        ));
    });
}

#[test]
fn test_terminate_lease_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        let lease_id = 0;
        let hotkey = U256::from(1);

        assert_noop!(
            SubtensorModule::terminate_lease(RuntimeOrigin::none(), lease_id, hotkey),
            DispatchError::BadOrigin,
        );

        assert_noop!(
            SubtensorModule::terminate_lease(RuntimeOrigin::root(), lease_id, hotkey),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn test_terminate_lease_fails_if_lease_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let lease_id = 0;
        let beneficiary = U256::from(1);
        let hotkey = U256::from(2);

        assert_noop!(
            SubtensorModule::terminate_lease(RuntimeOrigin::signed(beneficiary), lease_id, hotkey),
            Error::<Test>::LeaseDoesNotExist,
        );
    });
}

#[test]
fn test_terminate_lease_fails_if_origin_is_not_beneficiary() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(2), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, _lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Run to the end of the lease
        run_to_block(end_block);

        // Create a hotkey for the beneficiary
        let hotkey = U256::from(3);
        SubtensorModule::create_account_if_non_existent(&beneficiary, &hotkey);

        // Terminate the lease
        assert_noop!(
            SubtensorModule::terminate_lease(
                RuntimeOrigin::signed(U256::from(42)),
                lease_id,
                hotkey,
            ),
            Error::<Test>::ExpectedBeneficiaryOrigin,
        );
    });
}

#[test]
fn test_terminate_lease_fails_if_lease_has_no_end_block() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(2), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) =
            setup_leased_network!(beneficiary, emissions_share, None, Some(tao_to_stake));

        // Create a hotkey for the beneficiary
        let hotkey = U256::from(3);
        SubtensorModule::create_account_if_non_existent(&beneficiary, &hotkey);

        // Terminate the lease
        assert_noop!(
            SubtensorModule::terminate_lease(
                RuntimeOrigin::signed(lease.beneficiary),
                lease_id,
                hotkey,
            ),
            Error::<Test>::LeaseHasNoEndBlock,
        );
    });
}

#[test]
fn test_terminate_lease_fails_if_lease_has_not_ended() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(2), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Create a hotkey for the beneficiary
        let hotkey = U256::from(3);
        SubtensorModule::create_account_if_non_existent(&beneficiary, &hotkey);

        // Terminate the lease
        assert_noop!(
            SubtensorModule::terminate_lease(
                RuntimeOrigin::signed(lease.beneficiary),
                lease_id,
                hotkey,
            ),
            Error::<Test>::LeaseHasNotEnded,
        );
    });
}

#[test]
fn test_terminate_lease_fails_if_beneficiary_does_not_own_hotkey() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(2), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Run to the end of the lease
        run_to_block(end_block);

        // Terminate the lease
        assert_noop!(
            SubtensorModule::terminate_lease(
                RuntimeOrigin::signed(lease.beneficiary),
                lease_id,
                U256::from(42),
            ),
            Error::<Test>::BeneficiaryDoesNotOwnHotkey,
        );
    });
}
#[test]
fn test_distribute_lease_network_dividends_multiple_contributors_works() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Setup the correct block to distribute dividends
        run_to_block(<Test as Config>::LeaseDividendsDistributionInterval::get() as u64);

        // Get the initial alpha for the contributors and beneficiary and ensure they are zero
        let contributor1_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        );
        assert_eq!(contributor1_alpha_before, AlphaCurrency::ZERO);
        let contributor2_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        );
        assert_eq!(contributor2_alpha_before, AlphaCurrency::ZERO);
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // Setup some previously accumulated dividends
        let accumulated_dividends = AlphaCurrency::from(10_000_000_000);
        AccumulatedLeaseDividends::<Test>::insert(lease_id, accumulated_dividends);

        // Distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were distributed correctly relative to their shares
        let distributed_alpha =
            accumulated_dividends + emissions_share.mul_ceil(owner_cut_alpha.to_u64()).into();
        assert_ne!(distributed_alpha, AlphaCurrency::ZERO);

        let contributor1_alpha_delta = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        )
        .saturating_sub(contributor1_alpha_before);
        assert_ne!(contributor1_alpha_delta, AlphaCurrency::ZERO);

        let contributor2_alpha_delta = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        )
        .saturating_sub(contributor2_alpha_before);
        assert_ne!(contributor2_alpha_delta, AlphaCurrency::ZERO);

        let beneficiary_alpha_delta = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        )
        .saturating_sub(beneficiary_alpha_before);
        assert_ne!(beneficiary_alpha_delta, AlphaCurrency::ZERO);

        // What has been distributed should be equal to the sum of all contributors received alpha
        assert_eq!(
            distributed_alpha,
            (beneficiary_alpha_delta + contributor1_alpha_delta + contributor2_alpha_delta).into()
        );

        let expected_contributor1_alpha =
            SubnetLeaseShares::<Test>::get(lease_id, contributions[0].0)
                .saturating_mul(U64F64::from(distributed_alpha.to_u64()))
                .ceil()
                .to_num::<u64>();
        assert_eq!(contributor1_alpha_delta, expected_contributor1_alpha.into());
        assert_eq!(
            System::events()[2].event,
            RuntimeEvent::SubtensorModule(Event::SubnetLeaseDividendsDistributed {
                lease_id,
                contributor: contributions[0].0.into(),
                alpha: expected_contributor1_alpha.into(),
            },)
        );

        let expected_contributor2_alpha =
            SubnetLeaseShares::<Test>::get(lease_id, contributions[1].0)
                .saturating_mul(U64F64::from(distributed_alpha.to_u64()))
                .ceil()
                .to_num::<u64>();
        assert_eq!(contributor2_alpha_delta, expected_contributor2_alpha.into());
        assert_eq!(
            System::events()[5].event,
            RuntimeEvent::SubtensorModule(Event::SubnetLeaseDividendsDistributed {
                lease_id,
                contributor: contributions[1].0.into(),
                alpha: expected_contributor2_alpha.into(),
            },)
        );

        // The beneficiary should have received the remaining dividends
        let expected_beneficiary_alpha = distributed_alpha.to_u64()
            - (expected_contributor1_alpha + expected_contributor2_alpha);
        assert_eq!(beneficiary_alpha_delta, expected_beneficiary_alpha.into());
        assert_eq!(
            System::events()[8].event,
            RuntimeEvent::SubtensorModule(Event::SubnetLeaseDividendsDistributed {
                lease_id,
                contributor: beneficiary.into(),
                alpha: expected_beneficiary_alpha.into(),
            },)
        );

        // Ensure nothing was accumulated for later distribution
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            AlphaCurrency::ZERO
        );
    });
}

#[test]
fn test_distribute_lease_network_dividends_only_beneficiary_works() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![(U256::from(1), 990_000_000_000)]; // 990 TAO
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Setup the correct block to distribute dividends
        run_to_block(<Test as Config>::LeaseDividendsDistributionInterval::get() as u64);

        // Get the initial alpha for the beneficiary and ensure it is zero
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // Setup some previously accumulated dividends
        let accumulated_dividends = AlphaCurrency::from(10_000_000_000);
        AccumulatedLeaseDividends::<Test>::insert(lease_id, accumulated_dividends);

        // Distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were distributed correctly relative to their shares
        let distributed_alpha =
            accumulated_dividends + emissions_share.mul_ceil(owner_cut_alpha.to_u64()).into();
        assert_ne!(distributed_alpha, AlphaCurrency::ZERO);
        let beneficiary_alpha_delta = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        )
        .saturating_sub(beneficiary_alpha_before);
        assert_eq!(beneficiary_alpha_delta, distributed_alpha.into());
        assert_last_event::<Test>(RuntimeEvent::SubtensorModule(
            Event::SubnetLeaseDividendsDistributed {
                lease_id,
                contributor: beneficiary.into(),
                alpha: distributed_alpha,
            },
        ));

        // Ensure nothing was accumulated for later distribution
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            AlphaCurrency::ZERO
        );
    });
}

#[test]
fn test_distribute_lease_network_dividends_accumulates_if_not_the_correct_block() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Setup incorrect block to distribute dividends
        run_to_block(<Test as Config>::LeaseDividendsDistributionInterval::get() as u64 + 1);

        // Get the initial alpha for the contributors and beneficiary and ensure they are zero
        let contributor1_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        );
        assert_eq!(contributor1_alpha_before, AlphaCurrency::ZERO);
        let contributor2_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        );
        assert_eq!(contributor2_alpha_before, AlphaCurrency::ZERO);
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // Setup some previously accumulated dividends
        let accumulated_dividends = AlphaCurrency::from(10_000_000_000);
        AccumulatedLeaseDividends::<Test>::insert(lease_id, accumulated_dividends);

        // Distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were not distributed
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[0].0,
                lease.netuid
            ),
            contributor1_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[1].0,
                lease.netuid
            ),
            contributor2_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &beneficiary,
                lease.netuid
            ),
            beneficiary_alpha_before
        );

        // Ensure we correctly accumulated the dividends
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            (accumulated_dividends + emissions_share.mul_ceil(owner_cut_alpha.to_u64()).into())
                .into()
        );
    });
}

#[test]
fn test_distribute_lease_network_dividends_does_nothing_if_lease_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let lease_id = 0;
        let owner_cut_alpha = AlphaCurrency::from(5_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);
    });
}

#[test]
fn test_distribute_lease_network_dividends_does_nothing_if_lease_has_ended() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];
        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let tao_to_stake = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            Some(tao_to_stake)
        );

        // Run to the end of the lease
        run_to_block(end_block);

        // Get the initial alpha for the contributors and beneficiary and ensure they are zero
        let contributor1_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        );
        assert_eq!(contributor1_alpha_before, AlphaCurrency::ZERO);
        let contributor2_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        );
        assert_eq!(contributor2_alpha_before, AlphaCurrency::ZERO);
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // No dividends are present, lease is new
        let accumulated_dividends_before = AccumulatedLeaseDividends::<Test>::get(lease_id);
        assert_eq!(accumulated_dividends_before, AlphaCurrency::ZERO);

        // Try to distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were not distributed
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[0].0,
                lease.netuid
            ),
            contributor1_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[1].0,
                lease.netuid
            ),
            contributor2_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &beneficiary,
                lease.netuid
            ),
            beneficiary_alpha_before
        );
        // Ensure nothing was accumulated for later distribution
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            accumulated_dividends_before
        );
    });
}

#[test]
fn test_distribute_lease_network_dividends_accumulates_if_amount_is_too_low() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];

        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            None // We don't add any liquidity
        );

        // Get the initial alpha for the contributors and beneficiary and ensure they are zero
        let contributor1_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        );
        assert_eq!(contributor1_alpha_before, AlphaCurrency::ZERO);
        let contributor2_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        );
        assert_eq!(contributor2_alpha_before, AlphaCurrency::ZERO);
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // Try to distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were not distributed
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[0].0,
                lease.netuid
            ),
            contributor1_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[1].0,
                lease.netuid
            ),
            contributor2_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &beneficiary,
                lease.netuid
            ),
            beneficiary_alpha_before
        );
        // Ensure the correct amount of alpha was accumulated for later dividends distribution
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            emissions_share.mul_ceil(owner_cut_alpha.to_u64()).into()
        );
    });
}

#[test]
fn test_distribute_lease_network_dividends_accumulates_if_insufficient_liquidity() {
    new_test_ext(1).execute_with(|| {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary = U256::from(1);
        let deposit = 10_000_000_000; // 10 TAO
        let cap = 1_000_000_000_000; // 1000 TAO
        let contributions = vec![
            (U256::from(2), 600_000_000_000), // 600 TAO
            (U256::from(3), 390_000_000_000), // 390 TAO
        ];

        setup_crowdloan!(crowdloan_id, deposit, cap, beneficiary, &contributions);

        // Setup a leased network
        let end_block = 500;
        let emissions_share = Percent::from_percent(30);
        let (lease_id, lease) = setup_leased_network!(
            beneficiary,
            emissions_share,
            Some(end_block),
            None // We don't add any liquidity
        );

        let contributor1_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[0].0,
            lease.netuid,
        );
        assert_eq!(contributor1_alpha_before, AlphaCurrency::ZERO);
        let contributor2_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &contributions[1].0,
            lease.netuid,
        );
        assert_eq!(contributor2_alpha_before, AlphaCurrency::ZERO);
        let beneficiary_alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &lease.hotkey,
            &beneficiary,
            lease.netuid,
        );
        assert_eq!(beneficiary_alpha_before, AlphaCurrency::ZERO);

        // Try to distribute the dividends
        let owner_cut_alpha = AlphaCurrency::from(5_000_000);
        SubtensorModule::distribute_leased_network_dividends(lease_id, owner_cut_alpha);

        // Ensure the dividends were not distributed
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[0].0,
                lease.netuid
            ),
            contributor1_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &contributions[1].0,
                lease.netuid
            ),
            contributor2_alpha_before
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &lease.hotkey,
                &beneficiary,
                lease.netuid
            ),
            beneficiary_alpha_before
        );
        // Ensure the correct amount of alpha was accumulated for later dividends distribution
        assert_eq!(
            AccumulatedLeaseDividends::<Test>::get(lease_id),
            emissions_share.mul_ceil(owner_cut_alpha.to_u64()).into()
        );
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_works() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        setup_crowdloan!(
            crowdloan_id,
            0,
            100_000_000,
            seller,
            vec![] as Vec<(U256, u64)>
        );

        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, seller);

        let beneficiary = U256::from(2);
        let min_sale_price = TaoCurrency::from(100_000_000);
        assert_ok!(SubtensorModule::announce_subnet_sale_into_lease(
            RuntimeOrigin::signed(seller),
            netuid,
            beneficiary,
            min_sale_price,
        ));

        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            SubnetSaleIntoLeaseAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(seller, (now, beneficiary, netuid, crowdloan_id))]
        );
        assert_last_event::<Test>(RuntimeEvent::SubtensorModule(
            Event::SubnetSaleIntoLeaseAnnounced {
                beneficiary,
                netuid,
                min_sale_price,
            },
        ));
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let beneficiary = U256::from(1);
        let min_sale_price = TaoCurrency::from(100_000_000);

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::none(),
                netuid,
                beneficiary,
                min_sale_price,
            ),
            DispatchError::BadOrigin
        );

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::root(),
                netuid,
                beneficiary,
                min_sale_price,
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_fails_if_coldkey_swap_announcement_exists() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let beneficiary_hash = <Test as frame_system::Config>::Hashing::hash_of(&beneficiary);
        let now = frame_system::Pallet::<Test>::block_number();
        let min_sale_price = TaoCurrency::from(100_000_000);
        setup_crowdloan!(
            0,
            0,
            min_sale_price.to_u64(),
            seller,
            vec![] as Vec<(U256, u64)>
        );

        ColdkeySwapAnnouncements::<Test>::insert(seller, (now, beneficiary_hash));

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::signed(seller),
                netuid,
                beneficiary,
                min_sale_price,
            ),
            Error::<Test>::ColdkeySwapAnnouncementAlreadyExists
        );
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_fails_if_subnet_sale_into_lease_announcement_exists() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let netuid = NetUid::from(1);
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();
        let min_sale_price = TaoCurrency::from(100_000_000);
        setup_crowdloan!(
            crowdloan_id,
            0,
            min_sale_price.to_u64(),
            seller,
            vec![] as Vec<(U256, u64)>
        );

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::signed(seller),
                netuid,
                beneficiary,
                min_sale_price,
            ),
            Error::<Test>::SubnetSaleIntoLeaseAnnouncementAlreadyExists
        );
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_fails_if_caller_doesnt_owns_subnet() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let min_sale_price = TaoCurrency::from(100_000_000);
        setup_crowdloan!(
            crowdloan_id,
            0,
            min_sale_price.to_u64(),
            seller,
            vec![] as Vec<(U256, u64)>
        );

        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, U256::from(42));

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::signed(seller),
                netuid,
                beneficiary,
                min_sale_price,
            ),
            Error::<Test>::NotSubnetOwner
        );
    });
}

#[test]
fn test_announce_subnet_sale_into_lease_fails_if_caller_owns_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let min_sale_price = TaoCurrency::from(100_000_000);
        setup_crowdloan!(
            crowdloan_id,
            0,
            min_sale_price.to_u64(),
            seller,
            vec![] as Vec<(U256, u64)>
        );

        let netuid1 = NetUid::from(1);
        add_network(netuid1, 1, 0);
        SubnetOwner::<Test>::insert(netuid1, seller);
        let netuid2 = NetUid::from(2);
        add_network(netuid2, 1, 0);
        SubnetOwner::<Test>::insert(netuid2, seller);

        assert_noop!(
            SubtensorModule::announce_subnet_sale_into_lease(
                RuntimeOrigin::signed(seller),
                netuid1,
                beneficiary,
                min_sale_price,
            ),
            Error::<Test>::TooManySubnetsOwned
        );
    });
}

#[test]
fn test_settle_subnet_sale_into_lease_works() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let deposit = 10_000_000_000;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();
        let min_sale_price = TaoCurrency::from(100_000_000_000);
        let contributions = vec![
            (beneficiary, 80_000_000_000u64),
            (U256::from(3), 10_000_000_000),
            (U256::from(4), 10_000_000_000),
        ];
        setup_crowdloan!(
            crowdloan_id,
            deposit,
            min_sale_price.to_u64(),
            seller,
            contributions.clone()
        );

        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, seller);

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );

        let delay = ColdkeySwapAnnouncementDelay::<Test>::get();
        run_to_block(now + delay);

        assert_ok!(SubtensorModule::settle_subnet_sale_into_lease(
            RuntimeOrigin::signed(seller),
        ));

        // Ensure the lease was created
        let lease_id = 0;
        let lease = SubnetLeases::<Test>::get(lease_id).unwrap();
        assert_eq!(lease.beneficiary, beneficiary);
        assert_eq!(lease.emissions_share, Percent::from_percent(100));
        assert_eq!(lease.end_block, None);

        // Ensure the lease owns the subnet
        assert_eq!(SubnetOwner::<Test>::get(lease.netuid), lease.coldkey);

        // Ensure the subnet uid lease id mapping exists
        assert_eq!(SubnetUidToLeaseId::<Test>::get(netuid), Some(lease_id));

        // Ensure the beneficiary has been added as a proxy
        assert!(PROXIES.with_borrow(|proxies| proxies.0 == vec![(lease.coldkey, beneficiary)]));

        // Ensure lease shares have been created for each contributor
        let contributor1_share = U64F64::from(contributions[1].1)
            .saturating_div(U64F64::from(min_sale_price.to_u64() - deposit));
        assert_eq!(
            SubnetLeaseShares::<Test>::get(lease_id, contributions[1].0),
            contributor1_share
        );
        let contributor2_share = U64F64::from(contributions[2].1)
            .saturating_div(U64F64::from(min_sale_price.to_u64() - deposit));
        assert_eq!(
            SubnetLeaseShares::<Test>::get(lease_id, contributions[2].0),
            contributor2_share
        );
        let shares_count = SubnetLeaseShares::<Test>::iter_prefix(lease_id).count();
        assert_eq!(shares_count, 2);

        // Ensure the beneficiary has no lease shares because computed dynamically
        assert!(!SubnetLeaseShares::<Test>::contains_key(
            lease_id,
            beneficiary
        ));

        // Ensure the lease hotkey has 0 take from staking
        assert_eq!(SubtensorModule::get_hotkey_take(&lease.hotkey), 0);

        // Ensure the seller has been paid the leftover sale price
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&seller),
            min_sale_price.to_u64() - lease.cost
        );

        // Ensure events are emitted
        assert_eq!(
            nth_last_event(1),
            RuntimeEvent::SubtensorModule(Event::SubnetLeaseCreated {
                beneficiary,
                lease_id,
                netuid,
                end_block: None
            })
        );
        assert_eq!(
            nth_last_event(0),
            RuntimeEvent::SubtensorModule(Event::SubnetSaleIntoLeaseSettled {
                beneficiary,
                netuid
            })
        );
    });
}

#[test]
fn test_settle_subnet_sale_into_lease_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        assert_noop!(
            SubtensorModule::settle_subnet_sale_into_lease(RuntimeOrigin::none()),
            DispatchError::BadOrigin
        );
        assert_noop!(
            SubtensorModule::settle_subnet_sale_into_lease(RuntimeOrigin::root()),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_settle_subnet_sale_into_lease_fails_if_no_announcement_exists() {
    new_test_ext(1).execute_with(|| {
        let seller = U256::from(1);

        assert_noop!(
            SubtensorModule::settle_subnet_sale_into_lease(RuntimeOrigin::signed(seller)),
            Error::<Test>::SubnetSaleIntoLeaseAnnouncementNotFound
        );
    });
}

#[test]
fn test_settle_subnet_sale_into_lease_fails_if_announcement_delay_not_passed() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let deposit = 10_000_000_000;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();
        let min_sale_price = TaoCurrency::from(100_000_000_000);
        setup_crowdloan!(
            crowdloan_id,
            deposit,
            min_sale_price.to_u64(),
            seller,
            vec![(beneficiary, 90_000_000_000u64)]
        );

        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, seller);

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );

        assert_noop!(
            SubtensorModule::settle_subnet_sale_into_lease(RuntimeOrigin::signed(seller)),
            Error::<Test>::SubnetLeaseIntoSaleSettledTooEarly
        );
    });
}

#[test]
fn test_settle_subnet_sale_into_lease_fails_if_crowdloan_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let netuid = NetUid::from(1);
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );

        let delay = ColdkeySwapAnnouncementDelay::<Test>::get();
        run_to_block(now + delay);

        assert_noop!(
            SubtensorModule::settle_subnet_sale_into_lease(RuntimeOrigin::signed(seller)),
            pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
        );
    });
}

#[test]
fn test_cancel_subnet_sale_into_lease_works() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let deposit = 10_000_000_000;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();
        let min_sale_price = TaoCurrency::from(100_000_000_000);
        setup_crowdloan!(
            crowdloan_id,
            deposit,
            min_sale_price.to_u64(),
            seller,
            vec![
                (beneficiary, 80_000_000_000u64),
                (U256::from(3), 10_000_000_000)
            ]
        );

        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, seller);

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );
        pallet_crowdloan::Crowdloans::<Test>::mutate(crowdloan_id, |crowdloan| {
            if let Some(crowdloan) = crowdloan {
                crowdloan.finalized = true;
            }
        });

        // We can't refund a finalized crowdloan
        assert_noop!(
            pallet_crowdloan::Pallet::<Test>::refund(RuntimeOrigin::signed(seller), crowdloan_id),
            pallet_crowdloan::Error::<Test>::AlreadyFinalized
        );

        assert_ok!(SubtensorModule::cancel_subnet_sale_into_lease(
            RuntimeOrigin::signed(seller),
        ));

        assert_eq!(
            SubnetSaleIntoLeaseAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![]
        );
        assert_last_event::<Test>(RuntimeEvent::SubtensorModule(
            Event::SubnetSaleIntoLeaseCancelled { netuid },
        ));

        // We can emit refunds for the crowdloan
        assert_ok!(pallet_crowdloan::Pallet::<Test>::refund(
            RuntimeOrigin::signed(seller),
            crowdloan_id
        ));
    });
}

#[test]
fn test_cancel_subnet_sale_into_lease_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        setup_crowdloan!(
            crowdloan_id,
            0,
            100_000_000,
            seller,
            vec![] as Vec<(U256, u64)>
        );

        assert_noop!(
            SubtensorModule::cancel_subnet_sale_into_lease(RuntimeOrigin::none()),
            DispatchError::BadOrigin
        );

        assert_noop!(
            SubtensorModule::cancel_subnet_sale_into_lease(RuntimeOrigin::root()),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_cancel_subnet_sale_into_lease_fails_if_no_subnet_sale_into_lease_announcement_exists() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        setup_crowdloan!(
            crowdloan_id,
            0,
            100_000_000,
            seller,
            vec![] as Vec<(U256, u64)>
        );

        assert_noop!(
            SubtensorModule::cancel_subnet_sale_into_lease(RuntimeOrigin::signed(seller)),
            Error::<Test>::SubnetSaleIntoLeaseAnnouncementNotFound
        );
    });
}

#[test]
fn test_cancel_subnet_sale_into_lease_fails_if_crowdloan_does_not_exists() {
    new_test_ext(1).execute_with(|| {
        let crowdloan_id = 0;
        let seller = U256::from(1);
        let beneficiary = U256::from(2);
        let now = frame_system::Pallet::<Test>::block_number();
        let netuid = NetUid::from(1);

        SubnetSaleIntoLeaseAnnouncements::<Test>::insert(
            seller,
            (now, beneficiary, netuid, crowdloan_id),
        );

        assert_noop!(
            SubtensorModule::cancel_subnet_sale_into_lease(RuntimeOrigin::signed(seller)),
            pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
        );
    });
}

#[macro_export]
macro_rules! setup_crowdloan {
    ($id:expr, $deposit:expr, $cap:expr, $creator:expr, $contributions:expr) => {
        let funds_account = U256::from(42424242 + $id);

        pallet_crowdloan::Crowdloans::<Test>::insert(
            $id,
            pallet_crowdloan::CrowdloanInfo {
                creator: $creator,
                deposit: $deposit,
                min_contribution: 100_000_000,
                end: 0,
                cap: $cap,
                raised: $cap,
                finalized: false,
                funds_account,
                call: None,
                target_address: None,
                contributors_count: 1 + $contributions.len() as u32,
            },
        );

        // Simulate contributions
        pallet_crowdloan::Contributions::<Test>::insert($id, $creator, $deposit);
        for (contributor, amount) in $contributions {
            pallet_crowdloan::Contributions::<Test>::insert($id, contributor, amount);
        }

        SubtensorModule::add_balance_to_coldkey_account(&funds_account, $cap);

        // Mark the crowdloan as finalizing
        pallet_crowdloan::CurrentCrowdloanId::<Test>::set(Some($id));
    };
}

#[macro_export]
macro_rules! setup_leased_network {
    ($beneficiary:expr, $emissions_share:expr, $end_block:expr, $tao_to_stake:expr) => {{
        let lease_id = 0;
        assert_ok!(SubtensorModule::do_register_leased_network(
            RuntimeOrigin::signed($beneficiary),
            $emissions_share,
            $end_block,
        ));

        // Configure subnet and add some stake
        let lease = SubnetLeases::<Test>::get(lease_id).unwrap();
        let netuid = lease.netuid;
        SubtokenEnabled::<Test>::insert(netuid, true);

        if let Some(tao_to_stake) = $tao_to_stake {
            SubtensorModule::add_balance_to_coldkey_account(&lease.coldkey, tao_to_stake);
            assert_ok!(SubtensorModule::add_stake(
                RuntimeOrigin::signed(lease.coldkey),
                lease.hotkey,
                netuid,
                tao_to_stake.into()
            ));
        }

        (lease_id, lease)
    }};
}
