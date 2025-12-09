//! Subtensor pallet benchmarking.
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as Subtensor;
use crate::*;
use codec::Compact;
use frame_benchmarking::v2::*;
use frame_support::{StorageDoubleMap, assert_ok};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
pub use pallet::*;
use sp_core::H256;
use sp_runtime::{
    BoundedVec, Percent,
    traits::{BlakeTwo256, Hash},
};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

#[frame_benchmarking::v2::benchmarks]
mod pallet_benchmarks {
    use super::*;

    #[benchmark]
    fn register() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        let hotkey: T::AccountId = account("Alice", 0, 1);
        let coldkey: T::AccountId = account("Test", 0, 2);

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

        let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work): (u64, Vec<u8>) =
            Subtensor::<T>::create_work_for_block_number(netuid, block_number, 3, &hotkey);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuid,
            block_number,
            nonce,
            work,
            hotkey.clone(),
            coldkey.clone(),
        );
    }

    #[benchmark]
    fn set_weights() {
        let netuid = NetUid::from(1);
        let version_key: u64 = 1;
        let tempo: u16 = 1;

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_max_registrations_per_block(netuid, 4096);
        Subtensor::<T>::set_target_registrations_per_interval(netuid, 4096);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, false);

        let mut seed: u32 = 1;
        let mut dests = Vec::new();
        let mut weights = Vec::new();
        let signer: T::AccountId = account("Alice", 0, seed);

        for _ in 0..4096 {
            let hotkey: T::AccountId = account("Alice", 0, seed);
            let coldkey: T::AccountId = account("Test", 0, seed);
            seed += 1;

            Subtensor::<T>::set_burn(netuid, 1.into());
            let amount_to_be_staked: u64 = 1_000_000;
            Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);

            assert_ok!(Subtensor::<T>::do_burned_registration(
                RawOrigin::Signed(coldkey.clone()).into(),
                netuid,
                hotkey.clone()
            ));
            let uid = Subtensor::<T>::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();
            Subtensor::<T>::set_validator_permit_for_uid(netuid, uid, true);

            dests.push(uid);
            weights.push(uid);
        }

        #[extrinsic_call]
        _(
            RawOrigin::Signed(signer.clone()),
            netuid,
            dests,
            weights,
            version_key,
        );
    }

    #[benchmark]
    fn add_stake() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;

        Subtensor::<T>::init_new_network(netuid, tempo);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);

        let seed: u32 = 1;
        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);
        let total_stake = TaoCurrency::from(1_000_000_000);
        let amount = TaoCurrency::from(60_000_000);

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, total_stake.into());
        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            netuid,
            amount,
        );
    }

    #[benchmark]
    fn serve_axon() {
        let netuid = NetUid::from(1);
        let caller: T::AccountId = whitelisted_caller();
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;
        let protocol: u8 = 0;
        let placeholder1: u8 = 0;
        let placeholder2: u8 = 0;

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        let deposit = reg_fee.saturating_mul(2.into());
        Subtensor::<T>::add_balance_to_coldkey_account(&caller, deposit.into());

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(caller.clone()).into(),
            netuid,
            caller.clone()
        ));
        Subtensor::<T>::set_serving_rate_limit(netuid, 0);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        );
    }

    #[benchmark]
    fn serve_prometheus() {
        let netuid = NetUid::from(1);
        let caller: T::AccountId = whitelisted_caller();
        let version: u32 = 2;
        let ip: u128 = 1676056785;
        let port: u16 = 128;
        let ip_type: u8 = 4;

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        let deposit = reg_fee.saturating_mul(2.into());
        Subtensor::<T>::add_balance_to_coldkey_account(&caller, deposit.into());

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(caller.clone()).into(),
            netuid,
            caller.clone()
        ));
        Subtensor::<T>::set_serving_rate_limit(netuid, 0);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            netuid,
            version,
            ip,
            port,
            ip_type,
        );
    }

    #[benchmark]
    fn burned_register() {
        let netuid = NetUid::from(1);
        let seed: u32 = 1;
        let hotkey: T::AccountId = account("Alice", 0, seed);
        let coldkey: T::AccountId = account("Test", 0, seed);

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());

        let amount: u64 = 1_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), netuid, hotkey.clone());
    }

    #[benchmark]
    fn root_register() {
        let netuid = NetUid::from(1);
        let seed: u32 = 1;
        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

        let amount: u64 = 100_000_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount);

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone());
    }

    #[benchmark]
    fn register_network() {
        let seed: u32 = 1;
        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("TestHotkey", 0, seed);

        Subtensor::<T>::set_network_rate_limit(1);
        let amount: u64 = 100_000_000_000_000u64.saturating_mul(2);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone());
    }

    #[benchmark]
    fn commit_weights() {
        let tempo: u16 = 1;
        let netuid = NetUid::from(1);
        let version_key: u64 = 0;
        let uids: Vec<u16> = vec![0];
        let weight_values: Vec<u16> = vec![10];
        let hotkey: T::AccountId = account("hot", 0, 1);
        let coldkey: T::AccountId = account("cold", 0, 2);
        let start_nonce: u64 = 300_000;

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey.clone(),
            netuid,
            uids.clone(),
            weight_values.clone(),
            version_key,
        ));

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

        let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work) = Subtensor::<T>::create_work_for_block_number(
            netuid,
            block_number,
            start_nonce,
            &hotkey,
        );
        assert_ok!(Subtensor::<T>::register(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            block_number,
            nonce,
            work,
            hotkey.clone(),
            coldkey.clone()
        ));
        Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

        #[extrinsic_call]
        _(RawOrigin::Signed(hotkey.clone()), netuid, commit_hash);
    }

    #[benchmark]
    fn reveal_weights() {
        let tempo: u16 = 0;
        let netuid = NetUid::from(1);
        let version_key: u64 = 0;
        let uids: Vec<u16> = vec![0];
        let weight_values: Vec<u16> = vec![10];
        let salt: Vec<u16> = vec![8];
        let hotkey: T::AccountId = account("hot", 0, 1);
        let coldkey: T::AccountId = account("cold", 1, 2);

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

        let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work) =
            Subtensor::<T>::create_work_for_block_number(netuid, block_number, 3, &hotkey);

        let _ = Subtensor::<T>::register(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey.clone(),
            coldkey.clone(),
        );

        Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

        let commit_hash: H256 = BlakeTwo256::hash_of(&(
            hotkey.clone(),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        ));
        let _ = Subtensor::<T>::commit_weights(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            commit_hash,
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuid,
            uids.clone(),
            weight_values.clone(),
            salt.clone(),
            version_key,
        );
    }

    #[benchmark]
    fn sudo_set_tx_childkey_take_rate_limit() {
        let new_rate_limit: u64 = 100;

        #[extrinsic_call]
        _(RawOrigin::Root, new_rate_limit);
    }

    #[benchmark]
    fn set_childkey_take() {
        let netuid = NetUid::from(1);
        let coldkey: T::AccountId = account("Cold", 0, 1);
        let hotkey: T::AccountId = account("Hot", 0, 1);
        let take: u16 = 1000;

        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        let deposit = reg_fee.saturating_mul(2.into());
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit.into());

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            netuid,
            take,
        );
    }

    #[benchmark]
    fn announce_coldkey_swap() {
        let coldkey: T::AccountId = account("old_coldkey", 0, 0);
        let new_coldkey: T::AccountId = account("new_coldkey", 0, 0);
        let new_coldkey_hash: T::Hash = <T as frame_system::Config>::Hashing::hash_of(&new_coldkey);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), new_coldkey_hash);
    }

    #[benchmark]
    fn swap_coldkey_announced() {
        let old_coldkey: T::AccountId = account("old_coldkey", 0, 0);
        let new_coldkey: T::AccountId = account("new_coldkey", 0, 0);
        let new_coldkey_hash: T::Hash = <T as frame_system::Config>::Hashing::hash_of(&new_coldkey);
        let hotkey1: T::AccountId = account("hotkey1", 0, 0);

        let now = frame_system::Pallet::<T>::block_number();
        let delay = ColdkeySwapAnnouncementDelay::<T>::get();
        ColdkeySwapAnnouncements::<T>::insert(&old_coldkey, (now, new_coldkey_hash));
        frame_system::Pallet::<T>::set_block_number(now + delay);

        let swap_cost = Subtensor::<T>::get_key_swap_cost();
        Subtensor::<T>::add_balance_to_coldkey_account(&old_coldkey, swap_cost.into());

        let netuid = NetUid::from(1);
        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

        let block_number = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work) =
            Subtensor::<T>::create_work_for_block_number(netuid, block_number, 3, &hotkey1);
        let _ = Subtensor::<T>::register(
            RawOrigin::Signed(old_coldkey.clone()).into(),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey1.clone(),
            old_coldkey.clone(),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(old_coldkey), new_coldkey);
    }

    #[benchmark]
    fn swap_coldkey() {
        let old_coldkey: T::AccountId = account("old_coldkey", 0, 0);
        let new_coldkey: T::AccountId = account("new_coldkey", 0, 0);
        let hotkey1: T::AccountId = account("hotkey1", 0, 0);

        let swap_cost = Subtensor::<T>::get_key_swap_cost();
        Subtensor::<T>::add_balance_to_coldkey_account(&old_coldkey, swap_cost.into());

        let netuid = NetUid::from(1);
        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);

        let block_number = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work) =
            Subtensor::<T>::create_work_for_block_number(netuid, block_number, 3, &hotkey1);
        let _ = Subtensor::<T>::register(
            RawOrigin::Signed(old_coldkey.clone()).into(),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey1.clone(),
            old_coldkey.clone(),
        );

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            old_coldkey.clone(),
            new_coldkey.clone(),
            swap_cost,
        );
    }

    #[benchmark]
    fn remove_coldkey_swap_announcement() {
        let coldkey: T::AccountId = account("old_coldkey", 0, 0);
        let coldkey_hash: T::Hash = <T as frame_system::Config>::Hashing::hash_of(&coldkey);
        let now = frame_system::Pallet::<T>::block_number();

        ColdkeySwapAnnouncements::<T>::insert(&coldkey, (now, coldkey_hash));

        #[extrinsic_call]
        _(RawOrigin::Root, coldkey);
    }

    #[benchmark]
    fn batch_reveal_weights() {
        let tempo: u16 = 0;
        let netuid = NetUid::from(1);
        let num_commits: usize = 10;

        let hotkey: T::AccountId = account("hot", 0, 1);
        let coldkey: T::AccountId = account("cold", 0, 2);

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);
        Subtensor::<T>::set_weights_set_rate_limit(netuid, 0);

        let block_number: u64 = Subtensor::<T>::get_current_block_as_u64();
        let (nonce, work) =
            Subtensor::<T>::create_work_for_block_number(netuid, block_number, 3, &hotkey);
        let origin = T::RuntimeOrigin::from(RawOrigin::Signed(hotkey.clone()));
        assert_ok!(Subtensor::<T>::register(
            origin.clone(),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey.clone(),
            coldkey.clone()
        ));
        Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);

        let mut uids_list = Vec::new();
        let mut values_list = Vec::new();
        let mut salts_list = Vec::new();
        let mut version_keys = Vec::new();

        for i in 0..num_commits {
            let uids = vec![0u16];
            let values = vec![i as u16];
            let salts = vec![i as u16];
            let version_key_i: u64 = i as u64;

            let commit_hash: H256 = BlakeTwo256::hash_of(&(
                hotkey.clone(),
                netuid,
                uids.clone(),
                values.clone(),
                salts.clone(),
                version_key_i,
            ));

            assert_ok!(Subtensor::<T>::commit_weights(
                RawOrigin::Signed(hotkey.clone()).into(),
                netuid,
                commit_hash
            ));

            uids_list.push(uids);
            values_list.push(values);
            salts_list.push(salts);
            version_keys.push(version_key_i);
        }

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuid,
            uids_list,
            values_list,
            salts_list,
            version_keys,
        );
    }

    #[benchmark]
    fn recycle_alpha() {
        let netuid = NetUid::from(1);

        let coldkey: T::AccountId = account("Test", 0, 1);
        let hotkey: T::AccountId = account("Alice", 0, 1);

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());

        let amount_to_be_staked = 1_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);
        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        let alpha_amount = AlphaCurrency::from(1_000_000);
        SubnetAlphaOut::<T>::insert(netuid, alpha_amount * 2.into());

        Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha_amount,
        );

        assert_eq!(
            TotalHotkeyAlpha::<T>::get(&hotkey, netuid),
            alpha_amount.into()
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            alpha_amount,
            netuid,
        );
    }

    #[benchmark]
    fn burn_alpha() {
        let netuid = NetUid::from(1);
        let coldkey: T::AccountId = account("Test", 0, 1);
        let hotkey: T::AccountId = account("Alice", 0, 1);

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());

        let amount_to_be_staked: u64 = 1_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);
        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        let alpha_amount = 1_000_000;
        SubnetAlphaOut::<T>::insert(netuid, AlphaCurrency::from(alpha_amount * 2));
        Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha_amount.into(),
        );
        assert_eq!(
            TotalHotkeyAlpha::<T>::get(&hotkey, netuid),
            alpha_amount.into()
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            alpha_amount.into(),
            netuid,
        );
    }

    #[benchmark]
    fn start_call() {
        let netuid = NetUid::from(1);
        let coldkey: T::AccountId = account("Test", 0, 1);
        let hotkey: T::AccountId = account("Alice", 0, 1);

        Subtensor::<T>::init_new_network(netuid, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);

        Subtensor::<T>::set_burn(netuid, 1.into());
        let amount_to_be_staked = 1_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount_to_be_staked);
        SubnetOwner::<T>::set(netuid, coldkey.clone());

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));
        assert_eq!(SubnetOwner::<T>::get(netuid), coldkey.clone());
        assert_eq!(FirstEmissionBlockNumber::<T>::get(netuid), None);

        let current_block: u64 = Subtensor::<T>::get_current_block_as_u64();
        let duration = <T as Config>::DurationOfStartCall::get();
        let block: BlockNumberFor<T> = (current_block + duration)
            .try_into()
            .ok()
            .expect("can't convert to block number");
        frame_system::Pallet::<T>::set_block_number(block);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), netuid);
    }

    #[benchmark]
    fn add_stake_limit() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        let seed: u32 = 1;

        Subtensor::<T>::init_new_network(netuid, tempo);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_burn(netuid, 1.into());
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);

        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);

        let amount = 900_000_000_000;
        let limit = TaoCurrency::from(6_000_000_000);
        let amount_to_be_staked = TaoCurrency::from(44_000_000_000);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount);

        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<T>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<T>::insert(netuid, alpha_in);

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey,
            netuid,
            amount_to_be_staked,
            limit,
            false,
        );
    }

    #[benchmark]
    fn move_stake() {
        let coldkey: T::AccountId = whitelisted_caller();
        let origin: T::AccountId = account("A", 0, 1);
        let destination: T::AccountId = account("B", 0, 2);
        let netuid = NetUid::from(1);

        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::init_new_network(netuid, 1);

        let burn_fee = Subtensor::<T>::get_burn(netuid);
        let stake_tao = DefaultMinStake::<T>::get().saturating_mul(10.into());
        let deposit = burn_fee.saturating_mul(2.into()).saturating_add(stake_tao);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit.into());

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            origin.clone()
        ));

        SubnetTAO::<T>::insert(netuid, deposit);
        SubnetAlphaIn::<T>::insert(netuid, AlphaCurrency::from(deposit.to_u64()));
        TotalStake::<T>::set(deposit);

        assert_ok!(Subtensor::<T>::add_stake_limit(
            RawOrigin::Signed(coldkey.clone()).into(),
            origin.clone(),
            netuid,
            stake_tao,
            TaoCurrency::MAX,
            false
        ));

        let alpha_to_move =
            Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&origin, &coldkey, netuid);

        Subtensor::<T>::create_account_if_non_existent(&coldkey, &destination);

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((origin.clone(), coldkey.clone(), netuid));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            origin.clone(),
            destination.clone(),
            netuid,
            netuid,
            alpha_to_move,
        );
    }

    #[benchmark]
    fn remove_stake_limit() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        let seed: u32 = 1;

        // Set our total stake to 1000 TAO
        Subtensor::<T>::increase_total_stake(1_000_000_000_000.into());

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);
        Subtensor::<T>::set_burn(netuid, 1.into());

        let limit = TaoCurrency::from(1_000_000_000);
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<T>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<T>::insert(netuid, alpha_in);

        let wallet_bal = 1000000u32.into();
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), wallet_bal);

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        let u64_staked_amt = 100_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), u64_staked_amt);

        assert_ok!(Subtensor::<T>::add_stake(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            netuid,
            u64_staked_amt.into()
        ));

        let amount_unstaked = AlphaCurrency::from(30_000_000_000);

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((hotkey.clone(), coldkey.clone(), netuid));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            netuid,
            amount_unstaked,
            limit,
            false,
        );
    }

    #[benchmark]
    fn swap_stake_limit() {
        let coldkey: T::AccountId = whitelisted_caller::<AccountIdOf<T>>();
        let hot: T::AccountId = account("A", 0, 1);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let allow: bool = true;

        SubtokenEnabled::<T>::insert(netuid1, true);
        Subtensor::<T>::init_new_network(netuid1, 1);
        SubtokenEnabled::<T>::insert(netuid2, true);
        Subtensor::<T>::init_new_network(netuid2, 1);

        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<T>::insert(netuid1, tao_reserve);
        SubnetAlphaIn::<T>::insert(netuid1, alpha_in);
        SubnetTAO::<T>::insert(netuid2, tao_reserve);

        Subtensor::<T>::increase_total_stake(1_000_000_000_000.into());

        let amount = 900_000_000_000;
        let limit_stake = TaoCurrency::from(6_000_000_000);
        let limit_swap = TaoCurrency::from(1_000_000_000);
        let amount_to_be_staked = TaoCurrency::from(440_000_000_000);
        let amount_swapped = AlphaCurrency::from(30_000_000_000);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount);

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid1,
            hot.clone()
        ));

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid2,
            hot.clone()
        ));

        assert_ok!(Subtensor::<T>::add_stake_limit(
            RawOrigin::Signed(coldkey.clone()).into(),
            hot.clone(),
            netuid1,
            amount_to_be_staked,
            limit_stake,
            allow
        ));

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((hot.clone(), coldkey.clone(), netuid1));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hot.clone(),
            netuid1,
            netuid2,
            amount_swapped,
            limit_swap,
            allow,
        );
    }

    #[benchmark]
    fn transfer_stake() {
        let coldkey: T::AccountId = whitelisted_caller();
        let dest: T::AccountId = account("B", 0, 2);
        let hot: T::AccountId = account("A", 0, 1);
        let netuid = NetUid::from(1);

        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::init_new_network(netuid, 1);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        let stake_tao = DefaultMinStake::<T>::get().saturating_mul(10.into());
        let deposit = reg_fee.saturating_mul(2.into()).saturating_add(stake_tao);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit.into());

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hot.clone()
        ));

        SubnetTAO::<T>::insert(netuid, deposit);
        SubnetAlphaIn::<T>::insert(netuid, AlphaCurrency::from(deposit.to_u64()));
        TotalStake::<T>::set(deposit);

        assert_ok!(Subtensor::<T>::add_stake_limit(
            RawOrigin::Signed(coldkey.clone()).into(),
            hot.clone(),
            netuid,
            stake_tao,
            TaoCurrency::MAX,
            false
        ));

        let alpha_to_transfer =
            Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &coldkey, netuid);

        Subtensor::<T>::create_account_if_non_existent(&dest, &hot);

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((hot.clone(), coldkey.clone(), netuid));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            dest.clone(),
            hot.clone(),
            netuid,
            netuid,
            alpha_to_transfer,
        );
    }

    #[benchmark]
    fn swap_stake() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hot: T::AccountId = account("A", 0, 9);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        SubtokenEnabled::<T>::insert(netuid1, true);
        Subtensor::<T>::init_new_network(netuid1, 1);
        SubtokenEnabled::<T>::insert(netuid2, true);
        Subtensor::<T>::init_new_network(netuid2, 1);

        let reg_fee = Subtensor::<T>::get_burn(netuid1);
        let stake_tao = DefaultMinStake::<T>::get().saturating_mul(10.into());
        let deposit = reg_fee.saturating_mul(2.into()).saturating_add(stake_tao);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit.into());

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid1,
            hot.clone()
        ));

        SubnetTAO::<T>::insert(netuid1, deposit);
        SubnetAlphaIn::<T>::insert(netuid1, AlphaCurrency::from(deposit.to_u64()));
        SubnetTAO::<T>::insert(netuid2, deposit);
        SubnetAlphaIn::<T>::insert(netuid2, AlphaCurrency::from(deposit.to_u64()));
        TotalStake::<T>::set(deposit);

        assert_ok!(Subtensor::<T>::add_stake_limit(
            RawOrigin::Signed(coldkey.clone()).into(),
            hot.clone(),
            netuid1,
            stake_tao,
            TaoCurrency::MAX,
            false
        ));

        let alpha_to_swap =
            Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &coldkey, netuid1);

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((hot.clone(), coldkey.clone(), netuid1));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hot.clone(),
            netuid1,
            netuid2,
            alpha_to_swap,
        );
    }

    #[benchmark]
    fn batch_commit_weights() {
        let hotkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let count: usize = 3;
        let mut netuids: Vec<Compact<NetUid>> = Vec::new();
        let mut hashes: Vec<H256> = Vec::new();

        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        Subtensor::<T>::add_balance_to_coldkey_account(&hotkey, reg_fee.to_u64().saturating_mul(2));

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        Subtensor::<T>::set_validator_permit_for_uid(netuid, 0, true);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

        for i in 0..count {
            netuids.push(Compact(netuid));
            hashes.push(H256::repeat_byte(i as u8));
        }

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuids.clone(),
            hashes.clone(),
        );
    }

    #[benchmark]
    fn batch_set_weights() {
        let hotkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let version: u64 = 1;
        let entries: Vec<(Compact<u16>, Compact<u16>)> = vec![(Compact(0u16), Compact(0u16))];
        let netuids: Vec<Compact<NetUid>> = vec![Compact(netuid)];
        let weights: Vec<Vec<(Compact<u16>, Compact<u16>)>> = vec![entries.clone()];
        let keys: Vec<Compact<u64>> = vec![Compact(version)];

        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, false);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        Subtensor::<T>::add_balance_to_coldkey_account(&hotkey, reg_fee.to_u64().saturating_mul(2));

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuids.clone(),
            weights.clone(),
            keys.clone(),
        );
    }

    #[benchmark]
    fn decrease_take() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("Alice", 0, 1);
        let take: u16 = 100;

        Delegates::<T>::insert(&hotkey, 200u16);
        Owner::<T>::insert(&hotkey, &coldkey);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), take);
    }

    #[benchmark]
    fn increase_take() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("Alice", 0, 2);
        let take: u16 = 150;

        Delegates::<T>::insert(&hotkey, 100u16);
        Owner::<T>::insert(&hotkey, &coldkey);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone(), take);
    }

    #[benchmark]
    fn register_network_with_identity() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("Alice", 0, 1);
        let identity: Option<SubnetIdentityOfV3> = None;

        Subtensor::<T>::set_network_registration_allowed(1.into(), true);
        Subtensor::<T>::set_network_rate_limit(1);
        let amount: u64 = 9_999_999_999_999;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, amount);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            identity.clone(),
        );
    }

    #[benchmark]
    fn serve_axon_tls() {
        let caller: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let version: u32 = 1;
        let ip: u128 = 0xC0A8_0001;
        let port: u16 = 30333;
        let ip_type: u8 = 4;
        let proto: u8 = 0;
        let p1: u8 = 0;
        let p2: u8 = 0;
        let cert: Vec<u8> = vec![];

        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        let deposit = reg_fee.saturating_mul(2.into());
        Subtensor::<T>::add_balance_to_coldkey_account(&caller, deposit.into());

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(caller.clone()).into(),
            netuid,
            caller.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            netuid,
            version,
            ip,
            port,
            ip_type,
            proto,
            p1,
            p2,
            cert.clone(),
        );
    }

    #[benchmark]
    fn set_identity() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("Alice", 0, 5);
        let name = b"n".to_vec();
        let url = vec![];
        let repo = vec![];
        let img = vec![];
        let disc = vec![];
        let descr = vec![];
        let add = vec![];

        Subtensor::<T>::create_account_if_non_existent(&coldkey, &hotkey);
        Subtensor::<T>::init_new_network(1.into(), 1);
        let deposit: u64 = 1_000_000_000u64.saturating_mul(2);
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, deposit);
        SubtokenEnabled::<T>::insert(NetUid::from(1), true);

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            1.into(),
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            name.clone(),
            url.clone(),
            repo.clone(),
            img.clone(),
            disc.clone(),
            descr.clone(),
            add.clone(),
        );
    }

    #[benchmark]
    fn set_subnet_identity() {
        let coldkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let name = b"n".to_vec();
        let repo = vec![];
        let contact = vec![];
        let url = vec![];
        let disc = vec![];
        let descr = vec![];
        let logo_url = vec![];
        let add = vec![];

        SubnetOwner::<T>::insert(netuid, coldkey.clone());
        SubtokenEnabled::<T>::insert(netuid, true);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            netuid,
            name.clone(),
            repo.clone(),
            contact.clone(),
            url.clone(),
            disc.clone(),
            descr.clone(),
            logo_url.clone(),
            add.clone(),
        );
    }

    #[benchmark]
    fn swap_hotkey() {
        let coldkey: T::AccountId = whitelisted_caller();
        let old: T::AccountId = account("A", 0, 7);
        let new: T::AccountId = account("B", 0, 8);
        Owner::<T>::insert(&old, &coldkey);
        let cost = Subtensor::<T>::get_key_swap_cost();
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, cost.into());

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            old.clone(),
            new.clone(),
            None,
        );
    }

    #[benchmark]
    fn try_associate_hotkey() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hot: T::AccountId = account("A", 0, 1);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hot.clone());
    }

    #[benchmark]
    fn unstake_all() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("A", 0, 14);
        Subtensor::<T>::create_account_if_non_existent(&coldkey, &hotkey);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), hotkey.clone());
    }

    #[benchmark]
    fn unstake_all_alpha() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        let seed: u32 = 1;

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);
        Subtensor::<T>::set_burn(netuid, 1.into());

        SubnetTAO::<T>::insert(netuid, TaoCurrency::from(150_000_000_000));
        SubnetAlphaIn::<T>::insert(netuid, AlphaCurrency::from(100_000_000_000));

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), 1000000u32.into());

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        let staked_amt = 100_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), staked_amt);

        assert_ok!(Subtensor::<T>::add_stake(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            netuid,
            staked_amt.into()
        ));

        // Remove stake limit for benchmark
        StakingOperationRateLimiter::<T>::remove((hotkey.clone(), coldkey.clone(), netuid));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), hotkey);
    }

    #[benchmark]
    fn remove_stake_full_limit() {
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        let seed: u32 = 1;

        // Set our total stake to 1000 TAO
        Subtensor::<T>::increase_total_stake(1_000_000_000_000.into());

        Subtensor::<T>::init_new_network(netuid, tempo);
        Subtensor::<T>::set_network_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        Subtensor::<T>::set_max_allowed_uids(netuid, 4096);
        assert_eq!(Subtensor::<T>::get_max_allowed_uids(netuid), 4096);

        let coldkey: T::AccountId = account("Test", 0, seed);
        let hotkey: T::AccountId = account("Alice", 0, seed);
        Subtensor::<T>::set_burn(netuid, 1.into());

        let limit = TaoCurrency::from(1_000_000_000);
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<T>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<T>::insert(netuid, alpha_in);

        let wallet_bal = 1000000u32.into();
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), wallet_bal);

        assert_ok!(Subtensor::<T>::do_burned_registration(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        let u64_staked_amt = 100_000_000_000;
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), u64_staked_amt);

        assert_ok!(Subtensor::<T>::add_stake(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            netuid,
            u64_staked_amt.into()
        ));

        StakingOperationRateLimiter::<T>::remove((hotkey.clone(), coldkey.clone(), netuid));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            netuid,
            Some(limit),
        );
    }

    #[benchmark(extra)]
    fn register_leased_network(k: Linear<2, { T::MaxContributors::get() }>) {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary: T::AccountId = whitelisted_caller();
        let deposit = 20_000_000_000; // 20 TAO
        let now = frame_system::Pallet::<T>::block_number(); // not really important here
        let end = now + T::MaximumBlockDuration::get();
        let cap = 2_000_000_000_000; // 2000 TAO

        let funds_account: T::AccountId = account("funds", 0, 0);
        Subtensor::<T>::add_balance_to_coldkey_account(&funds_account, cap);

        pallet_crowdloan::Crowdloans::<T>::insert(
            crowdloan_id,
            pallet_crowdloan::CrowdloanInfo {
                creator: beneficiary.clone(),
                deposit,
                min_contribution: 0,
                end,
                cap,
                raised: cap,
                finalized: false,
                funds_account: funds_account.clone(),
                call: None,
                target_address: None,
                contributors_count: T::MaxContributors::get(),
            },
        );

        // Set the block to the end of the crowdloan
        frame_system::Pallet::<T>::set_block_number(end);

        // Simulate deposit
        pallet_crowdloan::Contributions::<T>::insert(crowdloan_id, &beneficiary, deposit);

        // Simulate k - 1 contributions, the deposit is already taken into account
        let contributors = k - 1;
        let amount = (cap - deposit) / contributors as u64;
        for i in 0..contributors {
            let contributor = account::<T::AccountId>("contributor", i.try_into().unwrap(), 0);
            pallet_crowdloan::Contributions::<T>::insert(crowdloan_id, contributor, amount);
        }

        // Mark the crowdloan as finalizing
        pallet_crowdloan::CurrentCrowdloanId::<T>::set(Some(0));

        let emissions_share = Percent::from_percent(30);
        #[extrinsic_call]
        _(
            RawOrigin::Signed(beneficiary.clone()),
            emissions_share,
            None,
        );

        // Ensure the lease was created
        let lease_id = 0;
        let lease = SubnetLeases::<T>::get(lease_id).unwrap();
        assert_eq!(lease.beneficiary, beneficiary);
        assert_eq!(lease.emissions_share, emissions_share);
        assert_eq!(lease.end_block, None);

        // Ensure the subnet exists
        assert!(SubnetMechanism::<T>::contains_key(lease.netuid));
    }

    #[benchmark(extra)]
    fn terminate_lease(k: Linear<2, { T::MaxContributors::get() }>) {
        // Setup a crowdloan
        let crowdloan_id = 0;
        let beneficiary: T::AccountId = whitelisted_caller();
        let deposit = 20_000_000_000; // 20 TAO
        let now = frame_system::Pallet::<T>::block_number(); // not really important here
        let crowdloan_end = now + T::MaximumBlockDuration::get();
        let cap = 2_000_000_000_000; // 2000 TAO

        let funds_account: T::AccountId = account("funds", 0, 0);
        Subtensor::<T>::add_balance_to_coldkey_account(&funds_account, cap);

        pallet_crowdloan::Crowdloans::<T>::insert(
            crowdloan_id,
            pallet_crowdloan::CrowdloanInfo {
                creator: beneficiary.clone(),
                deposit,
                min_contribution: 0,
                end: crowdloan_end,
                cap,
                raised: cap,
                finalized: false,
                funds_account: funds_account.clone(),
                call: None,
                target_address: None,
                contributors_count: T::MaxContributors::get(),
            },
        );

        // Set the block to the end of the crowdloan
        frame_system::Pallet::<T>::set_block_number(crowdloan_end);

        // Simulate deposit
        pallet_crowdloan::Contributions::<T>::insert(crowdloan_id, &beneficiary, deposit);

        // Simulate k - 1 contributions, the deposit is already taken into account
        let contributors = k - 1;
        let amount = (cap - deposit) / contributors as u64;
        for i in 0..contributors {
            let contributor = account::<T::AccountId>("contributor", i.try_into().unwrap(), 0);
            pallet_crowdloan::Contributions::<T>::insert(crowdloan_id, contributor, amount);
        }

        // Mark the crowdloan as finalizing
        pallet_crowdloan::CurrentCrowdloanId::<T>::set(Some(0));

        // Register the leased network
        let emissions_share = Percent::from_percent(30);
        let lease_end = crowdloan_end + 1000u32.into();
        assert_ok!(Subtensor::<T>::register_leased_network(
            RawOrigin::Signed(beneficiary.clone()).into(),
            emissions_share,
            Some(lease_end),
        ));

        // Set the block to the end of the lease
        frame_system::Pallet::<T>::set_block_number(lease_end);

        let lease_id = 0;
        let lease = SubnetLeases::<T>::get(0).unwrap();
        let hotkey = account::<T::AccountId>("beneficiary_hotkey", 0, 0);
        Subtensor::<T>::create_account_if_non_existent(&beneficiary, &hotkey);
        #[extrinsic_call]
        _(
            RawOrigin::Signed(beneficiary.clone()),
            lease_id,
            hotkey.clone(),
        );

        // Ensure the beneficiary is now the owner of the subnet
        assert_eq!(SubnetOwner::<T>::get(lease.netuid), beneficiary);
        assert_eq!(SubnetOwnerHotkey::<T>::get(lease.netuid), hotkey);

        // Ensure everything has been cleaned up
        assert_eq!(SubnetLeases::<T>::get(lease_id), None);
        assert!(!SubnetLeaseShares::<T>::contains_prefix(lease_id));
        assert!(!AccumulatedLeaseDividends::<T>::contains_key(lease_id));
    }

    #[benchmark]
    fn update_symbol() {
        let coldkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let tempo: u16 = 1;
        Subtensor::<T>::init_new_network(netuid, tempo);
        SubnetOwner::<T>::insert(netuid, coldkey.clone());

        let new_symbol = Subtensor::<T>::get_symbol_for_subnet(NetUid::from(42));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey), netuid, new_symbol.clone());

        assert_eq!(TokenSymbol::<T>::get(netuid), new_symbol);
    }

    #[benchmark]
    fn commit_timelocked_weights() {
        let hotkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let vec_commit: Vec<u8> = vec![0; MAX_CRV3_COMMIT_SIZE_BYTES as usize];
        let commit: BoundedVec<_, _> = vec_commit.try_into().unwrap();
        let round: u64 = 0;

        Subtensor::<T>::init_new_network(netuid, 1);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
        SubtokenEnabled::<T>::insert(netuid, true);

        let reg_fee = Subtensor::<T>::get_burn(netuid);
        Subtensor::<T>::add_balance_to_coldkey_account(
            &hotkey,
            reg_fee.saturating_mul(2.into()).into(),
        );

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(hotkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        Subtensor::<T>::set_commit_reveal_weights_enabled(netuid, true);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(hotkey.clone()),
            netuid,
            commit.clone(),
            round,
            Subtensor::<T>::get_commit_reveal_weights_version(),
        );
    }

    #[benchmark]
    fn set_coldkey_auto_stake_hotkey() {
        let coldkey: T::AccountId = whitelisted_caller();
        let netuid = NetUid::from(1);
        let hotkey: T::AccountId = account("A", 0, 1);
        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::init_new_network(netuid, 1);
        let amount = 900_000_000_000;

        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount);

        assert_ok!(Subtensor::<T>::burned_register(
            RawOrigin::Signed(coldkey.clone()).into(),
            netuid,
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), netuid, hotkey.clone());
    }
    #[benchmark]
    fn set_root_claim_type() {
        let coldkey: T::AccountId = whitelisted_caller();

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), RootClaimTypeEnum::Keep);
    }

    #[benchmark]
    fn claim_root() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("A", 0, 1);

        let netuid = Subtensor::<T>::get_next_netuid();

        let lock_cost = Subtensor::<T>::get_network_lock_cost();
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, lock_cost.into());

        assert_ok!(Subtensor::<T>::register_network(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone()
        ));

        SubtokenEnabled::<T>::insert(netuid, true);
        Subtensor::<T>::set_network_pow_registration_allowed(netuid, true);
        NetworkRegistrationAllowed::<T>::insert(netuid, true);
        FirstEmissionBlockNumber::<T>::insert(netuid, 0);

        SubnetMechanism::<T>::insert(netuid, 1);
        SubnetworkN::<T>::insert(netuid, 1);
        Subtensor::<T>::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 100_000_000u64;
        Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 100_000_000u64;
        Subtensor::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let pending_root_alpha = 10_000_000u64;
        Subtensor::<T>::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        let initial_stake =
            Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        assert_ok!(Subtensor::<T>::set_root_claim_type(
            RawOrigin::Signed(coldkey.clone()).into(),
            RootClaimTypeEnum::Keep
        ),);

        #[extrinsic_call]
        _(RawOrigin::Signed(coldkey.clone()), BTreeSet::from([netuid]));

        // Verification
        let new_stake =
            Subtensor::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        assert!(new_stake > initial_stake);
    }

    #[benchmark]
    fn sudo_set_num_root_claims() {
        #[extrinsic_call]
        _(RawOrigin::Root, 40);
    }

    #[benchmark]
    fn sudo_set_root_claim_threshold() {
        let coldkey: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("A", 0, 1);

        let netuid = Subtensor::<T>::get_next_netuid();

        let lock_cost = Subtensor::<T>::get_network_lock_cost();
        Subtensor::<T>::add_balance_to_coldkey_account(&coldkey, lock_cost.into());

        assert_ok!(Subtensor::<T>::register_network(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone()
        ));

        #[extrinsic_call]
        _(RawOrigin::Root, netuid, 100);
    }
}
