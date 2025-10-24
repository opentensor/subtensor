#![cfg(test)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]
use frame_support::{derive_impl, pallet_prelude::*, parameter_types};
use frame_system::{EnsureRoot, pallet_prelude::*};
use sp_core::U256;
use sp_runtime::{BuildStorage, traits::IdentityLookup};

use crate::{BalanceOf, pallet as pallet_governance};

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountOf<T> = <T as frame_system::Config>::AccountId;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: frame_system = 1,
      Balances: pallet_balances = 2,
      Preimage: pallet_preimage = 3,
      Governance: pallet_governance = 4,
      TestPallet: pallet_test = 5,
    }
);

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

impl pallet_preimage::Config for Test {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Test>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountOf<Test>>;
    type Consideration = ();
}

parameter_types! {
    pub const MaxAllowedProposers: u32 = 5;
    pub const MaxProposalWeight: Weight = Weight::from_parts(1_000_000_000_000, 0);
    pub const MaxProposals: u32 = 5;
}

impl pallet_governance::Config for Test {
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type Preimages = Preimage;
    type SetAllowedProposersOrigin = EnsureRoot<AccountOf<Test>>;
    type SetTriumvirateOrigin = EnsureRoot<AccountOf<Test>>;
    type MaxAllowedProposers = MaxAllowedProposers;
    type MaxProposalWeight = MaxProposalWeight;
    type MaxProposals = MaxProposals;
}

#[frame_support::pallet]
pub(crate) mod pallet_test {
    use super::MaxProposalWeight;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + Sized {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(MaxProposalWeight::get() * 2)]
        pub fn expensive_call(_origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}

impl pallet_test::Config for Test {}

pub(crate) struct TestState {
    block_number: BlockNumberFor<Test>,
    balances: Vec<(AccountOf<Test>, BalanceOf<Test>)>,
    allowed_proposers: Vec<AccountOf<Test>>,
    triumvirate: Vec<AccountOf<Test>>,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            block_number: 1,
            balances: vec![],
            allowed_proposers: vec![U256::from(1), U256::from(2), U256::from(3)],
            triumvirate: vec![U256::from(1001), U256::from(1002), U256::from(1003)],
        }
    }
}

impl TestState {
    pub(crate) fn with_allowed_proposers(
        mut self,
        allowed_proposers: Vec<AccountOf<Test>>,
    ) -> Self {
        self.allowed_proposers = allowed_proposers;
        self
    }

    pub(crate) fn with_triumvirate(mut self, triumvirate: Vec<AccountOf<Test>>) -> Self {
        self.triumvirate = triumvirate;
        self
    }

    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
            system: frame_system::GenesisConfig::default(),
            balances: pallet_balances::GenesisConfig {
                balances: self.balances,
                ..Default::default()
            },
            governance: pallet_governance::GenesisConfig {
                allowed_proposers: self.allowed_proposers,
                triumvirate: self.triumvirate,
            },
        }
        .build_storage()
        .unwrap()
        .into();
        ext.execute_with(|| System::set_block_number(self.block_number));
        ext
    }

    pub(crate) fn build_and_execute(self, test: impl FnOnce()) {
        self.build().execute_with(|| {
            test();
        });
    }
}

pub(crate) fn last_event() -> RuntimeEvent {
    System::events().pop().expect("RuntimeEvent expected").event
}
