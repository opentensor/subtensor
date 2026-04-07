use {
    frame_support::assert_ok,
    node_subtensor_runtime::ExistentialDeposit,
    node_subtensor_runtime::{BuildStorage, Runtime, RuntimeGenesisConfig, System},
    pallet_subtensor::{
        BurnHalfLife, BurnIncreaseMult, Error, FirstEmissionBlockNumber, Pallet as SubtensorPallet,
        SubnetAlphaIn, SubnetAlphaInProvided, SubnetTAO, SubtokenEnabled,
    },
    substrate_fixed::types::U64F64,
    subtensor_runtime_common::{AccountId, AlphaBalance, NetUid, TaoBalance},
};

pub const ONE: [u8; 32] = [1_u8; 32];
pub const TWO: [u8; 32] = [2_u8; 32];
pub const THREE: [u8; 32] = [3_u8; 32];
pub const ONE_NO_BALANCE: [u8; 32] = [4_u8; 32];

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let amount = TaoBalance::from(1_000_000_000_000_u64);
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![
                (AccountId::from(ONE), amount),
                (AccountId::from(TWO), amount),
                (AccountId::from(THREE), amount),
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

pub fn add_network_disable_commit_reveal(netuid: NetUid, tempo: u16, _modality: u16) {
    add_network(netuid, tempo, _modality);
    SubtensorPallet::<Runtime>::set_commit_reveal_weights_enabled(netuid, false);
    SubtensorPallet::<Runtime>::set_yuma3_enabled(netuid, false);
}

pub fn add_network(netuid: NetUid, tempo: u16, _modality: u16) {
    SubtensorPallet::<Runtime>::init_new_network(netuid, tempo);
    SubtensorPallet::<Runtime>::set_network_registration_allowed(netuid, true);
    FirstEmissionBlockNumber::<Runtime>::insert(netuid, 1);
    SubtokenEnabled::<Runtime>::insert(netuid, true);

    // make interval 1 block so tests can register by stepping 1 block.
    BurnHalfLife::<Runtime>::insert(netuid, 1);
    BurnIncreaseMult::<Runtime>::insert(netuid, U64F64::from_num(1));
}

pub(crate) fn setup_reserves(netuid: NetUid, tao: TaoBalance, alpha: AlphaBalance) {
    SubnetTAO::<Runtime>::set(netuid, tao);
    SubnetAlphaIn::<Runtime>::set(netuid, alpha);
}

pub fn register_ok_neuron(
    netuid: NetUid,
    hotkey_account_id: AccountId,
    coldkey_account_id: AccountId,
    _start_nonce: u64,
) {
    SubtensorPallet::<Runtime>::set_burn(netuid, TaoBalance::from(0));
    let reserve: u64 = 1_000_000_000_000;
    let tao_reserve = SubnetTAO::<Runtime>::get(netuid);
    let alpha_reserve =
        SubnetAlphaIn::<Runtime>::get(netuid) + SubnetAlphaInProvided::<Runtime>::get(netuid);

    if tao_reserve == 0.into() && alpha_reserve == 0.into() {
        setup_reserves(netuid, reserve.into(), reserve.into());
    }

    // Ensure coldkey has enough to pay the current burn AND is not fully drained to zero.
    // This avoids ZeroBalanceAfterWithdrawn in burned_register.
    let top_up_for_burn = |netuid: NetUid, cold: AccountId| {
        let burn: TaoBalance = SubtensorPallet::<Runtime>::get_burn(netuid);
        let burn_u64: TaoBalance = burn;

        // Make sure something remains after withdrawal even if ED is 0 in tests.
        let ed: TaoBalance = ExistentialDeposit::get();
        let min_remaining: TaoBalance = ed.max(1.into());

        // Small buffer for safety (fees / rounding / future changes).
        let buffer: TaoBalance = 10.into();

        let min_balance_needed: TaoBalance = burn_u64 + min_remaining + buffer;

        let bal: TaoBalance = SubtensorPallet::<Runtime>::get_coldkey_balance(&cold);
        if bal < min_balance_needed {
            SubtensorPallet::<Runtime>::add_balance_to_coldkey_account(
                &cold,
                min_balance_needed - bal,
            );
        }
    };

    top_up_for_burn(netuid, coldkey_account_id.clone());

    let origin =
        <<Runtime as frame_system::Config>::RuntimeOrigin>::signed(coldkey_account_id.clone());
    let result = SubtensorPallet::<Runtime>::burned_register(
        origin.clone(),
        netuid,
        hotkey_account_id.clone(),
    );

    match result {
        Ok(()) => {
            // success
        }
        Err(e)
            if e == Error::<Runtime>::TooManyRegistrationsThisInterval.into()
                || e == Error::<Runtime>::NotEnoughBalanceToStake.into()
                || e == Error::<Runtime>::ZeroBalanceAfterWithdrawn.into() =>
        {
            // Re-top-up and retry once (burn can be state-dependent).
            top_up_for_burn(netuid, coldkey_account_id.clone());

            assert_ok!(SubtensorPallet::<Runtime>::burned_register(
                origin,
                netuid,
                hotkey_account_id.clone()
            ));
        }
        Err(e) => {
            panic!("Expected Ok(_). Got Err({e:?})");
        }
    }
    SubtensorPallet::<Runtime>::set_burn(netuid, TaoBalance::from(0));
    log::info!(
        "Register ok neuron: netuid: {netuid:?}, coldkey: {coldkey_account_id:?}, hotkey: {hotkey_account_id:?}"
    );
}
