use crate::mock::*;
use frame_support::{assert_err, assert_ok};
mod mock;
use pallet_subtensor::*;
use sp_core::U256;

#[test]
fn test_do_set_child_singular_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            proportion
        ));

        // Verify child assignment
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion, child)]);
    });
}

#[test]
fn test_do_set_child_singular_network_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 999; // Non-existent network
        let proportion: u64 = 1000;

        // Attempt to set child
        assert_err!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid,
                proportion
            ),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn test_do_set_child_singular_invalid_child() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to set child as the same hotkey
        assert_err!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                hotkey, // Invalid child
                netuid,
                proportion
            ),
            Error::<Test>::InvalidChild
        );
    });
}

#[test]
fn test_do_set_child_singular_non_associated_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey with a different coldkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, U256::from(999), 0);

        // Attempt to set child
        assert_err!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid,
                proportion
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_do_set_child_singular_root_network() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = SubtensorModule::get_root_netuid(); // Root network
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);

        // Attempt to set child
        assert_err!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid,
                proportion
            ),
            Error::<Test>::RegistrationNotPermittedOnRootSubnet
        );
    });
}

#[test]
fn test_do_set_child_singular_old_children_cleanup() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let old_child = U256::from(3);
        let new_child = U256::from(4);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set old child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            old_child,
            netuid,
            proportion
        ));

        // Set new child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            new_child,
            netuid,
            proportion
        ));

        // Verify old child is removed
        let old_child_parents = SubtensorModule::get_parents(&old_child, netuid);
        assert!(old_child_parents.is_empty());

        // Verify new child assignment
        let new_child_parents = SubtensorModule::get_parents(&new_child, netuid);
        assert_eq!(new_child_parents, vec![(proportion, hotkey)]);
    });
}

#[test]
fn test_do_set_child_singular_new_children_assignment() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            proportion
        ));

        // Verify child assignment
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion, child)]);

        // Verify parent assignment
        let parents = SubtensorModule::get_parents(&child, netuid);
        assert_eq!(parents, vec![(proportion, hotkey)]);
    });
}

#[test]
fn test_do_set_child_singular_proportion_edge_cases() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set child with minimum proportion
        let min_proportion: u64 = 0;
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            min_proportion
        ));

        // Verify child assignment with minimum proportion
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(min_proportion, child)]);

        // Set child with maximum proportion
        let max_proportion: u64 = u64::MAX;
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            max_proportion
        ));

        // Verify child assignment with maximum proportion
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(max_proportion, child)]);
    });
}

#[test]
fn test_do_set_child_singular_multiple_children() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;
        let proportion1: u64 = 500;
        let proportion2: u64 = 500;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set first child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child1,
            netuid,
            proportion1
        ));

        // Set second child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child2,
            netuid,
            proportion2
        ));

        // Verify children assignment
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion2, child2)]);

        // Verify parent assignment for both children
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert!(parents1.is_empty()); // Old child should be removed

        let parents2 = SubtensorModule::get_parents(&child2, netuid);
        assert_eq!(parents2, vec![(proportion2, hotkey)]);
    });
}

// To run this test specifically, use the following command:
// cargo test --test children test_add_singular_child -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_add_singular_child() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let child = U256::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        assert_eq!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid,
                u64::MAX
            ),
            Err(Error::<Test>::SubNetworkDoesNotExist.into())
        );
        add_network(netuid, 0, 0);
        assert_eq!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                0,
                u64::MAX
            ),
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
        assert_eq!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(child),
                hotkey,
                child,
                netuid,
                u64::MAX
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        assert_eq!(
            SubtensorModule::do_set_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid,
                u64::MAX
            ),
            Err(Error::<Test>::InvalidChild.into())
        );
        let child = U256::from(3);
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            u64::MAX
        ));
    })
}

// To run this test specifically, use the following command:
// cargo test --test children test_get_stake_with_children_and_parents -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_get_stake_with_children_and_parents() {
    new_test_ext(1).execute_with(|| {
        // Define network ID
        let netuid: u16 = 1;
        // Define hotkeys and coldkeys
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        // Add network with netuid
        add_network(netuid, 0, 0);
        // Create accounts if they do not exist
        SubtensorModule::create_account_if_non_existent(&coldkey0, &hotkey0);
        SubtensorModule::create_account_if_non_existent(&coldkey1, &hotkey1);
        // Increase stake on coldkey-hotkey accounts
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey1, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000);
        // Assert total stake for hotkeys
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 2000);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2000);
        // Assert stake with children and parents for hotkeys
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            2000
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid),
            2000
        );
        // Create a child relationship of 100% from hotkey0 to hotkey1
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            hotkey1,
            netuid,
            u64::MAX
        ));
        // Assert stake with children and parents after relationship
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid),
            4000
        );
        // Recreate a child relationship of 50% from hotkey0 to hotkey1
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            hotkey1,
            netuid,
            u64::MAX / 2
        ));
        // Assert stake with children and parents after 50% relationship
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            1001
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid),
            2999
        );
        // Create a new inverse child relationship of 100% from hotkey1 to hotkey0
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey1),
            hotkey1,
            hotkey0,
            netuid,
            u64::MAX
        ));
        // Assert stake with children and parents after inverse relationship
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            3001
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid),
            999
        );
    });
}

#[test]
fn test_do_revoke_child_singular_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set child
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            proportion
        ));

        // Verify child assignment
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion, child)]);

        // Revoke child
        assert_ok!(SubtensorModule::do_revoke_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid
        ));

        // Verify child removal
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());

        // Verify parent removal
        let parents = SubtensorModule::get_parents(&child, netuid);
        assert!(parents.is_empty());
    });
}

#[test]
fn test_do_revoke_child_singular_network_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 999; // Non-existent network

        // Attempt to revoke child
        assert_err!(
            SubtensorModule::do_revoke_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid
            ),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn test_do_revoke_child_singular_non_associated_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;

        // Add network and register hotkey with a different coldkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, U256::from(999), 0);

        // Attempt to revoke child
        assert_err!(
            SubtensorModule::do_revoke_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_do_revoke_child_singular_child_not_associated() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to revoke child that is not associated
        assert_err!(
            SubtensorModule::do_revoke_child_singular(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                child,
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}
