#![allow(clippy::expect_used)]
use frame_support::{assert_noop, assert_ok};
use frame_system::Config;
use sp_core::U256;
use subtensor_runtime_common::NetUid;

use super::mock::*;
use crate::{SubnetOwner, utils::rate_limiting::TransactionType};

#[test]
fn ensure_subnet_owner_returns_who_and_checks_ownership() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 10, 0);

        let owner: U256 = U256::from(42);
        SubnetOwner::<Test>::insert(netuid, owner);

        // Non-owner signed should fail
        assert!(
            crate::Pallet::<Test>::ensure_subnet_owner(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(7)),
                netuid
            )
            .is_err()
        );

        // Owner signed returns who
        let who = crate::Pallet::<Test>::ensure_subnet_owner(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
        )
        .expect("owner must pass");
        assert_eq!(who, owner);
    });
}

#[test]
fn ensure_subnet_owner_or_root_distinguishes_root_and_owner() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(2);
        add_network(netuid, 10, 0);
        let owner: U256 = U256::from(9);
        SubnetOwner::<Test>::insert(netuid, owner);

        // Root path returns None
        let root = crate::Pallet::<Test>::ensure_subnet_owner_or_root(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
        )
        .expect("root allowed");
        assert!(root.is_none());

        // Owner path returns Some(owner)
        let maybe_owner = crate::Pallet::<Test>::ensure_subnet_owner_or_root(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
        )
        .expect("owner allowed");
        assert_eq!(maybe_owner, Some(owner));
    });
}

#[test]
fn ensure_admin_window_open_blocks_in_freeze_window() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(0);
        let tempo = 10;
        add_network(netuid, 10, 0);

        let freeze_window = 3;
        crate::Pallet::<Test>::set_admin_freeze_window(freeze_window);

        System::set_block_number((tempo - freeze_window).into());
        assert!(crate::Pallet::<Test>::ensure_admin_window_open(netuid).is_err());

        System::set_block_number((tempo - freeze_window - 1).into());
        assert!(crate::Pallet::<Test>::ensure_admin_window_open(netuid).is_ok());
    });
}

#[test]
fn ensure_owner_or_root_with_limits_checks_rate_limit_for_owner_only() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(3);
        add_network(netuid, 10, 0);
        let owner: U256 = U256::from(17);
        SubnetOwner::<Test>::insert(netuid, owner);

        let limits = [TransactionType::SetChildren];

        let owner_result = crate::Pallet::<Test>::ensure_sn_owner_or_root_with_limits(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            &limits,
        )
        .expect("owner should pass before any prior usage");
        assert_eq!(owner_result, Some(owner));

        let root_result = crate::Pallet::<Test>::ensure_sn_owner_or_root_with_limits(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            &limits,
        )
        .expect("root should bypass owner-only rate checks");
        assert_eq!(root_result, None);

        let now = crate::Pallet::<Test>::get_current_block_as_u64();
        TransactionType::SetChildren.set_last_block_on_subnet::<Test>(&owner, netuid, now);

        assert_noop!(
            crate::Pallet::<Test>::ensure_sn_owner_or_root_with_limits(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                &limits
            ),
            crate::Error::<Test>::TxRateLimitExceeded
        );

        let limit = TransactionType::SetChildren.rate_limit_on_subnet::<Test>(netuid);
        run_to_block(now + limit + 1);
        assert_ok!(crate::Pallet::<Test>::ensure_sn_owner_or_root_with_limits(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            &limits
        ));
    });
}
