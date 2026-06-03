mod common;
use common::new_test_ext;
use common::*;
use frame_support::assert_ok;
use frame_support::dispatch::GetDispatchInfo;
use frame_support::sp_runtime::traits::DispatchTransaction;
use node_subtensor_runtime::{
    Runtime, RuntimeCall, RuntimeOrigin,
    transaction_payment_wrapper::ChargeTransactionPaymentWrapper,
};
use pallet_subtensor::Pallet as SubtensorPallet;
use subtensor_runtime_common::{AccountId, NetUid};

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package node-subtensor-runtime --test subtensor_weights -- set_weights_fees_payed_by_coldkey --exact --nocapture
#[test]
fn set_weights_fees_payed_by_coldkey() {
    new_test_ext().execute_with(|| {
        let hotkey = AccountId::from(common::FOUR_NO_BALANCE);
        let coldkey = AccountId::from(common::TWO);
        let netuid0 = NetUid::from(1);
        let netuid1 = NetUid::from(2);

        SubtensorPallet::<Runtime>::set_weights_set_rate_limit(netuid0, 0);

        add_network_disable_commit_reveal(netuid0, 1, 0);
        add_network_disable_commit_reveal(netuid1, 1, 0);
        register_ok_neuron(netuid0, hotkey.clone(), coldkey.clone(), 2143124);
        register_ok_neuron(netuid1, hotkey.clone(), coldkey.clone(), 3124124);

        let hotkey_balance_before = pallet_balances::Pallet::<Runtime>::free_balance(&hotkey);
        let coldkey_balance_before = pallet_balances::Pallet::<Runtime>::free_balance(&coldkey);

        let weights_keys: Vec<u16> = vec![0];
        let weight_values: Vec<u16> = vec![1];

        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_weights {
            netuid: netuid0,
            dests: weights_keys,
            weights: weight_values,
            version_key: 0,
        });

        let info = call.get_dispatch_info();
        let ext = ChargeTransactionPaymentWrapper::<Runtime>::new(0.into());
        assert_ok!(ext.dispatch_transaction(
            RuntimeOrigin::signed(hotkey.clone()).into(),
            call,
            &info,
            0,
            0,
        ));

        let hotkey_balance_after = pallet_balances::Pallet::<Runtime>::free_balance(&hotkey);
        let coldkey_balance_after = pallet_balances::Pallet::<Runtime>::free_balance(&coldkey);

        assert_eq!(hotkey_balance_before, hotkey_balance_after);
        assert!(coldkey_balance_after < coldkey_balance_before); // Fee paid by coldkey
    });
}
