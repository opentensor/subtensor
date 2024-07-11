use crate::mock::*;
use frame_support::{assert_err, assert_ok};
mod mock;
use pallet_subtensor::*;
use sp_core::U256;
use substrate_fixed::types::I96F32;

////////////////////////////////////////////////////////////////////////////////////////////////
// Test cases
// ----    1. Setting single child
// done       - Simple happy path
// done       - Subnet doesn't exist - fails
// done       - Child is the same key as hotkey - fails
//            - Child is already set - fails
// done       - Coldkey is not owner of hotkey - fails
// done       - Set child on root subnet - fails
// done       - Old single child is removed
// done       - Hotkey is removed from parents of old single child - case with one parent
//            - Hotkey is removed from parents of old single child - case with multiple parents
//            - Old multiple children are removed
// done       - New children list of hotkey contains child + proportion
// done       - New parent list of child contains hotkey + proportion
//            - New parent list of child contains old parents (hotkeys and proportions) - case with multiple parents
//            - Set too many children (to DOS the epoch) - fails
// done       - Edge case: Set child with 0 proportion
// done       - Edge case: Set child with MAX proportion
// ----    2. get_stake_with_children_and_parents
//            - Single child inherits 100% stake of single parent, parent's stake is 0% 
//            - Single child inherits 50% stake of single parent, parent's stake is 50%
//            - Single child inherits 100% stake of two parents
//            - Two children inherit 30/70% stake of single parent
//            - Two children inherit 30/70% / 40/60% stake of two parents
//            - Child's stake does not depend on grand-parent's stake
// done       - Child with own stake, parent with own stake, 50% proportion
// ----    3. Setting multiple children
// done       - Set empty vector of children
//            - Set too many children (to DOS the epoch) - fails
// done       - Subnet doesn't exist - fails
// done       - One child, and it is the same key as hotkey - fails
//            - Multiple children, and one is the same key as hotkey (not first) - fails
// done       - Coldkey is not owner of hotkey - fails
// done       - Set children on root subnet - fails
// done       - Edge cases (min and max proportion)
// done       - Sum of proportions is 100% - ok
// done       - Sum of proportions is not 100% (greater or lower) - fails
// done       - Duplicate children in one transaction fails
//            - One of multiple children (not first) is already a child - fails
// done       - Old children list is cleaned
// done       - Hotkey is removed from old children's parent lists
//            - Set multiple children, some of them with 0 proportion
// ----    4. Set the same child for two different parents
//            - Single child: Parent list is correct
//            - Single child: Child lists are correct 
//            - Single child: Removing from one parent updates lists correctly
//            - Multiple children: Parent list is correct
//            - Multiple children: Child lists are correct 
//            - Multiple children: Removing from one parent updates lists correctly
// ----    5. Remove single child
// done       - Simple happy path
// done       - Subnet doesn't exist
// done       - Coldkey doesn't own hotkey
// done       - Key being removed is not a child
//            - Hotkey is removed from child's parent list - multiple parents case
//            - Child key is removed from parent's child list - multiple children case
// ----    6. Remove multiple children
// done       - Subnet doesn't exist
// done       - Coldkey doesn't own hotkey
// done       - Simple happy path: Parent list is correct, Child list is correct 
//            - Removing duplicate keys in one transaction - ok
// done       - Can remove different sets of chidren in multiple transactions
// done       - Key to remove is not a child - ok
//            - One of multiple keys to remove (not first) is not a child - ok 
//            - Removing keys that have already been removed - ok, noop
// done       - Hotkey is removed from children's parent lists - multiple parents case (test_do_revoke_children_multiple_complex_scenario)
// done       - Children keys are removed from parent's child list - multiple children case (test_do_revoke_children_multiple_complex_scenario)
// ----    7. Epoch function - tests based on Emission values after epoch function executes
//            - Simple happy path
//            - Edge case: There's difference in how epoch works for following children sets:
//                - Set 1 = { proportion1 = 1u64, proportion2 = u64::MAX - 1 } 
//                - Set 2 = { proportion1 = 2u64, proportion2 = u64::MAX - 2 }
//            - Edge case: There's difference in how epoch works for following children sets:
//                - Set 1 = { proportion1 = u64::MAX/2, proportion2 = u64::MAX/2 } 
//                - Set 2 = { proportion1 = u64::MAX/2-1, proportion2 = u64::MAX/2+1 } 
//            - Set multiple chidren, remove some, check that stake weight is as expected in epoch
//            - Set and then remove all chidren - epoch works
//            - Set the same child for two different parents
//            - Set same child for two parents, remove from one parent, epoch still working correctly
//            - Neuron hotkey sets non-neuron child with 100% proportion => parent stake is 0
// ----    8. Neuron registration
//            - Neuron with zero own stake and higher total stake (including stake from parent) 
//              has higher pruning score
//            - Neuron with own stake and higher child proportion has lower pruning score
// ----    9. Coldkey swap
//            - New coldkey can remove hotkey's children
// ----    10. Hotkey swap
//            - Swaps hotkey children, updates children's parents lists - multiple children with multiple parents case
// ----    11. Detecting internal loops (not sure this is needed)
//            - A --> B --> A
//            - A --> B --> C --> A
//            - A --> B --> C --> D --> A
//            - A --> B --> C --> D --> B
//            - 1000 key long loop
//            - A --> B --> A: Function get_stake_with_children_and_parents returns (TBD: error or OK)
//

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
fn test_get_stake_with_children_and_parents() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);

        add_network(netuid, 0, 0);

        let max_stake: u64 = 3000;
        SubtensorModule::set_network_max_stake(netuid, max_stake);

        SubtensorModule::create_account_if_non_existent(&coldkey0, &hotkey0);
        SubtensorModule::create_account_if_non_existent(&coldkey1, &hotkey1);

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey1, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey0, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey1, &hotkey1, 1000);

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 2000);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2000);

        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            2000
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid),
            2000
        );

        // Set child relationship
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            hotkey1,
            netuid,
            u64::MAX
        ));

        // Check stakes after setting child
        let stake0 = SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid);
        let stake1 = SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid);

        assert_eq!(stake0, 0);
        assert_eq!(stake1, max_stake);

        // Change child relationship to 50%
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            hotkey1,
            netuid,
            u64::MAX / 2
        ));

        // Check stakes after changing child relationship
        let stake0 = SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid);
        let stake1 = SubtensorModule::get_stake_with_children_and_parents(&hotkey1, netuid);

        assert_eq!(stake0, 1001);
        assert!(stake1 >= max_stake - 1 && stake1 <= max_stake);
    });
}

#[test]
fn test_get_stake_one_child() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey0 = U256::from(1);
        let child = U256::from(2);
        let coldkey0 = U256::from(3);
        let stake: u64 = 1_000_000_000_000; // 1000 TAO 

        add_network(netuid, 0, 0);

        let max_stake= stake;
        SubtensorModule::set_network_max_stake(netuid, max_stake);

        SubtensorModule::create_account_if_non_existent(&coldkey0, &hotkey0);

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0, &hotkey0, stake);

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), stake);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&child), 0);

        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid),
            stake
        );
        assert_eq!(
            SubtensorModule::get_stake_with_children_and_parents(&child, netuid),
            0
        );

        // Set child relationship
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            child,
            netuid,
            u64::MAX
        ));

        // Check stakes after setting child
        let stake_parent = SubtensorModule::get_stake_with_children_and_parents(&hotkey0, netuid);
        let stake_child = SubtensorModule::get_stake_with_children_and_parents(&child, netuid);

        assert_eq!(stake_parent, 0);
        assert_eq!(stake_child, stake);
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

#[test]
fn test_do_set_children_multiple_network_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let netuid: u16 = 999; // Non-existent network
        let proportion: u64 = 1000;

        // Attempt to set children
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child1, proportion)],
                netuid
            ),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn test_do_set_children_multiple_invalid_child() {
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
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(hotkey, proportion)], // Invalid child
                netuid
            ),
            Error::<Test>::InvalidChild
        );
    });
}

#[test]
fn test_do_set_children_multiple_non_associated_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        // Add network and register hotkey with a different coldkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, U256::from(999), 0);

        // Attempt to set children
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child, proportion)],
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_do_set_children_multiple_root_network() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = SubtensorModule::get_root_netuid(); // Root network
        let proportion: u64 = 1000;

        // Add network and register hotkey
        add_network(netuid, 13, 0);

        // Attempt to set children
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child, proportion)],
                netuid
            ),
            Error::<Test>::RegistrationNotPermittedOnRootSubnet
        );
    });
}

#[test]
fn test_do_set_children_multiple_proportion_edge_cases() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set children with minimum and maximum proportions
        let min_proportion: u64 = 0;
        let max_proportion: u64 = u64::MAX;
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, min_proportion), (child2, max_proportion)],
            netuid
        ));

        // Verify children assignment
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(
            children,
            vec![(min_proportion, child1), (max_proportion, child2)]
        );
    });
}

#[test]
fn test_do_set_children_multiple_duplicate_child() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let netuid: u16 = 1;
        let proportion: u64 = 1000;

        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to set children
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child1, proportion), (child1, proportion)],
                netuid
            ),
            Error::<Test>::DuplicateChild
        );
    });
}

#[test]
fn test_do_revoke_children_multiple_network_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let netuid: u16 = 999; // Non-existent network

        // Attempt to revoke children
        assert_err!(
            SubtensorModule::do_revoke_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![child1],
                netuid
            ),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn test_do_revoke_children_multiple_non_associated_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;

        // Add network and register hotkey with a different coldkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, U256::from(999), 0);

        // Attempt to revoke children
        assert_err!(
            SubtensorModule::do_revoke_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![child],
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_do_revoke_children_multiple_empty_list() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to revoke with an empty list
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![],
            netuid
        ));

        // Verify no changes in children
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());
    });
}

#[test]
fn test_do_revoke_children_multiple_complex_scenario() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let child3 = U256::from(5);
        let netuid: u16 = 1;
        let proportion1: u64 = u64::MAX / 3;
        let proportion2: u64 = u64::MAX / 3;
        let proportion3: u64 = u64::MAX - proportion1 - proportion2; // Ensure sum is u64::MAX

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set multiple children
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![
                (child1, proportion1),
                (child2, proportion2),
                (child3, proportion3)
            ],
            netuid
        ));

        // Revoke child2
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![child2],
            netuid
        ));

        // Verify remaining children
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion1, child1), (proportion3, child3)]);

        // Verify parent removal for child2
        let parents2 = SubtensorModule::get_parents(&child2, netuid);
        assert!(parents2.is_empty());

        // Revoke remaining children
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![child1, child3],
            netuid
        ));

        // Verify all children are removed
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());

        // Verify parent removal for all children
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert!(parents1.is_empty());
        let parents3 = SubtensorModule::get_parents(&child3, netuid);
        assert!(parents3.is_empty());
    });
}

#[test]
fn test_do_revoke_children_multiple_non_existent_children() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let non_existent_child = U256::from(999);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set one child
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, u64::MAX)],
            netuid
        ));

        // Attempt to revoke existing and non-existent children
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![child1, non_existent_child],
            netuid
        ));

        // Verify all children are removed
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());

        // Verify parent removal for the existing child
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert!(parents1.is_empty());
    });
}

#[test]
fn test_do_revoke_children_multiple_partial_revocation() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let child3 = U256::from(5);
        let netuid: u16 = 1;
        let proportion1: u64 = u64::MAX / 3;
        let proportion2: u64 = u64::MAX / 3;
        let proportion3: u64 = u64::MAX - proportion1 - proportion2; // Ensure sum is u64::MAX

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set multiple children
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![
                (child1, proportion1),
                (child2, proportion2),
                (child3, proportion3)
            ],
            netuid
        ));

        // Revoke only two children
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![child1, child2],
            netuid
        ));

        // Verify children removal
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(proportion3, child3)]);

        // Verify parent removal for revoked children
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert!(parents1.is_empty());
        let parents2 = SubtensorModule::get_parents(&child2, netuid);
        assert!(parents2.is_empty());

        // Verify parent remains for non-revoked child
        let parents3 = SubtensorModule::get_parents(&child3, netuid);
        assert_eq!(parents3, vec![(proportion3, hotkey)]);
    });
}

#[test]
fn test_do_revoke_children_multiple_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;
        let proportion1: u64 = u64::MAX / 2;
        let proportion2: u64 = u64::MAX - proportion1; // Ensure sum is u64::MAX

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set multiple children
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, proportion1), (child2, proportion2)],
            netuid
        ));

        // Revoke all children
        assert_ok!(SubtensorModule::do_revoke_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![child1, child2],
            netuid
        ));

        // Verify all children are removed
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());

        // Verify parent removal for all children
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert!(parents1.is_empty());
        let parents2 = SubtensorModule::get_parents(&child2, netuid);
        assert!(parents2.is_empty());
    });
}

#[test]
fn test_do_set_children_multiple_empty_list() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to set empty children list
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![],
                netuid
            ),
            Error::<Test>::ProportionSumIncorrect
        );

        // Verify no children are set
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert!(children.is_empty());
    });
}

#[test]
fn test_do_set_children_multiple_old_children_cleanup() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let old_child = U256::from(3);
        let new_child = U256::from(4);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set initial child
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(old_child, u64::MAX)],
            netuid
        ));

        // Set new child, replacing old one
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(new_child, u64::MAX)],
            netuid
        ));

        // Verify new child is set and old child is removed
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(u64::MAX, new_child)]);

        // Verify old child's parent is removed
        let old_child_parents = SubtensorModule::get_parents(&old_child, netuid);
        assert!(old_child_parents.is_empty());

        // Verify new child's parent is set
        let new_child_parents = SubtensorModule::get_parents(&new_child, netuid);
        assert_eq!(new_child_parents, vec![(u64::MAX, hotkey)]);
    });
}

#[test]
fn test_do_set_children_multiple_overwrite_existing() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;
        let initial_proportion: u64 = u64::MAX;
        let new_proportion1: u64 = u64::MAX / 2;
        let new_proportion2: u64 = u64::MAX - new_proportion1; // Ensure sum is u64::MAX

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set initial child
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, initial_proportion)],
            netuid
        ));

        // Overwrite existing child and add new one
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, new_proportion1), (child2, new_proportion2)],
            netuid
        ));

        // Verify new children are set with updated proportions
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(
            children,
            vec![(new_proportion1, child1), (new_proportion2, child2)]
        );

        // Verify parents are updated for both children
        let child1_parents = SubtensorModule::get_parents(&child1, netuid);
        assert_eq!(child1_parents, vec![(new_proportion1, hotkey)]);
        let child2_parents = SubtensorModule::get_parents(&child2, netuid);
        assert_eq!(child2_parents, vec![(new_proportion2, hotkey)]);
    });
}

#[test]
fn test_do_set_children_multiple_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let child3 = U256::from(5);
        let netuid: u16 = 1;
        let proportion1: u64 = u64::MAX / 3;
        let proportion2: u64 = u64::MAX / 3;
        let proportion3: u64 = u64::MAX - proportion1 - proportion2; // Ensure sum is u64::MAX

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set multiple children
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![
                (child1, proportion1),
                (child2, proportion2),
                (child3, proportion3)
            ],
            netuid
        ));

        // Verify children are set correctly
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(
            children,
            vec![
                (proportion1, child1),
                (proportion2, child2),
                (proportion3, child3)
            ]
        );

        // Verify parents are set correctly for all children
        let parents1 = SubtensorModule::get_parents(&child1, netuid);
        assert_eq!(parents1, vec![(proportion1, hotkey)]);
        let parents2 = SubtensorModule::get_parents(&child2, netuid);
        assert_eq!(parents2, vec![(proportion2, hotkey)]);
        let parents3 = SubtensorModule::get_parents(&child3, netuid);
        assert_eq!(parents3, vec![(proportion3, hotkey)]);
    });
}

#[test]
fn test_get_network_max_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let default_max_stake = SubtensorModule::get_network_max_stake(netuid);

        // Check that the default value is set correctly
        assert_eq!(default_max_stake, 500_000_000_000_000);

        // Set a new max stake value
        let new_max_stake: u64 = 1_000_000;
        SubtensorModule::set_network_max_stake(netuid, new_max_stake);

        // Check that the new value is retrieved correctly
        assert_eq!(
            SubtensorModule::get_network_max_stake(netuid),
            new_max_stake
        );
    });
}

#[test]
fn test_set_network_max_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let initial_max_stake = SubtensorModule::get_network_max_stake(netuid);

        // Set a new max stake value
        let new_max_stake: u64 = 500_000;
        SubtensorModule::set_network_max_stake(netuid, new_max_stake);

        // Check that the new value is set correctly
        assert_eq!(
            SubtensorModule::get_network_max_stake(netuid),
            new_max_stake
        );
        assert_ne!(
            SubtensorModule::get_network_max_stake(netuid),
            initial_max_stake
        );

        // Check that the event is emitted
        System::assert_last_event(Event::NetworkMaxStakeSet(netuid, new_max_stake).into());
    });
}

#[test]
fn test_set_network_max_stake_multiple_networks() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;

        // Set different max stake values for two networks
        let max_stake1: u64 = 1_000_000;
        let max_stake2: u64 = 2_000_000;
        SubtensorModule::set_network_max_stake(netuid1, max_stake1);
        SubtensorModule::set_network_max_stake(netuid2, max_stake2);

        // Check that the values are set correctly for each network
        assert_eq!(SubtensorModule::get_network_max_stake(netuid1), max_stake1);
        assert_eq!(SubtensorModule::get_network_max_stake(netuid2), max_stake2);
        assert_ne!(
            SubtensorModule::get_network_max_stake(netuid1),
            SubtensorModule::get_network_max_stake(netuid2)
        );
    });
}

#[test]
fn test_set_network_max_stake_update() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        // Set an initial max stake value
        let initial_max_stake: u64 = 1_000_000;
        SubtensorModule::set_network_max_stake(netuid, initial_max_stake);

        // Update the max stake value
        let updated_max_stake: u64 = 1_500_000;
        SubtensorModule::set_network_max_stake(netuid, updated_max_stake);

        // Check that the value is updated correctly
        assert_eq!(
            SubtensorModule::get_network_max_stake(netuid),
            updated_max_stake
        );
        assert_ne!(
            SubtensorModule::get_network_max_stake(netuid),
            initial_max_stake
        );

        // Check that the event is emitted for the update
        System::assert_last_event(Event::NetworkMaxStakeSet(netuid, updated_max_stake).into());
    });
}

#[test]
fn test_do_set_children_multiple_proportions_sum_to_one() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let child3 = U256::from(5);
        let netuid: u16 = 1;

        // Proportions that sum to u64::MAX (representing 1)
        let proportion1: u64 = u64::MAX / 3;
        let proportion2: u64 = u64::MAX / 3;
        let proportion3: u64 = u64::MAX - proportion1 - proportion2;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set multiple children with proportions summing to 1
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![
                (child1, proportion1),
                (child2, proportion2),
                (child3, proportion3)
            ],
            netuid
        ));

        // Verify children are set correctly
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(
            children,
            vec![
                (proportion1, child1),
                (proportion2, child2),
                (proportion3, child3)
            ]
        );

        // Verify the sum of proportions is exactly u64::MAX
        let sum: u64 = children.iter().map(|(prop, _)| prop).sum();
        assert_eq!(sum, u64::MAX);
    });
}

#[test]
fn test_do_set_children_multiple_proportions_sum_less_than_one() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;

        // Proportions that sum to less than u64::MAX
        let proportion1: u64 = u64::MAX / 3;
        let proportion2: u64 = u64::MAX / 3;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to set children with proportions summing to less than 1
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child1, proportion1), (child2, proportion2)],
                netuid
            ),
            Error::<Test>::ProportionSumIncorrect
        );
    });
}

#[test]
fn test_do_set_children_multiple_proportions_sum_greater_than_one() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let netuid: u16 = 1;

        // Proportions that sum to more than u64::MAX
        let proportion1: u64 = u64::MAX / 2 + 1;
        let proportion2: u64 = u64::MAX / 2 + 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Attempt to set children with proportions summing to more than 1
        assert_err!(
            SubtensorModule::do_set_children_multiple(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                vec![(child1, proportion1), (child2, proportion2)],
                netuid
            ),
            Error::<Test>::ProportionSumIncorrect
        );
    });
}

#[test]
fn test_do_set_children_multiple_single_child_full_proportion() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let netuid: u16 = 1;

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Set a single child with full proportion (1)
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child, u64::MAX)],
            netuid
        ));

        // Verify child is set correctly
        let children = SubtensorModule::get_children(&hotkey, netuid);
        assert_eq!(children, vec![(u64::MAX, child)]);
    });
}

//////////////////////////////////////////////////////////////////////////////////////////////////////
// Childkey-epoch tests

const COLDKEY: u16 = 1;
const PARENT1: u16 = 2;
// const PARENT2: u16 = 3;
const CHILD1: u16 = 4;
const CHILD2: u16 = 5;
const NETUID: u16 = 1;
const BLOCK_EMISSION: u64 = 1_000_000_000;
// const BLOCK_EMISSION: u64 = u64::MAX;

fn helper_epoch_setup(neurons: Vec::<u16>) {
    let coldkey = U256::from(COLDKEY);
    let netuid: u16 = NETUID;

    // Add network and register hotkey
    add_network(netuid, 13, 0);
    neurons.iter().for_each(|&uid| {
        register_ok_neuron(netuid, U256::from(uid), coldkey, 0);
    });
}

fn helper_set_children(parent: u16, children: &Vec::<u16>, proportions: &Vec::<u64>) {
    assert!(children.len() == proportions.len());
    let coldkey = U256::from(COLDKEY);
    let hotkey = U256::from(parent);

    // Create cold-hot account for parent
    SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);

    if children.len() == 1 {
        let child = U256::from(children[0]);
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            NETUID,
            proportions[0]
        ));
    } else {
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            children.iter().zip(proportions.iter()).map(|(&c, &p)| (U256::from(c), p)).collect(),
            NETUID
        ));
    }
}

fn helper_add_stake(hotkey: u16, stake: u64) {
    let coldkey = U256::from(COLDKEY);
    let hotkey = U256::from(hotkey);
    SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake);
}

fn helper_calculate_expected_emissions(proportions: &Vec::<u64>) -> Vec::<u64> {
    proportions.iter().map(|&p| {
        let pf = I96F32::from_num(p);
        let bef = I96F32::from_num(BLOCK_EMISSION);
        let one = I96F32::from_num(u64::MAX);
        let ratio = pf / one;
        (bef * ratio).to_num::<u64>()
    })
    .collect()
}

fn helper_verify_epoch(proportions: &Vec::<u64>) {
    let expected_emissions = helper_calculate_expected_emissions(proportions);
    let emissions = SubtensorModule::get_emission(NETUID);
    assert!(emissions.len() == expected_emissions.len());

    println!("{:?}", expected_emissions);
    println!("{:?}", emissions);
    
    expected_emissions.iter().zip(emissions.iter())
        .for_each(|(expected, actual)| {
            assert_eq!(expected, actual);
        })
}

#[test]
fn test_child_epoch_simple() {
    new_test_ext(1).execute_with(|| {
        let proportions = vec![1000000000000, u64::MAX - 1000000000000];

        helper_epoch_setup(vec![CHILD1, CHILD2]);
        helper_set_children(PARENT1, &vec![CHILD1, CHILD2], &proportions);
        helper_add_stake(PARENT1, 1000000000);

        SubtensorModule::epoch(NETUID, BLOCK_EMISSION);

        helper_verify_epoch(&proportions);
    });
}
