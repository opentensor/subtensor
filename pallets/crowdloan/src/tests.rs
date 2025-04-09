#![cfg(test)]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use frame_support::{assert_err, assert_ok, traits::StorePreimage};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::U256;
use sp_runtime::DispatchError;

use crate::{BalanceOf, CrowdloanId, CrowdloanInfo, mock::*, pallet as pallet_crowdloan};

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
            let call = pallet_preimage::Pallet::<Test>::bound(*noop_call()).unwrap();
            assert_eq!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id),
                Some(CrowdloanInfo {
                    creator,
                    deposit,
                    cap,
                    end,
                    raised: deposit,
                    target_address,
                    call,
                    finalized: false,
                })
            );
            // ensure the crowdloan account has the deposit
            assert_eq!(
                Balances::free_balance(pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(
                    crowdloan_id
                )),
                deposit
            );
            // ensure the creator has been deducted the deposit
            assert_eq!(Balances::free_balance(creator), 100 - deposit);
            // ensure the contributions has been updated
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, creator),
                Some(deposit)
            );
            // ensure the raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == deposit)
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
            // ensure next crowdloan id is incremented
            assert_eq!(
                pallet_crowdloan::NextCrowdloanId::<Test>::get(),
                crowdloan_id + 1
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
            assert_eq!(
                Balances::free_balance(creator),
                200 - amount - initial_deposit
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
            assert_eq!(Balances::free_balance(contributor1), 500 - amount);

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
            assert_eq!(Balances::free_balance(contributor2), 200 - amount);

            // ensure the contributions are present in the crowdloan account
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
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
fn test_contribute_succeeds_if_contribution_will_make_the_raised_amount_exceed_the_cap() {
    TestState::default()
        .with_balance(U256::from(1), 200)
        .with_balance(U256::from(2), 500)
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
            assert_eq!(
                Balances::free_balance(creator),
                200 - amount - initial_deposit
            );

            // second contribution to the crowdloan above the cap
            let contributor1: AccountOf<Test> = U256::from(2);
            let amount: BalanceOf<Test> = 300;
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
                    amount: 200, // the amount is capped at the cap
                }
                .into()
            );
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor1),
                Some(200)
            );
            assert_eq!(Balances::free_balance(contributor1), 500 - 200);

            // ensure the contributions are present in the crowdloan account up to the cap
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
                300
            );

            // ensure the crowdloan raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 300)
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

            // ensure the creator contribution has been removed
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, creator),
                None
            );

            // ensure the creator has the correct amount
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(creator), 100);

            // withdraw from contributor
            assert_ok!(Crowdloan::withdraw(
                RuntimeOrigin::signed(contributor),
                contributor,
                crowdloan_id
            ));

            // ensure the creator contribution has been removed
            assert_eq!(
                pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor),
                None
            );

            // ensure the contributor has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(contributor),
                100
            );

            // ensure the crowdloan account has the correct amount
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
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
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(creator), 100);

            // ensure the contributor has the correct amount
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(contributor),
                0
            );

            // ensure the crowdloan account has the correct amount
            let crowdloan_account_id: AccountOf<Test> =
                pallet_crowdloan::Pallet::<Test>::crowdloan_account_id(crowdloan_id);
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
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
        .with_balance(U256::from(6), 100)
        .with_balance(U256::from(7), 100)
        .build_and_execute(|| {
            // create a crowdloan
            let creator: AccountOf<Test> = U256::from(1);
            let initial_deposit: BalanceOf<Test> = 50;
            let cap: BalanceOf<Test> = 400;
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

            // make 6 contributions to reach 350 raised amount (initial deposit + contributions)
            let crowdloan_id: CrowdloanId = 0;
            let amount: BalanceOf<Test> = 50;
            for i in 2..8 {
                let contributor: AccountOf<Test> = U256::from(i);
                assert_ok!(Crowdloan::contribute(
                    RuntimeOrigin::signed(contributor),
                    crowdloan_id,
                    amount
                ));
            }

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
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
                350 - 5 * amount // 5 contributors have been refunded so far
            );
            // ensure raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 350 - 5 * amount)
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
                pallet_balances::Pallet::<Test>::free_balance(crowdloan_account_id),
                0
            );
            // ensure the raised amount is updated correctly
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.raised == 0)
            );

            // ensure creator has the correct amount
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(creator), 100);

            // ensure each contributor has been refunded and  removed from the crowdloan
            for i in 2..8 {
                let contributor: AccountOf<Test> = U256::from(i);
                assert_eq!(
                    pallet_balances::Pallet::<Test>::free_balance(contributor),
                    100
                );
                assert_eq!(
                    pallet_crowdloan::Contributions::<Test>::get(crowdloan_id, contributor),
                    None
                );
            }

            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::AllRefunded { crowdloan_id }.into()
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
                Box::new(RuntimeCall::TestPallet(
                    pallet_test::Call::<Test>::set_passed_crowdloan_id {}
                ))
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

            // ensure the target address has received the funds
            assert_eq!(
                pallet_balances::Pallet::<Test>::free_balance(target_address),
                100
            );

            // ensure the crowdloan is marked as finalized
            assert!(
                pallet_crowdloan::Crowdloans::<Test>::get(crowdloan_id)
                    .is_some_and(|c| c.finalized)
            );

            // ensure the event is emitted
            assert_eq!(
                last_event(),
                pallet_crowdloan::Event::<Test>::Finalized { crowdloan_id }.into()
            );

            // ensure the current crowdloan id was accessible from the dispatched call
            assert_eq!(
                pallet_test::PassedCrowdloanId::<Test>::get(),
                Some(crowdloan_id)
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

#[test]
fn test_finalize_fails_if_call_fails() {
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
                Box::new(RuntimeCall::TestPallet(
                    pallet_test::Call::<Test>::failing_extrinsic {}
                ))
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
                Crowdloan::finalize(RuntimeOrigin::signed(creator), crowdloan_id),
                pallet_test::Error::<Test>::ShouldFail
            );
        });
}
