#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use codec::Encode;
use frame_support::{assert_ok, traits::InstanceFilter};
use node_subtensor_runtime::{
    BalancesCall, BuildStorage, Commitments, Proxy, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeGenesisConfig, RuntimeOrigin, SubtensorModule, System, SystemCall,
};
use pallet_commitments::{CommitmentInfo, Data};
use pallet_subtensor_proxy as pallet_proxy;
use sp_core::{H160, H256, Pair, ecdsa, keccak_256};
use sp_runtime::BoundedVec;
use subtensor_runtime_common::{AccountId, NetUid, ProxyType, TaoBalance};

const ACCOUNT: [u8; 32] = [1_u8; 32];
const DELEGATE: [u8; 32] = [2_u8; 32];
const OTHER_ACCOUNT: [u8; 32] = [3_u8; 32];
const THIRD_ACCOUNT: [u8; 32] = [4_u8; 32];

type SystemError = frame_system::Error<Runtime>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let amount = TaoBalance::from(1_000_000_000_000_u64);
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![
                (AccountId::from(ACCOUNT), amount),
                (AccountId::from(DELEGATE), amount),
                (AccountId::from(OTHER_ACCOUNT), amount),
                (AccountId::from(THIRD_ACCOUNT), amount),
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

// transfer call
fn call_transfer() -> RuntimeCall {
    let value = TaoBalance::from(100);
    RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: AccountId::from(OTHER_ACCOUNT).into(),
        value,
    })
}

// remark call
fn call_remark() -> RuntimeCall {
    let remark = vec![1, 2, 3];
    RuntimeCall::System(SystemCall::remark { remark })
}

// owner call
fn call_owner_util() -> RuntimeCall {
    let netuid = NetUid::from(1);
    let serving_rate_limit = 2;
    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_serving_rate_limit {
        netuid,
        serving_rate_limit,
    })
}

// sn owner hotkey call
fn call_sn_owner_hotkey() -> RuntimeCall {
    let netuid = NetUid::from(1);
    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_sn_owner_hotkey {
        netuid,
        hotkey: AccountId::from(ACCOUNT).into(),
    })
}

// set subnet identity call
fn call_set_subnet_identity() -> RuntimeCall {
    let netuid = NetUid::from(1);
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_subnet_identity {
        netuid,
        subnet_name: vec![],
        github_repo: vec![],
        subnet_contact: vec![],
        subnet_url: vec![],
        discord: vec![],
        description: vec![],
        logo_url: vec![],
        additional: vec![],
    })
}

// update symbol call
fn call_update_symbol() -> RuntimeCall {
    let netuid = NetUid::from(1);
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::update_symbol {
        netuid,
        symbol: vec![],
    })
}

// critical call for Subtensor
fn call_root_register() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register {
        hotkey: AccountId::from(ACCOUNT),
    })
}

// staking call
fn call_add_stake() -> RuntimeCall {
    let netuid = NetUid::from(1);
    let amount_staked = 100;
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake {
        hotkey: AccountId::from(DELEGATE),
        netuid,
        amount_staked: amount_staked.into(),
    })
}

// register call, account as hotkey, delegate as coldkey
fn call_register() -> RuntimeCall {
    let block_number: u64 = 1;
    let netuid = NetUid::from(2);

    // lower diff first
    SubtensorModule::set_difficulty(netuid, 100);

    let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
        netuid,
        block_number,
        0,
        &AccountId::from(ACCOUNT),
    );

    RuntimeCall::SubtensorModule(pallet_subtensor::Call::register {
        netuid,
        block_number,
        nonce,
        work: work.clone(),
        hotkey: AccountId::from(ACCOUNT),
        coldkey: AccountId::from(DELEGATE),
    })
}

fn call_swap_hotkey() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey {
        hotkey: AccountId::from(ACCOUNT),
        new_hotkey: AccountId::from(OTHER_ACCOUNT),
        netuid: Some(NetUid::from(1)),
    })
}

fn call_set_children() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_children {
        hotkey: AccountId::from(ACCOUNT),
        netuid: NetUid::from(1),
        children: vec![(1, AccountId::from(OTHER_ACCOUNT))],
    })
}

fn call_serve_axon() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_axon {
        netuid: NetUid::from(1),
        version: 1,
        ip: 0,
        port: 8080,
        ip_type: 4,
        protocol: 0,
        placeholder1: 0,
        placeholder2: 0,
    })
}

fn call_serve_axon_tls() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_axon_tls {
        netuid: NetUid::from(1),
        version: 1,
        ip: 0,
        port: 8080,
        ip_type: 4,
        protocol: 0,
        placeholder1: 0,
        placeholder2: 0,
        certificate: b"CERT".to_vec(),
    })
}

fn call_associate_evm_key() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::associate_evm_key {
        netuid: NetUid::from(1),
        evm_key: H160::repeat_byte(1),
        block_number: 1,
        signature: ecdsa::Signature::from_raw([0u8; 65]),
    })
}

fn call_set_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
        netuid: NetUid::from(1),
        dests: vec![0],
        weights: vec![u16::MAX],
        version_key: 0,
    })
}

fn call_set_mechanism_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_mechanism_weights {
        netuid: NetUid::from(1),
        mecid: 0u8.into(),
        dests: vec![0],
        weights: vec![u16::MAX],
        version_key: 0,
    })
}

fn call_batch_set_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::batch_set_weights {
        netuids: vec![NetUid::from(1).into()],
        weights: vec![vec![(0u16.into(), u16::MAX.into())]],
        version_keys: vec![0u64.into()],
    })
}

fn call_commit_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_weights {
        netuid: NetUid::from(1),
        commit_hash: H256::repeat_byte(1),
    })
}

fn call_commit_mechanism_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_mechanism_weights {
        netuid: NetUid::from(1),
        mecid: 0u8.into(),
        commit_hash: H256::repeat_byte(2),
    })
}

fn call_batch_commit_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::batch_commit_weights {
        netuids: vec![NetUid::from(1).into()],
        commit_hashes: vec![H256::repeat_byte(3)],
    })
}

fn call_reveal_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::reveal_weights {
        netuid: NetUid::from(1),
        uids: vec![0],
        values: vec![u16::MAX],
        salt: vec![1, 2, 3],
        version_key: 0,
    })
}

fn call_reveal_mechanism_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::reveal_mechanism_weights {
        netuid: NetUid::from(1),
        mecid: 0u8.into(),
        uids: vec![0],
        values: vec![u16::MAX],
        salt: vec![1, 2, 3],
        version_key: 0,
    })
}

fn call_batch_reveal_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::batch_reveal_weights {
        netuid: NetUid::from(1),
        uids_list: vec![vec![0]],
        values_list: vec![vec![u16::MAX]],
        salts_list: vec![vec![1, 2, 3]],
        version_keys: vec![0],
    })
}

fn call_commit_timelocked_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_timelocked_weights {
        netuid: NetUid::from(1),
        commit: vec![1, 2, 3].try_into().unwrap(),
        reveal_round: 10,
        commit_reveal_version: 4,
    })
}

fn call_commit_crv3_mechanism_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::commit_crv3_mechanism_weights {
        netuid: NetUid::from(1),
        mecid: 0u8.into(),
        commit: vec![1, 2, 3].try_into().unwrap(),
        reveal_round: 10,
    })
}

fn call_commit_timelocked_mechanism_weights() -> RuntimeCall {
    RuntimeCall::SubtensorModule(
        pallet_subtensor::Call::commit_timelocked_mechanism_weights {
            netuid: NetUid::from(1),
            mecid: 0u8.into(),
            commit: vec![1, 2, 3].try_into().unwrap(),
            reveal_round: 10,
            commit_reveal_version: 4,
        },
    )
}

fn plain_commitment_info() -> Box<CommitmentInfo<<Runtime as pallet_commitments::Config>::MaxFields>>
{
    Box::new(CommitmentInfo {
        fields: BoundedVec::try_from(vec![Data::Raw(
            BoundedVec::try_from(b"knowledge".to_vec()).unwrap(),
        )])
        .unwrap(),
    })
}

fn timelocked_commitment_info()
-> Box<CommitmentInfo<<Runtime as pallet_commitments::Config>::MaxFields>> {
    Box::new(CommitmentInfo {
        fields: BoundedVec::try_from(vec![Data::TimelockEncrypted {
            encrypted: BoundedVec::try_from(vec![1, 2, 3]).unwrap(),
            reveal_round: 10,
        }])
        .unwrap(),
    })
}

fn call_set_commitment() -> RuntimeCall {
    RuntimeCall::Commitments(pallet_commitments::Call::set_commitment {
        netuid: NetUid::from(1),
        info: plain_commitment_info(),
    })
}

fn add_proxy_delegate(proxy_type: ProxyType) {
    assert_ok!(Proxy::add_proxy(
        RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
        AccountId::from(DELEGATE).into(),
        proxy_type,
        0
    ));
}

fn setup_hotkey_on_network(netuid: NetUid, hotkey: AccountId, coldkey: AccountId) -> u16 {
    if !SubtensorModule::if_subnet_exist(netuid) {
        SubtensorModule::init_new_network(netuid, 13);
        SubtensorModule::set_network_registration_allowed(netuid, true);
        SubtensorModule::set_network_pow_registration_allowed(netuid, true);
    }

    SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
    if SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).is_err() {
        SubtensorModule::append_neuron(
            netuid,
            &hotkey,
            SubtensorModule::get_current_block_as_u64(),
        );
    }
    SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap()
}

fn sign_evm_message(pair: &ecdsa::Pair, hotkey: &AccountId, block_number: u64) -> ecdsa::Signature {
    let hashed_block_number = keccak_256(block_number.encode().as_ref());
    let message = [hotkey.encode().as_slice(), hashed_block_number.as_slice()].concat();
    let hash = pallet_subtensor::Pallet::<Runtime>::hash_message_eip191(message);
    let mut signature = pair.sign_prehashed(&hash);
    if let Some(v) = signature.0.get_mut(64) {
        *v = v.wrapping_add(27);
    }
    signature
}

fn verify_call_with_proxy_type(proxy_type: &ProxyType, call: &RuntimeCall) {
    assert_ok!(Proxy::proxy(
        RuntimeOrigin::signed(AccountId::from(DELEGATE)),
        AccountId::from(ACCOUNT).into(),
        None,
        Box::new(call.clone()),
    ));

    let filtered_event: RuntimeEvent = pallet_proxy::Event::ProxyExecuted {
        result: Err(SystemError::CallFiltered.into()),
    }
    .into();

    // check if the filter works by checking the last event
    // filtered if the last event is SystemError::CallFiltered
    // not filtered if the last event is proxy executed done or any error from proxy call
    if proxy_type.filter(call) {
        let last_event = System::events().last().unwrap().event.clone();
        assert_ne!(last_event, filtered_event);
    } else {
        System::assert_last_event(filtered_event);
    }
}

#[test]
fn test_proxy_pallet() {
    let proxy_types = [
        ProxyType::Any,
        ProxyType::Owner,
        ProxyType::NonCritical,
        ProxyType::NonTransfer,
        ProxyType::NonFungible,
        ProxyType::Staking,
        ProxyType::Registration,
        ProxyType::Validate,
    ];

    let calls = [
        call_transfer,
        call_remark,
        call_owner_util,
        call_root_register,
        call_add_stake,
        call_register,
    ];

    for call in calls.iter() {
        for proxy_type in proxy_types.iter() {
            new_test_ext().execute_with(|| {
                assert_ok!(Proxy::add_proxy(
                    RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
                    AccountId::from(DELEGATE).into(),
                    *proxy_type,
                    0
                ));

                verify_call_with_proxy_type(proxy_type, &call());
            });
        }
    }
}

#[test]
fn test_non_transfer_cannot_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
            AccountId::from(DELEGATE).into(),
            ProxyType::NonTransfer,
            0
        ));

        let call = call_transfer();
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            Box::new(call.clone()),
        ));

        System::assert_last_event(
            pallet_proxy::Event::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
    });
}

#[test]
fn test_owner_type_cannot_set_sn_owner_hotkey() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
            AccountId::from(DELEGATE).into(),
            ProxyType::Owner,
            0
        ));

        let call = call_sn_owner_hotkey();
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            Box::new(call.clone()),
        ));

        System::assert_last_event(
            pallet_proxy::Event::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
    });
}

#[test]
fn test_owner_type_can_set_subnet_identity_and_update_symbol() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
            AccountId::from(DELEGATE).into(),
            ProxyType::Owner,
            0
        ));

        verify_call_with_proxy_type(&ProxyType::Owner, &call_set_subnet_identity());
        verify_call_with_proxy_type(&ProxyType::Owner, &call_update_symbol());
    });
}

#[test]
fn test_validate_proxy_allows_expected_calls() {
    let allowed_calls = [
        call_serve_axon,
        call_serve_axon_tls,
        call_associate_evm_key,
        call_set_weights,
        call_set_mechanism_weights,
        call_batch_set_weights,
        call_commit_weights,
        call_commit_mechanism_weights,
        call_batch_commit_weights,
        call_reveal_weights,
        call_reveal_mechanism_weights,
        call_batch_reveal_weights,
        call_commit_timelocked_weights,
        call_commit_crv3_mechanism_weights,
        call_commit_timelocked_mechanism_weights,
        call_set_commitment,
    ];

    for call in allowed_calls {
        new_test_ext().execute_with(|| {
            add_proxy_delegate(ProxyType::Validate);
            verify_call_with_proxy_type(&ProxyType::Validate, &call());
        });
    }
}

#[test]
fn test_validate_proxy_filters_disallowed_calls() {
    let denied_calls = [
        call_transfer,
        call_remark,
        call_owner_util,
        call_root_register,
        call_add_stake,
        call_register,
        call_swap_hotkey,
        call_set_children,
    ];

    for call in denied_calls {
        new_test_ext().execute_with(|| {
            add_proxy_delegate(ProxyType::Validate);
            verify_call_with_proxy_type(&ProxyType::Validate, &call());
        });
    }
}

#[test]
fn test_validate_proxy_hierarchy_and_escalation_rules() {
    new_test_ext().execute_with(|| {
        add_proxy_delegate(ProxyType::Validate);

        let add_validate_proxy = RuntimeCall::Proxy(pallet_proxy::Call::add_proxy {
            delegate: AccountId::from(OTHER_ACCOUNT).into(),
            proxy_type: ProxyType::Validate,
            delay: 0,
        });
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            Box::new(add_validate_proxy),
        ));
        assert!(
            Proxy::proxies(AccountId::from(ACCOUNT))
                .0
                .iter()
                .any(|proxy| proxy.delegate == AccountId::from(OTHER_ACCOUNT)
                    && proxy.proxy_type == ProxyType::Validate)
        );

        let add_any_proxy = RuntimeCall::Proxy(pallet_proxy::Call::add_proxy {
            delegate: AccountId::from(THIRD_ACCOUNT).into(),
            proxy_type: ProxyType::Any,
            delay: 0,
        });
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            Box::new(add_any_proxy),
        ));
        System::assert_last_event(
            pallet_proxy::Event::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );

        let remove_all = RuntimeCall::Proxy(pallet_proxy::Call::remove_proxies {});
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            Box::new(remove_all),
        ));
        System::assert_last_event(
            pallet_proxy::Event::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
    });
}

#[test]
fn test_validate_proxy_can_set_weights_statefully() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = AccountId::from(ACCOUNT);
        let coldkey = AccountId::from(OTHER_ACCOUNT);
        let peer_hotkey = AccountId::from(THIRD_ACCOUNT);

        let neuron_uid = setup_hotkey_on_network(netuid, hotkey.clone(), coldkey.clone());
        let peer_uid = setup_hotkey_on_network(netuid, peer_hotkey, coldkey);
        SubtensorModule::set_validator_permit_for_uid(netuid, neuron_uid, true);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);

        add_proxy_delegate(ProxyType::Validate);
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
            netuid,
            dests: vec![peer_uid],
            weights: vec![u16::MAX],
            version_key: 0,
        });

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            hotkey.into(),
            None,
            Box::new(call),
        ));

        assert!(System::events().iter().any(|record| {
            record.event
                == RuntimeEvent::SubtensorModule(pallet_subtensor::Event::WeightsSet(
                    SubtensorModule::get_mechanism_storage_index(netuid, 0u8.into()),
                    neuron_uid,
                ))
        }));
    });
}

#[test]
fn test_validate_proxy_can_set_plain_commitment_statefully() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = AccountId::from(ACCOUNT);
        let coldkey = AccountId::from(OTHER_ACCOUNT);

        setup_hotkey_on_network(netuid, hotkey.clone(), coldkey);
        add_proxy_delegate(ProxyType::Validate);

        let call = RuntimeCall::Commitments(pallet_commitments::Call::set_commitment {
            netuid,
            info: plain_commitment_info(),
        });

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            hotkey.clone().into(),
            None,
            Box::new(call),
        ));

        assert!(Commitments::commitment_of(netuid, hotkey).is_some());
    });
}

#[test]
fn test_validate_proxy_can_set_timelocked_commitment_statefully() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = AccountId::from(ACCOUNT);
        let coldkey = AccountId::from(OTHER_ACCOUNT);

        setup_hotkey_on_network(netuid, hotkey.clone(), coldkey);
        add_proxy_delegate(ProxyType::Validate);

        let call = RuntimeCall::Commitments(pallet_commitments::Call::set_commitment {
            netuid,
            info: timelocked_commitment_info(),
        });

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            hotkey.clone().into(),
            None,
            Box::new(call),
        ));

        assert!(System::events().iter().any(|record| {
            record.event
                == RuntimeEvent::Commitments(pallet_commitments::Event::TimelockCommitment {
                    netuid,
                    who: hotkey.clone(),
                    reveal_round: 10,
                })
        }));
    });
}

#[test]
fn test_validate_proxy_can_serve_axon_tls_statefully() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = AccountId::from(ACCOUNT);
        let coldkey = AccountId::from(OTHER_ACCOUNT);
        let certificate = b"CERT".to_vec();

        setup_hotkey_on_network(netuid, hotkey.clone(), coldkey);
        add_proxy_delegate(ProxyType::Validate);

        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::serve_axon_tls {
            netuid,
            version: 1,
            ip: 0,
            port: 8080,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
            certificate: certificate.clone(),
        });

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            hotkey.clone().into(),
            None,
            Box::new(call),
        ));

        let stored = pallet_subtensor::NeuronCertificates::<Runtime>::get(netuid, hotkey.clone())
            .expect("certificate should be stored");
        assert_eq!(
            stored.public_key.into_inner(),
            certificate.get(1..).unwrap().to_vec()
        );
        assert!(System::events().iter().any(|record| {
            record.event
                == RuntimeEvent::SubtensorModule(pallet_subtensor::Event::AxonServed(
                    netuid,
                    hotkey.clone(),
                ))
        }));
    });
}

#[test]
fn test_validate_proxy_can_associate_evm_key_statefully() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = AccountId::from(ACCOUNT);
        let coldkey = AccountId::from(OTHER_ACCOUNT);

        let neuron_uid = setup_hotkey_on_network(netuid, hotkey.clone(), coldkey);
        add_proxy_delegate(ProxyType::Validate);

        let pair = ecdsa::Pair::generate().0;
        let public = pair.public();
        let uncompressed = libsecp256k1::PublicKey::parse_compressed(&public.0)
            .unwrap()
            .serialize();
        let uncompressed_body = uncompressed.get(1..).unwrap_or_default();
        let hashed = keccak_256(uncompressed_body);
        let evm_key = H160::from_slice(hashed.get(12..).unwrap_or_default());
        let block_number = 1u64;
        let signature = sign_evm_message(&pair, &hotkey, block_number);

        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::associate_evm_key {
            netuid,
            evm_key,
            block_number,
            signature,
        });

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            hotkey.clone().into(),
            None,
            Box::new(call),
        ));

        assert_eq!(
            pallet_subtensor::AssociatedEvmAddress::<Runtime>::get(netuid, neuron_uid),
            Some((evm_key, block_number))
        );
        assert!(System::events().iter().any(|record| {
            record.event
                == RuntimeEvent::SubtensorModule(pallet_subtensor::Event::EvmKeyAssociated {
                    netuid,
                    hotkey: hotkey.clone(),
                    evm_key,
                    block_associated: block_number,
                })
        }));
    });
}
