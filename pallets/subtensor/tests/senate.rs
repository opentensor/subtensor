mod mock;
use mock::*;

use sp_core::{H256, U256, bounded_vec};
use frame_system::{EventRecord, Phase};
use frame_support::{assert_ok, assert_noop, codec::Encode};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	BuildStorage,
};

use frame_system::Config;
use pallet_subtensor::{Error};
use pallet_collective::{Event as CollectiveEvent};

pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();

	let mut ext: sp_io::TestExternalities = GenesisConfig {
		senate_members: pallet_membership::GenesisConfig::<Test, pallet_membership::Instance2> {
			members: bounded_vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()],
			phantom: Default::default()
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
	EventRecord { phase: Phase::Initialization, event, topics: vec![] }
}

#[test]
fn test_senate_join_works() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(7);
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, 100_000);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, 100_000));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), 100_000);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 100_000);

		assert_ok!(SubtensorModule::do_join_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);
	});
}

#[test]
fn test_senate_join_fails_stake_req() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, u16::MAX/2));

		SubtensorModule::increase_total_stake(100_000);
		assert_eq!(SubtensorModule::get_total_stake(), 100_000);

		assert_noop!(SubtensorModule::do_join_senate(
			<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
			&hotkey_account_id
		), Error::<Test>::BelowStakeThreshold);
	});
}

#[test]
fn test_senate_vote_works() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let senate_hotkey = U256::from(1);
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(7);
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, 100_000);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, 100_000));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), 100_000);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 100_000);

		assert_ok!(SubtensorModule::do_join_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);

		System::reset_events();

		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash = BlakeTwo256::hash_of(&proposal);
		assert_ok!(Triumvirate::propose(
			RuntimeOrigin::signed(senate_hotkey),
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_ok!(SubtensorModule::do_vote_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id, hash, 0, true));
		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Triumvirate(CollectiveEvent::Proposed {
					account: senate_hotkey,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 4
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
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let senate_hotkey = U256::from(1);
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash = BlakeTwo256::hash_of(&proposal);
		assert_ok!(Triumvirate::propose(
			RuntimeOrigin::signed(senate_hotkey),
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_noop!(SubtensorModule::do_vote_senate(
			<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
			&hotkey_account_id,
			hash,
			0,
			true
		), Error::<Test>::NotSenateMember);
	});
}

#[test]
fn test_senate_leave_works() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(7);
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, 100_000);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, 100_000));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), 100_000);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 100_000);

		assert_ok!(SubtensorModule::do_join_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);

		assert_ok!(SubtensorModule::do_leave_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), false);
	});
}

#[test]
fn test_senate_leave_not_member() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		assert_noop!(SubtensorModule::do_leave_senate(
			<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
			&hotkey_account_id
		), Error::<Test>::NotSenateMember);
	});
}

#[test]
fn test_senate_leave_vote_removal() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let senate_hotkey = U256::from(1);
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
		let coldkey_origin = <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id);

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(coldkey_origin.clone(), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(coldkey_origin.clone(), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(7);
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, 100_000);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, 100_000));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), 100_000);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 100_000);

		assert_ok!(SubtensorModule::do_join_senate(coldkey_origin.clone(), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);

		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash = BlakeTwo256::hash_of(&proposal);
		assert_ok!(Triumvirate::propose(
			RuntimeOrigin::signed(senate_hotkey),
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_ok!(SubtensorModule::do_vote_senate(coldkey_origin.clone(), &hotkey_account_id, hash, 0, true));
		assert_ok!(SubtensorModule::do_leave_senate(coldkey_origin, &hotkey_account_id));

		assert_eq!(Triumvirate::has_voted(hash, 0, &hotkey_account_id), Ok(false));
	});
}

#[test]
fn test_senate_leave_when_stake_removed() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let hotkey_account_id = U256::from(6);
		let burn_cost = 1000;
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har

		//add network
		SubtensorModule::set_burn( netuid, burn_cost);
		add_network(netuid, tempo, 0);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if neuron has added to the specified network(netuid)
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(7);
		let stake_amount: u64 = 100_000;
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake_amount);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, stake_amount));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), stake_amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), stake_amount);

		assert_ok!(SubtensorModule::do_join_senate(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);

		step_block(100);

		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, stake_amount));
		assert_eq!(Senate::is_member(&hotkey_account_id), false);
	});
}

#[test]
fn test_senate_replace_lowest_member() {
	new_test_ext().execute_with( || {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost = 1000;

		//add network
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids(netuid, 4096);
		SubtensorModule::set_burn( netuid, burn_cost );
		SubtensorModule::set_max_registrations_per_block(netuid, 100);

		for i in 1..11u64 {
			let coldkey_account_id = U256::from(100 + i);
			let coldkey_origin = <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id);

			let hotkey_account_id = U256::from(5 + i);

			// Give it some $$$ in his coldkey balance
			SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

			// Subscribe and check extrinsic output
			assert_ok!(SubtensorModule::burned_register(coldkey_origin.clone(), netuid,  hotkey_account_id));
			// Check if balance has  decreased to pay for the burn.
			assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
			// Check if neuron has added to the specified network(netuid)
			assert_eq!(u64::from(SubtensorModule::get_subnetwork_n(netuid)), i);
			// Check if hotkey is added to the Hotkeys
			assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

			// Lets make this new key a delegate with a 50% take.
			assert_ok!(SubtensorModule::do_become_delegate(coldkey_origin.clone(), hotkey_account_id, u16::MAX/2));

			let staker_coldkey = U256::from(200 + i);
			let staked_amount = 100_000 * i;
			SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, staked_amount);

			assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, staked_amount));
			assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), staked_amount);
			assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), staked_amount);

			assert_ok!(SubtensorModule::do_join_senate(coldkey_origin, &hotkey_account_id));
			assert_eq!(Senate::is_member(&hotkey_account_id), true);

			SubtensorModule::set_burn( netuid, burn_cost );
			step_block(100);
		}

		let hotkey_account_id = U256::from(16);
		let coldkey_account_id = U256::from(667); // Neighbour of the beast, har har
		let coldkey_origin = <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id);

		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::burned_register(coldkey_origin.clone(), netuid,  hotkey_account_id));
		// Check if balance has  decreased to pay for the burn.
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) as u64, 10000 - burn_cost); // funds drained on reg.
		// Check if hotkey is added to the Hotkeys
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Lets make this new key a delegate with a 50% take.
		assert_ok!(SubtensorModule::do_become_delegate(coldkey_origin.clone(), hotkey_account_id, u16::MAX/2));

		let staker_coldkey = U256::from(1000);
		let stake_amount = 1_000_001;
		SubtensorModule::add_balance_to_coldkey_account(&staker_coldkey, stake_amount);

		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(staker_coldkey), hotkey_account_id, stake_amount));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&staker_coldkey, &hotkey_account_id), stake_amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), stake_amount);

		assert_ok!(SubtensorModule::do_join_senate(coldkey_origin.clone(), &hotkey_account_id));
		assert_eq!(Senate::is_member(&hotkey_account_id), true);

		// Lowest stake amount should get kicked out
		assert_eq!(Senate::is_member(&U256::from(6)), false);
	});
}
