#![cfg(test)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]
use frame_support::{derive_impl, pallet_prelude::*, parameter_types, traits::EqualPrivilegeOnly};
use frame_system::{EnsureRoot, limits, pallet_prelude::*};
use sp_core::U256;
use sp_runtime::{BuildStorage, Perbill, Percent, traits::IdentityLookup};
use sp_std::cell::RefCell;
use std::marker::PhantomData;

use crate::{
    BUILDING_COLLECTIVE_SIZE, BalanceOf, CollectiveMembersProvider, ECONOMIC_COLLECTIVE_SIZE,
    pallet as pallet_governance,
};

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountOf<T> = <T as frame_system::Config>::AccountId;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: frame_system = 1,
      Balances: pallet_balances = 2,
      Preimage: pallet_preimage = 3,
      Scheduler: pallet_scheduler = 4,
      Governance: pallet_governance = 5,
      TestPallet: pallet_test = 6,
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
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::with_sensible_defaults(
        Weight::from_parts(2_000_000_000_000, u64::MAX),
        Perbill::from_percent(75),
    );
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountOf<Test>>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Test>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}

pub struct FakeCollectiveMembersProvider<T: pallet_governance::Config>(PhantomData<T>);
impl<T: pallet_governance::Config> CollectiveMembersProvider<T> for FakeCollectiveMembersProvider<T>
where
    T::AccountId: From<AccountOf<Test>>,
{
    fn get_economic_collective() -> BoundedVec<T::AccountId, ConstU32<ECONOMIC_COLLECTIVE_SIZE>> {
        BoundedVec::truncate_from(ECONOMIC_COLLECTIVE.with(|c| {
            c.borrow()
                .iter()
                .map(|a| T::AccountId::from(a.clone()))
                .collect()
        }))
    }
    fn get_building_collective() -> BoundedVec<T::AccountId, ConstU32<BUILDING_COLLECTIVE_SIZE>> {
        BoundedVec::truncate_from(BUILDING_COLLECTIVE.with(|c| {
            c.borrow()
                .iter()
                .map(|a| T::AccountId::from(a.clone()))
                .collect()
        }))
    }
}

thread_local! {
    pub static ECONOMIC_COLLECTIVE: RefCell<Vec<AccountOf<Test>>> = const { RefCell::new(vec![]) };
    pub static BUILDING_COLLECTIVE: RefCell<Vec<AccountOf<Test>>> = const { RefCell::new(vec![]) };
}

#[macro_export]
macro_rules! set_next_economic_collective {
    ($members:expr) => {{
        assert_eq!($members.len(), ECONOMIC_COLLECTIVE_SIZE as usize);
        ECONOMIC_COLLECTIVE.with_borrow_mut(|c| *c = $members.clone());
    }};
}

#[macro_export]
macro_rules! set_next_building_collective {
    ($members:expr) => {{
        assert_eq!($members.len(), BUILDING_COLLECTIVE_SIZE as usize);
        BUILDING_COLLECTIVE.with_borrow_mut(|c| *c = $members.clone());
    }};
}

parameter_types! {
    pub const MaxAllowedProposers: u32 = 5;
    pub const MaxProposalWeight: Weight = Weight::from_parts(1_000_000_000_000, 0);
    pub const MaxProposals: u32 = 5;
    pub const MaxScheduled: u32 = 10;
    pub const MotionDuration: BlockNumberFor<Test> = 20;
    pub const InitialSchedulingDelay: BlockNumberFor<Test> = 20;
    pub const CollectiveRotationPeriod: BlockNumberFor<Test> = 100;
    pub const CleanupPeriod: BlockNumberFor<Test> = 500;
    pub const FastTrackThreshold: Percent = Percent::from_percent(67); // ~2/3
    pub const CancellationThreshold: Percent = Percent::from_percent(51);
    pub const EligibilityLockCost: BalanceOf<Test> = 1_000_000_000;
}

impl pallet_governance::Config for Test {
    type RuntimeCall = RuntimeCall;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type Preimages = Preimage;
    type Scheduler = Scheduler;
    type SetAllowedProposersOrigin = EnsureRoot<AccountOf<Test>>;
    type SetTriumvirateOrigin = EnsureRoot<AccountOf<Test>>;
    type CollectiveMembersProvider = FakeCollectiveMembersProvider<Test>;
    type MaxAllowedProposers = MaxAllowedProposers;
    type MaxProposalWeight = MaxProposalWeight;
    type MaxProposals = MaxProposals;
    type MaxScheduled = MaxScheduled;
    type MotionDuration = MotionDuration;
    type InitialSchedulingDelay = InitialSchedulingDelay;
    type CollectiveRotationPeriod = CollectiveRotationPeriod;
    type CleanupPeriod = CleanupPeriod;
    type CancellationThreshold = CancellationThreshold;
    type FastTrackThreshold = FastTrackThreshold;
    type EligibilityLockCost = EligibilityLockCost;
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
    economic_collective: BoundedVec<AccountOf<Test>, ConstU32<ECONOMIC_COLLECTIVE_SIZE>>,
    building_collective: BoundedVec<AccountOf<Test>, ConstU32<BUILDING_COLLECTIVE_SIZE>>,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            block_number: 1,
            balances: vec![],
            allowed_proposers: vec![U256::from(1), U256::from(2), U256::from(3)],
            triumvirate: vec![U256::from(1001), U256::from(1002), U256::from(1003)],
            economic_collective: BoundedVec::truncate_from(
                (1..=ECONOMIC_COLLECTIVE_SIZE)
                    .map(|i| U256::from(2000 + i))
                    .collect::<Vec<_>>(),
            ),
            building_collective: BoundedVec::truncate_from(
                (1..=BUILDING_COLLECTIVE_SIZE)
                    .map(|i| U256::from(3000 + i))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl TestState {
    pub(crate) fn with_balance(mut self, who: AccountOf<Test>, balance: BalanceOf<Test>) -> Self {
        self.balances.push((who, balance));
        self
    }

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
        ext.execute_with(|| {
            set_next_economic_collective!(self.economic_collective.to_vec());
            set_next_building_collective!(self.building_collective.to_vec());
            run_to_block(self.block_number);
        });
        ext
    }

    pub(crate) fn build_and_execute(self, test: impl FnOnce()) {
        self.build().execute_with(|| {
            test();
        });
    }
}

pub(crate) fn nth_last_event(n: usize) -> RuntimeEvent {
    System::events()
        .into_iter()
        .rev()
        .nth(n)
        .expect("RuntimeEvent expected")
        .event
}

pub(crate) fn last_event() -> RuntimeEvent {
    nth_last_event(0)
}

pub(crate) fn run_to_block(n: BlockNumberFor<Test>) {
    System::run_to_block::<AllPalletsWithSystem>(n);
}
