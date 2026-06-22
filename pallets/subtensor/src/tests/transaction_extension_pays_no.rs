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
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};
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
        add_balance_to_coldkey_account(&hotkey, 1_000_000_000_000_u64.into());
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
        add_balance_to_coldkey_account(&hotkey, 1_000_000_000_000_u64.into());
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
fn extension_reveal_mechanism_weights_accepts_valid_commit() {
    assert_reveal_mechanism_weights_accepts_valid_commit(MechId::MAIN, None);
}

#[test]
fn extension_reveal_mechanism_weights_accepts_valid_non_main_mechanism_commit() {
    assert_reveal_mechanism_weights_accepts_valid_commit(
        MechId::from(1u8),
        Some(MechId::from(2u8)),
    );
}

fn assert_reveal_mechanism_weights_accepts_valid_commit(
    mecid: MechId,
    mechanism_count: Option<MechId>,
) {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let uids = vec![0];
        let values = vec![1];
        let salt = vec![1];
        let version_key = 0;
        add_network(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        if let Some(mechanism_count) = mechanism_count {
            MechanismCountCurrent::<Test>::insert(netuid, mechanism_count);
        }
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
        SubtensorModule::set_stake_threshold(0);
        add_balance_to_coldkey_account(&hotkey, 1_000_000_000_000_u64.into());
        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(hotkey),
            hotkey,
            netuid,
            TaoBalance::from(500_000_000_000_u64)
        ));

        let commit_hash = SubtensorModule::get_commit_hash(
            &hotkey,
            SubtensorModule::get_mechanism_storage_index(netuid, mecid),
            &uids,
            &values,
            &salt,
            version_key,
        );
        assert_ok!(SubtensorModule::commit_mechanism_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            mecid,
            commit_hash
        ));
        step_epochs(1, netuid);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_mechanism_weights {
            netuid,
            mecid,
            uids,
            values,
            salt,
            version_key,
        });
        assert_ok!(validate_signed(hotkey, &call));
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

        let call = RuntimeCall::SubtensorModule(SubtensorCall::increase_take {
            hotkey,
            take: SubtensorModule::get_min_delegate_take(),
        });
        let err = validate_signed(other_ck, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::NonAssociatedColdKey.into());
    });
}

#[test]
fn extension_increase_take_validates_take_bounds() {
    new_test_ext(0).execute_with(|| {
        let coldkey = U256::from(13);
        let hotkey = U256::from(14);
        crate::Owner::<Test>::insert(hotkey, coldkey);
        let min_take = SubtensorModule::get_min_delegate_take();
        let max_take = SubtensorModule::get_max_delegate_take();
        let increase_take_call =
            |take| RuntimeCall::SubtensorModule(SubtensorCall::increase_take { hotkey, take });

        let too_low_call = increase_take_call(min_take - 1);
        let err = validate_signed(coldkey, &too_low_call).unwrap_err();
        assert_eq!(err, CustomTransactionError::DelegateTakeTooLow.into());

        let in_scope_call = increase_take_call(min_take);
        assert_ok!(validate_signed(coldkey, &in_scope_call));

        let too_high_call = increase_take_call(max_take + 1);
        let err = validate_signed(coldkey, &too_high_call).unwrap_err();
        assert_eq!(err, CustomTransactionError::DelegateTakeTooHigh.into());
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
        let _ = SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
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

// ============================================================
// GHSA-2026-006 regression test — security audit (June 2026)
// Fails on the vulnerable code; passes with the fix in this PR.
// ============================================================
use frame_support::assert_err;

#[test]
fn ghsa_2026_006_set_weights_paysno_validate_omits_ratelimit() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Subnet with commit-reveal disabled so do_set_weights runs the
        // per-neuron SetWeightsRateLimit check (weights.rs step 9).
        add_network_disable_commit_reveal(netuid, 1, 0);
        setup_reserves(
            netuid,
            1_000_000_000_000_u64.into(),
            1_000_000_000_000_u64.into(),
        );
        // Register a real neuron (uid 0) so it exists on-network and its
        // LastUpdate vector is sized for set_last_update_for_uid below.
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();

        // Drop the min-stake threshold to 0 so the ONLY thing that validate()
        // could reject for is the rate limit. This isolates the fix: the
        // min-stake mempool gate passes, and the rate-limit gate must now also
        // be enforced in validate().
        SubtensorModule::set_stake_threshold(0);
        assert!(SubtensorModule::check_weights_min_stake(&hotkey, netuid));

        // Configure a non-zero per-neuron rate limit and mark this neuron as
        // having "just" set weights at the current block, so the next
        // set_weights is over-rate.
        SubtensorModule::set_weights_set_rate_limit(netuid, 100);
        System::set_block_number(10u64.into());
        let current_block = SubtensorModule::get_current_block_as_u64();
        let netuid_index = SubtensorModule::get_mechanism_storage_index(netuid, MechId::MAIN);
        SubtensorModule::set_last_update_for_uid(netuid_index, uid, current_block);

        // Sanity: the in-dispatch rate-limit helper now reports over-rate.
        assert!(!SubtensorModule::check_rate_limit(
            netuid_index,
            uid,
            current_block
        ));

        // Self-weight call (uids/weights == [uid]/[1]) avoids needing a
        // validator permit, so dispatch reaches the rate-limit gate.
        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests: vec![uid],
            weights: vec![1],
            version_key: 0,
        });

        // (a) set_weights is declared Pays::No -> if validate accepted it, it
        //     would be included into a block for free.
        let info = call.get_dispatch_info();
        assert_eq!(info.pays_fee, frame_support::dispatch::Pays::No);

        // (b) THE FIX: SubtensorTransactionExtension::validate now enforces the
        //     per-neuron SetWeightsRateLimit. An over-rate set_weights is
        //     rejected pre-dispatch with RateLimitExceeded, so it can never be
        //     admitted to the mempool / included for free.
        let err = validate_signed(hotkey, &call).unwrap_err();
        assert_eq!(err, CustomTransactionError::RateLimitExceeded.into());

        // (c) The dispatch path still enforces the rate limit as the
        //     authoritative check (defence in depth for any tx that slips past
        //     the pool-level filter, e.g. two over-rate txs in the same block).
        assert_err!(
            SubtensorModule::set_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                vec![uid],
                vec![1],
                0,
            ),
            Error::<Test>::SettingWeightsTooFast
        );
    });
}
