#![allow(clippy::unwrap_used)]

use approx::assert_abs_diff_eq;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use frame_system::Config;
use frame_system::pallet_prelude::*;
use frame_system::{EventRecord, Phase};
use pallet_subtensor_collective as pallet_collective;
use pallet_subtensor_collective::Event as CollectiveEvent;
use sp_core::{Get, H256, U256, bounded_vec};
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, Hash},
};
use subtensor_runtime_common::TaoCurrency;
use subtensor_swap_interface::SwapHandler;

use super::mock;
use super::mock::*;
use crate::Delegates;
use crate::Error;
use crate::migrations;
use crate::*;

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();

    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        senate_members: pallet_membership::GenesisConfig::<Test, pallet_membership::Instance2> {
            members: bounded_vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()],
            phantom: Default::default(),
        },
        triumvirate: pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
            members: vec![1.into()],
            phantom: Default::default(),
        },
        ..Default::default()
    }
    .build_storage()
    .unwrap()
    .into();

    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn make_proposal(value: u64) -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::remark_with_event {
        remark: value.to_be_bytes().to_vec(),
    })
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

#[test]
fn test_senate_join_works() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let stake = DefaultMinStake::<Test>::get() * 100.into();

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = 1_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Lets make this new key a delegate with a 10% take.
        Delegates::<Test>::insert(hotkey_account_id, u16::MAX / 10);

        let staker_coldkey = U256::from(7);
        SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            stake
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &staker_coldkey,
                netuid
            ),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));
        assert!(Senate::is_member(&hotkey_account_id));
    });
}

#[test]
fn test_senate_vote_works() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let senate_hotkey = U256::from(1);
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = 1_000_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Lets make this new key a delegate with a 10% take.
        Delegates::<Test>::insert(hotkey_account_id, u16::MAX / 10);

        let staker_coldkey = U256::from(7);
        let stake = DefaultMinStake::<Test>::get() * 10.into();
        SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            stake
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &staker_coldkey,
                netuid
            ),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));
        assert!(Senate::is_member(&hotkey_account_id));

        System::reset_events();

        let proposal = make_proposal(42);
        let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        let hash = BlakeTwo256::hash_of(&proposal);
        assert_ok!(Triumvirate::propose(
            RuntimeOrigin::signed(senate_hotkey),
            Box::new(proposal.clone()),
            proposal_len,
            TryInto::<BlockNumberFor<Test>>::try_into(100u64)
                .expect("convert u64 to block number.")
        ));

        assert_ok!(SubtensorModule::do_vote_root(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            &hotkey_account_id,
            hash,
            0,
            true
        ));
        assert_eq!(
            System::events(),
            vec![
                record(RuntimeEvent::Triumvirate(CollectiveEvent::Proposed {
                    account: senate_hotkey,
                    proposal_index: 0,
                    proposal_hash: hash,
                    threshold: 1
                })),
                record(RuntimeEvent::Triumvirate(CollectiveEvent::Voted {
                    account: hotkey_account_id,
                    proposal_hash: hash,
                    voted: true,
                    yes: 1,
                    no: 0
                }))
            ]
        );
    });
}

#[test]
fn test_senate_vote_not_member() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let senate_hotkey = U256::from(1);
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = 1_000_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        let proposal = make_proposal(42);
        let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        let hash = BlakeTwo256::hash_of(&proposal);
        assert_ok!(Triumvirate::propose(
            RuntimeOrigin::signed(senate_hotkey),
            Box::new(proposal.clone()),
            proposal_len,
            TryInto::<BlockNumberFor<Test>>::try_into(100u64)
                .expect("convert u64 to block number.")
        ));

        assert_noop!(
            SubtensorModule::do_vote_root(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                &hotkey_account_id,
                hash,
                0,
                true
            ),
            Error::<Test>::NotSenateMember
        );
    });
}

#[test]
fn test_senate_leave_works() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let stake = DefaultMinStake::<Test>::get() * 10.into();

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = stake.to_u64() * 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Lets make this new key a delegate with a 10% take.
        Delegates::<Test>::insert(hotkey_account_id, u16::MAX / 10);

        let staker_coldkey = U256::from(7);
        SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            stake
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &staker_coldkey,
                netuid
            ),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));
        assert!(Senate::is_member(&hotkey_account_id));
    });
}

#[test]
fn test_senate_leave_vote_removal() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let senate_hotkey = U256::from(1);
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
        let coldkey_origin = <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id);
        let stake = DefaultMinStake::<Test>::get() * 10.into();

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, stake.into());
        SubtokenEnabled::<Test>::insert(netuid, true);

        let reserve = stake.to_u64() * 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            coldkey_origin.clone(),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (stake.to_u64() - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Lets make this new key a delegate with a 10% take.
        Delegates::<Test>::insert(hotkey_account_id, u16::MAX / 10);

        let staker_coldkey = U256::from(7);
        SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            stake
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &staker_coldkey,
                netuid
            ),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            AlphaCurrency::from(stake.to_u64()),
            epsilon = 1.into()
        );

        assert_ok!(SubtensorModule::root_register(
            coldkey_origin.clone(),
            hotkey_account_id
        ));
        assert!(Senate::is_member(&hotkey_account_id));

        let proposal = make_proposal(42);
        let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        let hash = BlakeTwo256::hash_of(&proposal);
        assert_ok!(Triumvirate::propose(
            RuntimeOrigin::signed(senate_hotkey),
            Box::new(proposal.clone()),
            proposal_len,
            TryInto::<BlockNumberFor<Test>>::try_into(100u64)
                .expect("convert u64 to block number.")
        ));

        assert_ok!(SubtensorModule::do_vote_root(
            coldkey_origin.clone(),
            &hotkey_account_id,
            hash,
            0,
            true
        ));
        // Fill the root network with many large stake keys.
        // This removes all other keys.
        // Add two networks.
        let other_netuid = NetUid::from(5);
        add_network(other_netuid, 1, 0);
        SubtensorModule::set_burn(other_netuid, TaoCurrency::ZERO);
        SubtensorModule::set_max_registrations_per_block(other_netuid, 1000);
        SubtensorModule::set_target_registrations_per_interval(other_netuid, 1000);
        SubtensorModule::set_max_registrations_per_block(NetUid::ROOT, 1000);
        SubtensorModule::set_target_registrations_per_interval(NetUid::ROOT, 1000);

        let reserve = 1_000_000_000_000;
        mock::setup_reserves(other_netuid, reserve.into(), reserve.into());
        mock::setup_reserves(NetUid::ROOT, reserve.into(), reserve.into());
        SubtokenEnabled::<Test>::insert(NetUid::ROOT, true);
        SubtokenEnabled::<Test>::insert(other_netuid, true);

        for i in 0..200 {
            let hot = U256::from(i + 100);
            let cold = U256::from(i + 100);
            // Add balance
            SubtensorModule::add_balance_to_coldkey_account(&cold, 100_000_000 + (i as u64)); // lots ot stake
            // Register
            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                other_netuid,
                hot
            ));
            // Add stake on other network
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                NetUid::ROOT,
                (100_000_000 + (i as u64)).into()
            ));
            // Register them on the root network.
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
            ));
            // Check succesfull registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(other_netuid, &hot).is_ok());
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(NetUid::ROOT, &hot).is_ok());
            // Check that they are all delegates
            assert!(SubtensorModule::hotkey_is_delegate(&hot));
        }
        // No longer a root member
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(NetUid::ROOT, &hotkey_account_id).is_err()
        );
        // No longer a member of the senate
        assert!(!Senate::is_member(&hotkey_account_id));
        assert_eq!(
            // Vote is removed
            Triumvirate::has_voted(hash, 0, &hotkey_account_id),
            Ok(false)
        );
    });
}

#[test]
fn test_senate_not_leave_when_stake_removed() {
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = 1_000_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Lets make this new key a delegate with a 10% take.
        Delegates::<Test>::insert(hotkey_account_id, u16::MAX / 10);

        let staker_coldkey = U256::from(7);
        let stake_amount = DefaultMinStake::<Test>::get() * 10.into();
        SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake_amount.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            stake_amount
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &staker_coldkey,
                netuid
            ),
            AlphaCurrency::from(stake_amount.to_u64()),
            epsilon = 1.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            AlphaCurrency::from(stake_amount.to_u64()),
            epsilon = 1.into()
        );

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));
        assert!(Senate::is_member(&hotkey_account_id));

        step_block(100);

        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(staker_coldkey),
            hotkey_account_id,
            netuid,
            (stake_amount - 1.into()).to_u64().into()
        ));
        assert!(Senate::is_member(&hotkey_account_id));
    });
}

#[test]
fn test_senate_join_current_delegate() {
    // Test that a current delegate can join the senate
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667);

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give some coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        let reserve = 1_000_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (10000 - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Register in the root network
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));
        // But, remove from the senate
        assert_ok!(SenateMembers::remove_member(
            <<Test as Config>::RuntimeOrigin>::root(),
            hotkey_account_id
        ));

        // Should *NOT* be a member of the senate now
        assert!(!Senate::is_member(&hotkey_account_id));

        System::reset_events();

        // We can call now to adjust the senate
        assert_ok!(SubtensorModule::adjust_senate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        ));

        // This should make the hotkey a member of the senate
        assert!(Senate::is_member(&hotkey_account_id));

        // Check the events
        assert!(
            System::events().contains(&record(RuntimeEvent::SubtensorModule(
                SubtensorEvent::SenateAdjusted {
                    old_member: None,
                    new_member: hotkey_account_id
                }
            )))
        );
    });
}

#[test]
fn test_adjust_senate_events() {
    // Test the events emitted after adjusting the senate successfully
    new_test_ext().execute_with(|| {
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey_account_id = U256::from(6);
        let burn_cost = 1000;
        let coldkey_account_id = U256::from(667);

        let max_senate_size: u16 = SenateMaxMembers::get() as u16;
        let stake_threshold = {
            let default_stake = DefaultMinStake::<Test>::get();
            let fee =
                <Test as pallet::Config>::SwapInterface::approx_fee_amount(netuid, default_stake);
            (default_stake + fee).to_u64()
        };

        // We will be registering MaxMembers hotkeys and two more to try a replace
        let balance_to_add = DefaultMinStake::<Test>::get().to_u64() * 10
            + 50_000
            + (stake_threshold + burn_cost) * (max_senate_size + 2) as u64;

        let replacement_hotkey_account_id = U256::from(7); // Will be added to the senate to replace hotkey_account_id

        //add network
        SubtensorModule::set_burn(netuid, burn_cost.into());
        add_network(netuid, tempo, 0);
        // Give some coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, balance_to_add);

        // Allow all registrations in netuid in same block. Same for root network.
        SubtensorModule::set_max_registrations_per_block(netuid, max_senate_size + 1);
        SubtensorModule::set_target_registrations_per_interval(netuid, max_senate_size + 1);
        SubtensorModule::set_max_registrations_per_block(NetUid::ROOT, max_senate_size + 1);
        SubtensorModule::set_target_registrations_per_interval(NetUid::ROOT, max_senate_size + 1);
        SubtokenEnabled::<Test>::insert(netuid, true);
        SubtokenEnabled::<Test>::insert(NetUid::ROOT, true);

        let reserve = 100_000_000_000_000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));
        // Check if balance has  decreased to pay for the burn.
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (balance_to_add - burn_cost)
        ); // funds drained on reg.
        // Check if neuron has added to the specified network(netuid)
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        // Check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
            coldkey_account_id
        );

        // Should *NOT* be a member of the senate
        assert!(!Senate::is_member(&hotkey_account_id));

        // root register
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id
        )); // Has no stake, but is now a senate member

        // Check if they are a member of the senate
        assert!(Senate::is_member(&hotkey_account_id));

        // Register MaxMembers - 1 more hotkeys, add stake and join the senate
        for i in 0..(max_senate_size - 1) {
            let new_hotkey_account_id = U256::from(8 + i);

            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                netuid,
                new_hotkey_account_id
            ));
            // Check if this hotkey is added to the Hotkeys
            assert_eq!(
                SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkey_account_id),
                coldkey_account_id
            );
            // Add/delegate enough stake to join the senate
            // +1 to be above hotkey_account_id
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                new_hotkey_account_id,
                netuid,
                (stake_threshold + 1 + i as u64).into() // Increasing with i to make them ordered
            ));
            // Join senate
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                new_hotkey_account_id
            ));
            // Check if they are a member of the senate
            assert!(Senate::is_member(&new_hotkey_account_id));
        }

        // Verify we are at max senate size
        assert_eq!(Senate::members().len(), max_senate_size as usize);

        // Verify the replacement hotkey is not a member of the senate
        assert!(!Senate::is_member(&replacement_hotkey_account_id));

        // Register
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            replacement_hotkey_account_id
        ));

        // Register in root network
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            replacement_hotkey_account_id
        ));

        // Check if they are a member of the senate, should not be,
        // as they have no stake
        assert!(!Senate::is_member(&replacement_hotkey_account_id));
        // Add/delegate enough stake to join the senate
        let stake = DefaultMinStake::<Test>::get() * 10.into();

        let reserve = 100_000_000_000_000;
        mock::setup_reserves(NetUid::ROOT, reserve.into(), reserve.into());

        let (_, fee) = mock::swap_tao_to_alpha(NetUid::ROOT, stake);

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            replacement_hotkey_account_id,
            NetUid::ROOT,
            stake // Will be more than the last one in the senate by stake (has 0 stake)
        ));
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &replacement_hotkey_account_id,
                &coldkey_account_id,
                NetUid::ROOT
            ),
            AlphaCurrency::from(stake.to_u64() - fee),
            epsilon = (stake.to_u64() / 1000).into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(
                &replacement_hotkey_account_id,
                NetUid::ROOT
            ),
            AlphaCurrency::from(stake.to_u64() - fee),
            epsilon = (stake.to_u64() / 1000).into()
        );

        System::reset_events();

        // We can call now to adjust the senate
        assert_ok!(SubtensorModule::adjust_senate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            replacement_hotkey_account_id
        ));

        // This should make the hotkey a member of the senate
        assert!(Senate::is_member(&replacement_hotkey_account_id));

        // Check the events
        assert!(
            System::events().contains(&record(RuntimeEvent::SubtensorModule(
                SubtensorEvent::SenateAdjusted {
                    old_member: None,
                    new_member: replacement_hotkey_account_id
                }
            )))
        );
    });
}
