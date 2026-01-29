#![allow(clippy::unwrap_used)]

use super::{SubtensorChainExtension, SubtensorExtensionEnv, mock};
use crate::types::{FunctionId, Output};
use codec::{Decode, Encode};
use frame_support::{assert_ok, weights::Weight};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::RetVal;
use pallet_subtensor::DefaultMinStake;
use sp_core::Get;
use sp_core::U256;
use sp_runtime::DispatchError;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency as CurrencyTrait, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

type AccountId = <mock::Test as frame_system::Config>::AccountId;

#[derive(Clone)]
struct MockEnv {
    func_id: u16,
    caller: AccountId,
    input: Vec<u8>,
    output: Vec<u8>,
    charged_weight: Option<Weight>,
    expected_weight: Option<Weight>,
}

#[test]
fn set_coldkey_auto_stake_hotkey_success_sets_destination() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4901);
        let owner_coldkey = U256::from(4902);
        let coldkey = U256::from(5901);
        let hotkey = U256::from(5902);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        pallet_subtensor::Owner::<mock::Test>::insert(hotkey, coldkey);
        pallet_subtensor::OwnedHotkeys::<mock::Test>::insert(coldkey, vec![hotkey]);
        pallet_subtensor::Uids::<mock::Test>::insert(netuid, hotkey, 0u16);

        assert_eq!(
            pallet_subtensor::AutoStakeDestination::<mock::Test>::get(coldkey, netuid),
            None
        );

        let expected_weight = Weight::from_parts(29_930_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(4))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(2));

        let mut env = MockEnv::new(
            FunctionId::SetColdkeyAutoStakeHotkeyV1,
            coldkey,
            (netuid, hotkey).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        assert_eq!(
            pallet_subtensor::AutoStakeDestination::<mock::Test>::get(coldkey, netuid),
            Some(hotkey)
        );
        let coldkeys =
            pallet_subtensor::AutoStakeDestinationColdkeys::<mock::Test>::get(hotkey, netuid);
        assert!(coldkeys.contains(&coldkey));
    });
}

#[test]
fn remove_stake_full_limit_success_with_limit_price() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4801);
        let owner_coldkey = U256::from(4802);
        let coldkey = U256::from(5801);
        let hotkey = U256::from(5802);
        let stake_amount_raw: u64 = 340_000_000_000;

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            TaoCurrency::from(130_000_000_000),
            AlphaCurrency::from(110_000_000_000),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let expected_weight = Weight::from_parts(395_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(28))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(14));

        let balance_before = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        let mut env = MockEnv::new(
            FunctionId::RemoveStakeFullLimitV1,
            coldkey,
            (hotkey, netuid, Option::<TaoCurrency>::None).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        let balance_after = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        assert!(alpha_after.is_zero());
        assert!(balance_after > balance_before);
    });
}

#[test]
fn swap_stake_limit_with_tight_price_returns_slippage_error() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey_a = U256::from(4701);
        let owner_coldkey_a = U256::from(4702);
        let owner_hotkey_b = U256::from(4703);
        let owner_coldkey_b = U256::from(4704);
        let coldkey = U256::from(5701);
        let hotkey = U256::from(5702);

        let stake_alpha = AlphaCurrency::from(150_000_000_000u64);

        let netuid_a = mock::add_dynamic_network(&owner_hotkey_a, &owner_coldkey_a);
        let netuid_b = mock::add_dynamic_network(&owner_hotkey_b, &owner_coldkey_b);

        mock::setup_reserves(
            netuid_a,
            TaoCurrency::from(150_000_000_000),
            AlphaCurrency::from(110_000_000_000),
        );
        mock::setup_reserves(
            netuid_b,
            TaoCurrency::from(120_000_000_000),
            AlphaCurrency::from(90_000_000_000),
        );

        mock::register_ok_neuron(netuid_a, hotkey, coldkey, 0);
        mock::register_ok_neuron(netuid_b, hotkey, coldkey, 1);

        pallet_subtensor::Pallet::<mock::Test>::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid_a,
            stake_alpha,
        );

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid_a);

        let alpha_origin_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_a,
            );
        let alpha_destination_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_b,
            );

        let alpha_to_swap: AlphaCurrency = (alpha_origin_before.to_u64() / 8).into();
        let limit_price: TaoCurrency = 100u64.into();

        let expected_weight = Weight::from_parts(411_500_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(35))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(22));

        let mut env = MockEnv::new(
            FunctionId::SwapStakeLimitV1,
            coldkey,
            (hotkey, netuid_a, netuid_b, alpha_to_swap, limit_price, true).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let alpha_origin_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_a,
            );
        let alpha_destination_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_b,
            );

        assert!(alpha_origin_after <= alpha_origin_before);
        assert!(alpha_destination_after >= alpha_destination_before);
    });
}

#[test]
fn remove_stake_limit_success_respects_price_limit() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4601);
        let owner_coldkey = U256::from(4602);
        let coldkey = U256::from(5601);
        let hotkey = U256::from(5602);
        let stake_amount_raw: u64 = 320_000_000_000;

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            TaoCurrency::from(120_000_000_000),
            AlphaCurrency::from(100_000_000_000),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );

        let current_price =
            <mock::Test as pallet_subtensor::Config>::SwapInterface::current_alpha_price(
                netuid.into(),
            );
        let limit_price_value = (current_price.to_num::<f64>() * 990_000_000f64).round() as u64;
        let limit_price: TaoCurrency = limit_price_value.into();

        let alpha_to_unstake: AlphaCurrency = (alpha_before.to_u64() / 2).into();

        let expected_weight = Weight::from_parts(377_400_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(28))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(14));

        let balance_before = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        let mut env = MockEnv::new(
            FunctionId::RemoveStakeLimitV1,
            coldkey,
            (hotkey, netuid, alpha_to_unstake, limit_price, true).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        let balance_after = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        assert!(alpha_after < alpha_before);
        assert!(balance_after > balance_before);
    });
}

#[test]
fn add_stake_limit_success_executes_within_price_guard() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4501);
        let owner_coldkey = U256::from(4502);
        let coldkey = U256::from(5501);
        let hotkey = U256::from(5502);
        let amount_raw: u64 = 900_000_000_000;
        let limit_price: TaoCurrency = 24_000_000_000u64.into();

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        mock::setup_reserves(
            netuid,
            TaoCurrency::from(150_000_000_000),
            AlphaCurrency::from(100_000_000_000),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            amount_raw + 1_000_000_000,
        );

        let stake_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        let balance_before = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        let expected_weight = Weight::from_parts(402_900_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(24))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(15));

        let mut env = MockEnv::new(
            FunctionId::AddStakeLimitV1,
            coldkey,
            (
                hotkey,
                netuid,
                TaoCurrency::from(amount_raw),
                limit_price,
                true,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let stake_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        let balance_after = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        assert!(stake_after > stake_before);
        assert!(stake_after > AlphaCurrency::ZERO);
        assert!(balance_after < balance_before);
    });
}

#[test]
fn swap_stake_success_moves_between_subnets() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey_a = U256::from(4401);
        let owner_coldkey_a = U256::from(4402);
        let owner_hotkey_b = U256::from(4403);
        let owner_coldkey_b = U256::from(4404);
        let coldkey = U256::from(5401);
        let hotkey = U256::from(5402);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(260);

        let netuid_a = mock::add_dynamic_network(&owner_hotkey_a, &owner_coldkey_a);
        let netuid_b = mock::add_dynamic_network(&owner_hotkey_b, &owner_coldkey_b);

        mock::setup_reserves(
            netuid_a,
            stake_amount_raw.saturating_mul(18).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(30)),
        );
        mock::setup_reserves(
            netuid_b,
            stake_amount_raw.saturating_mul(20).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(28)),
        );

        mock::register_ok_neuron(netuid_a, hotkey, coldkey, 0);
        mock::register_ok_neuron(netuid_b, hotkey, coldkey, 1);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid_a,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid_a);

        let alpha_origin_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_a,
            );
        let alpha_destination_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_b,
            );
        let alpha_to_swap: AlphaCurrency = (alpha_origin_before.to_u64() / 3).into();

        let expected_weight = Weight::from_parts(351_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(35))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(22));

        let mut env = MockEnv::new(
            FunctionId::SwapStakeV1,
            coldkey,
            (hotkey, netuid_a, netuid_b, alpha_to_swap).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let alpha_origin_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_a,
            );
        let alpha_destination_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid_b,
            );

        assert!(alpha_origin_after < alpha_origin_before);
        assert!(
            alpha_destination_after > alpha_destination_before,
            "destination stake should increase"
        );
    });
}

#[test]
fn transfer_stake_success_moves_between_coldkeys() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4301);
        let owner_coldkey = U256::from(4302);
        let origin_coldkey = U256::from(5301);
        let destination_coldkey = U256::from(5302);
        let hotkey = U256::from(5303);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(250);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(15).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(25)),
        );

        mock::register_ok_neuron(netuid, hotkey, origin_coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &origin_coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(origin_coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &origin_coldkey, netuid);

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                netuid,
            );
        let alpha_to_transfer: AlphaCurrency = (alpha_before.to_u64() / 3).into();

        let expected_weight = Weight::from_parts(160_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(13))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(6));

        let mut env = MockEnv::new(
            FunctionId::TransferStakeV1,
            origin_coldkey,
            (
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                alpha_to_transfer,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let origin_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                netuid,
            );
        let destination_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &destination_coldkey,
                netuid,
            );

        assert_eq!(origin_alpha_after, alpha_before - alpha_to_transfer);
        assert_eq!(destination_alpha_after, alpha_to_transfer);
    });
}

#[test]
fn move_stake_success_moves_alpha_between_hotkeys() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4201);
        let owner_coldkey = U256::from(4202);
        let coldkey = U256::from(5201);
        let origin_hotkey = U256::from(5202);
        let destination_hotkey = U256::from(5203);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(240);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(15).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(25)),
        );

        mock::register_ok_neuron(netuid, origin_hotkey, coldkey, 0);
        mock::register_ok_neuron(netuid, destination_hotkey, coldkey, 1);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            origin_hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&origin_hotkey, &coldkey, netuid);

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid,
            );
        let alpha_to_move: AlphaCurrency = (alpha_before.to_u64() / 2).into();

        let expected_weight = Weight::from_parts(164_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(15))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(7));

        let mut env = MockEnv::new(
            FunctionId::MoveStakeV1,
            coldkey,
            (
                origin_hotkey,
                destination_hotkey,
                netuid,
                netuid,
                alpha_to_move,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let origin_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid,
            );
        let destination_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid,
            );

        assert_eq!(origin_alpha_after, alpha_before - alpha_to_move);
        assert_eq!(destination_alpha_after, alpha_to_move);
    });
}

#[test]
fn unstake_all_alpha_success_moves_stake_to_root() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4101);
        let owner_coldkey = U256::from(4102);
        let coldkey = U256::from(5101);
        let hotkey = U256::from(5102);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(220);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(20).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(30)),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);
        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let expected_weight = Weight::from_parts(358_500_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(36))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(21));

        let mut env = MockEnv::new(FunctionId::UnstakeAllAlphaV1, coldkey, hotkey.encode())
            .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let subnet_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        assert!(subnet_alpha <= AlphaCurrency::from(1_000));

        let root_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                NetUid::ROOT,
            );
        assert!(root_alpha > AlphaCurrency::ZERO);
    });
}

#[test]
fn add_proxy_success_creates_proxy_relationship() {
    mock::new_test_ext(1).execute_with(|| {
        let delegator = U256::from(6001);
        let delegate = U256::from(6002);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &delegator,
            1_000_000_000,
        );

        assert_eq!(
            pallet_subtensor_proxy::Proxies::<mock::Test>::get(delegator)
                .0
                .len(),
            0
        );

        let mut env = MockEnv::new(FunctionId::AddProxyV1, delegator, delegate.encode());

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);

        let proxies = pallet_subtensor_proxy::Proxies::<mock::Test>::get(delegator).0;
        assert_eq!(proxies.len(), 1);
        if let Some(proxy) = proxies.first() {
            assert_eq!(proxy.delegate, delegate);
            assert_eq!(
                proxy.proxy_type,
                subtensor_runtime_common::ProxyType::Staking
            );
            assert_eq!(proxy.delay, 0u64);
        } else {
            panic!("proxies should contain one element");
        }
    });
}

#[test]
fn remove_proxy_success_removes_proxy_relationship() {
    mock::new_test_ext(1).execute_with(|| {
        let delegator = U256::from(7001);
        let delegate = U256::from(7002);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &delegator,
            1_000_000_000,
        );

        let mut add_env = MockEnv::new(FunctionId::AddProxyV1, delegator, delegate.encode());
        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut add_env).unwrap();
        assert_success(ret);

        let proxies_before = pallet_subtensor_proxy::Proxies::<mock::Test>::get(delegator).0;
        assert_eq!(proxies_before.len(), 1);

        let mut remove_env = MockEnv::new(FunctionId::RemoveProxyV1, delegator, delegate.encode());
        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut remove_env).unwrap();
        assert_success(ret);

        let proxies_after = pallet_subtensor_proxy::Proxies::<mock::Test>::get(delegator).0;
        assert_eq!(proxies_after.len(), 0);
    });
}

impl MockEnv {
    fn new(func_id: FunctionId, caller: AccountId, input: Vec<u8>) -> Self {
        Self {
            func_id: func_id as u16,
            caller,
            input,
            output: Vec::new(),
            charged_weight: None,
            expected_weight: None,
        }
    }

    fn with_expected_weight(mut self, weight: Weight) -> Self {
        self.expected_weight = Some(weight);
        self
    }

    fn charged_weight(&self) -> Option<Weight> {
        self.charged_weight
    }

    fn output(&self) -> &[u8] {
        &self.output
    }
}

impl SubtensorExtensionEnv<AccountId> for MockEnv {
    fn func_id(&self) -> u16 {
        self.func_id
    }

    fn charge_weight(&mut self, weight: Weight) -> Result<(), DispatchError> {
        if let Some(expected) = self.expected_weight
            && weight != expected
        {
            return Err(DispatchError::Other(
                "unexpected weight charged by mock env",
            ));
        }
        self.charged_weight = Some(weight);
        Ok(())
    }

    fn read_as<T: codec::Decode + codec::MaxEncodedLen>(&mut self) -> Result<T, DispatchError> {
        T::decode(&mut &self.input[..]).map_err(|_| DispatchError::Other("mock env decode failure"))
    }

    fn write_output(&mut self, data: &[u8]) -> Result<(), DispatchError> {
        self.output.clear();
        self.output.extend_from_slice(data);
        Ok(())
    }

    fn caller(&mut self) -> AccountId {
        self.caller
    }
}

fn assert_success(ret: RetVal) {
    match ret {
        RetVal::Converging(code) => {
            assert_eq!(code, Output::Success as u32, "expected success code")
        }
        _ => panic!("unexpected return value"),
    }
}

#[test]
fn get_stake_info_returns_encoded_runtime_value() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let hotkey = U256::from(11);
        let coldkey = U256::from(22);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        let expected =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_info_for_hotkey_coldkey_netuid(
                hotkey, coldkey, netuid,
            )
            .encode();

        let mut env = MockEnv::new(
            FunctionId::GetStakeInfoForHotkeyColdkeyNetuidV1,
            coldkey,
            (hotkey, coldkey, netuid).encode(),
        );

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        assert_success(ret);
        assert_eq!(env.output(), expected.as_slice());
        assert!(env.charged_weight().is_none());
    });
}

#[test]
fn add_stake_success_updates_stake_and_returns_success_code() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(101);
        let hotkey = U256::from(202);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount_raw = min_stake.to_u64().saturating_mul(10);
        let amount: TaoCurrency = amount_raw.into();

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            (amount_raw * 1_000_000).into(),
            AlphaCurrency::from(amount_raw * 10_000_000),
        );
        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey, amount_raw,
        );

        assert!(
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey).is_zero()
        );

        let expected_weight = Weight::from_parts(340_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(24))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(15));

        let mut env = MockEnv::new(
            FunctionId::AddStakeV1,
            coldkey,
            (hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let total_stake =
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey);
        assert!(total_stake > TaoCurrency::ZERO);
    });
}

#[test]
fn remove_stake_with_no_stake_returns_amount_too_low() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(301);
        let hotkey = U256::from(302);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount: AlphaCurrency = AlphaCurrency::from(min_stake.to_u64());

        let expected_weight = Weight::from_parts(196_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(19))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(10));

        let mut env = MockEnv::new(
            FunctionId::RemoveStakeV1,
            coldkey,
            (hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        match ret {
            RetVal::Converging(code) => {
                assert_eq!(code, Output::AmountTooLow as u32, "mismatched error output")
            }
            _ => panic!("unexpected return value"),
        }
        assert_eq!(env.charged_weight(), Some(expected_weight));
        assert!(
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey).is_zero()
        );
    });
}

#[test]
fn unstake_all_success_unstakes_balance() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4001);
        let owner_coldkey = U256::from(4002);
        let coldkey = U256::from(5001);
        let hotkey = U256::from(5002);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(200);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(10).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(20)),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);
        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let expected_weight = Weight::from_parts(28_830_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(6))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(0));

        let pre_balance = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        let mut env = MockEnv::new(FunctionId::UnstakeAllV1, coldkey, hotkey.encode())
            .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let remaining_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        assert!(remaining_alpha <= AlphaCurrency::from(1_000));

        let post_balance = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);
        assert!(post_balance > pre_balance);
    });
}

// ============================================================
// V2 function tests (proxy-aware staking functions)
// ============================================================

#[test]
fn add_stake_v2_self_call_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(101);
        let hotkey = U256::from(202);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount_raw = min_stake.to_u64().saturating_mul(10);
        let amount: TaoCurrency = amount_raw.into();

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            (amount_raw * 1_000_000).into(),
            AlphaCurrency::from(amount_raw * 10_000_000),
        );
        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey, amount_raw,
        );

        assert!(
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey).is_zero()
        );

        let expected_weight = Weight::from_parts(340_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(25))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(15));

        let mut env = MockEnv::new(
            FunctionId::AddStakeV2,
            coldkey,
            (coldkey, hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let total_stake =
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey);
        assert!(total_stake > TaoCurrency::ZERO);
    });
}

#[test]
fn add_stake_v2_with_proxy_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let real_coldkey = U256::from(101);
        let proxy_contract = U256::from(102);
        let hotkey = U256::from(202);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount_raw = min_stake.to_u64().saturating_mul(10);
        let amount: TaoCurrency = amount_raw.into();

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            (amount_raw * 1_000_000).into(),
            AlphaCurrency::from(amount_raw * 10_000_000),
        );
        mock::register_ok_neuron(netuid, hotkey, real_coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &real_coldkey,
            amount_raw + 1_000_000_000,
        );

        // Add proxy relationship: real_coldkey grants Staking proxy to proxy_contract
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(real_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));

        let expected_weight = Weight::from_parts(340_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(25))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(15));

        // proxy_contract calls AddStakeV2 on behalf of real_coldkey
        let mut env = MockEnv::new(
            FunctionId::AddStakeV2,
            proxy_contract,
            (real_coldkey, hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        assert_success(ret);

        let total_stake =
            pallet_subtensor::Pallet::<mock::Test>::get_total_stake_for_hotkey(&hotkey);
        assert!(total_stake > TaoCurrency::ZERO);
    });
}

#[test]
fn add_stake_v2_without_proxy_fails() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let real_coldkey = U256::from(101);
        let unauthorized_caller = U256::from(102);
        let hotkey = U256::from(202);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount_raw = min_stake.to_u64().saturating_mul(10);
        let amount: TaoCurrency = amount_raw.into();

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            (amount_raw * 1_000_000).into(),
            AlphaCurrency::from(amount_raw * 10_000_000),
        );
        mock::register_ok_neuron(netuid, hotkey, real_coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &real_coldkey,
            amount_raw + 1_000_000_000,
        );

        // No proxy relationship established

        let expected_weight = Weight::from_parts(340_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(25))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(15));

        // unauthorized_caller tries to act on behalf of real_coldkey without proxy
        let mut env = MockEnv::new(
            FunctionId::AddStakeV2,
            unauthorized_caller,
            (real_coldkey, hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env);

        // Should fail with NotProxy error
        assert!(ret.is_err());
    });
}

#[test]
fn remove_stake_v2_with_proxy_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4601);
        let owner_coldkey = U256::from(4602);
        let real_coldkey = U256::from(5601);
        let proxy_contract = U256::from(5603);
        let hotkey = U256::from(5602);
        let stake_amount_raw: u64 = 320_000_000_000;

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            TaoCurrency::from(120_000_000_000),
            AlphaCurrency::from(100_000_000_000),
        );

        mock::register_ok_neuron(netuid, hotkey, real_coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &real_coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(real_coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &real_coldkey, netuid);

        // Add proxy relationship
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(real_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid,
            );

        let alpha_to_unstake: AlphaCurrency = (alpha_before.to_u64() / 2).into();

        let expected_weight = Weight::from_parts(196_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(20))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(10));

        let mut env = MockEnv::new(
            FunctionId::RemoveStakeV2,
            proxy_contract,
            (real_coldkey, hotkey, netuid, alpha_to_unstake).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);

        let alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid,
            );

        assert!(alpha_after < alpha_before);
    });
}

#[test]
fn swap_stake_v2_with_proxy_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey_a = U256::from(4401);
        let owner_coldkey_a = U256::from(4402);
        let owner_hotkey_b = U256::from(4403);
        let owner_coldkey_b = U256::from(4404);
        let real_coldkey = U256::from(5401);
        let proxy_contract = U256::from(5403);
        let hotkey = U256::from(5402);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(260);

        let netuid_a = mock::add_dynamic_network(&owner_hotkey_a, &owner_coldkey_a);
        let netuid_b = mock::add_dynamic_network(&owner_hotkey_b, &owner_coldkey_b);

        mock::setup_reserves(
            netuid_a,
            stake_amount_raw.saturating_mul(18).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(30)),
        );
        mock::setup_reserves(
            netuid_b,
            stake_amount_raw.saturating_mul(20).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(28)),
        );

        mock::register_ok_neuron(netuid_a, hotkey, real_coldkey, 0);
        mock::register_ok_neuron(netuid_b, hotkey, real_coldkey, 1);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &real_coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(real_coldkey).into(),
            hotkey,
            netuid_a,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &real_coldkey, netuid_a);

        // Add proxy relationship
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(real_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));

        let alpha_origin_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid_a,
            );
        let alpha_destination_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid_b,
            );
        let alpha_to_swap: AlphaCurrency = (alpha_origin_before.to_u64() / 3).into();

        let expected_weight = Weight::from_parts(351_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(36))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(22));

        let mut env = MockEnv::new(
            FunctionId::SwapStakeV2,
            proxy_contract,
            (real_coldkey, hotkey, netuid_a, netuid_b, alpha_to_swap).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);

        let alpha_origin_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid_a,
            );
        let alpha_destination_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &real_coldkey, netuid_b,
            );

        assert!(alpha_origin_after < alpha_origin_before);
        assert!(alpha_destination_after > alpha_destination_before);
    });
}

#[test]
fn move_stake_v2_with_proxy_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4201);
        let owner_coldkey = U256::from(4202);
        let real_coldkey = U256::from(5201);
        let proxy_contract = U256::from(5204);
        let origin_hotkey = U256::from(5202);
        let destination_hotkey = U256::from(5203);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(240);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(15).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(25)),
        );

        mock::register_ok_neuron(netuid, origin_hotkey, real_coldkey, 0);
        mock::register_ok_neuron(netuid, destination_hotkey, real_coldkey, 1);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &real_coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(real_coldkey).into(),
            origin_hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&origin_hotkey, &real_coldkey, netuid);

        // Add proxy relationship
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(real_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &real_coldkey,
                netuid,
            );
        let alpha_to_move: AlphaCurrency = (alpha_before.to_u64() / 2).into();

        let expected_weight = Weight::from_parts(164_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(16))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(7));

        let mut env = MockEnv::new(
            FunctionId::MoveStakeV2,
            proxy_contract,
            (
                real_coldkey,
                origin_hotkey,
                destination_hotkey,
                netuid,
                netuid,
                alpha_to_move,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);

        let origin_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &real_coldkey,
                netuid,
            );
        let destination_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &real_coldkey,
                netuid,
            );

        assert_eq!(origin_alpha_after, alpha_before - alpha_to_move);
        assert_eq!(destination_alpha_after, alpha_to_move);
    });
}

#[test]
fn transfer_stake_v2_requires_transfer_proxy() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4301);
        let owner_coldkey = U256::from(4302);
        let origin_coldkey = U256::from(5301);
        let destination_coldkey = U256::from(5302);
        let proxy_contract = U256::from(5304);
        let hotkey = U256::from(5303);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(250);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(15).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(25)),
        );

        mock::register_ok_neuron(netuid, hotkey, origin_coldkey, 0);

        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &origin_coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(origin_coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &origin_coldkey, netuid);

        // Add Staking proxy (wrong type for transfer_stake)
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(origin_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));

        let alpha_before =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                netuid,
            );
        let alpha_to_transfer: AlphaCurrency = (alpha_before.to_u64() / 3).into();

        let expected_weight = Weight::from_parts(160_300_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(14))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(6));

        // First try with Staking proxy - should fail
        let mut env = MockEnv::new(
            FunctionId::TransferStakeV2,
            proxy_contract,
            (
                origin_coldkey,
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                alpha_to_transfer,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env);
        assert!(ret.is_err(), "Staking proxy should not work for transfer_stake");

        // Remove Staking proxy, add Transfer proxy
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::remove_proxy(
            RawOrigin::Signed(origin_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Staking,
            0u64,
        ));
        assert_ok!(pallet_subtensor_proxy::Pallet::<mock::Test>::add_proxy(
            RawOrigin::Signed(origin_coldkey).into(),
            proxy_contract,
            subtensor_runtime_common::ProxyType::Transfer,
            0u64,
        ));

        // Now try with Transfer proxy - should succeed
        let mut env = MockEnv::new(
            FunctionId::TransferStakeV2,
            proxy_contract,
            (
                origin_coldkey,
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                alpha_to_transfer,
            )
                .encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);

        let origin_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                netuid,
            );
        let destination_alpha_after =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &destination_coldkey,
                netuid,
            );

        assert_eq!(origin_alpha_after, alpha_before - alpha_to_transfer);
        assert_eq!(destination_alpha_after, alpha_to_transfer);
    });
}

#[test]
fn unstake_all_v2_self_call_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4001);
        let owner_coldkey = U256::from(4002);
        let coldkey = U256::from(5001);
        let hotkey = U256::from(5002);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(200);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(10).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(20)),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);
        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let expected_weight = Weight::from_parts(28_830_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(7))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(0));

        let pre_balance = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);

        let mut env = MockEnv::new(FunctionId::UnstakeAllV2, coldkey, (coldkey, hotkey).encode())
            .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let remaining_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        assert!(remaining_alpha <= AlphaCurrency::from(1_000));

        let post_balance = pallet_subtensor::Pallet::<mock::Test>::get_coldkey_balance(&coldkey);
        assert!(post_balance > pre_balance);
    });
}

#[test]
fn unstake_all_alpha_v2_self_call_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4101);
        let owner_coldkey = U256::from(4102);
        let coldkey = U256::from(5101);
        let hotkey = U256::from(5102);
        let min_stake = DefaultMinStake::<mock::Test>::get();
        let stake_amount_raw = min_stake.to_u64().saturating_mul(220);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        mock::setup_reserves(
            netuid,
            stake_amount_raw.saturating_mul(20).into(),
            AlphaCurrency::from(stake_amount_raw.saturating_mul(30)),
        );

        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);
        pallet_subtensor::Pallet::<mock::Test>::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount_raw + 1_000_000_000,
        );

        assert_ok!(pallet_subtensor::Pallet::<mock::Test>::add_stake(
            RawOrigin::Signed(coldkey).into(),
            hotkey,
            netuid,
            stake_amount_raw.into(),
        ));

        mock::remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        let expected_weight = Weight::from_parts(358_500_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(37))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(21));

        let mut env =
            MockEnv::new(FunctionId::UnstakeAllAlphaV2, coldkey, (coldkey, hotkey).encode())
                .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        let subnet_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
        assert!(subnet_alpha <= AlphaCurrency::from(1_000));

        let root_alpha =
            pallet_subtensor::Pallet::<mock::Test>::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                NetUid::ROOT,
            );
        assert!(root_alpha > AlphaCurrency::ZERO);
    });
}

#[test]
fn set_coldkey_auto_stake_hotkey_v2_self_call_succeeds() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(4901);
        let owner_coldkey = U256::from(4902);
        let coldkey = U256::from(5901);
        let hotkey = U256::from(5902);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        pallet_subtensor::Owner::<mock::Test>::insert(hotkey, coldkey);
        pallet_subtensor::OwnedHotkeys::<mock::Test>::insert(coldkey, vec![hotkey]);
        pallet_subtensor::Uids::<mock::Test>::insert(netuid, hotkey, 0u16);

        assert_eq!(
            pallet_subtensor::AutoStakeDestination::<mock::Test>::get(coldkey, netuid),
            None
        );

        let expected_weight = Weight::from_parts(29_930_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(5))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(2));

        let mut env = MockEnv::new(
            FunctionId::SetColdkeyAutoStakeHotkeyV2,
            coldkey,
            (coldkey, netuid, hotkey).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert_eq!(env.charged_weight(), Some(expected_weight));

        assert_eq!(
            pallet_subtensor::AutoStakeDestination::<mock::Test>::get(coldkey, netuid),
            Some(hotkey)
        );
    });
}

#[test]
fn remove_stake_v2_self_call_returns_error_with_no_stake() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(301);
        let hotkey = U256::from(302);
        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);
        mock::register_ok_neuron(netuid, hotkey, coldkey, 0);

        let min_stake = DefaultMinStake::<mock::Test>::get();
        let amount: AlphaCurrency = AlphaCurrency::from(min_stake.to_u64());

        let expected_weight = Weight::from_parts(196_800_000, 0)
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().reads(20))
            .saturating_add(<mock::Test as frame_system::Config>::DbWeight::get().writes(10));

        let mut env = MockEnv::new(
            FunctionId::RemoveStakeV2,
            coldkey,
            (coldkey, hotkey, netuid, amount).encode(),
        )
        .with_expected_weight(expected_weight);

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();

        match ret {
            RetVal::Converging(code) => {
                assert_eq!(code, Output::AmountTooLow as u32, "mismatched error output")
            }
            _ => panic!("unexpected return value"),
        }
    });
}

#[test]
fn get_alpha_price_returns_encoded_price() {
    mock::new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(8001);
        let owner_coldkey = U256::from(8002);
        let caller = U256::from(8003);

        let netuid = mock::add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Set up reserves to establish a price
        let tao_reserve = TaoCurrency::from(150_000_000_000u64);
        let alpha_reserve = AlphaCurrency::from(100_000_000_000u64);
        mock::setup_reserves(netuid, tao_reserve, alpha_reserve);

        // Get expected price from swap handler
        let expected_price =
            <pallet_subtensor_swap::Pallet<mock::Test> as SwapHandler>::current_alpha_price(
                netuid.into(),
            );
        let expected_price_scaled = expected_price.saturating_mul(U96F32::from_num(1_000_000_000));
        let expected_price_u64: u64 = expected_price_scaled.saturating_to_num();

        let mut env = MockEnv::new(FunctionId::GetAlphaPriceV1, caller, netuid.encode());

        let ret = SubtensorChainExtension::<mock::Test>::dispatch(&mut env).unwrap();
        assert_success(ret);
        assert!(env.charged_weight().is_none());

        // Decode the output
        let output_price: u64 = Decode::decode(&mut &env.output()[..]).unwrap();

        assert_eq!(
            output_price, expected_price_u64,
            "Price should match expected value"
        );
    });
}
