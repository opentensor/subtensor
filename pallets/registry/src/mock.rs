#![cfg(test)]
#![allow(clippy::expect_used)]
use crate as pallet_registry;
use frame_support::{derive_impl, parameter_types};
use sp_core::U256;
use sp_runtime::{BuildStorage, traits::IdentityLookup};
use subtensor_runtime_common::TaoBalance;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system = 1,
        Balances: pallet_balances = 2,
        Registry: pallet_registry = 3,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = U256;
    type AccountData = pallet_balances::AccountData<TaoBalance>;
    type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
    pub const ExistentialDeposit: TaoBalance = TaoBalance::new(1);
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = TaoBalance;
    type ExistentialDeposit = ExistentialDeposit;
}

parameter_types! {
    pub const MaxAdditionalFields: u32 = 16;
    pub const InitialDeposit: TaoBalance = TaoBalance::new(100);
    pub const FieldDeposit: TaoBalance = TaoBalance::new(10);
}

pub struct CanRegister;
impl pallet_registry::CanRegisterIdentity<U256> for CanRegister {
    fn can_register(who: &U256, identified: &U256) -> bool {
        who == identified
    }
}

impl pallet_registry::Config for Test {
    type Currency = Balances;
    type WeightInfo = ();
    type MaxAdditionalFields = MaxAdditionalFields;
    type CanRegister = CanRegister;
    type InitialDeposit = InitialDeposit;
    type FieldDeposit = FieldDeposit;
    type RuntimeHoldReason = RuntimeHoldReason;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("system storage should build ok");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (U256::from(1), 10.into()),
            (U256::from(2), 10.into()),
            (U256::from(3), 10.into()),
            (U256::from(4), 10.into()),
            (U256::from(5), 3.into()),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .expect("balances storage should build ok");
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
