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
use subtensor_runtime_common::{AccountId, NetUid};

use common::ExtBuilder;

mod common;

fn assert_extrinsic_ok(account_id: &AccountId, pair: &sr25519::Pair, call: RuntimeCall) {
    let nonce = System::account(account_id).nonce;
    let xt = signed_extrinsic(call, pair, nonce);
    assert_ok!(Executive::apply_extrinsic(xt));
}

fn assert_extrinsic_rate_limited(account_id: &AccountId, pair: &sr25519::Pair, call: RuntimeCall) {
    let nonce = System::account(account_id).nonce;
    let xt = signed_extrinsic(call, pair, nonce);
    assert!(matches!(
        Executive::apply_extrinsic(xt).expect_err("rate limit enforced"),
        TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
    ));
}

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

            // Run runtime upgrades explicitly so rate-limiting config is seeded for tests.
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

            assert_extrinsic_ok(&coldkey, &coldkey_pair, call_a.clone());

            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, call_b.clone());

            // Migration sets register-network limit to 4 days (28_800 blocks).
            let limit = start_block.saturating_add(28_800);

            // Should still be rate-limited.
            System::set_block_number(limit - 1);
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, call_a.clone());

            // Should pass now.
            System::set_block_number(limit);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, call_b);

            // Both calls share the same usage key and window.
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, call_a.clone());

            System::set_block_number(limit.saturating_add(28_800));
            assert_extrinsic_ok(&coldkey, &coldkey_pair, call_a);
        });
}

#[test]
fn serving_is_rate_limited_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[4u8; 32]);
    let hotkey_pair = sr25519::Pair::from_seed(&[5u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let hotkey = AccountId::from(hotkey_pair.public());
    let balance = 10_000_000_000_000_u64;

    ExtBuilder::default()
        .with_balances(vec![(coldkey.clone(), balance)])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            // Run runtime upgrades explicitly so rate-limiting config is seeded for tests.
            Executive::execute_on_runtime_upgrade();

            assert_extrinsic_ok(
                &coldkey,
                &coldkey_pair,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register {
                    hotkey: hotkey.clone(),
                }),
            );

            let netuid = NetUid::ROOT;
            let start_block = System::block_number();
            let serve_axon = RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_axon {
                netuid,
                version: 1,
                ip: 0,
                port: 3030,
                ip_type: 4,
                protocol: 0,
                placeholder1: 0,
                placeholder2: 0,
            });
            let serve_axon_tls =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_axon_tls {
                    netuid,
                    version: 1,
                    ip: 0,
                    port: 3030,
                    ip_type: 4,
                    protocol: 0,
                    placeholder1: 0,
                    placeholder2: 0,
                    certificate: b"cert".to_vec(),
                });
            let serve_prometheus =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_prometheus {
                    netuid,
                    version: 1,
                    ip: 1_676_056_785,
                    port: 3031,
                    ip_type: 4,
                });

            assert_extrinsic_ok(&hotkey, &hotkey_pair, serve_axon.clone());

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, serve_axon_tls.clone());

            assert_extrinsic_ok(&hotkey, &hotkey_pair, serve_prometheus.clone());

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, serve_prometheus.clone());

            // Migration sets serving limit to 50 blocks by default.
            let limit = start_block.saturating_add(50);

            // Should still be rate-limited.
            System::set_block_number(limit - 1);
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, serve_axon.clone());

            // Should pass now.
            System::set_block_number(limit);
            assert_extrinsic_ok(&hotkey, &hotkey_pair, serve_axon_tls);

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, serve_axon);

            assert_extrinsic_ok(&hotkey, &hotkey_pair, serve_prometheus.clone());

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, serve_prometheus);
        });
}
