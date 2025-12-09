use crate::tests::mock::{RuntimeCall, RuntimeOrigin, SubtensorCall, new_test_ext};
use crate::transaction_extension::SubtensorTransactionExtension;
use frame_support::assert_ok;
use frame_support::dispatch::GetDispatchInfo;
use sp_core::U256;
use sp_runtime::traits::{TransactionExtension, TxBaseImplication, ValidateResult};
use subtensor_runtime_common::{NetUid, TaoCurrency};

use super::mock::*;
use crate::*;

fn some_call() -> RuntimeCall {
    let hotkey = U256::from(0);
    let amount_staked = TaoCurrency::from(5000);
    let netuid = NetUid::from(1);
    RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
        hotkey,
        netuid,
        amount_staked,
    })
}
fn sudo_extrinsic(inner: RuntimeCall) -> RuntimeCall {
    RuntimeCall::Sudo(pallet_sudo::Call::sudo {
        call: Box::new(inner),
    })
}

fn validate_ext(
    origin: RuntimeOrigin,
    call: &RuntimeCall,
) -> ValidateResult<Option<<Test as frame_system::Config>::AccountId>, RuntimeCall> {
    let ext = SubtensorTransactionExtension::<Test>::new();

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
    new_test_ext(1).execute_with(|| {
        let sudo_key: <Test as frame_system::Config>::AccountId = U256::from(42);
        pallet_sudo::Key::<Test>::put(sudo_key);
        let sudo_call = sudo_extrinsic(some_call());

        // Signed origin with correct sudo key
        let origin = RuntimeOrigin::signed(sudo_key);
        let res = validate_ext(origin, &sudo_call);
        assert_ok!(res);
    });
}

#[test]
fn sudo_signed_by_wrong_account_is_rejected() {
    new_test_ext(1).execute_with(|| {
        let sudo_key: <Test as frame_system::Config>::AccountId = U256::from(42);
        // Set sudo key in storage
        pallet_sudo::Key::<Test>::put(sudo_key);
        let sudo_call = sudo_extrinsic(some_call());
        // Wrong signer
        let origin = RuntimeOrigin::signed(U256::from(99));
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
    new_test_ext(1).execute_with(|| {
        // Remove sudo key
        pallet_sudo::Key::<Test>::kill();
        let sudo_call = sudo_extrinsic(some_call());
        let origin = RuntimeOrigin::signed(U256::from(42));
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
    new_test_ext(1).execute_with(|| {
        let origin = RuntimeOrigin::signed(U256::from(42));
        let call = some_call();
        let res = validate_ext(origin, &call);
        assert!(res.is_ok());
    });
}
