use crate::*;
use frame_support::{assert_err, assert_ok, traits::Currency};
use frame_system::RawOrigin;
use sp_core::U256;
use sp_runtime::Percent;

use super::mock::*;

// #[test]
// fn test_create_subnet_lending_pool_successfully() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_balance = 5_000_000_000; // 5 TAO
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let cap = 100_000_000_000; // 100 TAO
//         let emissions_share = Percent::from_percent(10);
//         let end: BlockNumber = 100;

//         SubtensorModule::add_balance_to_coldkey_account(&creator_coldkey, initial_balance);

//         assert_ok!(SubtensorModule::create_subnet_crowdloan(
//             RuntimeOrigin::signed(creator_coldkey),
//             initial_deposit,
//             cap,
//             emissions_share,
//             end
//         ));

//         // // Check that the pool was created successfully.
//         // assert_eq!(
//         //     LendingPools::<Test>::get(0),
//         //     Some(LendingPool {
//         //         creator: creator_coldkey,
//         //         initial_deposit,
//         //         max_lending_cap,
//         //         emissions_share,
//         //     })
//         // );
//         // // Check that the creator coldkey was debited the initial deposit.
//         // assert_eq!(
//         //     SubtensorModule::get_coldkey_balance(&creator_coldkey),
//         //     initial_balance - initial_deposit
//         // );
//         // // Check that the pool coldkey was credited the initial deposit.
//         // let pool_id = 0; // the first pool to be created has id 0
//         // let pool_coldkey = SubtensorModule::get_lending_pool_coldkey(pool_id);
//         // assert_eq!(
//         //     SubtensorModule::get_coldkey_balance(&pool_coldkey),
//         //     initial_deposit
//         // );
//         // // Check that the initial deposit was added to the individual contributions.
//         // assert_eq!(
//         //     LendingPoolIndividualContributions::<Test>::get(pool_id, creator_coldkey),
//         //     initial_deposit
//         // );
//         // // Check that the total contributions to the pool are equal to the initial deposit.
//         // assert_eq!(
//         //     LendingPoolTotalContributions::<Test>::get(pool_id),
//         //     initial_deposit
//         // );
//     });
// }

#[test]
fn test_create_subnet_crowdloan_fails_if_bad_origin() {
    new_test_ext(1).execute_with(|| {
        let initial_deposit = 2_000_000_000; // 2 TAO
        let cap = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(10);
        let end = frame_system::Pallet::<Test>::block_number() + 50; // 50 blocks from now

        assert_err!(
            SubtensorModule::create_subnet_crowdloan(
                RawOrigin::None.into(),
                initial_deposit,
                cap,
                emissions_share,
                end
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_create_subnet_crowdloan_fails_if_end_is_in_the_past() {
    new_test_ext(10).execute_with(|| {
        let creator_coldkey = U256::from(1);
        let initial_deposit = 2_000_000_000; // 2 TAO
        let cap = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(10);
        let end = frame_system::Pallet::<Test>::block_number() - 1; // 1 block in the past

        assert_err!(
            SubtensorModule::create_subnet_crowdloan(
                RuntimeOrigin::signed(creator_coldkey),
                initial_deposit,
                cap,
                emissions_share,
                end
            ),
            Error::<Test>::CrowdloanCannotEndInPast
        )
    });
}

#[test]
fn test_create_subnet_crowdloan_fails_if_duration_is_too_short() {
    new_test_ext(10).execute_with(|| {
        let creator_coldkey = U256::from(1);
        let initial_deposit = 2_000_000_000; // 2 TAO
        let cap = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(10);
        let end = frame_system::Pallet::<Test>::block_number() + 5; // 5 blocks from now

        assert_err!(
            SubtensorModule::create_subnet_crowdloan(
                RuntimeOrigin::signed(creator_coldkey),
                initial_deposit,
                cap,
                emissions_share,
                end
            ),
            Error::<Test>::CrowdloanBlocksDurationTooShort
        );
    });
}

#[test]
fn test_create_subnet_crowdloan_fails_if_initial_deposit_is_too_low() {
    new_test_ext(10).execute_with(|| {
        let creator_coldkey = U256::from(1);
        let initial_deposit = 1_000_000_000; // 1 TAO
        let cap = 100_000_000_000; // 100 TAO
        let emissions_share = Percent::from_percent(10);
        let end = frame_system::Pallet::<Test>::block_number() + 50; // 50 blocks from now

        assert_err!(
            SubtensorModule::create_subnet_crowdloan(
                RuntimeOrigin::signed(creator_coldkey),
                initial_deposit,
                cap,
                emissions_share,
                end
            ),
            Error::<Test>::CrowdloanInitialDepositTooLow
        );
    })
}

#[test]
fn test_create_subnet_crowdloan_fails_if_cap_is_inferior_to_initial_deposit() {
    new_test_ext(10).execute_with(|| {
        let creator_coldkey = U256::from(1);
        let initial_deposit = 5_000_000_000; // 5 TAO
        let cap = 4_000_000_000; // 4 TAO
        let emissions_share = Percent::from_percent(10);
        let end = frame_system::Pallet::<Test>::block_number() + 50; // 50 blocks from now

        assert_err!(
            SubtensorModule::create_subnet_crowdloan(
                RuntimeOrigin::signed(creator_coldkey),
                initial_deposit,
                cap,
                emissions_share,
                end
            ),
            Error::<Test>::CrowdloanCapInferiorToInitialDeposit
        );
    })
}

// #[test]
// fn test_create_subnet_lending_pool_fails_if_pool_limit_reached() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let max_lending_cap = 100_000_000_000; // 100 TAO
//         let emissions_share = 10; // 10%

//         // Simulate the fact that we have reached the maximum number of lending pools.
//         NextLendingPoolId::<Test>::set(5);

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolsLimitReached
//         );
//     });
// }

// #[test]
// fn test_create_subnet_lending_pool_fails_if_initial_deposit_too_low() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 500_000_000; // 0.5 TAO
//         let max_lending_cap = 100_000_000_000; // 100 TAO
//         let emissions_share = 10; // 10%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolInitialDepositTooLow
//         );
//     });
// }

// #[test]
// fn test_create_subnet_lending_pool_fails_if_lending_cap_inferior_to_initial_deposit() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 5_000_000_000; // 5 TAO
//         let max_lending_cap = 4_000_000_000; // 4 TAO
//         let emissions_share = 10; // 10%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolLendingCapInferiorToInitialDeposit
//         );
//     });
// }

// #[test]
// fn test_create_subnet_lending_pool_fails_if_lending_cap_too_high() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let max_lending_cap = 2_000_000_000_000; // 2000 TAO
//         let emissions_share = 10; // 10%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolLendingCapTooHigh
//         );
//     });
// }

// #[test]
// fn test_create_subnet_lending_pool_fails_if_emissions_share_too_low() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let max_lending_cap = 100_000_000_000; // 100 TAO
//         let emissions_share = 4; // 4%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolEmissionsShareTooLow
//         );
//     });
// }

// #[test]
// fn test_create_subnet_lending_pool_fails_if_emissions_share_too_high() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let max_lending_cap = 100_000_000_000; // 100 TAO
//         let emissions_share = 101; // 101%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolEmissionsShareTooHigh
//         );
//     });
// }

// #[test]
// fn create_subnet_lending_pool_fails_if_creator_coldkey_does_not_contains_initial_deposit() {
//     new_test_ext(1).execute_with(|| {
//         let creator_coldkey = U256::from(1);
//         let initial_deposit = 2_000_000_000; // 2 TAO
//         let max_lending_cap = 100_000_000_000; // 100 TAO
//         let emissions_share = 10; // 10%

//         assert_err!(
//             SubtensorModule::create_subnet_lending_pool(
//                 RuntimeOrigin::signed(creator_coldkey),
//                 initial_deposit,
//                 max_lending_cap,
//                 emissions_share
//             ),
//             Error::<Test>::LendingPoolNotEnoughBalanceToPayInitialDeposit
//         );
//     });
// }
