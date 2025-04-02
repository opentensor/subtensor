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
    pub const MinimumContribution: u64 = 10;
    pub const MinimumBlockDuration: u64 = 20;
    pub const MaximumBlockDuration: u64 = 100;
    pub const RefundContributorsLimit: u32 = 2;
}

impl pallet_crowdloan::Config for Test {
    type PalletId = CrowdloanPalletId;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MinimumDeposit = MinimumDeposit;
    type MinimumContribution = MinimumContribution;
    type MinimumBlockDuration = MinimumBlockDuration;
    type MaximumBlockDuration = MaximumBlockDuration;
    type RefundContributorsLimit = RefundContributorsLimit;
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

fn noop_call() -> Box<RuntimeCall> {
    Box::new(RuntimeCall::System(frame_system::Call::<Test>::remark {
        remark: vec![],
    }))
}

#[test]
fn test_create_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call(),
            ));

            let crowdloan_id = 0;
            // ensure the crowdloan is stored correctly
            assert_eq!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id),
                Some(CrowdloanInfo {
                    creator,
                    deposit,
                    cap,
                    end,
                    raised: deposit,
                    target_address,
                    call: noop_call(),
                    finalized: false,
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
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, creator),
                Some(deposit)
            );
            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Created {
                    crowdloan_id,
                    creator,
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
        let cap: BalanceOf<Test> = 300;
        let end: BlockNumberFor<Test> = 50;
        let target_address: AccountOf<Test> = U256::from(42);

        assert_err!(
            Crowdloan::create(
                RuntimeOrigin::none(),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ),
            DispatchError::BadOrigin
        );

        assert_err!(
            Crowdloan::create(
                RuntimeOrigin::root(),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 20;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 40;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
                ),
                pallet_crowdloan::Error::<Test>::CapTooLow
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
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = current_block_number - 5;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 11;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 1000;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
                ),
                pallet_crowdloan::Error::<Test>::BlockDurationTooLong
            );
        });
}

#[test]
fn test_create_fails_if_creator_has_insufficient_balance() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 200;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_err!(
                Crowdloan::create(
                    RuntimeOrigin::signed(creator),
                    deposit,
                    cap,
                    end,
                    target_address,
                    noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // first contribution to the crowdloan from creator
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(creator),
                crowdloan_id,
                amount
            ));
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Contributed {
                    crowdloan_id,
                    contributor: creator,
                    amount,
                }
                .into()
            );
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, creator),
                Some(100)
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
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor1),
                Some(100)
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
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor2),
                Some(50)
            );

            // ensure the contributions are present in the crowdloan account
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                250
            );

            // ensure the crowdloan raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 250)
            );
        });
}

#[test]
fn test_contribute_fails_if_bad_origin() {
    TestState::default().build_and_execute(|| {
        let crowdloan_id: CrowdloanId = 0;
        let amount: BalanceOf<Test> = 100;

        assert_err!(
            Crowdloan::contribute(RuntimeOrigin::none(), crowdloan_id, amount),
            DispatchError::BadOrigin
        );

        assert_err!(
            Crowdloan::contribute(RuntimeOrigin::root(), crowdloan_id, amount),
            DispatchError::BadOrigin
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
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);

            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
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
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
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

#[test]
fn test_contribute_fails_if_contributor_has_insufficient_balance() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 50)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 100;
            assert_err!(
                Crowdloan::contribute(RuntimeOrigin::signed(contributor), crowdloan_id, amount),
                pallet_crowdloan::Error::<Test>::InsufficientBalance
            );
        });
}

#[test]
fn test_withdraw_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .with_balance(U256::from(3), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // withdraw from creator
            assert_ok!(Crowdloan::withdraw(
                RuntimeOrigin::signed(creator),
                creator,
                crowdloan_id
            ));
            // ensure the creator has the correct amount
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(&creator), 100);

            // withdraw from contributor
            assert_ok!(Crowdloan::withdraw(
                RuntimeOrigin::signed(contributor),
                contributor,
                crowdloan_id
            ));
            // ensure the contributor has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor),
                100
            );

            // ensure the crowdloan account has the correct amount
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                0
            );
            // ensure the crowdloan raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 0)
            );
        });
}

#[test]
fn test_withdraw_fails_if_bad_origin() {
    TestState::default().build_and_execute(|| {
        let crowdloan_id: CrowdloanId = 0;

        assert_err!(
            Crowdloan::withdraw(RuntimeOrigin::none(), U256::from(1), crowdloan_id),
            DispatchError::BadOrigin
        );

        assert_err!(
            Crowdloan::withdraw(RuntimeOrigin::root(), U256::from(1), crowdloan_id),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_withdraw_succeeds_for_another_contributor() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // withdraw for creator as a contributor
            assert_ok!(Crowdloan::withdraw(
                RuntimeOrigin::signed(contributor),
                creator,
                crowdloan_id
            ));

            // ensure the creator has the correct amount
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(&creator), 100);

            // ensure the contributor has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor),
                0
            );

            // ensure the crowdloan account has the correct amount
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                100
            );

            // ensure the crowdloan raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 100)
            );
        });
}

#[test]
fn test_withdraw_fails_if_crowdloan_does_not_exists() {
    TestState::default().build_and_execute(|| {
        let contributor: AccountOf<Test> = U256::from(1);
        let crowdloan_id: CrowdloanId = 0;

        assert_err!(
            Crowdloan::withdraw(
                RuntimeOrigin::signed(contributor),
                contributor,
                crowdloan_id
            ),
            pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
        );
    });
}

#[test]
fn test_withdraw_fails_if_contribution_period_has_not_ended() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks
            run_to_block(20);

            // try to withdraw
            assert_err!(
                Crowdloan::withdraw(
                    RuntimeOrigin::signed(contributor),
                    contributor,
                    crowdloan_id
                ),
                pallet_crowdloan::Error::<Test>::ContributionPeriodNotEnded
            );
        });
}

#[test]
fn test_withdraw_fails_if_cap_was_fully_raised() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 200)
        .with_balance(U256::from(3), 200)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 150;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks
            run_to_block(20);

            // another contribution to the crowdloan
            let contributor2: AccountOf<Test> = U256::from(3);
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor2),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // try to withdraw
            assert_err!(
                Crowdloan::withdraw(
                    RuntimeOrigin::signed(contributor),
                    contributor,
                    crowdloan_id
                ),
                pallet_crowdloan::Error::<Test>::CapRaised
            );
        });
}

#[test]
fn test_withdraw_fails_if_contribution_is_not_found() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 200)
        .with_balance(U256::from(3), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // contribute to the crowdloan
            let contributor: AccountOf<Test> = U256::from(2);
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // try to withdraw
            let contributor2: AccountOf<Test> = U256::from(3);
            assert_err!(
                Crowdloan::withdraw(
                    RuntimeOrigin::signed(contributor2),
                    contributor2,
                    crowdloan_id
                ),
                pallet_crowdloan::Error::<Test>::NoContribution
            );
        });
}

#[test]
fn test_refund_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .with_balance(U256::from(3), 100)
        .with_balance(U256::from(4), 100)
        .with_balance(U256::from(5), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // first contribution to the crowdloan
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // second contribution to the crowdloan
            let contributor2: AccountOf<Test> = U256::from(3);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor2),
                crowdloan_id,
                amount
            ));

            // third contribution to the crowdloan
            let contributor3: AccountOf<Test> = U256::from(4);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor3),
                crowdloan_id,
                amount
            ));

            // fourth contribution to the crowdloan
            let contributor4: AccountOf<Test> = U256::from(5);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor4),
                crowdloan_id,
                amount,
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            //  first round of refund
            assert_ok!(Crowdloan::refund(
                RuntimeOrigin::signed(creator),
                crowdloan_id
            ));

            // ensure the crowdloan account has the correct amount
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                150 // 2 contributors have been refunded so far
            );
            // ensure raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 150)
            );
            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::PartiallyRefunded { crowdloan_id }.into()
            );

            // run some more blocks
            run_to_block(70);

            //  second round of refund
            assert_ok!(Crowdloan::refund(
                RuntimeOrigin::signed(creator),
                crowdloan_id
            ));

            // ensure the crowdloan account has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                50
            );
            // ensure raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 50)
            );
            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::PartiallyRefunded { crowdloan_id }.into()
            );

            // run some more blocks
            run_to_block(80);

            //  third round of refund
            assert_ok!(Crowdloan::refund(
                RuntimeOrigin::signed(creator),
                crowdloan_id
            ));

            // ensure the crowdloan account has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&crowdloan_account_id),
                0
            );
            // ensure the raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 0)
            );
            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Refunded { crowdloan_id }.into()
            );

            // ensure creator has the correct amount
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(&creator), 100);

            // ensure each contributor has been refunded
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor),
                100
            );
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor2),
                100
            );
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor3),
                100
            );
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&contributor4),
                100
            );
        })
}

#[test]
fn test_refund_fails_if_bad_origin() {
    TestState::default().build_and_execute(|| {
        let crowdloan_id: CrowdloanId = 0;

        assert_err!(
            Crowdloan::refund(RuntimeOrigin::none(), crowdloan_id),
            DispatchError::BadOrigin
        );

        assert_err!(
            Crowdloan::refund(RuntimeOrigin::root(), crowdloan_id),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_refund_fails_if_crowdloan_does_not_exist() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let creator: AccountOf<Test> = U256::from(1);
            let crowdloan_id: CrowdloanId = 0;

            assert_err!(
                Crowdloan::refund(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
            );
        });
}

#[test]
fn test_refund_fails_if_crowdloan_has_not_ended() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // try to refund
            let crowdloan_id: CrowdloanId = 0;
            assert_err!(
                Crowdloan::refund(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::ContributionPeriodNotEnded
            );
        });
}

#[test]
fn test_refund_fails_if_crowdloan_has_fully_raised() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 200)
        .with_balance(U256::from(3), 200)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 300;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                initial_deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // first contribution to the crowdloan
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 150;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks
            run_to_block(20);

            // second contribution to the crowdloan
            let contributor2: AccountOf<Test> = U256::from(3);
            let amount: BalanceOf<Test> = 100;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor2),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // try to refund
            assert_err!(
                Crowdloan::refund(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::CapRaised
            );
        });
}

#[test]
fn test_finalize_succeeds() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 100;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // some contribution
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // finalize the crowdloan
            assert_ok!(Crowdloan::finalize(
                RuntimeOrigin::signed(creator),
                crowdloan_id
            ));

            // ensure the crowdloan account has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(&target_address),
                100
            );

            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Finalized { crowdloan_id }.into()
            );
        })
}

#[test]
fn test_finalize_fails_if_bad_origin() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let crowdloan_id: CrowdloanId = 0;

            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::none(), crowdloan_id),
                DispatchError::BadOrigin
            );

            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::root(), crowdloan_id),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn test_finalize_fails_if_crowdloan_does_not_exist() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .build_and_execute(|| {
            let creator: AccountOf<Test> = U256::from(1);
            let crowdloan_id: CrowdloanId = 0;

            // try to finalize
            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::InvalidCrowdloanId
            );
        });
}

#[test]
fn test_finalize_fails_if_crowdloan_has_not_ended() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 100;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // some contribution
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks before end of contribution period
            run_to_block(10);

            // try to finalize
            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::ContributionPeriodNotEnded
            );
        });
}

#[test]
fn test_finalize_fails_if_crowdloan_cap_is_not_raised() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 100;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // some contribution
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 49; // below cap
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // try finalize the crowdloan
            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::CapNotRaised
            );
        });
}

#[test]
fn test_finalize_fails_if_crowdloan_has_already_been_finalized() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 100;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // some contribution
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // finalize the crowdloan
            assert_ok!(Crowdloan::finalize(
                RuntimeOrigin::signed(creator),
                crowdloan_id
            ));

            // try finalize the crowdloan a second time
            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_crowdloan::Error::<Test>::AlreadyFinalized
            );
        });
}

#[test]
fn test_finalize_fails_if_not_creator_origin() {
    TestState::default()
        .with_balance(U256::from(1), 100)
        .with_balance(U256::from(2), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 100;
            let end: BlockNumberFor<Test> = 50;
            let target_address: AccountOf<Test> = U256::from(42);
            assert_ok!(Crowdloan::create(
                RuntimeOrigin::signed(creator),
                deposit,
                cap,
                end,
                target_address,
                noop_call()
            ));

            // run some blocks
            run_to_block(10);

            // some contribution
            let crowdloan_id: CrowdloanId = 0;
            let contributor: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 50;
            assert_ok!(Crowdloan::contribute(
                RuntimeOrigin::signed(contributor),
                crowdloan_id,
                amount
            ));

            // run some more blocks past the end of the contribution period
            run_to_block(60);

            // try finalize the crowdloan
            assert_err!(
                Crowdloan::finalize(RuntimeOrigin::signed(contributor), crowdloan_id),
                pallet_crowdloan::Error::<Test>::ExpectedCreatorOrigin
            );
        });
}
