#![allow(
    unused,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use approx::assert_abs_diff_eq;
use codec::Encode;
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo};
use frame_support::error::BadOrigin;
use frame_support::traits::OnInitialize;
use frame_support::traits::schedule::DispatchTime;
use frame_support::traits::schedule::v3::Named as ScheduleNamed;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
use sp_core::{Get, H256, U256};
use sp_runtime::traits::Hash;
use sp_runtime::traits::{DispatchInfoOf, DispatchTransaction, TransactionExtension};
use sp_runtime::{DispatchError, traits::TxBaseImplication};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, SubnetInfo, TaoCurrency};
use subtensor_swap_interface::{SwapEngine, SwapHandler};

use super::mock;
use super::mock::*;
use crate::transaction_extension::SubtensorTransactionExtension;
use crate::*;
use crate::{Call, Error};

#[test]
fn test_announce_coldkey_swap_works() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who, (now, new_coldkey_hash))]
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::ColdkeySwapAnnounced {
                who,
                new_coldkey_hash,
                block_number: now,
            })
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_existing_announcement_past_delay_works() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let new_coldkey_2 = U256::from(3);
        let new_coldkey_2_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey_2);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who, (now, new_coldkey_hash))]
        );

        let delay = ColdkeySwapAnnouncementDelay::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_2_hash,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who, (now, new_coldkey_2_hash))]
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_bad_origin_fails() {
    new_test_ext(1).execute_with(|| {
        let new_coldkey = U256::from(1);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(RuntimeOrigin::none(), new_coldkey_hash),
            BadOrigin
        );

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(RuntimeOrigin::root(), new_coldkey_hash),
            BadOrigin
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_existing_announcement_not_past_delay_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let new_coldkey_2 = U256::from(3);
        let new_coldkey_2_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey_2);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who, (now, new_coldkey_hash))]
        );

        let unmet_delay = ColdkeySwapAnnouncementDelay::<Test>::get();
        System::run_to_block::<AllPalletsWithSystem>(now + unmet_delay);

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(RuntimeOrigin::signed(who), new_coldkey_2_hash,),
            Error::<Test>::ColdKeySwapReannouncedTooEarly
        );
    });
}

#[test]
fn test_swap_coldkey_announced_works() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let hotkey1 = U256::from(1001);
        let hotkey2 = U256::from(1002);
        let hotkey3 = U256::from(1003);
        let swap_cost = SubtensorModule::get_key_swap_cost();
        let left_over = 12345;
        let min_stake = DefaultMinStake::<Test>::get().to_u64();
        let stake1 = min_stake * 10;
        let stake2 = min_stake * 20;
        let stake3 = min_stake * 30;
        let now = System::block_number();

        SubtensorModule::add_balance_to_coldkey_account(
            &who,
            stake1 + stake2 + stake3 + swap_cost.to_u64() + left_over,
        );

        // Announce the coldkey swap
        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who, (now, new_coldkey_hash))]
        );

        // Run some blocks for the announcement to be past the delay
        // WARN: this is required before staking to neurons to avoid
        // value mismatch due to coinbase run
        let delay = ColdkeySwapAnnouncementDelay::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        // Setup networks and subnet ownerships
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        SubnetOwner::<Test>::insert(netuid1, who);
        SubnetOwner::<Test>::insert(netuid2, who);

        // Setup reserves
        let reserve1 = (stake1 + stake3) * 10;
        let reserve2 = stake2 * 10;
        mock::setup_reserves(netuid1, reserve1.into(), reserve1.into());
        mock::setup_reserves(netuid2, reserve2.into(), reserve2.into());

        // Setup auto stake destinations
        AutoStakeDestination::<Test>::insert(who, netuid1, hotkey1);
        AutoStakeDestination::<Test>::insert(who, netuid2, hotkey2);
        AutoStakeDestinationColdkeys::<Test>::insert(
            hotkey1,
            netuid1,
            vec![who, U256::from(3), U256::from(4)],
        );
        AutoStakeDestinationColdkeys::<Test>::insert(
            hotkey2,
            netuid2,
            vec![U256::from(7), U256::from(8), who],
        );

        // Setup neurons with stake
        register_ok_neuron(netuid1, hotkey1, who, 0);
        register_ok_neuron(netuid2, hotkey2, who, 0);
        register_ok_neuron(netuid1, hotkey3, who, 0);

        let hotkeys = vec![hotkey1, hotkey2, hotkey3];
        assert_eq!(StakingHotkeys::<Test>::get(who), hotkeys);
        assert_eq!(OwnedHotkeys::<Test>::get(who), hotkeys);
        assert_eq!(Owner::<Test>::get(hotkey1), who);
        assert_eq!(Owner::<Test>::get(hotkey2), who);
        assert_eq!(Owner::<Test>::get(hotkey3), who);

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(who),
            hotkey1,
            netuid1,
            stake1.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(who),
            hotkey2,
            netuid2,
            stake2.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(who),
            hotkey3,
            netuid1,
            stake3.into()
        ));
        let hk1_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &who, netuid1);
        let hk2_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &who, netuid2);
        let hk3_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey3, &who, netuid1);
        let total_ck_stake = SubtensorModule::get_total_stake_for_coldkey(&who);

        // Setup identity
        let identity = ChainIdentityV2::default();
        IdentitiesV2::<Test>::insert(who, identity.clone());
        assert_eq!(IdentitiesV2::<Test>::get(who), Some(identity.clone()));
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_none());

        let balance_before = SubtensorModule::get_coldkey_balance(&who);
        let total_stake_before = SubtensorModule::get_total_stake();

        assert_ok!(SubtensorModule::swap_coldkey_announced(
            <Test as frame_system::Config>::RuntimeOrigin::signed(who),
            new_coldkey
        ));

        // Ensure the announcement has been consumed
        assert_eq!(ColdkeySwapAnnouncements::<Test>::get(who), None);

        // Ensure the cost has been withdrawn from the old coldkey and recycled
        let balance_after = SubtensorModule::get_coldkey_balance(&who);
        assert_eq!(
            balance_before - swap_cost.to_u64(),
            balance_after + left_over
        );

        // Ensure the identity is correctly swapped
        assert!(IdentitiesV2::<Test>::get(who).is_none());
        assert_eq!(IdentitiesV2::<Test>::get(new_coldkey), Some(identity));

        // Ensure the subnet ownerships are correctly swapped
        assert_eq!(SubnetOwner::<Test>::get(netuid1), new_coldkey);
        assert_eq!(SubnetOwner::<Test>::get(netuid2), new_coldkey);

        // Ensure the auto stake destinations are correctly swapped
        assert!(AutoStakeDestination::<Test>::get(who, netuid1).is_none());
        assert!(AutoStakeDestination::<Test>::get(who, netuid2).is_none());
        assert_eq!(
            AutoStakeDestination::<Test>::get(new_coldkey, netuid1),
            Some(hotkey1)
        );
        assert_eq!(
            AutoStakeDestination::<Test>::get(new_coldkey, netuid2),
            Some(hotkey2)
        );
        assert_eq!(
            AutoStakeDestinationColdkeys::<Test>::get(hotkey1, netuid1),
            vec![U256::from(3), U256::from(4), new_coldkey]
        );
        assert_eq!(
            AutoStakeDestinationColdkeys::<Test>::get(hotkey2, netuid2),
            vec![U256::from(7), U256::from(8), new_coldkey]
        );

        // Ensure the coldkey stake is correctly swapped
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &who, netuid1),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &who, netuid2),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey3, &who, netuid1),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &new_coldkey,
                netuid1
            ),
            hk1_alpha
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &new_coldkey,
                netuid2
            ),
            hk2_alpha
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey3,
                &new_coldkey,
                netuid1
            ),
            hk3_alpha
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&who),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            total_ck_stake,
        );

        // Ensure the staking hotkeys are correctly swapped
        assert!(StakingHotkeys::<Test>::get(who).is_empty());
        assert_eq!(StakingHotkeys::<Test>::get(new_coldkey), hotkeys);

        // Ensure the hotkey ownership is correctly swapped
        assert!(OwnedHotkeys::<Test>::get(who).is_empty());
        assert_eq!(OwnedHotkeys::<Test>::get(new_coldkey), hotkeys);
        assert_eq!(Owner::<Test>::get(hotkey1), new_coldkey);
        assert_eq!(Owner::<Test>::get(hotkey2), new_coldkey);
        assert_eq!(Owner::<Test>::get(hotkey3), new_coldkey);

        // Ensure the remaining balance is transferred to the new coldkey
        assert_eq!(SubtensorModule::get_coldkey_balance(&who), 0);
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey),
            left_over
        );

        // Ensure total stake is unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            total_stake_before,
            "Total stake changed unexpectedly"
        );

        // Verify event emission
        System::assert_last_event(
            Event::ColdkeySwapped {
                old_coldkey: who,
                new_coldkey,
                swap_cost,
            }
            .into(),
        );
    });
}

#[test]
fn test_do_swap_coldkey_preserves_new_coldkey_identity() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);

        let old_identity = ChainIdentityV2 {
            name: b"Old identity".to_vec(),
            ..Default::default()
        };
        IdentitiesV2::<Test>::insert(who, old_identity.clone());

        let new_identity = ChainIdentityV2 {
            name: b"New identity".to_vec(),
            ..Default::default()
        };
        IdentitiesV2::<Test>::insert(new_coldkey, new_identity.clone());

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&who, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&who, &new_coldkey));

        // Identity is preserved
        assert_eq!(IdentitiesV2::<Test>::get(who), Some(old_identity));
        assert_eq!(IdentitiesV2::<Test>::get(new_coldkey), Some(new_identity));
    });
}

#[test]
fn test_swap_coldkey_announced_with_bad_origin_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(RuntimeOrigin::none(), new_coldkey),
            BadOrigin
        );

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(RuntimeOrigin::root(), new_coldkey),
            BadOrigin
        );
    });
}

#[test]
fn test_swap_coldkey_announced_without_announcement_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(RuntimeOrigin::signed(who), new_coldkey),
            Error::<Test>::ColdKeySwapAnnouncementNotFound
        );
    })
}

#[test]
fn test_swap_coldkey_announced_with_mismatched_coldkey_hash_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let other_coldkey = U256::from(3);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(RuntimeOrigin::signed(who), other_coldkey),
            Error::<Test>::AnnouncedColdkeyHashDoesNotMatch
        );
    })
}

#[test]
fn test_swap_coldkey_announced_too_early_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(
                <Test as frame_system::Config>::RuntimeOrigin::signed(who),
                new_coldkey
            ),
            Error::<Test>::ColdKeySwapTooEarly
        );
    })
}

#[test]
fn test_swap_coldkey_announced_with_already_associated_coldkey_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let hotkey = U256::from(3);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));

        let now = System::block_number();
        let delay = ColdkeySwapAnnouncementDelay::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&who, swap_cost.to_u64());

        SubtensorModule::create_account_if_non_existent(&new_coldkey, &hotkey);

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(
                <Test as frame_system::Config>::RuntimeOrigin::signed(who),
                new_coldkey
            ),
            Error::<Test>::ColdKeyAlreadyAssociated
        );
    })
}

#[test]
fn test_swap_coldkey_announced_with_hotkey_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let hotkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&hotkey);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who),
            hotkey_hash,
        ));

        let now = System::block_number();
        let delay = ColdkeySwapAnnouncementDelay::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&who, swap_cost.to_u64());

        SubtensorModule::create_account_if_non_existent(&new_coldkey, &hotkey);

        assert_noop!(
            SubtensorModule::swap_coldkey_announced(
                <Test as frame_system::Config>::RuntimeOrigin::signed(who),
                hotkey
            ),
            Error::<Test>::NewColdKeyIsHotkey
        );
    })
}

#[test]
fn test_do_swap_coldkey_with_not_enough_balance_to_pay_swap_cost_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);

        let now = System::block_number();
        let delay = ColdkeySwapAnnouncementDelay::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        assert_noop!(
            SubtensorModule::do_swap_coldkey(&who, &new_coldkey),
            Error::<Test>::NotEnoughBalanceToPaySwapColdKey
        );
    });
}

#[test]
fn test_do_swap_coldkey_with_no_stake() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            TaoCurrency::ZERO
        );
    });
}

#[test]
fn test_do_swap_coldkey_with_max_values() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let old_coldkey2 = U256::from(3);
        let new_coldkey2 = U256::from(4);
        let hotkey = U256::from(5);
        let hotkey2 = U256::from(6);
        let other_coldkey = U256::from(7);
        let netuid = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let stake = 10_000;
        let max_stake = 21_000_000_000_000_000; // 21 Million TAO; max possible balance.

        // Add a network
        add_network(netuid, 1, 0);
        add_network(netuid2, 1, 0);

        // Register hotkey on each subnet.
        // hotkey2 is owned by other_coldkey.
        register_ok_neuron(netuid, hotkey, old_coldkey, 1001000);
        register_ok_neuron(netuid2, hotkey2, other_coldkey, 1001000);

        // Give balance to old_coldkey and old_coldkey2.
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, max_stake + 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey2, max_stake + 1_000);

        let reserve = max_stake * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());

        // Stake to hotkey on each subnet.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey,
            netuid,
            max_stake.into()
        ));
        let expected_stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &old_coldkey,
            netuid,
        );

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey2),
            hotkey2,
            netuid2,
            max_stake.into()
        ));
        let expected_stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey2,
            netuid2,
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey2, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));
        assert_ok!(SubtensorModule::do_swap_coldkey(
            &old_coldkey2,
            &new_coldkey2
        ));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            expected_stake1.to_u64().into(),
            epsilon = TaoCurrency::from(expected_stake1.to_u64()) / 1000.into()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey2),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey2),
            expected_stake2.to_u64().into(),
            epsilon = TaoCurrency::from(expected_stake2.to_u64()) / 1000.into()
        );
    });
}

#[test]
fn test_do_swap_coldkey_effect_on_delegated_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let delegator = U256::from(3);
        let hotkey = U256::from(4);
        let stake = 100_000_000_000;

        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);
        StakingHotkeys::<Test>::insert(delegator, vec![hotkey]);
        SubtensorModule::create_account_if_non_existent(&old_coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake);
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake);

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(old_coldkey),
            hotkey,
            netuid,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            hotkey,
            netuid,
            stake.into()
        ));
        let coldkey_stake_before = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);
        let delegator_stake_before = SubtensorModule::get_total_stake_for_coldkey(&delegator);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            coldkey_stake_before,
            epsilon = 500.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            delegator_stake_before,
            epsilon = 500.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO,
            epsilon = 500.into()
        );
    });
}

#[test]
fn test_swap_delegated_stake_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let other_coldkey = U256::from(3);
        let hotkey1 = U256::from(4);
        let hotkey2 = U256::from(5);
        let stake_amount1 = DefaultMinStake::<Test>::get().to_u64() * 10;
        let stake_amount2 = DefaultMinStake::<Test>::get().to_u64() * 20;
        let netuid = NetUid::from(1);

        // Setup initial state
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey1, other_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, other_coldkey, 0);

        let reserve = (stake_amount1 + stake_amount2) * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Notice hotkey1 and hotkey2 are Owned by other_coldkey
        // old_coldkey and new_coldkey therefore delegates stake to them
        // === Give old_coldkey some balance ===
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + 1_000_000,
        );

        // === Stake to hotkeys ===
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            netuid,
            stake_amount1.into()
        ));
        let expected_stake_alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );

        let (expected_stake_alpha2, fee) = mock::swap_tao_to_alpha(netuid, stake_amount2.into());
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            netuid,
            stake_amount2.into()
        ));
        let expected_stake_alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );
        let fee = (expected_stake_alpha2.to_u64() as f64 * 0.003) as u64;

        // Record initial values
        let initial_total_issuance = SubtensorModule::get_total_issuance();
        let initial_total_stake = SubtensorModule::get_total_stake();
        let coldkey_stake = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);
        let stake_coldkey_hotkey1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );
        let stake_coldkey_hotkey2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );
        let total_hotkey1_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let total_hotkey2_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify stake transfer
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha2
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        // Verify TotalColdkeyStake
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            coldkey_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );

        // Verify TotalHotkeyStake remains unchanged
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey1),
            total_hotkey1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey2),
            total_hotkey2_stake
        );

        // Verify total stake and issuance remain unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake,
            "Total stake changed unexpectedly"
        );
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_total_issuance,
            "Total issuance changed unexpectedly"
        );
    });
}

#[test]
fn test_coldkey_swap_total() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let nominator1 = U256::from(2);
        let nominator2 = U256::from(3);
        let nominator3 = U256::from(4);
        let delegate1 = U256::from(5);
        let delegate2 = U256::from(6);
        let delegate3 = U256::from(7);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let hotkey3 = U256::from(4);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let netuid3 = NetUid::from(3);
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake * 6);
        SubtensorModule::add_balance_to_coldkey_account(&delegate1, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&delegate2, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&delegate3, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator3, stake * 2);

        let reserve = stake * 10;
        mock::setup_reserves(netuid1, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());
        mock::setup_reserves(netuid3, reserve.into(), reserve.into());

        // Setup initial state
        add_network(netuid1, 13, 0);
        add_network(netuid2, 14, 0);
        add_network(netuid3, 15, 0);
        register_ok_neuron(netuid1, hotkey1, coldkey, 0);
        register_ok_neuron(netuid2, hotkey2, coldkey, 0);
        register_ok_neuron(netuid3, hotkey3, coldkey, 0);
        register_ok_neuron(netuid1, delegate1, delegate1, 0);
        register_ok_neuron(netuid2, delegate2, delegate2, 0);
        register_ok_neuron(netuid3, delegate3, delegate3, 0);
        Delegates::<Test>::insert(hotkey1, u16::MAX / 10);
        Delegates::<Test>::insert(hotkey2, u16::MAX / 10);
        Delegates::<Test>::insert(hotkey3, u16::MAX / 10);
        Delegates::<Test>::insert(delegate1, u16::MAX / 10);
        Delegates::<Test>::insert(delegate2, u16::MAX / 10);
        Delegates::<Test>::insert(delegate3, u16::MAX / 10);

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey3,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            hotkey3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            hotkey3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        let ck_stake = SubtensorModule::get_total_stake_for_coldkey(&coldkey);
        let hk1_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let hk2_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey2);
        let hk3_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey3);
        let d1_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate1);
        let d2_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate2);
        let d3_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate3);

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost.to_u64());

        // Perform the swap
        let new_coldkey = U256::from(1100);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            ck_stake
        );
        assert_ok!(SubtensorModule::do_swap_coldkey(&coldkey, &new_coldkey,));
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            ck_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            TaoCurrency::ZERO
        );

        // Check everything is swapped.
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        // Shouldn't change.
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey1),
            hk1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey2),
            hk2_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey3),
            hk3_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate1),
            d1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate2),
            d2_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate3),
            d3_stake
        );

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );
    });
}

#[test]
fn test_do_swap_coldkey_effect_on_delegations() {
    new_test_ext(1).execute_with(|| {
        let new_coldkey = U256::from(0);
        let owner = U256::from(1);
        let coldkey = U256::from(4);
        let delegate = U256::from(2);
        let netuid = NetUid::from(0); // Stake to 0
        let netuid2 = NetUid::from(1); // Stake to 1
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        let reserve = stake * 1000;

        mock::setup_reserves(netuid, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());

        add_network(netuid, 13, 0); // root
        add_network(netuid2, 13, 0);

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            delegate
        )); // register on root
        register_ok_neuron(netuid2, delegate, owner, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake * 10);

        // since the reserves are equal and we stake the same amount to both networks, we can reuse
        // this values for different networks. but you should take it into account in case of tests
        // changes
        let (expected_stake, fee) = mock::swap_tao_to_alpha(netuid, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate,
            netuid,
            stake.into()
        ));

        // Add stake to netuid2
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate,
            netuid2,
            stake.into()
        ));

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&coldkey, &new_coldkey,));

        // Verify stake was moved for the delegate
        let approx_total_stake = TaoCurrency::from(stake * 2 - fee * 2);
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate),
            approx_total_stake,
            epsilon = approx_total_stake / 100.into()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            approx_total_stake,
            epsilon = approx_total_stake / 100.into()
        );
        assert_eq!(
            expected_stake,
            Alpha::<Test>::get((delegate, new_coldkey, netuid))
                .to_num::<u64>()
                .into(),
        );
        assert_eq!(Alpha::<Test>::get((delegate, coldkey, netuid)), 0);

        assert_eq!(
            expected_stake,
            Alpha::<Test>::get((delegate, new_coldkey, netuid2))
                .to_num::<u64>()
                .into()
        );
        assert_eq!(Alpha::<Test>::get((delegate, coldkey, netuid2)), 0);
    });
}

#[test]
fn test_subtensor_extension_rejects_any_call_that_is_not_swap_coldkey_announced() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let who = U256::from(0);
        let new_coldkey = U256::from(1);
        let new_coldkey_hash = <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let hotkey = U256::from(2);
        let stake = DefaultMinStake::<Test>::get().to_u64();
        assert_ne!(hotkey, who);

        // Setup reserves
        let reserve = stake * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Setup network and neuron
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, who, 0);

        SubtensorModule::add_balance_to_coldkey_account(&who, u64::MAX);

        // Schedule the coldkey for a swap
        assert_ok!(SubtensorModule::announce_coldkey_swap(
            <Test as frame_system::Config>::RuntimeOrigin::signed(who),
            new_coldkey_hash,
        ));
        assert!(ColdkeySwapAnnouncements::<Test>::contains_key(who));

        let forbidden_calls: Vec<RuntimeCall> = vec![
            RuntimeCall::SubtensorModule(SubtensorCall::dissolve_network {
                netuid,
                coldkey: who,
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
                hotkey,
                netuid,
                amount_staked: stake.into(),
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::add_stake_limit {
                hotkey,
                netuid,
                amount_staked: stake.into(),
                limit_price: stake.into(),
                allow_partial: false,
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::swap_stake {
                hotkey,
                origin_netuid: netuid,
                destination_netuid: netuid,
                alpha_amount: stake.into(),
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::swap_stake_limit {
                hotkey,
                origin_netuid: netuid,
                destination_netuid: netuid,
                alpha_amount: stake.into(),
                limit_price: stake.into(),
                allow_partial: false,
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::move_stake {
                origin_hotkey: hotkey,
                destination_hotkey: hotkey,
                origin_netuid: netuid,
                destination_netuid: netuid,
                alpha_amount: stake.into(),
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::transfer_stake {
                destination_coldkey: new_coldkey,
                hotkey,
                origin_netuid: netuid,
                destination_netuid: netuid,
                alpha_amount: stake.into(),
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
                hotkey,
                netuid,
                amount_unstaked: (DefaultMinStake::<Test>::get().to_u64() * 2).into(),
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::remove_stake_limit {
                hotkey,
                netuid,
                amount_unstaked: (stake * 2).into(),
                limit_price: 123456789.into(),
                allow_partial: true,
            }),
            RuntimeCall::SubtensorModule(SubtensorCall::burned_register { netuid, hotkey }),
            RuntimeCall::Balances(BalancesCall::transfer_all {
                dest: new_coldkey,
                keep_alive: false,
            }),
            RuntimeCall::Balances(BalancesCall::transfer_keep_alive {
                dest: new_coldkey,
                value: 100_000_000_000,
            }),
            RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                dest: new_coldkey,
                value: 100_000_000_000,
            }),
        ];

        for call in forbidden_calls {
            let info = call.get_dispatch_info();
            let ext = SubtensorTransactionExtension::<Test>::new();
            assert_noop!(
                ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
                CustomTransactionError::ColdkeySwapAnnounced
            );
        }

        // Swap coldkey announced should succeed
        let call =
            RuntimeCall::SubtensorModule(SubtensorCall::swap_coldkey_announced { new_coldkey });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_ok!(ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0));
    });
}

#[test]
#[allow(deprecated)]
fn test_swap_coldkey_deprecated() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::swap_coldkey(
                <<Test as Config>::RuntimeOrigin>::root(),
                old_coldkey,
                new_coldkey,
                TaoCurrency::MAX
            ),
            Error::<Test>::Deprecated
        );
    });
}

#[test]
#[allow(deprecated)]
fn test_schedule_swap_coldkey_deprecated() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::schedule_swap_coldkey(
                <<Test as Config>::RuntimeOrigin>::root(),
                new_coldkey,
            ),
            Error::<Test>::Deprecated
        );
    });
}

// TEST TRANSFER ROOT CLAIM WITH NEW KEYS + ROOT CASE
