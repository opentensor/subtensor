//! Transaction extension coverage for `Pays::No` subtensor calls.

#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::extensions::SubtensorTransactionExtension;
use crate::*;
use codec::Compact;
use frame_support::dispatch::GetDispatchInfo;
use frame_system::RawOrigin;
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;
use subtensor_runtime_common::{CustomTransactionError, NetUid};

#[test]
fn extension_set_weights_rejects_commit_reveal_enabled() {
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
        SubtensorModule::add_balance_to_coldkey_account(&hotkey, u64::MAX.into());
        SubtensorModule::set_stake_threshold(0);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests: vec![1],
            weights: vec![1],
            version_key: 0,
        });
        let info = DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorTransactionExtension::<Test>::new();
        let err = extension
            .validate(
                RawOrigin::Signed(hotkey).into(),
                &call,
                &info,
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(err, CustomTransactionError::CommitRevealEnabled.into());
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
        let info = DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorTransactionExtension::<Test>::new();
        let err = extension
            .validate(
                RawOrigin::Signed(hotkey).into(),
                &call,
                &info,
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(err, CustomTransactionError::InputLengthsUnequal.into());
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
        let info = DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorTransactionExtension::<Test>::new();
        let err = extension
            .validate(
                RawOrigin::Signed(other_ck).into(),
                &call,
                &info,
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(err, CustomTransactionError::NonAssociatedColdKey.into());
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

        let extension = SubtensorTransactionExtension::<Test>::new();
        let err = extension
            .validate(
                RawOrigin::Signed(hotkey).into(),
                &call,
                &info,
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(
            err,
            CustomTransactionError::HotKeyNotRegisteredInNetwork.into()
        );
    });
}

#[test]
fn extension_commit_weights_rejects_commit_reveal_disabled() {
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
        SubtensorModule::add_balance_to_coldkey_account(&hotkey, u64::MAX.into());
        SubtensorModule::set_stake_threshold(0);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::commit_weights {
            netuid,
            commit_hash: sp_core::H256::zero(),
        });
        let info = DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorTransactionExtension::<Test>::new();
        let err = extension
            .validate(
                RawOrigin::Signed(hotkey).into(),
                &call,
                &info,
                0,
                (),
                &TxBaseImplication(()),
                TransactionSource::External,
            )
            .unwrap_err();
        assert_eq!(err, CustomTransactionError::CommitRevealDisabled.into());
    });
}
