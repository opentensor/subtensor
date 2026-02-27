#![allow(clippy::unwrap_used)]

use frame_support::{
    assert_ok,
    dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo},
};
use node_subtensor_runtime::{
    BuildStorage, Proxy, Runtime, RuntimeCall, RuntimeGenesisConfig, RuntimeOrigin, System,
    SystemCall, transaction_payment_wrapper, NORMAL_DISPATCH_BASE_PRIORITY,
    OPERATIONAL_DISPATCH_PRIORITY,
};
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_utility as pallet_utility;
use pallet_transaction_payment::{ChargeTransactionPayment, Val};
use sp_runtime::traits::{TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError, ValidTransaction};
use subtensor_runtime_common::{AccountId, ProxyType};

const SIGNER: [u8; 32] = [1_u8; 32];
const REAL_A: [u8; 32] = [2_u8; 32];
const REAL_B: [u8; 32] = [3_u8; 32];
const OTHER: [u8; 32] = [4_u8; 32];
const BALANCE: u64 = 1_000_000_000_000;

fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![
                (AccountId::from(SIGNER), BALANCE),
                (AccountId::from(REAL_A), BALANCE),
                (AccountId::from(REAL_B), BALANCE),
                (AccountId::from(OTHER), BALANCE),
            ],
            dev_accounts: None,
        },
        ..Default::default()
    }
    .build_storage()
    .unwrap()
    .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn signer() -> AccountId {
    AccountId::from(SIGNER)
}
fn real_a() -> AccountId {
    AccountId::from(REAL_A)
}
fn real_b() -> AccountId {
    AccountId::from(REAL_B)
}
fn other() -> AccountId {
    AccountId::from(OTHER)
}

// -- Call builders --

fn call_remark() -> RuntimeCall {
    RuntimeCall::System(SystemCall::remark {
        remark: vec![1, 2, 3],
    })
}

fn proxy_call(real: AccountId, inner: RuntimeCall) -> RuntimeCall {
    RuntimeCall::Proxy(pallet_proxy::Call::proxy {
        real: real.into(),
        force_proxy_type: None,
        call: Box::new(inner),
    })
}

fn proxy_announced_call(delegate: AccountId, real: AccountId, inner: RuntimeCall) -> RuntimeCall {
    RuntimeCall::Proxy(pallet_proxy::Call::proxy_announced {
        delegate: delegate.into(),
        real: real.into(),
        force_proxy_type: None,
        call: Box::new(inner),
    })
}

fn batch_call(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::batch { calls })
}

fn batch_all_call(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::batch_all { calls })
}

fn force_batch_call(calls: Vec<RuntimeCall>) -> RuntimeCall {
    RuntimeCall::Utility(pallet_utility::Call::force_batch { calls })
}

// -- Setup helpers --

fn add_proxy(real: &AccountId, delegate: &AccountId) {
    assert_ok!(Proxy::add_proxy(
        RuntimeOrigin::signed(real.clone()),
        delegate.clone().into(),
        ProxyType::Any,
        0,
    ));
}

fn enable_real_pays_fee(real: &AccountId, delegate: &AccountId) {
    assert_ok!(Proxy::set_real_pays_fee(
        RuntimeOrigin::signed(real.clone()),
        delegate.clone().into(),
        true,
    ));
}

// -- Validate helpers --

fn validate_call(
    origin: RuntimeOrigin,
    call: &RuntimeCall,
) -> Result<(ValidTransaction, Val<Runtime>), TransactionValidityError> {
    validate_call_with_info(origin, call, &call.get_dispatch_info())
}

fn validate_call_with_info(
    origin: RuntimeOrigin,
    call: &RuntimeCall,
    info: &DispatchInfo,
) -> Result<(ValidTransaction, Val<Runtime>), TransactionValidityError> {
    let ext = transaction_payment_wrapper::ChargeTransactionPaymentWrapper::<Runtime>::new(
        ChargeTransactionPayment::from(0u64),
    );
    let (valid_tx, val, _origin) = ext.validate(
        origin,
        call,
        info,
        100,
        (),
        &TxBaseImplication(()),
        TransactionSource::External,
    )?;
    Ok((valid_tx, val))
}

/// Extract the fee payer from the validate result.
fn fee_payer(val: &Val<Runtime>) -> AccountId {
    match val {
        Val::Charge { who, .. } => who.clone(),
        _ => panic!("expected Val::Charge"),
    }
}

// ============================================================
// Case 0: Non-proxy calls
// ============================================================

#[test]
fn non_proxy_call_charges_signer() {
    new_test_ext().execute_with(|| {
        let call = call_remark();
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), signer());
    });
}

// ============================================================
// Case 1: Simple proxy (1 level)
// ============================================================

#[test]
fn simple_proxy_charges_real_when_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_a(), &signer());
        enable_real_pays_fee(&real_a(), &signer());

        let call = proxy_call(real_a(), call_remark());
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_a());
    });
}

#[test]
fn simple_proxy_charges_signer_when_not_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_a(), &signer());
        // No enable_real_pays_fee

        let call = proxy_call(real_a(), call_remark());
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), signer());
    });
}

#[test]
fn proxy_announced_always_charges_signer() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        // Fee propagation intentionally ignores proxy_announced; signer always pays.
        let call = proxy_announced_call(real_b(), real_a(), call_remark());
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), signer());
    });
}

// ============================================================
// Case 2: Nested proxy (2 levels)
// ============================================================

#[test]
fn nested_proxy_charges_inner_real_when_both_opted_in() {
    new_test_ext().execute_with(|| {
        // Chain: signer → real_b → real_a
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        let call = proxy_call(real_b(), proxy_call(real_a(), call_remark()));
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_a());
    });
}

#[test]
fn nested_proxy_charges_outer_real_when_only_outer_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        // No enable_real_pays_fee for A→B

        let call = proxy_call(real_b(), proxy_call(real_a(), call_remark()));
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

#[test]
fn nested_proxy_charges_signer_when_neither_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        // No enable_real_pays_fee at all

        let call = proxy_call(real_b(), proxy_call(real_a(), call_remark()));
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), signer());
    });
}

#[test]
fn nested_proxy_charges_signer_when_only_inner_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        // No enable_real_pays_fee for B→signer
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        let call = proxy_call(real_b(), proxy_call(real_a(), call_remark()));
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        // Outer RealPaysFee not set → signer pays (inner opt-in is irrelevant)
        assert_eq!(fee_payer(&val), signer());
    });
}

// ============================================================
// Case 3: Batch of proxy calls
// ============================================================

#[test]
fn batch_charges_inner_real_when_all_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        let batch = batch_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(real_a(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_a());
    });
}

#[test]
fn batch_all_charges_inner_real_when_all_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        let batch = batch_all_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(real_a(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_a());
    });
}

#[test]
fn force_batch_charges_inner_real_when_all_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());

        let batch = force_batch_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(real_a(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_a());
    });
}

#[test]
fn batch_charges_outer_real_when_only_outer_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        // No enable_real_pays_fee for A→B

        let batch = batch_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(real_a(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

#[test]
fn batch_charges_outer_real_when_mixed_inner_reals() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        enable_real_pays_fee(&real_a(), &real_b());
        add_proxy(&other(), &real_b());
        enable_real_pays_fee(&other(), &real_b());

        // Different inner reals → can't push deeper
        let batch = batch_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(other(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

#[test]
fn batch_charges_outer_real_when_non_proxy_in_batch() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());

        // Batch contains a non-proxy call → extract_proxy_parts fails
        let batch = batch_call(vec![
            proxy_call(real_a(), call_remark()),
            call_remark(),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

#[test]
fn batch_charges_outer_real_when_empty() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());

        let batch = batch_call(vec![]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

#[test]
fn batch_charges_outer_real_when_inner_real_not_opted_in() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_b(), &signer());
        enable_real_pays_fee(&real_b(), &signer());
        add_proxy(&real_a(), &real_b());
        // real_a has NOT opted in to pay for real_b

        // Even with same real in all batch items, if RealPaysFee<A, B> not set → outer_real pays
        let batch = batch_call(vec![
            proxy_call(real_a(), call_remark()),
            proxy_call(real_a(), call_remark()),
        ]);
        let call = proxy_call(real_b(), batch);
        let (_valid_tx, val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(fee_payer(&val), real_b());
    });
}

// ============================================================
// Priority override
// ============================================================

#[test]
fn priority_override_normal_dispatch() {
    new_test_ext().execute_with(|| {
        let call = call_remark();
        let (valid_tx, _val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        assert_eq!(valid_tx.priority, NORMAL_DISPATCH_BASE_PRIORITY);
    });
}

#[test]
fn priority_override_operational_dispatch() {
    new_test_ext().execute_with(|| {
        let call = call_remark();
        let mut info = call.get_dispatch_info();
        info.class = DispatchClass::Operational;

        let (valid_tx, _val) =
            validate_call_with_info(RuntimeOrigin::signed(signer()), &call, &info).unwrap();
        assert_eq!(valid_tx.priority, OPERATIONAL_DISPATCH_PRIORITY);
    });
}

#[test]
fn priority_override_mandatory_dispatch() {
    new_test_ext().execute_with(|| {
        let call = call_remark();
        let mut info = call.get_dispatch_info();
        info.class = DispatchClass::Mandatory;

        let (valid_tx, _val) =
            validate_call_with_info(RuntimeOrigin::signed(signer()), &call, &info).unwrap();
        // Mandatory uses the same base as Normal
        assert_eq!(valid_tx.priority, NORMAL_DISPATCH_BASE_PRIORITY);
    });
}

#[test]
fn priority_override_applies_with_real_pays_fee() {
    new_test_ext().execute_with(|| {
        add_proxy(&real_a(), &signer());
        enable_real_pays_fee(&real_a(), &signer());

        let call = proxy_call(real_a(), call_remark());
        let (valid_tx, _val) = validate_call(RuntimeOrigin::signed(signer()), &call).unwrap();
        // Priority override should still apply when real pays fee
        assert_eq!(valid_tx.priority, NORMAL_DISPATCH_BASE_PRIORITY);
    });
}
