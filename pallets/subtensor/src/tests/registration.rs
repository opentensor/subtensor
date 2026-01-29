#![allow(clippy::unwrap_used)]

use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::dispatch::DispatchInfo;
use frame_support::sp_runtime::{DispatchError, transaction_validity::TransactionSource};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, TransactionExtension, TxBaseImplication};
use subtensor_runtime_common::{AlphaCurrency, Currency as CurrencyT, NetUid, NetUidStorageIndex};

use super::mock;
use super::mock::*;
use crate::extensions::SubtensorTransactionExtension;
use crate::{AxonInfoOf, CustomTransactionError, Error};

/********************************************
    subscribing::subscribe() tests
*********************************************/

#[test]
fn test_init_new_network_registration_defaults() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;

        SubtensorModule::init_new_network(netuid, tempo);

        assert_eq!(BurnHalfLife::<Test>::get(netuid), 360);
        assert_eq!(BurnIncreaseMult::<Test>::get(netuid), 2);

        assert_eq!(
            SubtensorModule::get_burn(netuid),
            TaoCurrency::from(RAO_PER_TAO)
        );

        assert_eq!(
            BurnLastHalvingBlock::<Test>::get(netuid),
            SubtensorModule::get_current_block_as_u64()
        );
    });
}

#[test]
fn test_registration_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;

        add_network(netuid, tempo, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // Make burn small and stable for this test.
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 50_000);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        // neuron inserted
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);

        // ownership set
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );

        // uid exists
        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();
        assert_eq!(uid, 0);

        // no stake by default
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, uid),
            AlphaCurrency::ZERO
        );
    });
}

#[test]
fn test_registration_failed_no_signature() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        let hotkey = U256::from(1);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::none(),
            netuid,
            hotkey,
        );

        assert_eq!(result, Err(sp_runtime::DispatchError::BadOrigin));
    });
}

#[test]
fn test_registration_disabled() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        SubtensorModule::set_network_registration_allowed(netuid, false);

        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey,
        );

        assert_eq!(
            result,
            Err(Error::<Test>::SubNetRegistrationDisabled.into())
        );
    });
}

#[test]
fn test_registration_root_not_permitted() {
    new_test_ext(1).execute_with(|| {
        let tempo: u16 = 13;
        // Ensure root exists in this test env.
        SubtensorModule::init_new_network(NetUid::ROOT, tempo);

        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            NetUid::ROOT,
            hotkey,
        );

        assert_eq!(
            result,
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
    });
}

#[test]
fn test_registration_not_enough_balance() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // burn cost is 10_000, but coldkey only has 9_999.
        SubtensorModule::set_burn(netuid, 10_000u64.into());

        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 9_999);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);
    });
}

#[test]
fn test_registration_non_associated_coldkey() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let hotkey = U256::from(1);
        let true_owner = U256::from(111);
        let attacker = U256::from(222);

        // Pre-own the hotkey by a different coldkey.
        Owner::<Test>::insert(hotkey, true_owner);

        // Attacker has enough funds, but doesn't own the hotkey.
        SubtensorModule::add_balance_to_coldkey_account(&attacker, 50_000);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(attacker),
            netuid,
            hotkey,
        );

        assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);
    });
}

#[test]
fn test_registration_without_neuron_slot_doesnt_burn() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 10_000);
        let before = SubtensorModule::get_coldkey_balance(&coldkey);

        // No slots => should fail before burning.
        SubtensorModule::set_max_allowed_uids(netuid, 0);

        assert_noop!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                netuid,
                hotkey
            ),
            Error::<Test>::NoNeuronIdAvailable
        );

        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey), before);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 0);
    });
}

#[test]
fn test_registration_too_many_registrations_this_interval() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let coldkey = U256::from(667);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000);

        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);

        // First ok
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey1
        ));

        // Same interval (same block) => reject
        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey2,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::TooManyRegistrationsThisInterval.into())
        );

        // Advance 1 block: add_network sets BurnHalfLife=1 for tests => interval resets each block.
        step_block(1);

        // Now allowed
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey2
        ));
    });
}

#[test]
fn test_registration_already_active_hotkey_error() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let coldkey = U256::from(667);
        let hotkey = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        // Step to a new interval so we don't hit TooManyRegistrationsThisInterval first.
        step_block(1);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyAlreadyRegisteredInSubNet.into())
        );
    });
}

#[test]
fn test_registration_too_many_registrations_this_block_when_block_cap_zero() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        // Block cap at 0 => first reg should fail with TooManyRegistrationsThisBlock
        SubtensorModule::set_max_registrations_per_block(netuid, 0);

        let coldkey = U256::from(667);
        let hotkey = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000);

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey,
        );

        assert_eq!(
            result,
            Err(Error::<Test>::TooManyRegistrationsThisBlock.into())
        );
    });
}

// -----------------------------
// Burn price dynamics tests
// -----------------------------

#[test]
fn test_burn_increases_next_block_after_registration() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // Override to make behavior deterministic:
        BurnHalfLife::<Test>::insert(netuid, 1_000); // no halving during this test
        BurnIncreaseMult::<Test>::insert(netuid, 2);
        BurnLastHalvingBlock::<Test>::insert(netuid, SubtensorModule::get_current_block_as_u64());

        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let coldkey = U256::from(100);
        let hotkey = U256::from(200);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000);

        // Register in this block. Burn doesn't change until next block.
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));
        assert_eq!(SubtensorModule::get_burn(netuid), 1_000u64.into());

        // Next block => bump applies from previous block's registration
        step_block(1);

        assert_eq!(SubtensorModule::get_burn(netuid), 2_000u64.into());
    });
}

#[test]
fn test_burn_halves_every_half_life() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        BurnHalfLife::<Test>::insert(netuid, 2);
        BurnIncreaseMult::<Test>::insert(netuid, 1);
        BurnLastHalvingBlock::<Test>::insert(netuid, SubtensorModule::get_current_block_as_u64());

        SubtensorModule::set_burn(netuid, 1024u64.into());

        step_block(2);
        assert_eq!(SubtensorModule::get_burn(netuid), 512u64.into());

        step_block(2);
        assert_eq!(SubtensorModule::get_burn(netuid), 256u64.into());
    });
}

#[test]
fn test_burn_floor_prevents_zero_stuck_and_allows_bump() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // Half-life every block; multiplier 2.
        BurnHalfLife::<Test>::insert(netuid, 1);
        BurnIncreaseMult::<Test>::insert(netuid, 2);
        BurnLastHalvingBlock::<Test>::insert(netuid, SubtensorModule::get_current_block_as_u64());

        // Start at 1 => halving would go to 0, but floor keeps it at 1.
        SubtensorModule::set_burn(netuid, 1u64.into());

        // Step one block => halving applies, but floor => burn stays 1.
        step_block(1);
        assert_eq!(SubtensorModule::get_burn(netuid), 1u64.into());

        // Register now; bump should apply next block and not be stuck at 0.
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 10_000);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        step_block(1);

        // burn should become 2 (after halving floor then bump)
        assert_eq!(SubtensorModule::get_burn(netuid), 2u64.into());
    });
}

#[test]
fn test_registration_increases_recycled_rao_per_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        BurnHalfLife::<Test>::insert(netuid, 1); // allow 1 reg / block
        BurnIncreaseMult::<Test>::insert(netuid, 1); // keep burn stable aside from halving
        BurnLastHalvingBlock::<Test>::insert(netuid, SubtensorModule::get_current_block_as_u64());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let coldkey = U256::from(667);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000);

        // First registration
        let burn1 = SubtensorModule::get_burn(netuid);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            U256::from(1)
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), burn1);

        // Next interval
        step_block(1);

        // Second registration (burn may have changed; read it)
        let burn2 = SubtensorModule::get_burn(netuid);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            U256::from(2)
        ));

        assert_eq!(SubtensorModule::get_rao_recycled(netuid), burn1 + burn2);
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn test_registration_get_uid_to_prune_all_in_immunity_period() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // Neutralize safety floor and owner-immortality for deterministic selection
        SubtensorModule::set_min_non_immune_uids(netuid, 0);
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 0);
        SubnetOwner::<Test>::insert(netuid, U256::from(999_999u64));

        // Register uid0 @ block 0
        register_ok_neuron(netuid, U256::from(0), U256::from(0), 0);

        // Move to next block so interval resets; register uid1 @ block 1
        step_block(1);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 0);

        // Both immune with immunity_period=2 at current block=1.
        SubtensorModule::set_immunity_period(netuid, 2);
        assert_eq!(SubtensorModule::get_current_block_as_u64(), 1);

        // Both immune; prune lowest emission among immune (uid0=100, uid1=110 => uid0)
        Emission::<Test>::mutate(netuid, |v| {
            v[0] = 100u64.into();
            v[1] = 110u64.into();
        });

        assert_eq!(SubtensorModule::get_neuron_to_prune(netuid), Some(0));
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn test_registration_get_uid_to_prune_none_in_immunity_period() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        // Neutralize safety floor and owner-immortality for deterministic selection
        SubtensorModule::set_min_non_immune_uids(netuid, 0);
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 0);
        SubnetOwner::<Test>::insert(netuid, U256::from(999_999u64));

        // Register uid0 @ block 0
        register_ok_neuron(netuid, U256::from(0), U256::from(0), 0);

        // Register uid1 @ block 1
        step_block(1);
        register_ok_neuron(netuid, U256::from(1), U256::from(1), 0);

        SubtensorModule::set_immunity_period(netuid, 2);

        // Advance beyond immunity -> both non-immune (current=1, step 3 => 4)
        step_block(3);
        assert_eq!(SubtensorModule::get_current_block_as_u64(), 4);

        Emission::<Test>::mutate(netuid, |v| {
            v[0] = 100u64.into();
            v[1] = 110u64.into();
        });

        assert_eq!(SubtensorModule::get_neuron_to_prune(netuid), Some(0));
    });
}

// These owner-immortality tests do not depend on registration paths; keep as-is.

#[test]
fn test_registration_get_uid_to_prune_owner_immortality() {
    new_test_ext(1).execute_with(|| {
        [
            (1, 1), // limit=1 => prune other owner hk (uid1)
            (2, 2), // limit=2 => both owner hks immortal => prune non-owner (uid2)
        ]
        .iter()
        .for_each(|(limit, uid_to_prune)| {
            let subnet_owner_ck = U256::from(0);
            let subnet_owner_hk = U256::from(1);

            let other_owner_hk = U256::from(2);
            Owner::<Test>::insert(other_owner_hk, subnet_owner_ck);
            OwnedHotkeys::<Test>::insert(subnet_owner_ck, vec![subnet_owner_hk, other_owner_hk]);

            let non_owner_hk = U256::from(3);

            let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

            BlockAtRegistration::<Test>::insert(netuid, 1, 1);
            BlockAtRegistration::<Test>::insert(netuid, 2, 2);
            Uids::<Test>::insert(netuid, other_owner_hk, 1);
            Uids::<Test>::insert(netuid, non_owner_hk, 2);
            Keys::<Test>::insert(netuid, 1, other_owner_hk);
            Keys::<Test>::insert(netuid, 2, non_owner_hk);
            ImmunityPeriod::<Test>::insert(netuid, 1);
            SubnetworkN::<Test>::insert(netuid, 3);

            SubtensorModule::set_min_non_immune_uids(netuid, 0);

            step_block(10); // all non-immune

            ImmuneOwnerUidsLimit::<Test>::insert(netuid, *limit);

            Emission::<Test>::insert(
                netuid,
                vec![AlphaCurrency::from(0), 0u64.into(), 1u64.into()],
            );

            assert_eq!(
                SubtensorModule::get_neuron_to_prune(netuid),
                Some(*uid_to_prune)
            );
        });
    });
}

#[test]
fn test_registration_get_uid_to_prune_owner_immortality_all_immune() {
    new_test_ext(1).execute_with(|| {
        let limit = 2;
        let uid_to_prune = 2;
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        let other_owner_hk = U256::from(2);
        Owner::<Test>::insert(other_owner_hk, subnet_owner_ck);
        OwnedHotkeys::<Test>::insert(subnet_owner_ck, vec![subnet_owner_hk, other_owner_hk]);

        let non_owner_hk = U256::from(3);
        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        BlockAtRegistration::<Test>::insert(netuid, 0, 12);
        BlockAtRegistration::<Test>::insert(netuid, 1, 11);
        BlockAtRegistration::<Test>::insert(netuid, 2, 10);
        Uids::<Test>::insert(netuid, other_owner_hk, 1);
        Uids::<Test>::insert(netuid, non_owner_hk, 2);
        Keys::<Test>::insert(netuid, 1, other_owner_hk);
        Keys::<Test>::insert(netuid, 2, non_owner_hk);
        ImmunityPeriod::<Test>::insert(netuid, 100);
        SubnetworkN::<Test>::insert(netuid, 3);

        SubtensorModule::set_min_non_immune_uids(netuid, 0);

        step_block(20); // all still immune

        ImmuneOwnerUidsLimit::<Test>::insert(netuid, limit);

        Emission::<Test>::insert(
            netuid,
            vec![AlphaCurrency::from(0), 0u64.into(), 1u64.into()],
        );

        assert_eq!(
            SubtensorModule::get_neuron_to_prune(netuid),
            Some(uid_to_prune)
        );
    });
}

#[test]
fn test_registration_get_neuron_metadata() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        let hotkey = U256::from(1);
        let coldkey = U256::from(667);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        let neuron: AxonInfoOf = SubtensorModule::get_axon_info(netuid, &hotkey);
        assert_eq!(neuron.ip, 0);
        assert_eq!(neuron.version, 0);
        assert_eq!(neuron.port, 0);
    });
}

#[test]
fn test_last_update_correctness() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        SubtensorModule::set_burn(netuid, 1_000u64.into());

        // Simulate existing neurons
        let existing_neurons: u16 = 3;
        SubnetworkN::<Test>::insert(netuid, existing_neurons);

        // Simulate no LastUpdate so far (can happen on mechanisms)
        LastUpdate::<Test>::remove(NetUidStorageIndex::from(netuid));

        let coldkey = U256::from(667);
        let hotkey = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        assert_eq!(
            LastUpdate::<Test>::get(NetUidStorageIndex::from(netuid)).len(),
            (existing_neurons + 1) as usize
        );
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn test_registration_pruning() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(5);
        add_network(netuid, 10_000, 0);
        mock::setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());

        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 0);

        MaxRegistrationsPerBlock::<Test>::insert(netuid, 1024);
        SubtensorModule::set_max_allowed_uids(netuid, 3);

        let coldkeys = [U256::from(20_001), U256::from(20_002), U256::from(20_003)];
        let hotkeys = [U256::from(30_001), U256::from(30_002), U256::from(30_003)];

        for i in 0..3 {
            register_ok_neuron(netuid, hotkeys[i], coldkeys[i], 0);
            step_block(1);
        }

        let uid0 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[0]).unwrap();
        let uid1 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[1]).unwrap();
        let uid2 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[2]).unwrap();

        assert_eq!(uid0, 0);
        assert_eq!(uid1, 1);
        assert_eq!(uid2, 2);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        let now: u64 = 1_000;
        frame_system::Pallet::<Test>::set_block_number(now);

        SubtensorModule::set_immunity_period(netuid, 100);

        BlockAtRegistration::<Test>::insert(netuid, uid0, now - 150);
        BlockAtRegistration::<Test>::insert(netuid, uid1, now - 200);
        BlockAtRegistration::<Test>::insert(netuid, uid2, now - 10);

        assert!(!SubtensorModule::get_neuron_is_immune(netuid, uid0));
        assert!(!SubtensorModule::get_neuron_is_immune(netuid, uid1));
        assert!(SubtensorModule::get_neuron_is_immune(netuid, uid2));

        Emission::<Test>::mutate(netuid, |v| {
            v[uid0 as usize] = 10u64.into();
            v[uid1 as usize] = 10u64.into();
            v[uid2 as usize] = 1u64.into();
        });

        SubtensorModule::set_min_non_immune_uids(netuid, 0);

        assert_eq!(SubtensorModule::get_neuron_to_prune(netuid), Some(uid1));

        let new_hotkey = U256::from(40_000);
        let new_coldkey = U256::from(50_000);

        // Ensure interval allows another registration
        step_block(1);

        register_ok_neuron(netuid, new_hotkey, new_coldkey, 0);

        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[1]).is_err(),
            "Hotkey for pruned UID should no longer be registered"
        );

        let new_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &new_hotkey).unwrap();
        assert_eq!(new_uid, uid1);

        assert_eq!(
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[0]).unwrap(),
            uid0
        );
        assert_eq!(
            SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkeys[2]).unwrap(),
            uid2
        );
    });
}

// DEPRECATED #[test]
// fn test_network_connection_requirement() {
//     new_test_ext(1).execute_with(|| {
//         // Add a networks and connection requirements.
//         let netuid_a: u16 = 0;
//         let netuid_b: u16 = 1;
//         add_network(netuid_a, 10, 0);
//         add_network(netuid_b, 10, 0);

//         // Bulk values.
//         let hotkeys: Vec<U256> = (0..=10).map(|x| U256::from(x)).collect();
//         let coldkeys: Vec<U256> = (0..=10).map(|x| U256::from(x)).collect();

//         // Add a connection requirement between the A and B. A requires B.
//         SubtensorModule::add_connection_requirement(netuid_a, netuid_b, u16::MAX);
//         SubtensorModule::set_max_registrations_per_block(netuid_a, 10); // Enough for the below tests.
//         SubtensorModule::set_max_registrations_per_block(netuid_b, 10); // Enough for the below tests.
//         SubtensorModule::set_max_allowed_uids(netuid_a, 10); // Enough for the below tests.
//         SubtensorModule::set_max_allowed_uids(netuid_b, 10); // Enough for the below tests.

//         // Attempt registration on A fails because the hotkey is not registered on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 3942084, &U256::from(0));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[0],
//                 coldkeys[0]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Attempt registration on B passes because there is no exterior requirement.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 5942084, &U256::from(0));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[0],
//             coldkeys[0]
//         ));

//         // Attempt registration on A passes because this key is in the top 100 of keys on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 6942084, &U256::from(0));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[0]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[0],
//             coldkeys[0]
//         ));

//         // Lets attempt the key registration on A. Fails because we are not in B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 634242084, &U256::from(1));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[1],
//                 coldkeys[1]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Lets register the next key on B. Passes, np.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 7942084, &U256::from(1));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[1],
//             coldkeys[1]
//         ));

//         // Lets make the connection requirement harder. Top 0th percentile.
//         SubtensorModule::add_connection_requirement(netuid_a, netuid_b, 0);

//         // Attempted registration passes because the prunning score for hotkey_1 is the top keys on network B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 8942084, &U256::from(1));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[1]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[1],
//             coldkeys[1]
//         ));

//         // Lets register key 3 with lower prunning score.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 9942084, &U256::from(2));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[2]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[2],
//             coldkeys[2]
//         ));
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[2]).unwrap(),
//             0,
//         ); // Set prunning score to 0.
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[1]).unwrap(),
//             0,
//         ); // Set prunning score to 0.
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[0]).unwrap(),
//             0,
//         ); // Set prunning score to 0.

//         // Lets register key 4 with higher prunining score.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 10142084, &U256::from(3));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[3]),
//             netuid_b,
//             0,
//             nonce,
//             work,
//             hotkeys[3],
//             coldkeys[3]
//         ));
//         SubtensorModule::set_pruning_score_for_uid(
//             netuid_b,
//             SubtensorModule::get_uid_for_net_and_hotkey(netuid_b, &hotkeys[3]).unwrap(),
//             1,
//         ); // Set prunning score to 1.

//         // Attempted register of key 3 fails because of bad prunning score on B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_a, 0, 11142084, &U256::from(2));
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as Config>::RuntimeOrigin>::signed(hotkeys[2]),
//                 netuid_a,
//                 0,
//                 nonce,
//                 work,
//                 hotkeys[2],
//                 coldkeys[2]
//             ),
//             Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into())
//         );

//         // Attempt to register key 4 passes because of best prunning score on B.
//         let (nonce, work): (u64, Vec<u8>) =
//             SubtensorModule::create_work_for_block_number(netuid_b, 0, 12142084, &U256::from(3));
//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkeys[3]),
//             netuid_a,
//             0,
//             nonce,
//             work,
//             hotkeys[3],
//             coldkeys[3]
//         ));
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_ok() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = NetUid::from(1);
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(667);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10_000_000_000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(1337);
//         assert_ok!(SubtensorModule::swap_hotkey(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             hotkey_account_id,
//             new_hotkey
//         ));
//         assert_ne!(
//             SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id),
//             coldkey_account_id
//         );
//         assert_eq!(
//             SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkey),
//             coldkey_account_id
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_not_owner() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = NetUid::from(1);
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);
//         let not_owner_coldkey = U256::from(3);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(4);
//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(not_owner_coldkey),
//                 hotkey_account_id,
//                 new_hotkey
//             ),
//             Error::<Test>::NonAssociatedColdKey
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_same_key() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = NetUid::from(1);
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 hotkey_account_id
//             ),
//             Error::<Test>::HotKeyAlreadyRegisteredInSubNet
//         );
//     });
// }

// #[ignore]
// #[test]
// fn test_hotkey_swap_registered_key() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = NetUid::from(1);
//         let tempo: u16 = 13;
//         let hotkey_account_id = U256::from(1);
//         let burn_cost = 1000;
//         let coldkey_account_id = U256::from(2);

//         SubtensorModule::set_burn(netuid, burn_cost);
//         add_network(netuid, tempo, 0);

//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 100_000_000_000);

//         // Subscribe and check extrinsic output
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             hotkey_account_id
//         ));

//         let new_hotkey = U256::from(3);
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid,
//             new_hotkey
//         ));

//         assert_err!(
//             SubtensorModule::swap_hotkey(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 new_hotkey
//             ),
//             Error::<Test>::HotKeyAlreadyRegisteredInSubNet
//         );
//     });
// }
