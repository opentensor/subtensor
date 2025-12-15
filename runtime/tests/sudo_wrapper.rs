#![allow(clippy::unwrap_used)]

use frame_support::assert_ok;
use frame_support::dispatch::GetDispatchInfo;
use node_subtensor_runtime::{
    BuildStorage, Runtime, RuntimeCall, RuntimeGenesisConfig, RuntimeOrigin, System, SystemCall,
    sudo_wrapper,
};
use sp_runtime::traits::{TransactionExtension, TxBaseImplication, ValidateResult};
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError,
};
use subtensor_runtime_common::AccountId;

const SUDO_ACCOUNT: [u8; 32] = [1_u8; 32];
const OTHER_ACCOUNT: [u8; 32] = [3_u8; 32];

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        sudo: pallet_sudo::GenesisConfig { key: None },
        ..Default::default()
    }
    .build_storage()
    .unwrap()
    .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn call_remark() -> RuntimeCall {
    let remark = vec![1, 2, 3];
    RuntimeCall::System(SystemCall::remark { remark })
}

fn sudo_extrinsic(inner: RuntimeCall) -> RuntimeCall {
    RuntimeCall::Sudo(pallet_sudo::Call::sudo {
        call: Box::new(inner),
    })
}

fn validate_ext(origin: RuntimeOrigin, call: &RuntimeCall) -> ValidateResult<(), RuntimeCall> {
    let ext = sudo_wrapper::SudoTransactionExtension::<Runtime>::new();

    ext.validate(
        origin,
        call,
        &call.get_dispatch_info(),
        0,
        (),
        &TxBaseImplication(()),
        TransactionSource::External,
    )
}
#[test]
fn sudo_signed_by_correct_key_is_valid() {
    new_test_ext().execute_with(|| {
        let sudo_key = AccountId::from(SUDO_ACCOUNT);
        pallet_sudo::Key::<Runtime>::put(sudo_key.clone());
        let sudo_call = sudo_extrinsic(call_remark());

        // Signed origin with correct sudo key
        let origin = RuntimeOrigin::signed(sudo_key);
        let res = validate_ext(origin, &sudo_call);
        assert_ok!(res);
    });
}

#[test]
fn sudo_signed_by_wrong_account_is_rejected() {
    new_test_ext().execute_with(|| {
        let sudo_key = AccountId::from(SUDO_ACCOUNT);
        // Set sudo key in storage
        pallet_sudo::Key::<Runtime>::put(sudo_key.clone());
        let sudo_call = sudo_extrinsic(call_remark());
        // Wrong signer
        let origin = RuntimeOrigin::signed(AccountId::from(OTHER_ACCOUNT));
        let res = validate_ext(origin, &sudo_call);
        assert!(matches!(
            res,
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::BadSigner
            ))
        ));
    });
}

#[test]
fn sudo_when_no_sudo_key_configured_is_rejected() {
    new_test_ext().execute_with(|| {
        // Remove sudo key
        pallet_sudo::Key::<Runtime>::kill();
        let sudo_call = sudo_extrinsic(call_remark());
        let origin = RuntimeOrigin::signed(AccountId::from(SUDO_ACCOUNT));
        let res = validate_ext(origin, &sudo_call);
        assert!(matches!(
            res,
            Err(TransactionValidityError::Invalid(
                InvalidTransaction::BadSigner
            ))
        ));
    });
}

#[test]
fn non_sudo_extrinsic_does_not_trigger_filter() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(AccountId::from(OTHER_ACCOUNT));
        let call = call_remark();
        let res = validate_ext(origin, &call);
        assert!(res.is_ok());
    });
}
