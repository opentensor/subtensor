#![cfg(feature = "integration-tests")]
#![allow(clippy::unwrap_used)]

use codec::Encode;
use frame_support::assert_ok;
use node_subtensor_runtime::{
    Executive, Runtime, RuntimeCall, SignedPayload, System, TransactionExtensions,
    UncheckedExtrinsic, check_nonce, sudo_wrapper, transaction_payment_wrapper,
};
use sp_core::{Pair, sr25519};
use sp_runtime::{
    MultiSignature,
    generic::Era,
    traits::SaturatedConversion,
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};
use subtensor_runtime_common::AccountId;

use common::ExtBuilder;

mod common;

fn signed_extrinsic(call: RuntimeCall, pair: &sr25519::Pair, nonce: u32) -> UncheckedExtrinsic {
    let check_metadata_hash =
        frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(false);

    let extra: TransactionExtensions = (
        frame_system::CheckNonZeroSender::<Runtime>::new(),
        frame_system::CheckSpecVersion::<Runtime>::new(),
        frame_system::CheckTxVersion::<Runtime>::new(),
        frame_system::CheckGenesis::<Runtime>::new(),
        frame_system::CheckEra::<Runtime>::from(Era::Immortal),
        check_nonce::CheckNonce::<Runtime>::from(nonce).into(),
        frame_system::CheckWeight::<Runtime>::new(),
        transaction_payment_wrapper::ChargeTransactionPaymentWrapper::new(
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
        ),
        sudo_wrapper::SudoTransactionExtension::<Runtime>::new(),
        pallet_subtensor::transaction_extension::SubtensorTransactionExtension::<Runtime>::new(),
        (
            pallet_drand::drand_priority::DrandPriority::<Runtime>::new(),
            check_metadata_hash,
        ),
        pallet_rate_limiting::RateLimitTransactionExtension::<Runtime>::new(),
    );

    let payload = SignedPayload::new(call.clone(), extra.clone()).expect("signed payload");
    let signature = MultiSignature::from(pair.sign(payload.encode().as_slice()));
    let address = sp_runtime::MultiAddress::Id(AccountId::from(pair.public()));
    UncheckedExtrinsic::new_signed(call, address, signature, extra)
}

#[test]
fn register_network_is_rate_limited_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[1u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let hotkey_a = AccountId::from([2u8; 32]);
    let hotkey_b = AccountId::from([3u8; 32]);
    let balance = 10_000_000_000_000_u64;

    ExtBuilder::default()
        .with_balances(vec![(coldkey.clone(), balance)])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            // Run runtime upgrades explicitly; ExtBuilder sets up genesis only.
            Executive::execute_on_runtime_upgrade();

            let call_a = RuntimeCall::SubtensorModule(pallet_subtensor::Call::register_network {
                hotkey: hotkey_a,
            });
            let call_b = RuntimeCall::SubtensorModule(
                pallet_subtensor::Call::register_network_with_identity {
                    hotkey: hotkey_b,
                    identity: None,
                },
            );

            let start_block =
                pallet_subtensor::NetworkRegistrationStartBlock::<Runtime>::get().saturated_into();
            System::set_block_number(start_block);

            let mut nonce = System::account(coldkey.clone()).nonce;
            let xt_a = signed_extrinsic(call_a, &coldkey_pair, nonce);
            assert_ok!(Executive::apply_extrinsic(xt_a));

            nonce = System::account(coldkey.clone()).nonce;
            let xt_b = signed_extrinsic(call_b.clone(), &coldkey_pair, nonce);
            assert!(matches!(
                Executive::apply_extrinsic(xt_b).expect_err("rate limit enforced"),
                TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
            ));

            // Migration sets register-network limit to 4 days (28_800 blocks).
            let limit = start_block.saturating_add(28_800);

			// Should still be rate-limited.
            System::set_block_number(limit - 1);
            nonce = System::account(coldkey.clone()).nonce;
            let xt_b = signed_extrinsic(call_b.clone(), &coldkey_pair, nonce);
            assert!(matches!(
                Executive::apply_extrinsic(xt_b).expect_err("rate limit enforced"),
                TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
            ));

			// Should pass now.
            System::set_block_number(limit);
            nonce = System::account(coldkey.clone()).nonce;
            let xt_c = signed_extrinsic(call_b, &coldkey_pair, nonce);
            assert_ok!(Executive::apply_extrinsic(xt_c));
        });
}
