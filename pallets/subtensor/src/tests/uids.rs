#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
use sp_core::{H160, U256};
use subtensor_runtime_common::{AlphaCurrency, NetUidStorageIndex};

/********************************************
    tests for uids.rs file
*********************************************/

/********************************************
    tests uids::replace_neuron()
*********************************************/

#[test]
fn test_replace_neuron() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            111111,
            &hotkey_account_id,
        );
        let coldkey_account_id = U256::from(1234);

        let new_hotkey_account_id = U256::from(2);
        let _new_colkey_account_id = U256::from(12345);
        let certificate = NeuronCertificate::try_from(vec![1, 2, 3]).unwrap();
        let evm_address = H160::from_slice(&[1_u8; 20]);
        //add network
        add_network(netuid, tempo, 0);

        // Register a neuron.
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));

        // Get UID
        let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id);
        assert_ok!(neuron_uid);
        let neuron_uid = neuron_uid.unwrap();

        // set non-default values
        Trust::<Test>::mutate(netuid, |v| {
            SubtensorModule::set_element_at(v, neuron_uid as usize, 5u16)
        });
        Emission::<Test>::mutate(netuid, |v| {
            SubtensorModule::set_element_at(v, neuron_uid as usize, 5.into())
        });
        Consensus::<Test>::mutate(netuid, |v| {
            SubtensorModule::set_element_at(v, neuron_uid as usize, 5u16)
        });
        Incentive::<Test>::mutate(NetUidStorageIndex::from(netuid), |v| {
            SubtensorModule::set_element_at(v, neuron_uid as usize, 5u16)
        });
        Dividends::<Test>::mutate(netuid, |v| {
            SubtensorModule::set_element_at(v, neuron_uid as usize, 5u16)
        });
        Bonds::<Test>::insert(NetUidStorageIndex::from(netuid), neuron_uid, vec![(0, 1)]);

        // serve axon mock address
        let ip: u128 = 1676056785;
        let port: u16 = 9999;
        let ip_type: u8 = 4;
        assert!(
            SubtensorModule::serve_axon(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                0,
                ip,
                port,
                ip_type,
                0,
                0,
                0
            )
            .is_ok()
        );

        // Set a neuron certificate for it
        NeuronCertificates::<Test>::insert(netuid, hotkey_account_id, certificate);
        AssociatedEvmAddress::<Test>::insert(netuid, neuron_uid, (evm_address, 1));
        // Replace the neuron.
        SubtensorModule::replace_neuron(netuid, neuron_uid, &new_hotkey_account_id, block_number);

        // Check old hotkey is not registered on any network.
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_err());
        assert!(!SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey_account_id
        ));

        let curr_hotkey = SubtensorModule::get_hotkey_for_net_and_uid(netuid, neuron_uid);
        assert_ok!(curr_hotkey);
        assert_ne!(curr_hotkey.unwrap(), hotkey_account_id);

        // Check new hotkey is registered on the network.
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &new_hotkey_account_id).is_ok()
        );
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &new_hotkey_account_id
        ));
        assert_eq!(curr_hotkey.unwrap(), new_hotkey_account_id);

        // Check neuron certificate was reset
        let certificate = NeuronCertificates::<Test>::get(netuid, hotkey_account_id);
        assert_eq!(certificate, None);

        // Check trust, emission, consensus, incentive, dividends have been reset to 0.
        assert_eq!(SubtensorModule::get_trust_for_uid(netuid, neuron_uid), 0);
        assert_eq!(
            SubtensorModule::get_emission_for_uid(netuid, neuron_uid),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_consensus_for_uid(netuid, neuron_uid),
            0
        );
        assert_eq!(
            SubtensorModule::get_incentive_for_uid(netuid.into(), neuron_uid),
            0
        );
        assert_eq!(
            SubtensorModule::get_dividends_for_uid(netuid, neuron_uid),
            0
        );

        // Check axon info is reset.
        let axon_info = SubtensorModule::get_axon_info(netuid, &curr_hotkey.unwrap());
        assert_eq!(axon_info.ip, 0);
        assert_eq!(axon_info.port, 0);
        assert_eq!(axon_info.ip_type, 0);

        // Check bonds are cleared.
        assert_eq!(Bonds::<Test>::get(NetUidStorageIndex::from(netuid), neuron_uid), vec![]);
        assert_eq!(AssociatedEvmAddress::<Test>::get(netuid, neuron_uid), None);
    });
}

#[test]
fn test_bonds_cleared_on_replace() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            111111,
            &hotkey_account_id,
        );
        let coldkey_account_id = U256::from(1234);

        let new_hotkey_account_id = U256::from(2);
        let _new_colkey_account_id = U256::from(12345);
        let evm_address = H160::from_slice(&[1_u8; 20]);

        //add network
        add_network(netuid, tempo, 0);

        // Register a neuron.
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));

        // Get UID
        let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id);
        assert_ok!(neuron_uid);
        let neuron_uid = neuron_uid.unwrap();
        AssociatedEvmAddress::<Test>::insert(netuid, neuron_uid, (evm_address, 1));
        // set non-default bonds
        Bonds::<Test>::insert(NetUidStorageIndex::from(netuid), neuron_uid, vec![(0, 1)]);

        // Replace the neuron.
        SubtensorModule::replace_neuron(netuid, neuron_uid, &new_hotkey_account_id, block_number);

        // Check old hotkey is not registered on any network.
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_err());
        assert!(!SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey_account_id
        ));

        let curr_hotkey = SubtensorModule::get_hotkey_for_net_and_uid(netuid, neuron_uid);
        assert_ok!(curr_hotkey);
        assert_ne!(curr_hotkey.unwrap(), hotkey_account_id);

        // Check new hotkey is registered on the network.
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &new_hotkey_account_id).is_ok()
        );
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &new_hotkey_account_id
        ));
        assert_eq!(curr_hotkey.unwrap(), new_hotkey_account_id);

        // Check bonds are cleared.
        assert_eq!(Bonds::<Test>::get(NetUidStorageIndex::from(netuid), neuron_uid), vec![]);
        assert_eq!(AssociatedEvmAddress::<Test>::get(netuid, neuron_uid), None);
    });
}

#[test]
fn test_replace_neuron_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid = NetUid::from(1);
        let netuid1 = NetUid::from(2);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(1);
        let new_hotkey_account_id = U256::from(2);

        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            111111,
            &hotkey_account_id,
        );
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid1,
            block_number,
            111111 * 5,
            &hotkey_account_id,
        );

        let coldkey_account_id = U256::from(1234);

        let _new_colkey_account_id = U256::from(12345);
        let evm_address = H160::from_slice(&[1_u8; 20]);
        //add network
        add_network(netuid, tempo, 0);
        add_network(netuid1, tempo, 0);

        // Register a neuron on both networks.
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work,
            hotkey_account_id,
            coldkey_account_id
        ));
        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid1,
            block_number,
            nonce1,
            work1,
            hotkey_account_id,
            coldkey_account_id
        ));

        // Get UID
        let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id);
        assert_ok!(neuron_uid);

        // Verify neuron is registered on both networks.
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid,
            &hotkey_account_id
        ));
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid1,
            &hotkey_account_id
        ));
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey_account_id
        ));

        AssociatedEvmAddress::<Test>::insert(netuid, neuron_uid.unwrap(), (evm_address, 1));

        // Replace the neuron.
        // Only replace on ONE network.
        SubtensorModule::replace_neuron(
            netuid,
            neuron_uid.unwrap(),
            &new_hotkey_account_id,
            block_number,
        );

        // Check old hotkey is not registered on netuid network.
        assert!(SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_err());

        // Verify still registered on netuid1 network.
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey_account_id
        ));
        assert!(SubtensorModule::is_hotkey_registered_on_network(
            netuid1,
            &hotkey_account_id
        ));

        assert_eq!(
            AssociatedEvmAddress::<Test>::get(netuid, neuron_uid.unwrap()),
            None
        );
    });
}

#[test]
fn test_neuron_certificate() {
    new_test_ext(1).execute_with(|| {
        // 512 bits key
        let mut data = [0; 65].to_vec();
        assert_ok!(NeuronCertificate::try_from(data));

        // 256 bits key
        data = [1; 33].to_vec();
        assert_ok!(NeuronCertificate::try_from(data));

        // too much data
        data = [8; 88].to_vec();
        assert_err!(NeuronCertificate::try_from(data), ());

        // no data
        data = vec![];
        assert_err!(NeuronCertificate::try_from(data), ());
    });
}

#[test]
fn test_replace_neuron_subnet_owner_not_replaced() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(100);
        let owner_coldkey = U256::from(999);
        let new_hotkey_account_id = U256::from(2);
        let evm_address = H160::from_slice(&[1_u8; 20]);

        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
            .expect("Owner neuron should be registered by add_dynamic_network");
        AssociatedEvmAddress::<Test>::insert(netuid, neuron_uid, (evm_address, 1));
        let current_block = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::replace_neuron(netuid, neuron_uid, &new_hotkey_account_id, current_block);

        let still_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey);
        assert_ok!(still_uid);
        assert_eq!(
            still_uid.unwrap(),
            neuron_uid,
            "UID should remain unchanged for subnet owner"
        );

        let new_key_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &new_hotkey_account_id);
        assert_err!(new_key_uid, Error::<Test>::HotKeyNotRegisteredInSubNet,);
        assert!(AssociatedEvmAddress::<Test>::get(netuid, neuron_uid).is_some());
    });
}

#[test]
fn test_replace_neuron_subnet_owner_not_replaced_if_in_sn_owner_hotkey_map() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(123);
        let owner_coldkey = U256::from(999);
        let other_owner_hotkey = U256::from(456);
        let evm_address = H160::from_slice(&[1_u8; 20]);

        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        SubtensorModule::set_max_registrations_per_block(netuid, 100);
        SubtensorModule::set_target_registrations_per_interval(netuid, 100);
        SubnetOwner::<Test>::insert(netuid, owner_coldkey);

        let owner_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
            .expect("Owner neuron should already be registered by add_dynamic_network");

        AssociatedEvmAddress::<Test>::insert(netuid, owner_uid, (evm_address, 1));

        // Register another hotkey for the owner
        register_ok_neuron(netuid, other_owner_hotkey, owner_coldkey, 0);
        let other_owner_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &other_owner_hotkey)
                .expect("Should be registered");

        let additional_hotkey_1 = U256::from(1000);
        let additional_hotkey_2 = U256::from(1001);

        let current_block = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::replace_neuron(netuid, owner_uid, &additional_hotkey_1, current_block);

        let still_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey);
        assert_ok!(still_uid);
        assert_eq!(
            still_uid.unwrap(),
            owner_uid,
            "Owner's first hotkey should remain registered"
        );
        assert!(AssociatedEvmAddress::<Test>::get(netuid, owner_uid).is_some());

        let new_key_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_1);
        assert_err!(new_key_uid, Error::<Test>::HotKeyNotRegisteredInSubNet,);
        AssociatedEvmAddress::<Test>::insert(netuid, other_owner_uid, (evm_address, 1));

        // Try to replace the other owner hotkey
        SubtensorModule::replace_neuron(
            netuid,
            other_owner_uid,
            &additional_hotkey_1,
            current_block,
        );
        let still_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &other_owner_hotkey);
        assert_err!(still_uid, Error::<Test>::HotKeyNotRegisteredInSubNet,); // Was replaced
        assert!(AssociatedEvmAddress::<Test>::get(netuid, other_owner_uid).is_none());

        // Re-register this hotkey
        register_ok_neuron(netuid, other_owner_hotkey, owner_coldkey, 0);
        let _other_owner_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &other_owner_hotkey)
                .expect("Should be registered");

        // Set this hotkey as the SubnetOwnerHotkey
        SubnetOwnerHotkey::<Test>::insert(netuid, other_owner_hotkey);

        SubtensorModule::replace_neuron(netuid, owner_uid, &additional_hotkey_2, current_block);

        // The owner's first hotkey should be replaceable; it's not the top-stake owner hotkey
        let still_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey);
        assert_err!(still_uid, Error::<Test>::HotKeyNotRegisteredInSubNet,);

        let new_key_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_2);
        assert_ok!(new_key_uid);
        assert!(AssociatedEvmAddress::<Test>::get(netuid, owner_uid).is_none());
    });
}

#[test]
fn test_get_neuron_to_prune_owner_not_pruned() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(123);
        let owner_coldkey = U256::from(999);

        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        SubtensorModule::set_max_registrations_per_block(netuid, 100);
        SubtensorModule::set_target_registrations_per_interval(netuid, 100);
        SubnetOwner::<Test>::insert(netuid, owner_coldkey);

        let owner_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
            .expect("Owner neuron should already be registered by add_dynamic_network");

        let additional_hotkey_1 = U256::from(1000);
        let additional_coldkey_1 = U256::from(2000);

        let additional_hotkey_2 = U256::from(1001);
        let additional_coldkey_2 = U256::from(2001);

        register_ok_neuron(netuid, additional_hotkey_1, additional_coldkey_1, 0);
        let uid_1 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_1)
            .expect("Should be registered");

        register_ok_neuron(netuid, additional_hotkey_2, additional_coldkey_2, 1);
        let uid_2 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_2)
            .expect("Should be registered");

        SubtensorModule::set_pruning_score_for_uid(netuid, owner_uid, 0);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_1, 1);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_2, 2);

        let pruned_uid = SubtensorModule::get_neuron_to_prune(netuid);

        // - The pruned UID must be `uid_1` (score=1).
        // - The owner's UID remains unpruned.
        assert_eq!(
            pruned_uid, uid_1,
            "Should prune the neuron with pruning score=1, not the owner (score=0)."
        );

        let pruned_score = SubtensorModule::get_pruning_score_for_uid(netuid, uid_1);
        assert_eq!(
            pruned_score,
            u16::MAX,
            "Pruned neuron's score should be set to u16::MAX"
        );

        let owner_score = SubtensorModule::get_pruning_score_for_uid(netuid, owner_uid);
        assert_eq!(
            owner_score, 0,
            "Owner's pruning score remains 0, indicating it was skipped"
        );
    });
}

#[test]
fn test_get_neuron_to_prune_owner_pruned_if_not_in_sn_owner_hotkey_map() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(123);
        let owner_coldkey = U256::from(999);
        let other_owner_hotkey = U256::from(456);

        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        SubtensorModule::set_max_registrations_per_block(netuid, 100);
        SubtensorModule::set_target_registrations_per_interval(netuid, 100);
        SubnetOwner::<Test>::insert(netuid, owner_coldkey);

        let owner_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
            .expect("Owner neuron should already be registered by add_dynamic_network");

        // Register another hotkey for the owner
        register_ok_neuron(netuid, other_owner_hotkey, owner_coldkey, 0);
        let other_owner_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &other_owner_hotkey)
                .expect("Should be registered");

        let additional_hotkey_1 = U256::from(1000);
        let additional_coldkey_1 = U256::from(2000);

        let additional_hotkey_2 = U256::from(1001);
        let additional_coldkey_2 = U256::from(2001);

        register_ok_neuron(netuid, additional_hotkey_1, additional_coldkey_1, 1);
        let uid_2 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_1)
            .expect("Should be registered");

        register_ok_neuron(netuid, additional_hotkey_2, additional_coldkey_2, 2);
        let uid_3 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &additional_hotkey_2)
            .expect("Should be registered");

        SubtensorModule::set_pruning_score_for_uid(netuid, owner_uid, 0);
        // Other owner key has pruning score not worse than the owner's first hotkey, but worse than the additional hotkeys
        SubtensorModule::set_pruning_score_for_uid(netuid, other_owner_uid, 1);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_2, 2);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_3, 3);

        let pruned_uid = SubtensorModule::get_neuron_to_prune(netuid);
        assert_eq!(pruned_uid, other_owner_uid, "Should prune the owner");

        // Set the owner's other hotkey as the SubnetOwnerHotkey
        SubnetOwnerHotkey::<Test>::insert(netuid, other_owner_hotkey);

        // Reset pruning scores
        SubtensorModule::set_pruning_score_for_uid(netuid, owner_uid, 0);
        SubtensorModule::set_pruning_score_for_uid(netuid, other_owner_uid, 1);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_2, 2);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid_3, 3);

        let pruned_uid = SubtensorModule::get_neuron_to_prune(netuid);

        assert_eq!(
            pruned_uid, owner_uid,
            "Should prune the owner, not the top-stake owner hotkey and not the additional hotkeys"
        );
    });
}
