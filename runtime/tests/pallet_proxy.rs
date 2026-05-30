#![allow(clippy::unwrap_used)]

use frame_support::{assert_ok, traits::InstanceFilter};
use node_subtensor_runtime::{
    BalancesCall, BuildStorage, Proxy, Runtime, RuntimeCall, RuntimeEvent, RuntimeGenesisConfig,
    RuntimeOrigin, SubtensorModule, System, SystemCall, get_all_proxy_filters,
    get_all_proxy_type_infos,
};
use pallet_subtensor_proxy as pallet_proxy;
use subtensor_runtime_common::{
    AccountId, CallCondition, FilterMode, NetUid, ProxyType, SMALL_ALPHA_TRANSFER_LIMIT,
    SMALL_TRANSFER_LIMIT, TaoBalance,
};

const ACCOUNT: [u8; 32] = [1_u8; 32];
const DELEGATE: [u8; 32] = [2_u8; 32];
const OTHER_ACCOUNT: [u8; 32] = [3_u8; 32];

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

// --- ProxyFilter RuntimeAPI sync tests ---

fn call_small_transfer() -> RuntimeCall {
    let value = TaoBalance::from(100u64);
    RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: AccountId::new([2u8; 32]).into(),
        value,
    })
}

fn call_large_transfer() -> RuntimeCall {
    let value = TaoBalance::from(1_000_000_000u64);
    RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: AccountId::new([2u8; 32]).into(),
        value,
    })
}

fn call_sudo_remark() -> RuntimeCall {
    RuntimeCall::Sudo(pallet_sudo::Call::sudo {
        call: Box::new(RuntimeCall::System(SystemCall::remark {
            remark: vec![1, 2, 3],
        })),
    })
}

#[test]
fn proxy_filter_api_behavior_matches_instance_filter() {
    new_test_ext().execute_with(|| {
        // Part 1: Ground truth — verify InstanceFilter::filter() returns expected results
        // based on reading the InstanceFilter source code

        // Any allows everything
        assert!(ProxyType::Any.filter(&call_transfer()));
        assert!(ProxyType::Any.filter(&call_remark()));
        assert!(ProxyType::Any.filter(&call_add_stake()));

        // NonTransfer denies Balances and specific transfer calls
        assert!(!ProxyType::NonTransfer.filter(&call_transfer()));
        assert!(ProxyType::NonTransfer.filter(&call_remark()));
        assert!(ProxyType::NonTransfer.filter(&call_add_stake()));

        // NonFungible denies Balances and staking-related calls
        assert!(!ProxyType::NonFungible.filter(&call_transfer()));
        assert!(!ProxyType::NonFungible.filter(&call_add_stake()));
        assert!(ProxyType::NonFungible.filter(&call_remark()));

        // Transfer allows only specific transfer calls
        assert!(ProxyType::Transfer.filter(&call_transfer()));
        assert!(!ProxyType::Transfer.filter(&call_remark()));
        assert!(!ProxyType::Transfer.filter(&call_add_stake()));

        // SmallTransfer allows transfers below limit
        assert!(ProxyType::SmallTransfer.filter(&call_small_transfer()));
        assert!(!ProxyType::SmallTransfer.filter(&call_large_transfer()));
        assert!(!ProxyType::SmallTransfer.filter(&call_remark()));

        // Owner allows AdminUtils and specific SubtensorModule calls
        assert!(ProxyType::Owner.filter(&call_owner_util()));
        assert!(ProxyType::Owner.filter(&call_set_subnet_identity()));
        assert!(ProxyType::Owner.filter(&call_update_symbol()));
        assert!(!ProxyType::Owner.filter(&call_sn_owner_hotkey()));
        assert!(!ProxyType::Owner.filter(&call_remark()));

        // NonCritical denies critical calls
        assert!(!ProxyType::NonCritical.filter(&call_root_register()));
        assert!(ProxyType::NonCritical.filter(&call_remark()));
        assert!(ProxyType::NonCritical.filter(&call_transfer()));
        assert!(!ProxyType::NonCritical.filter(&call_sudo_remark()));

        // Staking allows staking calls only
        assert!(ProxyType::Staking.filter(&call_add_stake()));
        assert!(!ProxyType::Staking.filter(&call_transfer()));
        assert!(!ProxyType::Staking.filter(&call_remark()));

        // Registration allows registration calls only
        assert!(ProxyType::Registration.filter(&call_register()));
        assert!(!ProxyType::Registration.filter(&call_remark()));

        // Deprecated types deny everything
        assert!(!ProxyType::Triumvirate.filter(&call_remark()));
        assert!(!ProxyType::Senate.filter(&call_remark()));
        assert!(!ProxyType::Governance.filter(&call_remark()));
        assert!(!ProxyType::RootWeights.filter(&call_remark()));
    });
}

#[test]
fn proxy_filter_api_structural_validation() {
    new_test_ext().execute_with(|| {
        let type_infos = get_all_proxy_type_infos();
        let filters = get_all_proxy_filters();

        // Part 2: Structural validation of API output

        // Verify total count equals number of ProxyType variants (18)
        assert_eq!(type_infos.len(), 18);
        assert_eq!(filters.len(), 18);

        // Verify ProxyTypeInfo correctness
        let any_info = type_infos.iter().find(|t| t.index == 0).unwrap();
        assert_eq!(any_info.name, b"Any");
        assert!(!any_info.deprecated);

        let triumvirate_info = type_infos.iter().find(|t| t.index == 6).unwrap();
        assert_eq!(triumvirate_info.name, b"Triumvirate");
        assert!(triumvirate_info.deprecated);

        let senate_info = type_infos.iter().find(|t| t.index == 4).unwrap();
        assert_eq!(senate_info.name, b"Senate");
        assert!(senate_info.deprecated);

        // Verify deprecated ProxyTypes have DenyAll filter mode
        for filter in &filters {
            let pt = ProxyType::try_from(filter.proxy_type).unwrap();
            if pt.is_deprecated() {
                assert_eq!(
                    filter.filter_mode,
                    FilterMode::DenyAll,
                    "Deprecated ProxyType {:?} should have DenyAll filter mode",
                    pt
                );
                assert!(filter.calls.is_empty());
            }
        }

        // Verify Any has AllowAll
        let any_filter = filters.iter().find(|f| f.proxy_type == 0).unwrap();
        assert_eq!(any_filter.filter_mode, FilterMode::AllowAll);
        assert!(any_filter.calls.is_empty());

        // Verify NonTransfer has Deny mode with Balances wildcard
        let non_transfer = filters.iter().find(|f| f.proxy_type == 3).unwrap();
        assert_eq!(non_transfer.filter_mode, FilterMode::Deny);
        assert!(
            non_transfer
                .calls
                .iter()
                .any(|c| c.call_name.is_none() && c.pallet_name == b"Balances")
        );

        // Verify Owner has Allow mode with exceptions
        let owner = filters.iter().find(|f| f.proxy_type == 1).unwrap();
        assert_eq!(owner.filter_mode, FilterMode::Allow);
        assert!(
            owner
                .calls
                .iter()
                .any(|c| c.call_name.is_none() && c.pallet_name == b"AdminUtils")
        );
        assert!(!owner.exceptions.is_empty());
        assert!(
            owner
                .exceptions
                .iter()
                .any(|c| c.call_name.as_deref() == Some(b"sudo_set_sn_owner_hotkey".as_slice()))
        );

        // Verify SmallTransfer has conditions with correct limits
        let small_transfer = filters.iter().find(|f| f.proxy_type == 11).unwrap();
        assert_eq!(small_transfer.filter_mode, FilterMode::Allow);
        let has_tao_limit = small_transfer.calls.iter().any(|c| {
            matches!(
                &c.condition,
                Some(CallCondition::ParamLessThan { limit, .. })
                if *limit == Into::<u64>::into(SMALL_TRANSFER_LIMIT) as u128
            )
        });
        assert!(
            has_tao_limit,
            "SmallTransfer should have TAO limit condition"
        );

        let has_alpha_limit = small_transfer.calls.iter().any(|c| {
            matches!(
                &c.condition,
                Some(CallCondition::ParamLessThan { param_name, limit, .. })
                if param_name == b"alpha_amount"
                    && *limit == Into::<u64>::into(SMALL_ALPHA_TRANSFER_LIMIT) as u128
            )
        });
        assert!(
            has_alpha_limit,
            "SmallTransfer should have Alpha limit condition"
        );

        // Verify NonCritical has Deny mode with Sudo wildcard
        let non_critical = filters.iter().find(|f| f.proxy_type == 2).unwrap();
        assert_eq!(non_critical.filter_mode, FilterMode::Deny);
        assert!(
            non_critical
                .calls
                .iter()
                .any(|c| c.call_name.is_none() && c.pallet_name == b"Sudo")
        );

        // Verify SudoUncheckedSetCode has NestedCallMustBe condition
        let sudo_set_code = filters.iter().find(|f| f.proxy_type == 14).unwrap();
        assert_eq!(sudo_set_code.filter_mode, FilterMode::Allow);
        let has_nested_condition = sudo_set_code.calls.iter().any(|c| {
            matches!(
                &c.condition,
                Some(CallCondition::NestedCallMustBe {
                    pallet_name,
                    call_name,
                }) if pallet_name == b"System" && call_name == b"set_code"
            )
        });
        assert!(
            has_nested_condition,
            "SudoUncheckedSetCode should have NestedCallMustBe condition"
        );

        // Verify pallet indices are correct (from construct_runtime!)
        // Balances = 5, SubtensorModule = 7, Sudo = 12, AdminUtils = 19
        let balances_wildcard = non_transfer
            .calls
            .iter()
            .find(|c| c.call_name.is_none() && c.pallet_name == b"Balances")
            .unwrap();
        assert_eq!(balances_wildcard.pallet_index, 5);

        let sudo_wildcard = non_critical
            .calls
            .iter()
            .find(|c| c.call_name.is_none() && c.pallet_name == b"Sudo")
            .unwrap();
        assert_eq!(sudo_wildcard.pallet_index, 12);

        let admin_wildcard = owner
            .calls
            .iter()
            .find(|c| c.call_name.is_none() && c.pallet_name == b"AdminUtils")
            .unwrap();
        assert_eq!(admin_wildcard.pallet_index, 19);
    });
}

#[test]
fn proxy_filter_api_cross_check_filter_behavior() {
    new_test_ext().execute_with(|| {
        let filters = get_all_proxy_filters();

        // Part 3: For each non-wildcard, non-conditional call in the API output,
        // verify that InstanceFilter::filter() agrees with the declared filter_mode

        // Build a set of test calls indexed by (pallet_index, call_index)
        let test_calls: Vec<(u8, u8, RuntimeCall)> = vec![
            // Balances calls
            {
                let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                    dest: AccountId::new([0u8; 32]).into(),
                    value: Default::default(),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            // SubtensorModule calls
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake {
                    hotkey: AccountId::new([0u8; 32]),
                    netuid: Default::default(),
                    amount_staked: Default::default(),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
                    hotkey: AccountId::new([0u8; 32]),
                    netuid: Default::default(),
                    amount_unstaked: Default::default(),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register {
                    netuid: Default::default(),
                    hotkey: AccountId::new([0u8; 32]),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register {
                    hotkey: AccountId::new([0u8; 32]),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey {
                    hotkey: AccountId::new([0u8; 32]),
                    new_hotkey: AccountId::new([0u8; 32]),
                    netuid: Default::default(),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_children {
                    hotkey: AccountId::new([0u8; 32]),
                    netuid: Default::default(),
                    children: Default::default(),
                });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            // AdminUtils calls
            {
                let call = RuntimeCall::AdminUtils(
                    pallet_admin_utils::Call::sudo_set_serving_rate_limit {
                        netuid: Default::default(),
                        serving_rate_limit: Default::default(),
                    },
                );
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
            {
                let call =
                    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_sn_owner_hotkey {
                        netuid: Default::default(),
                        hotkey: AccountId::new([0u8; 32]),
                    });
                let encoded = codec::Encode::encode(&call);
                (encoded[0], encoded[1], call)
            },
        ];

        for filter_info in &filters {
            let proxy_type = ProxyType::try_from(filter_info.proxy_type).unwrap();

            match filter_info.filter_mode {
                FilterMode::AllowAll => {
                    for (_, _, call) in &test_calls {
                        assert!(
                            proxy_type.filter(call),
                            "AllowAll ProxyType {:?} should allow all calls",
                            proxy_type
                        );
                    }
                }
                FilterMode::DenyAll => {
                    for (_, _, call) in &test_calls {
                        assert!(
                            !proxy_type.filter(call),
                            "DenyAll ProxyType {:?} should deny all calls",
                            proxy_type
                        );
                    }
                }
                FilterMode::Allow => {
                    for call_info in &filter_info.calls {
                        if call_info.call_name.is_none() || call_info.condition.is_some() {
                            continue;
                        }
                        if let Some((_, _, call)) = test_calls.iter().find(|(pi, ci, _)| {
                            *pi == call_info.pallet_index && Some(*ci) == call_info.call_index
                        }) {
                            assert!(
                                proxy_type.filter(call),
                                "Allow-mode ProxyType {:?} should allow call {:?}",
                                proxy_type,
                                call_info
                                    .call_name
                                    .as_ref()
                                    .map(|n| core::str::from_utf8(n).unwrap_or("?"))
                                    .unwrap_or("*")
                            );
                        }
                    }
                    // Verify exceptions are denied
                    for exc_info in &filter_info.exceptions {
                        if let Some((_, _, call)) = test_calls.iter().find(|(pi, ci, _)| {
                            *pi == exc_info.pallet_index && Some(*ci) == exc_info.call_index
                        }) {
                            assert!(
                                !proxy_type.filter(call),
                                "ProxyType {:?} should deny exception {:?}",
                                proxy_type,
                                exc_info
                                    .call_name
                                    .as_ref()
                                    .map(|n| core::str::from_utf8(n).unwrap_or("?"))
                                    .unwrap_or("*")
                            );
                        }
                    }
                }
                FilterMode::Deny => {
                    for call_info in &filter_info.calls {
                        if call_info.call_name.is_none() || call_info.condition.is_some() {
                            continue;
                        }
                        if let Some((_, _, call)) = test_calls.iter().find(|(pi, ci, _)| {
                            *pi == call_info.pallet_index && Some(*ci) == call_info.call_index
                        }) {
                            assert!(
                                !proxy_type.filter(call),
                                "Deny-mode ProxyType {:?} should deny call {:?}",
                                proxy_type,
                                call_info
                                    .call_name
                                    .as_ref()
                                    .map(|n| core::str::from_utf8(n).unwrap_or("?"))
                                    .unwrap_or("*")
                            );
                        }
                    }
                }
            }
        }
    });
}

#[test]
fn proxy_filter_api_deprecated_consistency() {
    new_test_ext().execute_with(|| {
        let type_infos = get_all_proxy_type_infos();

        for pt_info in &type_infos {
            let pt = ProxyType::try_from(pt_info.index).unwrap();
            if pt_info.deprecated {
                assert!(
                    pt.is_deprecated(),
                    "ProxyTypeInfo reports {:?} as deprecated but is_deprecated() disagrees",
                    pt
                );
                assert!(
                    !pt.filter(&call_remark()),
                    "Deprecated ProxyType {:?} should deny all calls",
                    pt
                );
                assert!(!pt.filter(&call_transfer()));
                assert!(!pt.filter(&call_add_stake()));
            }
        }
    });
}
