#![cfg(test)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]
use crate::DerivativeSwapInterface;
use frame_support::{
    PalletId, derive_impl, parameter_types,
    traits::{OnFinalize, OnInitialize},
};
use sp_core::U256;
use sp_runtime::{BuildStorage, traits::IdentityLookup};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

use crate::pallet as pallet_derivatives;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: frame_system = 1,
      Balances: pallet_balances = 2,
      Crowdloan: pallet_derivatives = 3,
    }
);

#[allow(unused)]
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("Expected to not panic");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (U256::from(1), 10),
            (U256::from(2), 10),
            (U256::from(3), 10),
            (U256::from(4), 10),
            (U256::from(5), 3),
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
    type AccountId = U256;
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

pub struct MockSwap {
    // Reserves
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    // Price (not related to reserves for testing purposes)
    pub price: U96F32,
}

impl MockSwap {
    pub fn new(tao: TaoCurrency, alpha: AlphaCurrency, price: U96F32) -> Self {
        MockSwap {
            tao, alpha, price
        }
    }
    pub fn set_price(&mut self, new_price: U96F32) {
        self.price = new_price;
    }
}

impl DerivativeSwapInterface for MockSwap {
    fn buy(&mut self, _netuid: NetUid, _tao: TaoCurrency) -> AlphaCurrency {
        todo!();
    }
    fn sell(&mut self, _netuid: NetUid, _alpha: AlphaCurrency) -> TaoCurrency {
        todo!();
    }
    fn get_tao_for_alpha_amount(&mut self, _netuid: NetUid, _alpha: AlphaCurrency) -> TaoCurrency {
        todo!();
    }
    fn mint_alpha(&mut self, _netuid: NetUid, _alpha: AlphaCurrency) {
        todo!();
    }
    fn burn_alpha(&mut self, _netuid: NetUid, _alpha: AlphaCurrency) {
        todo!();
    }
}



impl pallet_derivatives::Config for Test {
    type PalletId = DerivativesPalletId;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
    type SwapInterface = MockSwap;
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
