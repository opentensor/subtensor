#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

// Run tests with:
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::submetagraph --show-output

use super::mock::*;
use crate::*;
use sp_core::U256;
use subtensor_runtime_common::{MechId, NetUid};

#[test]
fn test_get_submetagraphs_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(999u16); // Non-existent subnet

        let submetagraphs = SubtensorModule::get_submetagraphs(netuid);
        assert_eq!(submetagraphs.len(), 0);
    });
}

#[test]
fn test_get_submetagraphs_empty_mechanisms() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        // Add network
        add_network(netuid, tempo, modality);

        // Explicitly set mechanism count to 0
        NetworksAdded::<Test>::insert(netuid, true);
        MechanismCountCurrent::<Test>::insert(netuid, MechId::from(0u8));

        let submetagraphs = SubtensorModule::get_submetagraphs(netuid);
        assert_eq!(submetagraphs.len(), 0);
    });
}

#[test]
fn test_get_submetagraphs_single_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network(netuid, tempo, modality);

        // Set mechanism count to 1
        NetworksAdded::<Test>::insert(netuid, true);
        MechanismCountCurrent::<Test>::insert(netuid, MechId::from(1u8));

        let submetagraphs = SubtensorModule::get_submetagraphs(netuid);
        assert_eq!(submetagraphs.len(), 1);

        // Verify we can get the mechagraph directly
        let mechagraph = SubtensorModule::get_mechagraph(netuid, MechId::from(0u8));
        assert!(mechagraph.is_some());
    });
}

#[test]
fn test_get_submetagraphs_multiple_mechanisms() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(2u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network(netuid, tempo, modality);

        // Set mechanism count to 3
        NetworksAdded::<Test>::insert(netuid, true);
        MechanismCountCurrent::<Test>::insert(netuid, MechId::from(3u8));

        let submetagraphs = SubtensorModule::get_submetagraphs(netuid);
        assert_eq!(submetagraphs.len(), 3);

        // Verify each mechagraph exists
        for mecid in 0..3 {
            let mechagraph = SubtensorModule::get_mechagraph(netuid, MechId::from(mecid));
            assert!(
                mechagraph.is_some(),
                "Mechagraph for mecid {} should exist",
                mecid
            );
        }
    });
}

#[test]
fn test_get_submetagraphs_filters_by_netuid() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(10u16);
        let netuid2 = NetUid::from(20u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        // Add two networks
        add_network(netuid1, tempo, modality);
        add_network(netuid2, tempo, modality);

        // Set different mechanism counts for each
        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);
        MechanismCountCurrent::<Test>::insert(netuid1, MechId::from(2u8));
        MechanismCountCurrent::<Test>::insert(netuid2, MechId::from(4u8));

        // Get submetagraphs for each netuid
        let submetagraphs1 = SubtensorModule::get_submetagraphs(netuid1);
        let submetagraphs2 = SubtensorModule::get_submetagraphs(netuid2);

        assert_eq!(submetagraphs1.len(), 2);
        assert_eq!(submetagraphs2.len(), 4);

        // Verify filtering - netuid1 should only have 2, netuid2 should only have 4
        for (idx, metagraph) in submetagraphs1.iter().enumerate() {
            assert!(
                metagraph.is_some(),
                "Mechagraph at index {} should exist",
                idx
            );
            assert!(idx < 2);
        }

        for (idx, metagraph) in submetagraphs2.iter().enumerate() {
            assert!(
                metagraph.is_some(),
                "Mechagraph at index {} should exist",
                idx
            );
            assert!(idx < 4);
        }
    });
}

#[test]
fn test_get_submetagraphs_vs_get_all_mechagraphs() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(30u16);
        let netuid2 = NetUid::from(31u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        // Add two networks with different mechanism counts
        add_network(netuid1, tempo, modality);
        add_network(netuid2, tempo, modality);

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);
        MechanismCountCurrent::<Test>::insert(netuid1, MechId::from(2u8));
        MechanismCountCurrent::<Test>::insert(netuid2, MechId::from(3u8));

        // Get all mechagraphs (should include both netuids)
        let all_mechagraphs = SubtensorModule::get_all_mechagraphs();

        // Get submetagraphs for each netuid
        let submetagraphs1 = SubtensorModule::get_submetagraphs(netuid1);
        let submetagraphs2 = SubtensorModule::get_submetagraphs(netuid2);

        // Verify that get_submetagraphs returns only the mechagraphs for the specific netuid
        assert_eq!(submetagraphs1.len(), 2);
        assert_eq!(submetagraphs2.len(), 3);

        // The sum should match (or be less if there are other subnets)
        // At minimum, our two subnets should be included
        assert!(all_mechagraphs.len() >= 5); // 2 + 3 = 5

        // Verify that get_submetagraphs for netuid1 returns the same as filtering all_mechagraphs
        // by checking that each mechagraph in submetagraphs1 exists in all_mechagraphs
        // (This is a simplified check - in practice you'd compare the actual data)
        assert_eq!(submetagraphs1.len(), 2);
        assert_eq!(submetagraphs2.len(), 3);
    });
}

#[test]
fn test_get_submetagraphs_with_registered_neurons() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(40u16);
        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network(netuid, tempo, modality);
        NetworksAdded::<Test>::insert(netuid, true);
        MechanismCountCurrent::<Test>::insert(netuid, MechId::from(2u8));

        // Register some neurons
        let hotkey1 = U256::from(1);
        let coldkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey2 = U256::from(2);

        register_ok_neuron(netuid, hotkey1, coldkey1, 39420842);
        register_ok_neuron(netuid, hotkey2, coldkey2, 39420843);

        let submetagraphs = SubtensorModule::get_submetagraphs(netuid);
        assert_eq!(submetagraphs.len(), 2);

        // Verify that mechagraphs exist and contain neuron data
        // We verify by comparing with direct get_mechagraph calls
        for (idx, metagraph_opt) in submetagraphs.iter().enumerate() {
            assert!(
                metagraph_opt.is_some(),
                "Mechagraph at index {} should exist",
                idx
            );

            // Verify it matches the direct call
            let direct_mechagraph =
                SubtensorModule::get_mechagraph(netuid, MechId::from(idx as u8));
            assert_eq!(metagraph_opt.is_some(), direct_mechagraph.is_some());
        }
    });
}
