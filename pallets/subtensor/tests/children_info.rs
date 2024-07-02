use crate::mock::*;
use codec::{Compact, Encode};
use frame_support::assert_ok;
use sp_core::U256;

mod mock;

#[test]
fn test_get_child_info() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let proportion: u64 = 500_000_000;

        // Add network and register hotkey and child
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, child, coldkey, 1);

        // Set child relationship
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            proportion
        ));

        // Add some stake to both hotkey and child
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &child, 500);
        assert_ok!(SubtensorModule::set_emission_values(&[netuid], vec![1000]));

        // Simulate network activity to generate emissions
        run_to_block(1);
        // Get child info
        let child_info = SubtensorModule::get_child_info(netuid, child.encode(), proportion);

        assert_eq!(child_info.child_ss58, child);
        assert_eq!(child_info.proportion, Compact(proportion));
        assert!(child_info.total_stake.0 > 0);
        assert_eq!(child_info.take, Compact(11796)); // 18%
    });
}

#[test]
fn test_get_children_info() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child1 = U256::from(3);
        let child2 = U256::from(4);
        let proportion1: u64 = 300_000_000;
        let proportion2: u64 = u64::MAX - proportion1;

        // Add network and register hotkey and children
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, child1, coldkey, 0);
        register_ok_neuron(netuid, child2, coldkey, 0);

        // Set child relationships
        assert_ok!(SubtensorModule::do_set_children_multiple(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            vec![(child1, proportion1), (child2, proportion2)],
            netuid
        ));

        // Add some stake
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &child1, 300);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &child2, 200);

        // Get children info
        let children_info = SubtensorModule::get_children_info(netuid);

        assert_eq!(children_info.len(), 1); // Only one parent (hotkey)
        let (parent, children) = &children_info[0];
        assert_eq!(*parent, hotkey);
        assert_eq!(children.len(), 2);

        let child1_info = &children[0];
        let child2_info = &children[1];

        assert_eq!(child1_info.child_ss58, child1);
        assert_eq!(child1_info.proportion, Compact(proportion1));
        assert!(child1_info.total_stake.0 > 0);

        assert_eq!(child2_info.child_ss58, child2);
        assert_eq!(child2_info.proportion, Compact(proportion2));
        assert!(child2_info.total_stake.0 > 0);
    });
}

#[test]
fn test_get_children_info_multiple_parents() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey = U256::from(1);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let child = U256::from(4);
        let proportion1: u64 = 300_000_000; // 30% of u64::MAX
        let proportion2: u64 = 200_000_000; // 20% of u64::MAX

        // Add network and register hotkeys and child
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey1, coldkey, 0);
        register_ok_neuron(netuid, hotkey2, coldkey, 0);
        register_ok_neuron(netuid, child, coldkey, 0);

        // Set child relationships
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey1,
            child,
            netuid,
            proportion1
        ));
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey2,
            child,
            netuid,
            proportion2
        ));

        // Add some stake
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey1, 1000);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey2, 800);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &child, 500);

        // Get children info
        let children_info = SubtensorModule::get_children_info(netuid);

        assert_eq!(children_info.len(), 2); // Two parents (hotkey1 and hotkey2)

        let (parent1, children1) = &children_info[0];
        let (parent2, children2) = &children_info[1];

        assert!(
            (*parent1 == hotkey1 && *parent2 == hotkey2)
                || (*parent1 == hotkey2 && *parent2 == hotkey1)
        );
        assert_eq!(children1.len(), 1);
        assert_eq!(children2.len(), 1);

        let child_info1 = &children1[0];
        let child_info2 = &children2[0];

        assert_eq!(child_info1.child_ss58, child);
        assert_eq!(child_info2.child_ss58, child);
        assert!(
            child_info1.proportion == Compact(proportion1)
                || child_info1.proportion == Compact(proportion2)
        );
        assert!(
            child_info2.proportion == Compact(proportion1)
                || child_info2.proportion == Compact(proportion2)
        );
        assert!(child_info1.total_stake.0 > 0);
        assert!(child_info2.total_stake.0 > 0);
    });
}

#[test]
fn test_get_children_info_no_children() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        // Add network and register hotkey
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        // Get children info
        let children_info = SubtensorModule::get_children_info(netuid);

        assert!(children_info.is_empty());
    });
}

#[test]
fn test_get_children_info_after_revoke() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let child = U256::from(3);
        let proportion: u64 = 500_000_000; // 50% of u64::MAX

        // Add network and register hotkey and child
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, child, coldkey, 0);

        // Set child relationship
        assert_ok!(SubtensorModule::do_set_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid,
            proportion
        ));

        // Get children info before revoke
        let children_info_before = SubtensorModule::get_children_info(netuid);
        assert_eq!(children_info_before.len(), 1);
        assert_eq!(children_info_before[0].1.len(), 1);

        // Revoke child
        assert_ok!(SubtensorModule::do_revoke_child_singular(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            child,
            netuid
        ));

        // Get children info after revoke
        let children_info_after = SubtensorModule::get_children_info(netuid);
        assert!(children_info_after.is_empty());
    });
}
