#![allow(clippy::unwrap_used)]

use codec::Encode;
use frame_support::{BoundedVec, assert_ok, traits::InstanceFilter};
use node_subtensor_runtime::{
    BalancesCall, BuildStorage, Proxy, Runtime, RuntimeCall, RuntimeEvent, RuntimeGenesisConfig,
    RuntimeOrigin, SubtensorModule, System, SystemCall,
};
use pallet_subtensor_collective as pallet_collective;
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_swap::tick::TickIndex;
use subtensor_runtime_common::{AccountId, NetUid, ProxyType};

const ACCOUNT: [u8; 32] = [1_u8; 32];
const DELEGATE: [u8; 32] = [2_u8; 32];
const OTHER_ACCOUNT: [u8; 32] = [3_u8; 32];

type SystemError = frame_system::Error<Runtime>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let amount = 1_000_000_000_000;
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![
                (AccountId::from(ACCOUNT), amount),
                (AccountId::from(DELEGATE), amount),
                (AccountId::from(OTHER_ACCOUNT), amount),
            ],
            dev_accounts: None,
        },

        triumvirate: pallet_collective::GenesisConfig {
            members: vec![AccountId::from(ACCOUNT)],
            phantom: Default::default(),
        },
        senate_members: pallet_membership::GenesisConfig {
            members: BoundedVec::try_from(vec![AccountId::from(ACCOUNT)]).unwrap(),
            phantom: Default::default(),
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
    let value = 100;
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
fn call_propose() -> RuntimeCall {
    let proposal = call_remark();
    let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);

    RuntimeCall::Triumvirate(pallet_collective::Call::propose {
        proposal: Box::new(call_remark()),
        length_bound: proposal_len,
        duration: 100_000_000_u32,
    })
}

// critical call for Subtensor
fn call_root_register() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register {
        hotkey: AccountId::from(ACCOUNT),
    })
}

// triumvirate call
fn call_triumvirate() -> RuntimeCall {
    RuntimeCall::TriumvirateMembers(pallet_membership::Call::change_key {
        new: AccountId::from(ACCOUNT).into(),
    })
}

// senate call
fn call_senate() -> RuntimeCall {
    RuntimeCall::SenateMembers(pallet_membership::Call::change_key {
        new: AccountId::from(ACCOUNT).into(),
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

fn call_add_liquidity() -> RuntimeCall {
    let netuid = NetUid::from(1);
    let liquidity = 100;
    let hotkey = U256::from(hotkey);
    let tick_low = TickIndex::new(0).unwrap();
    let tick_high = TickIndex::new(100).unwrap();

    RuntimeCall::Swap(pallet_subtensor_swap::Call::add_liquidity {
        netuid,
        hotkey,
        tick_low,
        tick_high,
        liquidity,
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
        ProxyType::Senate,
        ProxyType::NonFungibile,
        ProxyType::Triumvirate,
        ProxyType::Governance,
        ProxyType::Staking,
        ProxyType::Registration,
        ProxyType::Liquidity,
    ];

    let calls = [
        call_transfer,
        call_remark,
        call_owner_util,
        call_propose,
        call_root_register,
        call_triumvirate,
        call_senate,
        call_add_stake,
        call_register,
        call_add_liquidity,
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
