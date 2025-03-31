#![cfg(test)]

use frame_support::{
    PalletId, assert_err, assert_ok, derive_impl, parameter_types,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::U256;
use sp_runtime::{BuildStorage, DispatchError, traits::IdentityLookup};

use crate::{BalanceOf, CrowdloanId, CrowdloanInfo, pallet as pallet_crowdloan};

type Block = frame_system::mocking::MockBlock<Test>;
type AccountOf<T> = <T as frame_system::Config>::AccountId;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: frame_system = 1,
      Balances: pallet_balances = 2,
      Crowdloan: pallet_crowdloan = 3,
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

parameter_types! {
    pub const CrowdloanPalletId: PalletId = PalletId(*b"bt/cloan");
    pub const MinimumDeposit: u64 = 50;
    pub const AbsoluteMinimumContribution: u64 = 10;
    pub const MinimumBlockDuration: u64 = 20;
    pub const MaximumBlockDuration: u64 = 100;
}

impl pallet_crowdloan::Config for Test {
    type PalletId = CrowdloanPalletId;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MinimumDeposit = MinimumDeposit;
    type AbsoluteMinimumContribution = AbsoluteMinimumContribution;
    type MinimumBlockDuration = MinimumBlockDuration;
    type MaximumBlockDuration = MaximumBlockDuration;
}

pub(crate) struct TestState {
    block_number: BlockNumberFor<Test>,
    balances: Vec<(AccountOf<Test>, BalanceOf<Test>)>,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            block_number: 1,
            balances: vec![],
        }
    }
}

impl TestState {
    fn with_block_number(mut self, block_number: BlockNumberFor<Test>) -> Self {
        self.block_number = block_number;
        self
    }

    fn with_balance(mut self, who: AccountOf<Test>, balance: BalanceOf<Test>) -> Self {
        self.balances.push((who, balance));
        self
    }

    fn build_and_execute(self, test: impl FnOnce() -> ()) {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: self
                .balances
                .iter()
                .map(|(who, balance)| (*who, *balance))
                .collect::<Vec<_>>(),
            ..Default::default()
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(self.block_number));
        ext.execute_with(test);
    }
}

pub(crate) fn last_event() -> RuntimeEvent {
    System::events().pop().expect("RuntimeEvent expected").event
}

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

#[test]
fn test_create_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;

            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                deposit,
                minimum_contribution,
                cap,
                end,
            ));

            let crowdloan_id = 0;
            // ensure the crowdloan is stored correctly
            assert_eq!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id),
                Some(CrowdloanInfo {
                    depositor,
                    deposit,
                    minimum_contribution,
                    cap,
                    end,
                    raised: deposit,
                })
            );
            // ensure the crowdloan account has the deposit
            assert_eq!(
                Balances::free_balance(&pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(
                    crowdloan_id
                )),
                deposit
            );
            // ensure the contributions  has been updated
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, depositor),
                Some(deposit)
            );
            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Created {
                    crowdloan_id,
                    depositor,
                    end,
                    cap,
                }
                .into()
            );
        });
}

#[test]
fn test_create_fails_if_bad_origin() {
    TestState::default().build_and_execute(|| {
        let deposit: BalanceOf<Test> = 50;
        let minimum_contribution: BalanceOf<Test> = 10;
        let cap: BalanceOf<Test> = 300;
        let end: BlockNumberFor<Test> = 50;

        assert_err!(
            Crowdloan::create(
                RuntimeOrigin::none(),
                deposit,
                minimum_contribution,
                cap,
                end
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_create_fails_if_deposit_is_too_low() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 20;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::DepositTooLow
            );
        });
}

#[test]
fn test_create_fails_if_cap_is_not_greater_than_deposit() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 40;
            let end: BlockNumberFor<Test> = 50;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::CapTooLow
            );
        });
}

#[test]
fn test_create_fails_if_minimum_contribution_is_too_low() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 5;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::MinimumContributionTooLow
            );
        });
}

#[test]
fn test_create_fails_if_end_is_in_the_past() {
    let current_block_number: BlockNumberFor<Test> = 10;

    TestState::default()
        .with_block_number(current_block_number)
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = current_block_number - 5;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::CannotEndInPast
            );
        });
}

#[test]
fn test_create_fails_if_block_duration_is_too_short() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 11;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::BlockDurationTooShort
            );
        });
}

#[test]
fn test_create_fails_if_block_duration_is_too_long() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 1000;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::BlockDurationTooLong
            );
        });
}

#[test]
fn test_create_fails_if_depositor_has_insufficient_balance() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let depositor: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 200;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(depositor),
                    deposit,
                    minimum_contribution,
                    cap,
                    end
                ),
                pallet_crowdloan::Error::<Test>::InsufficientBalance
            );
        });
}

#[test]
fn test_contribute_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 200)
        .with_balance(U256::from(2), 500)
        .with_balance(U256::from(3), 200)
        .build_and_execute(|| {
            // create a crowdloan
            let depositor: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                initial_deposit,
                minimum_contribution,
                cap,
                end
            ));

            // run some blocks
            run_to_block(10);

            // first contribution to the crowdloan from depositor
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(depositor),
                crowdloan_id,
                amount
            ));
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Contributed {
                    crowdloan_id,
                    contributor: depositor,
                    amount,
                }
                .into()
            );

            // second contribution to the crowdloan
            let contributor1: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor1),
                crowdloan_id,
                amount
            ));
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Contributed {
                    crowdloan_id,
                    contributor: contributor1,
                    amount,
                }
                .into()
            );

            // third contribution to the crowdloan
            let contributor2: AccountOf<Test> = U256::from(3);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor2),
                crowdloan_id,
                amount
            ));
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Contributed {
                    crowdloan_id,
                    contributor: contributor2,
                    amount,
                }
                .into()
            );

            // ensure the contributions are updated correctly
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, depositor),
                Some(100)
            );
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor1),
                Some(100)
            );
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor2),
                Some(50)
            );

            // ensure the crowdloan raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 250)
            );
        });
}

#[test]
fn test_contribute_fails_if_crowdloan_does_not_exist() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let contributor: AccountOf<Test> = U256::from(1);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 20;

            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
            );
        });
}

#[test]
fn test_contribute_fails_if_crowdloan_has_ended() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let depositor: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                initial_deposit,
                minimum_contribution,
                cap,
                end
            ));

            // run past the end of the crowdloan
            run_to_block(60);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 20;
            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::ContributionPeriodEnded
            );
        });
}

#[test]
fn test_contribute_fails_if_cap_has_been_raised() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 1000)
        .with_balance(U256::from(3), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let depositor: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                initial_deposit,
                minimum_contribution,
                cap,
                end
            ));

            // run some blocks
            run_to_block(10);

            // first contribution to the crowdloan fully raise the cap
            let crowdloan_id: CrowdloanId = 0;
            let contributor1: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = cap - initial_deposit;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor1),
                crowdloan_id,
                amount
            ));

            // second contribution to the crowdloan
            let contributor2: AccountOf<Test> = U256::from(3);
            let amount: BalanceOf<Test> = 10;
            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor2), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::CapRaised
            );
        });
}

#[test]
fn test_contribute_fails_if_contribution_is_below_minimum_contribution() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let depositor: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                initial_deposit,
                minimum_contribution,
                cap,
                end
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 5;
            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::ContributionTooLow
            )
        });
}

#[test]
fn test_contribute_fails_if_contribution_will_make_the_raised_amount_exceed_the_cap() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 1000)
        .build_and_execute(|| {
            // create a crowdloan
            let depositor: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let minimum_contribution: BalanceOf<Test> = 10;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(depositor),
                initial_deposit,
                minimum_contribution,
                cap,
                end
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 300;
            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::CapExceeded
            );
        });
}
