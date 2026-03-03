#![cfg(test)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]
use crate::{DerivativeSwapInterface, DispatchError};
use core::num::NonZeroU64;
use frame_support::{
    PalletId, derive_impl, parameter_types,
    traits::{OnFinalize, OnInitialize},
};
use sp_runtime::{BuildStorage, traits::IdentityLookup};
use std::{cell::RefCell, collections::HashMap};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{
    AlphaCurrency, BalanceOps, CurrencyReserve, NetUid, SubnetInfo, TaoCurrency,
};
use subtensor_swap_interface::{Order, SwapHandler};

use crate::pallet as pallet_derivatives;

type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u32;
pub const COLDKEY1: AccountId = 1;
pub const HOTKEY1: AccountId = 1001;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: frame_system = 1,
      Balances: pallet_balances = 2,
      Derivatives: pallet_derivatives = 3,
      Swap: pallet_subtensor_swap = 4,
    }
);

#[allow(unused)]
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("Expected to not panic");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1_u32, 10),
            (2_u32, 10),
            (3_u32, 10),
            (4_u32, 10),
            (5_u32, 3),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .expect("Expected to not panic");
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = u32;
    type AccountData = pallet_balances::AccountData<u64>;
    type Lookup = IdentityLookup<Self::AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

parameter_types! {
    pub const DerivativesPalletId: PalletId = PalletId(*b"bt/deriv");
}

pub struct MockSwap;

pub type GetAlphaForTao = subtensor_swap_interface::GetAlphaForTao<TaoReserve, AlphaReserve>;
pub type GetTaoForAlpha = subtensor_swap_interface::GetTaoForAlpha<AlphaReserve, TaoReserve>;

impl MockSwap {
    pub fn get_alpha_out(netuid: NetUid) -> AlphaCurrency {
        if let Some(val) = MOCK_ALPHA_OUT.with(|m| m.borrow().get(&netuid).cloned()) {
            val
        } else {
            0.into()
        }
    }

    pub fn get_current_price(netuid: NetUid) -> U96F32 {
        <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::current_alpha_price(netuid)
    }
}

impl DerivativeSwapInterface for MockSwap {
    fn buy(netuid: NetUid, tao: TaoCurrency) -> AlphaCurrency {
        let order = GetAlphaForTao::with_amount(tao);
        let max_price = <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::max_price();
        let swap_result = <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::swap(
            netuid.into(),
            order,
            max_price,
            true,
            false,
        )
        .unwrap();
        swap_result.amount_paid_out
    }
    fn sell(netuid: NetUid, alpha: AlphaCurrency) -> TaoCurrency {
        let order = GetTaoForAlpha::with_amount(alpha);
        let min_price = <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::min_price();
        let swap_result = <pallet_subtensor_swap::Pallet<Test> as SwapHandler>::swap(
            netuid.into(),
            order,
            min_price,
            true,
            false,
        )
        .unwrap();
        swap_result.amount_paid_out
    }
    fn get_tao_for_alpha_amount(_netuid: NetUid, _alpha: AlphaCurrency) -> TaoCurrency {
        todo!();
    }
    fn mint_alpha(netuid: NetUid, alpha: AlphaCurrency) {
        let old = Self::get_alpha_out(netuid);
        MOCK_ALPHA_OUT.with(|m| {
            m.borrow_mut().insert(netuid, old + alpha);
        });
    }
    fn burn_alpha(netuid: NetUid, alpha: AlphaCurrency) {
        let old = Self::get_alpha_out(netuid);
        MOCK_ALPHA_OUT.with(|m| {
            m.borrow_mut().insert(netuid, old - alpha);
        });
    }
    fn get_alpha_ema_price(_netuid: NetUid) -> U96F32 {
        U96F32::from_num(0.001)
    }
}

parameter_types! {
    pub const CollateralRatio: u64 = 2_000_000_000;
}

impl pallet_derivatives::Config for Test {
    type PalletId = DerivativesPalletId;
    type BalanceOps = MockBalanceOps;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
    type SwapInterface = MockSwap;
    type CollateralRatio = CollateralRatio;
}

#[allow(dead_code)]
pub(crate) fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::reset_events();
        System::set_block_number(System::block_number() + 1);
        Balances::on_initialize(System::block_number());
        System::on_initialize(System::block_number());
    }
}

parameter_types! {
    pub const SwapProtocolId: PalletId = PalletId(*b"ten/swap");
    pub const SwapMaxFeeRate: u16 = 10000; // 15.26%
    pub const SwapMaxPositions: u32 = 100;
    pub const SwapMinimumLiquidity: u64 = 1_000;
    pub const SwapMinimumReserve: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1_000_000) };
}

impl pallet_subtensor_swap::Config for Test {
    type SubnetInfo = MockLiquidityProvider;
    type BalanceOps = MockBalanceOps;
    type ProtocolId = SwapProtocolId;
    type TaoReserve = TaoReserve;
    type AlphaReserve = AlphaReserve;
    type MaxFeeRate = SwapMaxFeeRate;
    type MaxPositions = SwapMaxPositions;
    type MinimumLiquidity = SwapMinimumLiquidity;
    type MinimumReserve = SwapMinimumReserve;
    type WeightInfo = pallet_subtensor_swap::weights::DefaultWeight<Test>;
}

// Mock implementor of SubnetInfo trait
pub struct MockLiquidityProvider;

impl SubnetInfo<AccountId> for MockLiquidityProvider {
    fn exists(_netuid: NetUid) -> bool {
        true
    }

    fn mechanism(netuid: NetUid) -> u16 {
        if netuid == NetUid::from(0) { 0 } else { 1 }
    }

    fn is_owner(_account_id: &AccountId, _netuid: NetUid) -> bool {
        true
    }

    // Only disable one subnet for testing
    fn is_subtoken_enabled(_netuid: NetUid) -> bool {
        true
    }

    fn get_validator_trust(_netuid: NetUid) -> Vec<u16> {
        vec![1000, 800, 600, 400]
    }

    fn get_validator_permit(_netuid: NetUid) -> Vec<bool> {
        vec![true, true, true, true]
    }

    fn hotkey_of_uid(_netuid: NetUid, uid: u16) -> Option<AccountId> {
        Some(uid as AccountId)
    }
}

pub struct MockBalanceOps;

thread_local! {
   // maps AccountId -> mocked tao balance
   static MOCK_TAO_BALANCES: RefCell<HashMap<AccountId, TaoCurrency>> =
       RefCell::new(HashMap::new());
   // maps AccountId -> mocked alpha balance
   static MOCK_ALPHA_BALANCES: RefCell<HashMap<AccountId, AlphaCurrency>> =
       RefCell::new(HashMap::new());
   // maps netuid -> mocked tao reserve
   static MOCK_TAO_RESERVES: RefCell<HashMap<NetUid, TaoCurrency>> =
       RefCell::new(HashMap::new());
   // maps netuid -> mocked alpha reserve
   static MOCK_ALPHA_RESERVES: RefCell<HashMap<NetUid, AlphaCurrency>> =
       RefCell::new(HashMap::new());
   // maps netuid -> mocked alpha outstanding
   static MOCK_ALPHA_OUT: RefCell<HashMap<NetUid, AlphaCurrency>> =
       RefCell::new(HashMap::new());
}

impl BalanceOps<AccountId> for MockBalanceOps {
    fn tao_balance(account_id: &AccountId) -> TaoCurrency {
        if let Some(val) = MOCK_TAO_BALANCES.with(|m| m.borrow().get(&account_id).cloned()) {
            val
        } else {
            0.into()
        }
    }

    fn alpha_balance(
        _: NetUid,
        coldkey_account_id: &AccountId,
        _hotkey_account_id: &AccountId,
    ) -> AlphaCurrency {
        if let Some(val) =
            MOCK_ALPHA_BALANCES.with(|m| m.borrow().get(&coldkey_account_id).cloned())
        {
            val
        } else {
            0.into()
        }
    }

    fn increase_balance(coldkey: &AccountId, tao: TaoCurrency) {
        let old = Self::tao_balance(coldkey);
        MOCK_TAO_BALANCES.with(|m| {
            m.borrow_mut().insert(*coldkey, old + tao);
        });
    }

    fn decrease_balance(
        coldkey: &AccountId,
        tao: TaoCurrency,
    ) -> Result<TaoCurrency, DispatchError> {
        let old = Self::tao_balance(coldkey);
        MOCK_TAO_BALANCES.with(|m| {
            // Just panic on underflows
            m.borrow_mut().insert(*coldkey, old - tao);
        });
        Ok(tao)
    }

    fn increase_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<(), DispatchError> {
        let old = Self::alpha_balance(netuid, coldkey, hotkey);
        MOCK_ALPHA_BALANCES.with(|m| {
            m.borrow_mut().insert(*coldkey, old + alpha);
        });
        Ok(())
    }

    fn decrease_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<AlphaCurrency, DispatchError> {
        let old = Self::alpha_balance(netuid, coldkey, hotkey);
        MOCK_ALPHA_BALANCES.with(|m| {
            // Just panic on underflows
            m.borrow_mut().insert(*coldkey, old - alpha);
        });
        Ok(alpha)
    }
}

#[derive(Clone)]
pub struct TaoReserve;

impl TaoReserve {
    pub fn set_mock_reserve(netuid: NetUid, value: TaoCurrency) {
        MOCK_TAO_RESERVES.with(|m| {
            m.borrow_mut().insert(netuid, value);
        });
    }
}

impl CurrencyReserve<TaoCurrency> for TaoReserve {
    fn reserve(netuid: NetUid) -> TaoCurrency {
        // If test has set an override, use it
        if let Some(val) = MOCK_TAO_RESERVES.with(|m| m.borrow().get(&netuid).cloned()) {
            val
        } else {
            0.into()
        }
    }

    fn increase_provided(_: NetUid, _: TaoCurrency) {}
    fn decrease_provided(_: NetUid, _: TaoCurrency) {}
}

#[derive(Clone)]
pub struct AlphaReserve;

impl AlphaReserve {
    pub fn set_mock_reserve(netuid: NetUid, value: AlphaCurrency) {
        MOCK_ALPHA_RESERVES.with(|m| {
            m.borrow_mut().insert(netuid, value);
        });
    }
}

impl CurrencyReserve<AlphaCurrency> for AlphaReserve {
    fn reserve(netuid: NetUid) -> AlphaCurrency {
        // If test has set an override, use it
        if let Some(val) = MOCK_ALPHA_RESERVES.with(|m| m.borrow().get(&netuid).cloned()) {
            val
        } else {
            0.into()
        }
    }

    fn increase_provided(_: NetUid, _: AlphaCurrency) {}
    fn decrease_provided(_: NetUid, _: AlphaCurrency) {}
}
