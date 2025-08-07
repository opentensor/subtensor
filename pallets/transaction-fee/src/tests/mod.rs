#![allow(clippy::indexing_slicing, clippy::unwrap_used)]
use crate::TransactionSource;
use frame_support::assert_ok;
use frame_support::dispatch::GetDispatchInfo;
use pallet_subtensor_swap::AlphaSqrtPrice;
use sp_runtime::{
    traits::{DispatchTransaction, TransactionExtension, TxBaseImplication},
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::AlphaCurrency;

use mock::*;
mod mock;

// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_fees_tao --exact --show-output
#[test]
fn test_remove_stake_fees_tao() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let unstake_amount = AlphaCurrency::from(2_000_000);
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Simulate stake removal to get how much TAO should we get for unstaked Alpha
        let (expected_unstaked_tao, _swap_fee) =
            mock::swap_alpha_to_tao(sn[0].netuid, unstake_amount);

        // Remove stake
        let balance_before = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: unstake_amount,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            call,
            &info,
            0,
            0,
        ));

        let final_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        let actual_tao_fee = balance_before + expected_unstaked_tao - final_balance;
        let actual_alpha_fee = alpha_before - alpha_after - unstake_amount;

        // Remove stake extrinsic should pay fees in TAO because ck has sufficient TAO balance
        assert!(actual_tao_fee > 0);
        assert_eq!(actual_alpha_fee, AlphaCurrency::from(0));
    });
}

// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_fees_alpha --exact --show-output
#[test]
fn test_remove_stake_fees_alpha() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let unstake_amount = AlphaCurrency::from(2_000_000);
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Simulate stake removal to get how much TAO should we get for unstaked Alpha
        let (expected_unstaked_tao, _swap_fee) =
            mock::swap_alpha_to_tao(sn[0].netuid, unstake_amount);

        // Forse-set signer balance to ED
        let current_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let _ = SubtensorModule::remove_balance_from_coldkey_account(
            &sn[0].ck_neurons[0],
            current_balance - ExistentialDeposit::get(),
        );

        // Remove stake
        let balance_before = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: unstake_amount,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            call,
            &info,
            0,
            0,
        ));

        let final_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        let actual_tao_fee = balance_before + expected_unstaked_tao - final_balance;
        let actual_alpha_fee = alpha_before - alpha_after - unstake_amount;

        // Remove stake extrinsic should pay fees in Alpha
        assert_eq!(actual_tao_fee, 0);
        assert!(actual_alpha_fee > 0.into());
    });
}

// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_completely_fees_alpha --exact --show-output
#[test]
fn test_remove_stake_completely_fees_alpha() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Simulate stake removal to get how much TAO should we get for unstaked Alpha
        let unstake_amount = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );
        let (expected_unstaked_tao, _swap_fee) =
            mock::swap_alpha_to_tao(sn[0].netuid, unstake_amount);

        // Forse-set signer balance to ED
        let current_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let _ = SubtensorModule::remove_balance_from_coldkey_account(
            &sn[0].ck_neurons[0],
            current_balance - ExistentialDeposit::get(),
        );

        // Remove stake
        let balance_before = Balances::free_balance(sn[0].ck_neurons[0]);
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: unstake_amount,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            call,
            &info,
            0,
            0,
        ));

        let final_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        // Effectively, the fee is paid in TAO in this case because user receives less TAO,
        // and all Alpha is gone, and it is not measurable in Alpha
        let actual_fee = balance_before + expected_unstaked_tao - final_balance;
        assert_eq!(alpha_after, 0.into());
        assert!(actual_fee > 0);
    });
}

// Validation should fail if both TAO and Alpha balance are lower than tx fees,
// so that transaction is not included in the block
#[test]
fn test_remove_stake_not_enough_balance_for_fees() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Simulate stake removal to get how much TAO should we get for unstaked Alpha
        let current_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        // Forse-set signer balance to ED
        let current_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let _ = SubtensorModule::remove_balance_from_coldkey_account(
            &sn[0].ck_neurons[0],
            current_balance - ExistentialDeposit::get(),
        );

        // For-set Alpha balance to low
        let new_current_stake = AlphaCurrency::from(1_000);
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
            current_stake - new_current_stake,
        );

        // Remove stake
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: new_current_stake,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        let result = ext.validate(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            &call.clone(),
            &info,
            10,
            (),
            &TxBaseImplication(()),
            TransactionSource::External,
        );

        assert_eq!(
            result.unwrap_err(),
            TransactionValidityError::Invalid(InvalidTransaction::Payment)
        );
    });
}

// No TAO balance, Alpha fees. If Alpha price is high, it is enough to pay fees, but when Alpha price
// is low, the validation fails
//
// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_edge_alpha --exact --show-output
#[test]
fn test_remove_stake_edge_alpha() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Simulate stake removal to get how much TAO should we get for unstaked Alpha
        let current_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        // Forse-set signer balance to ED
        let current_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let _ = SubtensorModule::remove_balance_from_coldkey_account(
            &sn[0].ck_neurons[0],
            current_balance - ExistentialDeposit::get(),
        );

        // For-set Alpha balance to low, but enough to pay tx fees at the current Alpha price
        let new_current_stake = AlphaCurrency::from(1_000_000);
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
            current_stake - new_current_stake,
        );

        // Remove stake
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: new_current_stake,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        let result = ext.validate(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            &call.clone(),
            &info,
            10,
            (),
            &TxBaseImplication(()),
            TransactionSource::External,
        );

        // Ok - Validation passed
        assert_ok!(result);

        // Lower Alpha price to 0.0001 so that there is not enough alpha to cover tx fees
        AlphaSqrtPrice::<Test>::insert(sn[0].netuid, U64F64::from_num(0.01));
        let result_low_alpha_price = ext.validate(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            &call.clone(),
            &info,
            10,
            (),
            &TxBaseImplication(()),
            TransactionSource::External,
        );
        assert_eq!(
            result_low_alpha_price.unwrap_err(),
            TransactionValidityError::Invalid(InvalidTransaction::Payment)
        );
    });
}

// Validation passes, but transaction fails => TAO fees are paid
//
// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_failing_transaction_tao_fees --exact --show-output
#[test]
fn test_remove_stake_failing_transaction_tao_fees() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let unstake_amount = AlphaCurrency::from(2_000_000);
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Make unstaking fail by reducing liquidity to critical
        SubnetTAO::<Test>::insert(sn[0].netuid, 1);

        // Remove stake
        let balance_before = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: unstake_amount,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            call,
            &info,
            0,
            0,
        ));

        let final_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        let actual_tao_fee = balance_before - final_balance;

        // Remove stake extrinsic should pay fees in TAO because ck has sufficient TAO balance
        assert!(actual_tao_fee > 0);
        assert_eq!(alpha_before, alpha_after);
    });
}

// Validation passes, but transaction fails => Alpha fees are paid
//
// cargo test --package subtensor-transaction-fee --lib -- tests::test_remove_stake_failing_transaction_alpha_fees --exact --show-output
#[test]
fn test_remove_stake_failing_transaction_alpha_fees() {
    new_test_ext().execute_with(|| {
        let stake_amount = 1_000_000_000;
        let unstake_amount = AlphaCurrency::from(2_000_000);
        let sn = setup_subnets(1, 1);
        setup_stake(&sn[0], stake_amount);

        // Make unstaking fail by reducing liquidity to critical
        SubnetTAO::<Test>::insert(sn[0].netuid, 1);

        // Forse-set signer balance to ED
        let current_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let _ = SubtensorModule::remove_balance_from_coldkey_account(
            &sn[0].ck_neurons[0],
            current_balance - ExistentialDeposit::get(),
        );

        // Remove stake
        let balance_before = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake {
            hotkey: sn[0].hk_neurons[0],
            netuid: sn[0].netuid,
            amount_unstaked: unstake_amount,
        });

        // Dispatch the extrinsic with ChargeTransactionPayment extension
        let info = call.get_dispatch_info();
        let ext = pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(0);
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(sn[0].ck_neurons[0]).into(),
            call,
            &info,
            0,
            0,
        ));

        let final_balance = Balances::free_balance(sn[0].ck_neurons[0]);
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &sn[0].hk_neurons[0],
            &sn[0].ck_neurons[0],
            sn[0].netuid,
        );

        let actual_tao_fee = balance_before - final_balance;
        let actual_alpha_fee = alpha_before - alpha_after;

        // Remove stake extrinsic should pay fees in Alpha
        assert_eq!(actual_tao_fee, 0);
        assert!(actual_alpha_fee > 0.into());
        assert!(actual_alpha_fee < unstake_amount);
    });
}
