use frame_support::assert_ok;
use node_subtensor_runtime::{
    AccountId, BalancesCall, BlockNumber, BuildStorage, Proxy, ProxyType, Runtime, RuntimeCall,
    RuntimeOrigin, SubtensorModule, System, SystemCall,
};
use sp_runtime::traits::{BlakeTwo256, Hash};
const ACCOUNT: [u8; 32] = [1_u8; 32];
const DELEGATE: [u8; 32] = [2_u8; 32];
const OTHER_ACCOUNT: [u8; 32] = [3_u8; 32];

pub fn new_test_ext(block_number: BlockNumber) -> sp_io::TestExternalities {
    let amount = 100_000_000_000;

    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (AccountId::from(ACCOUNT), amount),
            (AccountId::from(DELEGATE), amount),
            (AccountId::from(OTHER_ACCOUNT), amount),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);

    ext.execute_with(|| System::set_block_number(block_number));
    ext
}

pub fn add_network(netuid: u16, tempo: u16) {
    SubtensorModule::init_new_network(netuid, tempo);
    SubtensorModule::set_network_registration_allowed(netuid, true);
    SubtensorModule::set_network_pow_registration_allowed(netuid, true);
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
    let default_take = 0;
    RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_default_take { default_take })
}

// critical call for Subtensor
fn call_set_member() -> RuntimeCall {
    RuntimeCall::Triumvirate(pallet_collective::Call::set_members {
        new_members: vec![AccountId::from(OTHER_ACCOUNT).into()],
        prime: None,
        old_count: 0,
    })
}

// critical call for Subtensor
fn call_root_register() -> RuntimeCall {
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register {
        hotkey: AccountId::from(OTHER_ACCOUNT),
    })
}

// triumvirate call
fn call_triumvirate() -> RuntimeCall {
    RuntimeCall::TriumvirateMembers(pallet_membership::Call::add_member {
        who: AccountId::from(OTHER_ACCOUNT).into(),
    })
}

// senate call
fn call_senate() -> RuntimeCall {
    RuntimeCall::SenateMembers(pallet_membership::Call::add_member {
        who: AccountId::from(OTHER_ACCOUNT).into(),
    })
}

// staking call
fn call_add_stake() -> RuntimeCall {
    let amount_staked = 100;
    RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake {
        hotkey: AccountId::from(OTHER_ACCOUNT).into(),
        amount_staked,
    })
}

// register call
fn call_register() -> RuntimeCall {
    let block_number: u64 = 0;
    let netuid: u16 = 1;
    let tempo: u16 = 2;
    let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
        netuid,
        block_number,
        0,
        &AccountId::from(DELEGATE).into(),
    );

    add_network(netuid, tempo);

    RuntimeCall::SubtensorModule(pallet_subtensor::Call::register {
        netuid,
        block_number,
        nonce,
        work: work.clone(),
        hotkey: AccountId::from(DELEGATE).into(),
        coldkey: AccountId::from(OTHER_ACCOUNT).into(),
    })
}

#[test]
fn test_any_type() {
    new_test_ext(1).execute_with(|| {
        let block_number = 1;
        System::set_block_number(block_number);
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(AccountId::from(ACCOUNT)),
            AccountId::from(DELEGATE).into(),
            ProxyType::Any,
            0
        ));

        let call = Box::new(call_transfer());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_remark());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_owner_util());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_set_member());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_root_register());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_triumvirate());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_senate());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_add_stake());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));

        let call = Box::new(call_register());
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            call_hash,
        ));

        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(AccountId::from(DELEGATE)),
            AccountId::from(ACCOUNT).into(),
            None,
            call,
        ));
    });
}
