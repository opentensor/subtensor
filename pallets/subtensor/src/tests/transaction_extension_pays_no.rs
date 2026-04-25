//! Transaction extension coverage for extrinsics handled by [`crate::extensions::SubtensorTransactionExtension`].

#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::extensions::SubtensorTransactionExtension;
use crate::*;
use codec::Compact;
use frame_support::dispatch::GetDispatchInfo;
use frame_support::{BoundedVec, assert_ok, traits::ConstU32};
use frame_system::RawOrigin;
use pallet_drand::LastStoredRound;
use sp_core::H256;
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError,
};
use subtensor_runtime_common::{CustomTransactionError, MechId, NetUid, TaoBalance};

fn dispatch_info() -> sp_runtime::traits::DispatchInfoOf<<Test as frame_system::Config>::RuntimeCall>
{
    DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default()
}

fn validate_signed(
    signer: U256,
    call: &RuntimeCall,
) -> Result<sp_runtime::transaction_validity::ValidTransaction, TransactionValidityError> {
    SubtensorTransactionExtension::<Test>::new()
        .validate(
            RawOrigin::Signed(signer).into(),
            call,
            &dispatch_info(),
            0,
            (),
            &TxBaseImplication(()),
            TransactionSource::External,
        )
        .map(|(v, _, _)| v)
}

#[test]
fn extension_set_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network_disable_commit_reveal(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests: vec![1],
            weights: vec![1],
            version_key: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_set_mechanism_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network_disable_commit_reveal(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_mechanism_weights {
            netuid,
            mecid: MechId::MAIN,
            dests: vec![1],
            weights: vec![1],
            version_key: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_batch_set_weights_rejects_mismatched_lengths() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::batch_set_weights {
            netuids: vec![Compact(netuid)],
            weights: vec![],
            version_keys: vec![Compact(0_u64)],
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InputLengthsUnequal.into());
    });
}

#[test]
fn extension_batch_set_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network_disable_commit_reveal(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::batch_set_weights {
            netuids: vec![Compact(netuid)],
            weights: vec![vec![(Compact(0u16), Compact(1u16))]],
            version_keys: vec![Compact(0u64)],
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_commit_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_weights {
            netuid,
            commit_hash: H256::zero(),
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_commit_mechanism_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_mechanism_weights {
            netuid,
            mecid: MechId::MAIN,
            commit_hash: H256::zero(),
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_batch_commit_weights_rejects_mismatched_lengths() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::batch_commit_weights {
            netuids: vec![Compact(netuid)],
            commit_hashes: vec![],
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InputLengthsUnequal.into());
    });
}

#[test]
fn extension_reveal_weights_rejects_stake_too_low() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(1_000_000_000_000u64);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_weights {
            netuid,
            uids: vec![0],
            values: vec![1],
            salt: vec![1],
            version_key: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::StakeAmountTooLow.into());
    });
}

#[test]
fn extension_reveal_weights_rejects_commit_not_found() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(0);
        SubtensorModule::add_balance_to_coldkey_account(&hotkey, u64::MAX.into());
        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(hotkey),
            hotkey,
            netuid,
            TaoBalance::from(500_000_000_000_u64)
        ));

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_weights {
            netuid,
            uids: vec![0],
            values: vec![1],
            salt: vec![1],
            version_key: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::CommitNotFound.into());
    });
}

#[test]
fn extension_reveal_mechanism_weights_rejects_commit_not_found() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(0);
        SubtensorModule::add_balance_to_coldkey_account(&hotkey, u64::MAX.into());
        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(hotkey),
            hotkey,
            netuid,
            TaoBalance::from(500_000_000_000_u64)
        ));

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_mechanism_weights {
            netuid,
            mecid: MechId::MAIN,
            uids: vec![0],
            values: vec![1],
            salt: vec![1],
            version_key: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::CommitNotFound.into());
    });
}

#[test]
fn extension_batch_reveal_weights_rejects_mismatched_vector_lengths() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 1, 0);
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        SubtensorModule::set_stake_threshold(0);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::batch_reveal_weights {
            netuid,
            uids_list: vec![vec![0]],
            values_list: vec![],
            salts_list: vec![vec![1]],
            version_keys: vec![0],
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InputLengthsUnequal.into());
    });
}

#[test]
fn extension_commit_timelocked_weights_rejects_invalid_reveal_round() {
    new_test_ext(0).execute_with(|| {
        LastStoredRound::<Test>::put(1_000_u64);
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::set_stake_threshold(0);

        let commit =
            BoundedVec::<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>::try_from(vec![0u8]).unwrap();
        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_weights {
            netuid,
            commit,
            reveal_round: 500,
            commit_reveal_version: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InvalidRevealRound.into());
    });
}

#[test]
fn extension_commit_timelocked_mechanism_weights_rejects_invalid_reveal_round() {
    new_test_ext(0).execute_with(|| {
        LastStoredRound::<Test>::put(2_000_u64);
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::set_stake_threshold(0);

        let commit =
            BoundedVec::<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>::try_from(vec![1u8]).unwrap();
        let call =
            RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_mechanism_weights {
                netuid,
                mecid: MechId::MAIN,
                commit,
                reveal_round: 100,
                commit_reveal_version: 0,
            });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InvalidRevealRound.into());
    });
}

#[test]
fn extension_commit_crv3_mechanism_weights_rejects_invalid_reveal_round() {
    new_test_ext(0).execute_with(|| {
        LastStoredRound::<Test>::put(500u64);
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::set_stake_threshold(0);

        let commit =
            BoundedVec::<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>::try_from(vec![2u8]).unwrap();
        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_crv3_mechanism_weights {
            netuid,
            mecid: MechId::MAIN,
            commit,
            reveal_round: 100,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::InvalidRevealRound.into());
    });
}

#[test]
fn extension_decrease_take_rejects_non_owner_coldkey() {
    new_test_ext(0).execute_with(|| {
        let owner_ck = U256::from(1);
        let other_ck = U256::from(2);
        let hotkey = U256::from(3);
        crate::Owner::<Test>::insert(hotkey, owner_ck);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::decrease_take {
            hotkey,
            take: MinDelegateTake::<Test>::get(),
        });
        let err = validate_signed(other_ck, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::NonAssociatedColdKey.into());
    });
}

#[test]
fn extension_decrease_take_rejects_missing_hotkey_owner() {
    new_test_ext(0).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(99);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::decrease_take {
            hotkey,
            take: MinDelegateTake::<Test>::get(),
        });
        let err = validate_signed(coldkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::HotkeyAccountDoesntExist.into());
    });
}

#[test]
fn extension_increase_take_rejects_non_owner_coldkey() {
    new_test_ext(0).execute_with(|| {
        let owner_ck = U256::from(10);
        let other_ck = U256::from(11);
        let hotkey = U256::from(12);
        crate::Owner::<Test>::insert(hotkey, owner_ck);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::increase_take { hotkey, take: 100 });
        let err = validate_signed(other_ck, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::NonAssociatedColdKey.into());
    });
}

#[test]
fn extension_swap_hotkey_v2_rejects_non_owner_coldkey() {
    new_test_ext(0).execute_with(|| {
        let owner_ck = U256::from(20);
        let other_ck = U256::from(21);
        let old_hk = U256::from(22);
        let new_hk = U256::from(23);
        crate::Owner::<Test>::insert(old_hk, owner_ck);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::swap_hotkey_v2 {
            hotkey: old_hk,
            new_hotkey: new_hk,
            netuid: None,
            keep_stake: false,
        });
        let err = validate_signed(other_ck, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::NonAssociatedColdKey.into());
    });
}

#[test]
fn extension_swap_hotkey_v2_rejects_rate_limited() {
    new_test_ext(0).execute_with(|| {
        let coldkey = U256::from(30);
        let old_hk = U256::from(31);
        let new_hk = U256::from(32);
        crate::Owner::<Test>::insert(old_hk, coldkey);

        SubtensorModule::set_tx_rate_limit(100);
        SubtensorModule::set_last_tx_block(&coldkey, 1);
        System::set_block_number(1u64.into());

        let err = SubtensorTransactionExtension::<Test>::new()
            .validate(
                RawOrigin::Signed(coldkey).into(),
                &RuntimeCall::SubtensorModule(SubtensorCall::swap_hotkey_v2 {
                    hotkey: old_hk,
                    new_hotkey: new_hk,
                    netuid: None,
                    keep_stake: false,
                }),
                &dispatch_info(),
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(err, CustomTransactionError::RateLimitExceeded.into());
    });
}

#[test]
fn extension_serve_axon_rejects_unregistered_hotkey() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(40);
        let ip = u128::from(u32::from_be_bytes([8, 8, 8, 8]));
        let call = RuntimeCall::SubtensorModule(SubtensorCall::serve_axon {
            netuid,
            version: 1,
            ip,
            port: 1,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(
            err,
            CustomTransactionError::HotKeyNotRegisteredInNetwork.into()
        );
    });
}

#[test]
fn extension_serve_axon_tls_rejects_unregistered_hotkey() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(41);
        let ip = u128::from(u32::from_be_bytes([8, 8, 8, 8]));
        let call = RuntimeCall::SubtensorModule(SubtensorCall::serve_axon_tls {
            netuid,
            version: 1,
            ip,
            port: 1,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
            certificate: vec![],
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(
            err,
            CustomTransactionError::HotKeyNotRegisteredInNetwork.into()
        );
    });
}

#[test]
fn extension_serve_prometheus_rejects_unregistered_hotkey() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(99);
        let ip = u128::from(u32::from_be_bytes([8, 8, 8, 8]));
        let call = RuntimeCall::SubtensorModule(SubtensorCall::serve_prometheus {
            netuid,
            version: 1,
            ip,
            port: 1,
            ip_type: 4,
        });
        let info = call.get_dispatch_info();
        assert_eq!(info.pays_fee, frame_support::dispatch::Pays::No);

        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(
            err,
            CustomTransactionError::HotKeyNotRegisteredInNetwork.into()
        );
    });
}

#[test]
fn extension_associate_evm_key_rejects_uid_not_found() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey = U256::from(50);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::associate_evm_key {
            netuid,
            evm_key: sp_core::H160::zero(),
            block_number: 0,
            signature: sp_core::ecdsa::Signature::from_raw([0u8; 65]),
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::UidNotFound.into());
    });
}

#[test]
fn extension_add_stake_burn_rejects_not_subnet_owner() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let owner = U256::from(60);
        let not_owner = U256::from(61);
        let hotkey = U256::from(62);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, owner);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake_burn {
            hotkey,
            netuid,
            amount: TaoBalance::from(1u64),
            limit: None,
        });
        let err = SubtensorTransactionExtension::<Test>::new()
            .validate(
                RawOrigin::Signed(not_owner).into(),
                &call,
                &dispatch_info(),
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(
            err,
            TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
        );
    });
}

#[test]
fn extension_register_network_rejects_global_rate_limit() {
    new_test_ext(0).execute_with(|| {
        let limit = 50u64;
        NetworkRateLimit::<Test>::put(limit);
        System::set_block_number(200u64.into());
        SubtensorModule::set_network_last_lock_block(170);

        let coldkey = U256::from(70);
        let hotkey = U256::from(71);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::register_network { hotkey });
        let err = validate_signed(coldkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::RateLimitExceeded.into());
    });
}

#[test]
fn extension_register_network_accepts_after_global_cooldown() {
    new_test_ext(0).execute_with(|| {
        let limit = 50u64;
        NetworkRateLimit::<Test>::put(limit);
        System::set_block_number(200u64.into());
        SubtensorModule::set_network_last_lock_block(150);

        let coldkey = U256::from(72);
        let hotkey = U256::from(73);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::register_network { hotkey });
        assert!(validate_signed(coldkey, &call).is_ok());
    });
}

#[test]
fn extension_associate_evm_key_rejects_associate_rate_limit() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 2;
        let modality: u16 = 2;
        add_network(netuid, tempo, modality);

        let coldkey = U256::from(80);
        let hotkey = U256::from(81);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();
        System::set_block_number(300u64.into());
        let now = SubtensorModule::get_current_block_as_u64();
        AssociatedEvmAddress::<Test>::insert(netuid, uid, (sp_core::H160::zero(), now));

        let call = RuntimeCall::SubtensorModule(SubtensorCall::associate_evm_key {
            netuid,
            evm_key: sp_core::H160::zero(),
            block_number: 0,
            signature: sp_core::ecdsa::Signature::from_raw([0u8; 65]),
        });
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(
            err,
            CustomTransactionError::EvmKeyAssociateRateLimitExceeded.into()
        );
    });
}

#[test]
fn extension_add_stake_burn_boosts_priority_for_subnet_owner() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let owner = U256::from(90);
        let hotkey = U256::from(91);
        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, owner);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake_burn {
            hotkey,
            netuid,
            amount: TaoBalance::from(1u64),
            limit: None,
        });
        let v = validate_signed(owner, &call).unwrap();
        assert_eq!(v.priority, 100);
    });
}
