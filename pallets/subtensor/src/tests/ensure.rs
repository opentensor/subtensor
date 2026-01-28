#![allow(clippy::expect_used)]
use frame_system::Config;
use sp_core::U256;
use subtensor_runtime_common::NetUid;

use super::mock::*;
use crate::SubnetOwner;

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
