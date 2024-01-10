use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::Error;
use sp_core::{H256, U256};

/***********************************************************
    subnet_staking::add_subnet_stake() tests
************************************************************/
#[test]
fn test_add_subnet_stake_ok_no_emission() 
{
    new_test_ext().execute_with(|| {
        let hotkey_account_id:  U256    = U256::from(533453);
        let coldkey_account_id: U256    = U256::from(55453);
        let netuid:             u16     = 1;
        let tempo:              u16     = 13;
        let start_nonce:        u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, 10000 + 1);

        // Check we have zero staked before transfer
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            10000
        );

        // Check if balance has  decreased
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 10000);
    });
}