#![allow(clippy::unwrap_used)]

use codec::{Compact, Encode};
use frame_support::{assert_ok, traits::Get};
use node_subtensor_runtime::{
    Executive, Runtime, RuntimeCall, SignedPayload, SubtensorInitialTxDelegateTakeRateLimit,
    System, TransactionExtensions, UncheckedExtrinsic, check_nonce,
    rate_limiting::legacy::storage as legacy_storage, sudo_wrapper, transaction_payment_wrapper,
};
use sp_core::{H256, Pair, sr25519};
use sp_runtime::{
    BoundedVec, MultiSignature,
    generic::Era,
    traits::SaturatedConversion,
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};
use subtensor_runtime_common::{AccountId, AlphaCurrency, Currency, MechId, NetUid};

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

fn setup_weights_network(netuid: NetUid, hotkey: &AccountId, block: u64, mechanisms: u8) {
    pallet_subtensor::Pallet::<Runtime>::init_new_network(netuid, 1);
    if mechanisms > 1 {
        pallet_subtensor::MechanismCountCurrent::<Runtime>::insert(
            netuid,
            MechId::from(mechanisms),
        );
    }
    System::set_block_number(block.saturated_into());
    pallet_subtensor::Pallet::<Runtime>::append_neuron(netuid, hotkey, block);
}

fn setup_staking_network(netuid: NetUid) {
    pallet_subtensor::Pallet::<Runtime>::init_new_network(netuid, 1);
    pallet_subtensor::SubtokenEnabled::<Runtime>::insert(netuid, true);
    pallet_subtensor::TransferToggle::<Runtime>::insert(netuid, true);
}

fn seed_stake(netuid: NetUid, hotkey: &AccountId, coldkey: &AccountId, alpha: u64) {
    pallet_subtensor::Pallet::<Runtime>::create_account_if_non_existent(coldkey, hotkey);
    pallet_subtensor::Pallet::<Runtime>::increase_stake_for_hotkey_and_coldkey_on_subnet(
        hotkey,
        coldkey,
        netuid,
        AlphaCurrency::from(alpha),
    );
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
            let limit = start_block + 28_800;

            // Should still be rate-limited.
            System::set_block_number(limit - 1);
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, call_a.clone());

            // Should pass now.
            System::set_block_number(limit);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, call_b);

            // Both calls share the same usage key and window.
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, call_a.clone());

            System::set_block_number(limit + 28_800);
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
            let limit = start_block + 50;

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

#[test]
fn delegate_take_increase_is_rate_limited_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[6u8; 32]);
    let hotkey_pair = sr25519::Pair::from_seed(&[7u8; 32]);
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

            // Seed current take so increase_take passes take checks.
            pallet_subtensor::Delegates::<Runtime>::insert(&hotkey, 1u16);

            let increase_once =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::increase_take {
                    hotkey: hotkey.clone(),
                    take: 2u16,
                });
            let increase_twice =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::increase_take {
                    hotkey: hotkey.clone(),
                    take: 3u16,
                });

            let start_block = System::block_number();

            assert_extrinsic_ok(&coldkey, &coldkey_pair, increase_once);

            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, increase_twice.clone());

            let limit = SubtensorInitialTxDelegateTakeRateLimit::get();
            let limit_block = start_block + limit.saturated_into::<u32>();
            let allowed_block = limit_block + 1;

            System::set_block_number(limit_block - 1);
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, increase_twice.clone());

            System::set_block_number(allowed_block);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, increase_twice);
        });
}

#[test]
fn delegate_take_decrease_is_not_rate_limited_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[10u8; 32]);
    let hotkey_pair = sr25519::Pair::from_seed(&[11u8; 32]);
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

            // Seed current take so decreases are valid and deterministic.
            pallet_subtensor::Delegates::<Runtime>::insert(&hotkey, 3u16);

            let decrease_once =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::decrease_take {
                    hotkey: hotkey.clone(),
                    take: 2u16,
                });
            let decrease_twice =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::decrease_take {
                    hotkey: hotkey.clone(),
                    take: 1u16,
                });

            assert_extrinsic_ok(&coldkey, &coldkey_pair, decrease_once);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, decrease_twice);
        });
}

#[test]
fn delegate_take_decrease_blocks_immediate_increase_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[8u8; 32]);
    let hotkey_pair = sr25519::Pair::from_seed(&[9u8; 32]);
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

            // Seed current take so decrease then increase remains valid.
            pallet_subtensor::Delegates::<Runtime>::insert(&hotkey, 2u16);

            let decrease = RuntimeCall::SubtensorModule(pallet_subtensor::Call::decrease_take {
                hotkey: hotkey.clone(),
                take: 1u16,
            });
            let increase = RuntimeCall::SubtensorModule(pallet_subtensor::Call::increase_take {
                hotkey: hotkey.clone(),
                take: 2u16,
            });

            let start_block = System::block_number();

            assert_extrinsic_ok(&coldkey, &coldkey_pair, decrease);

            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, increase.clone());

            let limit = SubtensorInitialTxDelegateTakeRateLimit::get();
            let limit_block = start_block + limit.saturated_into::<u32>();
            let allowed_block = limit_block + 1;

            System::set_block_number(limit_block - 1);
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, increase.clone());

            System::set_block_number(allowed_block);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, increase);
        });
}

#[test]
fn weights_set_is_rate_limited_after_migration() {
    let hotkey_pair = sr25519::Pair::from_seed(&[12u8; 32]);
    let hotkey = AccountId::from(hotkey_pair.public());
    let netuid = NetUid::from(1u16);
    let span = 3u64;
    let registration_block = 1u64;

    ExtBuilder::default()
        .with_balances(vec![(hotkey.clone(), 10_000_000_000_000_u64)])
        .build()
        .execute_with(|| {
            setup_weights_network(netuid, &hotkey, registration_block, 1);
            legacy_storage::set_weights_set_rate_limit(netuid, span);

            Executive::execute_on_runtime_upgrade();

            pallet_subtensor::Pallet::<Runtime>::set_commit_reveal_weights_enabled(netuid, false);

            let version_key = pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid);
            let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
                netuid,
                dests: vec![0],
                weights: vec![u16::MAX],
                version_key,
            });

            System::set_block_number(registration_block.saturated_into());
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, call.clone());

            System::set_block_number((registration_block + span - 1).saturated_into());
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, call.clone());

            System::set_block_number((registration_block + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, call.clone());
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, call.clone());

            System::set_block_number((registration_block + span + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, call);
        });
}

#[test]
fn commit_weights_shares_rate_limit_with_set_weights() {
    let hotkey_pair = sr25519::Pair::from_seed(&[13u8; 32]);
    let hotkey = AccountId::from(hotkey_pair.public());
    let netuid = NetUid::from(2u16);
    let span = 4u64;
    let registration_block = 1u64;
    let commit_hash = H256::from_low_u64_be(42);

    ExtBuilder::default()
        .with_balances(vec![(hotkey.clone(), 10_000_000_000_000_u64)])
        .build()
        .execute_with(|| {
            setup_weights_network(netuid, &hotkey, registration_block, 1);
            legacy_storage::set_weights_set_rate_limit(netuid, span);

            Executive::execute_on_runtime_upgrade();

            let commit_call =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_weights {
                    netuid,
                    commit_hash,
                });

            System::set_block_number((registration_block + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, commit_call);

            pallet_subtensor::Pallet::<Runtime>::set_commit_reveal_weights_enabled(netuid, false);

            let version_key = pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid);
            let set_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
                netuid,
                dests: vec![0],
                weights: vec![u16::MAX],
                version_key,
            });

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, set_call.clone());

            System::set_block_number((registration_block + span + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, set_call);
        });
}

#[test]
fn commit_timelocked_weights_is_rate_limited_after_migration() {
    let hotkey_pair = sr25519::Pair::from_seed(&[14u8; 32]);
    let hotkey = AccountId::from(hotkey_pair.public());
    let netuid = NetUid::from(3u16);
    let span = 4u64;
    let registration_block = 1u64;
    let commit = BoundedVec::try_from(vec![1u8; 16]).expect("commit payload within limit");
    let reveal_round = 10u64;

    ExtBuilder::default()
        .with_balances(vec![(hotkey.clone(), 10_000_000_000_000_u64)])
        .build()
        .execute_with(|| {
            setup_weights_network(netuid, &hotkey, registration_block, 1);
            legacy_storage::set_weights_set_rate_limit(netuid, span);

            Executive::execute_on_runtime_upgrade();

            let commit_reveal_version =
                pallet_subtensor::Pallet::<Runtime>::get_commit_reveal_weights_version();
            let commit_call =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_timelocked_weights {
                    netuid,
                    commit: commit.clone(),
                    reveal_round,
                    commit_reveal_version,
                });

            System::set_block_number((registration_block + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, commit_call.clone());
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, commit_call.clone());

            System::set_block_number((registration_block + span + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, commit_call);
        });
}

#[test]
fn commit_crv3_mechanism_weights_are_rate_limited_per_mechanism() {
    let hotkey_pair = sr25519::Pair::from_seed(&[15u8; 32]);
    let hotkey = AccountId::from(hotkey_pair.public());
    let netuid = NetUid::from(4u16);
    let span = 4u64;
    let registration_block = 1u64;
    let commit = BoundedVec::try_from(vec![1u8; 16]).expect("commit payload within limit");
    let reveal_round = 10u64;
    let mecid_a = MechId::from(0u8);
    let mecid_b = MechId::from(1u8);

    ExtBuilder::default()
        .with_balances(vec![(hotkey.clone(), 10_000_000_000_000_u64)])
        .build()
        .execute_with(|| {
            setup_weights_network(netuid, &hotkey, registration_block, 2);
            legacy_storage::set_weights_set_rate_limit(netuid, span);

            Executive::execute_on_runtime_upgrade();

            let commit_a = RuntimeCall::SubtensorModule(
                pallet_subtensor::Call::commit_crv3_mechanism_weights {
                    netuid,
                    mecid: mecid_a,
                    commit: commit.clone(),
                    reveal_round,
                },
            );
            let commit_b = RuntimeCall::SubtensorModule(
                pallet_subtensor::Call::commit_crv3_mechanism_weights {
                    netuid,
                    mecid: mecid_b,
                    commit: commit.clone(),
                    reveal_round,
                },
            );

            System::set_block_number((registration_block + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, commit_a.clone());
            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, commit_a);
            assert_extrinsic_ok(&hotkey, &hotkey_pair, commit_b);
        });
}

#[test]
fn batch_set_weights_is_rate_limited_if_any_scope_is_within_span() {
    let hotkey_pair = sr25519::Pair::from_seed(&[16u8; 32]);
    let hotkey = AccountId::from(hotkey_pair.public());
    let netuid_a = NetUid::from(5u16);
    let netuid_b = NetUid::from(6u16);
    let span = 3u64;
    let registration_block = 1u64;

    ExtBuilder::default()
        .with_balances(vec![(hotkey.clone(), 10_000_000_000_000_u64)])
        .build()
        .execute_with(|| {
            setup_weights_network(netuid_a, &hotkey, registration_block, 1);
            setup_weights_network(netuid_b, &hotkey, registration_block, 1);
            legacy_storage::set_weights_set_rate_limit(netuid_a, span);
            legacy_storage::set_weights_set_rate_limit(netuid_b, span);

            Executive::execute_on_runtime_upgrade();

            pallet_subtensor::Pallet::<Runtime>::set_commit_reveal_weights_enabled(netuid_a, false);
            pallet_subtensor::Pallet::<Runtime>::set_commit_reveal_weights_enabled(netuid_b, false);

            let version_key_a = pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid_a);
            let version_key_b = pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid_b);

            let set_call_a = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
                netuid: netuid_a,
                dests: vec![0],
                weights: vec![u16::MAX],
                version_key: version_key_a,
            });

            System::set_block_number((registration_block + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, set_call_a);

            let batch_call =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::batch_set_weights {
                    netuids: vec![Compact(netuid_a), Compact(netuid_b)],
                    weights: vec![
                        vec![(Compact(0u16), Compact(1u16))],
                        vec![(Compact(0u16), Compact(1u16))],
                    ],
                    version_keys: vec![Compact(version_key_a), Compact(version_key_b)],
                });

            assert_extrinsic_rate_limited(&hotkey, &hotkey_pair, batch_call.clone());

            System::set_block_number((registration_block + span + span).saturated_into());
            assert_extrinsic_ok(&hotkey, &hotkey_pair, batch_call);
        });
}

#[test]
fn staking_add_then_remove_is_rate_limited_after_migration() {
    let coldkey_pair = sr25519::Pair::from_seed(&[20u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let hotkey = AccountId::from([21u8; 32]);
    let netuid = NetUid::from(10u16);
    let stake_amount = pallet_subtensor::DefaultMinStake::<Runtime>::get().to_u64() * 10;
    let balance = stake_amount * 10;

    ExtBuilder::default()
        .with_balances(vec![(coldkey.clone(), balance)])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            setup_staking_network(netuid);
            pallet_subtensor::Pallet::<Runtime>::create_account_if_non_existent(&coldkey, &hotkey);

            Executive::execute_on_runtime_upgrade();

            let add_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake {
                hotkey: hotkey.clone(),
                netuid,
                amount_staked: stake_amount.into(),
            });
            assert_extrinsic_ok(&coldkey, &coldkey_pair, add_call);

            let alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey, &coldkey, netuid,
                );
            let remove_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
                hotkey,
                netuid,
                amount_unstaked: alpha,
            });

            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, remove_call.clone());

            System::set_block_number(2);
            assert_extrinsic_ok(&coldkey, &coldkey_pair, remove_call);
        });
}

#[test]
fn transfer_stake_is_rate_limited_after_add_stake() {
    let coldkey_pair = sr25519::Pair::from_seed(&[22u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let destination_coldkey = AccountId::from([23u8; 32]);
    let hotkey = AccountId::from([24u8; 32]);
    let netuid = NetUid::from(11u16);
    let stake_amount = pallet_subtensor::DefaultMinStake::<Runtime>::get().to_u64() * 10;
    let balance = stake_amount * 10;

    ExtBuilder::default()
        .with_balances(vec![(coldkey.clone(), balance)])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            setup_staking_network(netuid);
            pallet_subtensor::Pallet::<Runtime>::create_account_if_non_existent(&coldkey, &hotkey);

            Executive::execute_on_runtime_upgrade();

            let add_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake {
                hotkey: hotkey.clone(),
                netuid,
                amount_staked: stake_amount.into(),
            });
            assert_extrinsic_ok(&coldkey, &coldkey_pair, add_call);

            let alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey, &coldkey, netuid,
                );
            let transfer_call =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake {
                    destination_coldkey,
                    hotkey,
                    origin_netuid: netuid,
                    destination_netuid: netuid,
                    alpha_amount: alpha,
                });

            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, transfer_call);
        });
}

#[test]
fn transfer_stake_does_not_limit_destination_coldkey() {
    let coldkey_pair = sr25519::Pair::from_seed(&[25u8; 32]);
    let destination_pair = sr25519::Pair::from_seed(&[26u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let destination_coldkey = AccountId::from(destination_pair.public());
    let hotkey = AccountId::from([27u8; 32]);
    let origin_netuid = NetUid::from(12u16);
    let destination_netuid = NetUid::from(13u16);
    let stake_amount = pallet_subtensor::DefaultMinStake::<Runtime>::get().to_u64() * 10;

    ExtBuilder::default()
        .with_balances(vec![
            (coldkey.clone(), stake_amount * 10),
            (destination_coldkey.clone(), stake_amount * 10),
        ])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            setup_staking_network(origin_netuid);
            setup_staking_network(destination_netuid);
            seed_stake(origin_netuid, &hotkey, &coldkey, stake_amount);

            Executive::execute_on_runtime_upgrade();

            let alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey,
                    &coldkey,
                    origin_netuid,
                );
            let transfer_call =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake {
                    destination_coldkey: destination_coldkey.clone(),
                    hotkey: hotkey.clone(),
                    origin_netuid,
                    destination_netuid,
                    alpha_amount: alpha,
                });

            assert_extrinsic_ok(&coldkey, &coldkey_pair, transfer_call);

            let destination_alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey,
                    &destination_coldkey,
                    destination_netuid,
                );
            let remove_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
                hotkey,
                netuid: destination_netuid,
                amount_unstaked: destination_alpha,
            });

            assert_extrinsic_ok(&destination_coldkey, &destination_pair, remove_call);
        });
}

#[test]
fn swap_stake_limits_destination_netuid() {
    let coldkey_pair = sr25519::Pair::from_seed(&[28u8; 32]);
    let coldkey = AccountId::from(coldkey_pair.public());
    let hotkey = AccountId::from([29u8; 32]);
    let origin_netuid = NetUid::from(14u16);
    let destination_netuid = NetUid::from(15u16);
    let stake_amount = pallet_subtensor::DefaultMinStake::<Runtime>::get().to_u64() * 10;

    ExtBuilder::default()
        .with_balances(vec![(coldkey.clone(), stake_amount * 10)])
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            setup_staking_network(origin_netuid);
            setup_staking_network(destination_netuid);
            seed_stake(origin_netuid, &hotkey, &coldkey, stake_amount);

            Executive::execute_on_runtime_upgrade();

            let alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey,
                    &coldkey,
                    origin_netuid,
                );
            let swap_alpha = AlphaCurrency::from(alpha.to_u64() / 2);
            let swap_call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake {
                hotkey: hotkey.clone(),
                origin_netuid,
                destination_netuid,
                alpha_amount: swap_alpha,
            });

            assert_extrinsic_ok(&coldkey, &coldkey_pair, swap_call);

            let destination_alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey,
                    &coldkey,
                    destination_netuid,
                );
            let remove_destination =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
                    hotkey: hotkey.clone(),
                    netuid: destination_netuid,
                    amount_unstaked: destination_alpha,
                });
            assert_extrinsic_rate_limited(&coldkey, &coldkey_pair, remove_destination);

            let origin_alpha =
                pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    &hotkey,
                    &coldkey,
                    origin_netuid,
                );
            let remove_origin =
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
                    hotkey,
                    netuid: origin_netuid,
                    amount_unstaked: origin_alpha,
                });
            assert_extrinsic_ok(&coldkey, &coldkey_pair, remove_origin);
        });
}
