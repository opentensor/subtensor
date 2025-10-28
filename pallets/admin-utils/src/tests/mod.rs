use frame_support::sp_runtime::DispatchError;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::{DispatchClass, GetDispatchInfo, Pays},
    traits::Hooks,
};
use frame_system::Config;
use pallet_subtensor::{
    Error as SubtensorError, MaxRegistrationsPerBlock, Rank, SubnetOwner,
    TargetRegistrationsPerInterval, Tempo, WeightsVersionKeyRateLimit, *,
};
// use pallet_subtensor::{migrations, Event};
use pallet_subtensor::{Event, utils::rate_limiting::TransactionType};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Get, Pair, U256, ed25519};
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::{Currency, MechId, NetUid, TaoCurrency};

use crate::Error;
use crate::pallet::PrecompileEnable;
use mock::*;

mod mock;

#[test]
fn test_sudo_set_default_take() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_default_delegate_take();
        assert_eq!(
            AdminUtils::sudo_set_default_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_default_delegate_take(), init_value);
        assert_ok!(AdminUtils::sudo_set_default_take(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_default_delegate_take(), to_be_set);
    });
}

#[test]
fn test_sudo_set_serving_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(3);
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_serving_rate_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_serving_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_serving_rate_limit(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_serving_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_serving_rate_limit(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_min_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_min_difficulty(netuid);
        assert_eq!(
            AdminUtils::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_difficulty(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_min_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_difficulty(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_max_difficulty(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_difficulty(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_difficulty(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_version_key() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_weights_version_key(netuid);
        assert_eq!(
            AdminUtils::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_version_key_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;

        let sn_owner = U256::from(1);
        add_network(netuid, 10);
        // Set the Subnet Owner
        SubnetOwner::<Test>::insert(netuid, sn_owner);

        let rate_limit = WeightsVersionKeyRateLimit::<Test>::get();
        let tempo = Tempo::<Test>::get(netuid);

        let rate_limit_period = rate_limit * (tempo as u64);

        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), to_be_set);

        // Try to set again with
        // Assert rate limit not passed
        assert!(
            !TransactionType::SetWeightsVersionKey
                .passes_rate_limit_on_subnet::<Test>(&sn_owner, netuid)
        );

        // Try transaction
        assert_noop!(
            AdminUtils::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
                netuid,
                to_be_set + 1
            ),
            pallet_subtensor::Error::<Test>::TxRateLimitExceeded
        );

        // Wait for rate limit to pass
        run_to_block(rate_limit_period + 1);
        assert!(
            TransactionType::SetWeightsVersionKey
                .passes_rate_limit_on_subnet::<Test>(&sn_owner, netuid)
        );

        // Try transaction
        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            to_be_set + 1
        ));
        assert_eq!(
            SubtensorModule::get_weights_version_key(netuid),
            to_be_set + 1
        );
    });
}

#[test]
fn test_sudo_set_weights_version_key_rate_limit_root() {
    // root should not be effected by rate limit
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;

        let sn_owner = U256::from(1);
        add_network(netuid, 10);
        // Set the Subnet Owner
        SubnetOwner::<Test>::insert(netuid, sn_owner);

        let rate_limit = WeightsVersionKeyRateLimit::<Test>::get();
        let tempo: u16 = Tempo::<Test>::get(netuid);

        let rate_limit_period = rate_limit * (tempo as u64);
        // Verify the rate limit is more than 0 blocks
        assert!(rate_limit_period > 0);

        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), to_be_set);

        // Try transaction
        assert_ok!(AdminUtils::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            to_be_set + 1
        ));
        assert_eq!(
            SubtensorModule::get_weights_version_key(netuid),
            to_be_set + 1
        );
    });
}

#[test]
fn test_sudo_set_weights_set_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_weights_set_rate_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_weights_set_rate_limit(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_weights_set_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_weights_set_rate_limit(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_adjustment_interval() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_adjustment_interval(netuid);
        assert_eq!(
            AdminUtils::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_interval(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_adjustment_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_adjustment_interval(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_adjustment_alpha() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_adjustment_alpha(netuid);
        assert_eq!(
            AdminUtils::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_alpha(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_adjustment_alpha(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_adjustment_alpha(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_subnet_owner_cut() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_subnet_owner_cut();
        assert_eq!(
            AdminUtils::sudo_set_subnet_owner_cut(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), init_value);
        assert_ok!(AdminUtils::sudo_set_subnet_owner_cut(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), to_be_set);
    });
}

#[test]
fn test_sudo_set_issuance() {
    new_test_ext().execute_with(|| {
        let to_be_set = TaoCurrency::from(10);
        assert_eq!(
            AdminUtils::sudo_set_total_issuance(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_ok!(AdminUtils::sudo_set_total_issuance(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_total_issuance(), to_be_set);
    });
}

#[test]
fn test_sudo_set_immunity_period() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_immunity_period(netuid);
        assert_eq!(
            AdminUtils::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_immunity_period(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_immunity_period(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_immunity_period(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_min_allowed_weights() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_min_allowed_weights(netuid);
        assert_eq!(
            AdminUtils::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_min_allowed_weights(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 12;
        add_network(netuid, 10);
        MaxRegistrationsPerBlock::<Test>::insert(netuid, 256);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 256);

        // Register some neurons
        for i in 0..=8 {
            register_ok_neuron(netuid, U256::from(i * 1000), U256::from(i * 1000 + i), 0);
        }

        // Bad origin that is not root or subnet owner
        assert_noop!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(42)),
                netuid,
                to_be_set
            ),
            DispatchError::BadOrigin
        );

        // Random netuid that doesn't exist
        assert_noop!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                NetUid::from(42),
                to_be_set
            ),
            Error::<Test>::SubnetDoesNotExist
        );

        // Trying to set max allowed uids less than min allowed uids
        assert_noop!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_min_allowed_uids(netuid) - 1
            ),
            Error::<Test>::MaxAllowedUidsLessThanMinAllowedUids
        );

        // Trying to set max allowed uids less than current uids
        assert_noop!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_subnetwork_n(netuid) - 1
            ),
            Error::<Test>::MaxAllowedUIdsLessThanCurrentUIds
        );

        // Trying to set max allowed uids greater than default max allowed uids
        assert_noop!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                DefaultMaxAllowedUids::<Test>::get() + 1
            ),
            Error::<Test>::MaxAllowedUidsGreaterThanDefaultMaxAllowedUids
        );

        // Normal case
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), to_be_set);

        // Exact current case
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            SubtensorModule::get_subnetwork_n(netuid)
        ));
        assert_eq!(
            SubtensorModule::get_max_allowed_uids(netuid),
            SubtensorModule::get_subnetwork_n(netuid)
        );

        // Lower bound case
        SubtensorModule::set_min_allowed_uids(netuid, SubtensorModule::get_subnetwork_n(netuid));
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            SubtensorModule::get_min_allowed_uids(netuid)
        ));
        assert_eq!(
            SubtensorModule::get_max_allowed_uids(netuid),
            SubtensorModule::get_min_allowed_uids(netuid)
        );

        // Upper bound case
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            DefaultMaxAllowedUids::<Test>::get(),
        ));
        assert_eq!(
            SubtensorModule::get_max_allowed_uids(netuid),
            DefaultMaxAllowedUids::<Test>::get()
        );
    });
}

#[test]
fn test_sudo_set_kappa() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_kappa(netuid);
        assert_eq!(
            AdminUtils::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_kappa(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_kappa(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_kappa(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_rho() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_rho(netuid);
        assert_eq!(
            AdminUtils::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_rho(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_rho(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_rho(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_activity_cutoff() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = pallet_subtensor::MinActivityCutoff::<Test>::get();
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_activity_cutoff(netuid);
        assert_eq!(
            AdminUtils::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_activity_cutoff(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_activity_cutoff(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_activity_cutoff(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_target_registrations_per_interval() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_target_registrations_per_interval(netuid);
        assert_eq!(
            AdminUtils::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_target_registrations_per_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_difficulty() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(
            AdminUtils::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), to_be_set);

        // Test that SN owner can't set difficulty
        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, U256::from(1));
        assert_eq!(
            AdminUtils::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                init_value
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), to_be_set); // no change
    });
}

#[test]
fn test_sudo_set_max_allowed_validators() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_validators(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_max_allowed_validators(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_max_allowed_validators(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_max_allowed_validators(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_stake_threshold() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_stake_threshold();
        assert_eq!(
            AdminUtils::sudo_set_stake_threshold(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_stake_threshold(), init_value);
        assert_ok!(AdminUtils::sudo_set_stake_threshold(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_stake_threshold(), to_be_set);
    });
}

#[test]
fn test_sudo_set_bonds_moving_average() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_bonds_moving_average(netuid.into());
        assert_eq!(
            AdminUtils::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_bonds_moving_average(netuid.into()),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_bonds_moving_average(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_bonds_moving_average(netuid.into()),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_bonds_penalty() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_bonds_penalty(netuid);
        assert_eq!(
            AdminUtils::sudo_set_bonds_penalty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_bonds_penalty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_bonds_penalty(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_bonds_penalty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_penalty(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_rao_recycled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set = TaoCurrency::from(10);
        add_network(netuid, 10);
        let init_value = SubtensorModule::get_rao_recycled(netuid);

        // Need to run from genesis block
        run_to_block(1);

        assert_eq!(
            AdminUtils::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), init_value);

        // Verify no events emitted matching the expected event
        assert_eq!(
            System::events()
                .iter()
                .filter(|r| r.event
                    == RuntimeEvent::SubtensorModule(Event::RAORecycledForRegistrationSet(
                        netuid, to_be_set
                    )))
                .count(),
            0
        );

        assert_ok!(AdminUtils::sudo_set_rao_recycled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), to_be_set);

        // Verify event emitted with correct values
        assert_eq!(
            System::events()
                .last()
                .unwrap_or_else(|| panic!(
                    "Expected there to be events: {:?}",
                    System::events().to_vec()
                ))
                .event,
            RuntimeEvent::SubtensorModule(Event::RAORecycledForRegistrationSet(netuid, to_be_set))
        );
    });
}

#[test]
fn test_sudo_set_network_lock_reduction_interval() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 7200;
        add_network(netuid, 10);

        let init_value: u64 = SubtensorModule::get_lock_reduction_interval();
        assert_eq!(
            AdminUtils::sudo_set_lock_reduction_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_lock_reduction_interval(), init_value);
        assert_ok!(AdminUtils::sudo_set_lock_reduction_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_lock_reduction_interval(), to_be_set);
    });
}

#[test]
fn test_sudo_set_network_pow_registration_allowed() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: bool = true;
        add_network(netuid, 10);

        let init_value: bool = SubtensorModule::get_network_pow_registration_allowed(netuid);
        assert_eq!(
            AdminUtils::sudo_set_network_pow_registration_allowed(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            SubtensorModule::get_network_pow_registration_allowed(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_network_pow_registration_allowed(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_network_pow_registration_allowed(netuid),
            to_be_set
        );
    });
}

mod sudo_set_nominator_min_required_stake {
    use super::*;

    #[test]
    fn can_only_be_called_by_admin() {
        new_test_ext().execute_with(|| {
            let to_be_set = SubtensorModule::get_nominator_min_required_stake() + 5;
            assert_eq!(
                AdminUtils::sudo_set_nominator_min_required_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                    to_be_set
                ),
                Err(DispatchError::BadOrigin)
            );
        });
    }

    #[test]
    fn sets_a_lower_value() {
        new_test_ext().execute_with(|| {
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                10
            ));
            let default_min_stake = pallet_subtensor::DefaultMinStake::<Test>::get();
            assert_eq!(
                SubtensorModule::get_nominator_min_required_stake(),
                10 * default_min_stake.to_u64() / 1_000_000
            );

            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                5
            ));
            assert_eq!(
                SubtensorModule::get_nominator_min_required_stake(),
                5 * default_min_stake.to_u64() / 1_000_000
            );
        });
    }

    #[test]
    fn sets_a_higher_value() {
        new_test_ext().execute_with(|| {
            let to_be_set = SubtensorModule::get_nominator_min_required_stake() + 5;
            let default_min_stake = pallet_subtensor::DefaultMinStake::<Test>::get();
            assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
                <<Test as Config>::RuntimeOrigin>::root(),
                to_be_set
            ));
            assert_eq!(
                SubtensorModule::get_nominator_min_required_stake(),
                to_be_set * default_min_stake.to_u64() / 1_000_000
            );
        });
    }
}

#[test]
fn test_sudo_set_tx_delegate_take_rate_limit() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_tx_delegate_take_rate_limit();
        assert_eq!(
            AdminUtils::sudo_set_tx_delegate_take_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            SubtensorModule::get_tx_delegate_take_rate_limit(),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_tx_delegate_take_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_tx_delegate_take_rate_limit(),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_min_delegate_take() {
    new_test_ext().execute_with(|| {
        let to_be_set = u16::MAX / 100;
        let init_value = SubtensorModule::get_min_delegate_take();
        assert_eq!(
            AdminUtils::sudo_set_min_delegate_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_min_delegate_take(), init_value);
        assert_ok!(AdminUtils::sudo_set_min_delegate_take(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_delegate_take(), to_be_set);
    });
}

#[test]
fn test_sudo_set_commit_reveal_weights_enabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 10);

        let to_be_set: bool = false;
        let init_value: bool = SubtensorModule::get_commit_reveal_weights_enabled(netuid);

        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));

        assert!(init_value != to_be_set);
        assert_eq!(
            SubtensorModule::get_commit_reveal_weights_enabled(netuid),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_liquid_alpha_enabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let enabled: bool = true;
        assert_eq!(!enabled, SubtensorModule::get_liquid_alpha_enabled(netuid));

        assert_ok!(AdminUtils::sudo_set_liquid_alpha_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            enabled
        ));

        assert_eq!(enabled, SubtensorModule::get_liquid_alpha_enabled(netuid));
    });
}

#[test]
fn test_sudo_set_alpha_sigmoid_steepness() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: i16 = 5000;
        add_network(netuid, 10);
        let init_value = SubtensorModule::get_alpha_sigmoid_steepness(netuid);
        assert_eq!(
            AdminUtils::sudo_set_alpha_sigmoid_steepness(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_alpha_sigmoid_steepness(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid.next(),
                to_be_set
            ),
            Err(Error::<Test>::SubnetDoesNotExist.into())
        );

        let owner = U256::from(10);
        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, owner);
        assert_eq!(
            AdminUtils::sudo_set_alpha_sigmoid_steepness(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                -to_be_set
            ),
            Err(Error::<Test>::NegativeSigmoidSteepness.into())
        );
        assert_eq!(
            SubtensorModule::get_alpha_sigmoid_steepness(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_alpha_sigmoid_steepness(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_alpha_sigmoid_steepness(netuid),
            to_be_set
        );
        assert_ok!(AdminUtils::sudo_set_alpha_sigmoid_steepness(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            -to_be_set
        ));
        assert_eq!(
            SubtensorModule::get_alpha_sigmoid_steepness(netuid),
            -to_be_set
        );
    });
}

#[test]
fn test_set_alpha_values_dispatch_info_ok() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let alpha_low: u16 = 1638_u16;
        let alpha_high: u16 = u16::MAX - 10;
        let call = RuntimeCall::AdminUtils(crate::Call::sudo_set_alpha_values {
            netuid,
            alpha_low,
            alpha_high,
        });

        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.class, DispatchClass::Normal);
        assert_eq!(dispatch_info.pays_fee, Pays::Yes);
    });
}

#[test]
fn test_sudo_get_set_alpha() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let alpha_low: u16 = 1638_u16;
        let alpha_high: u16 = u16::MAX - 10;

        let hotkey: U256 = U256::from(1);
        let coldkey: U256 = U256::from(1 + 456);
        let signer = <<Test as Config>::RuntimeOrigin>::signed(coldkey);

        // Enable Liquid Alpha and setup
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        pallet_subtensor::migrations::migrate_create_root_network::migrate_create_root_network::<
            Test,
        >();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey,));

        // Should fail as signer does not own the subnet
        assert_err!(
            AdminUtils::sudo_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            DispatchError::BadOrigin
        );

        assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));

        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
        let (grabbed_alpha_low, grabbed_alpha_high): (u16, u16) =
            SubtensorModule::get_alpha_values(netuid);

        log::info!("alpha_low: {grabbed_alpha_low:?} alpha_high: {grabbed_alpha_high:?}");
        assert_eq!(grabbed_alpha_low, alpha_low);
        assert_eq!(grabbed_alpha_high, alpha_high);

        // Convert the u16 values to decimal values
        fn unnormalize_u16_to_float(normalized_value: u16) -> f32 {
            const MAX_U16: u16 = 65535;
            normalized_value as f32 / MAX_U16 as f32
        }

        let alpha_low_decimal = unnormalize_u16_to_float(alpha_low);
        let alpha_high_decimal = unnormalize_u16_to_float(alpha_high);

        let (alpha_low_32, alpha_high_32) = SubtensorModule::get_alpha_values_32(netuid);

        let tolerance: f32 = 1e-6; // 0.000001

        // Check if the values are equal to the sixth decimal
        assert!(
            (alpha_low_32.to_num::<f32>() - alpha_low_decimal).abs() < tolerance,
            "alpha_low mismatch: {} != {}",
            alpha_low_32.to_num::<f32>(),
            alpha_low_decimal
        );
        assert!(
            (alpha_high_32.to_num::<f32>() - alpha_high_decimal).abs() < tolerance,
            "alpha_high mismatch: {} != {}",
            alpha_high_32.to_num::<f32>(),
            alpha_high_decimal
        );

        // 1. Liquid alpha disabled
        SubtensorModule::set_liquid_alpha_enabled(netuid, false);
        assert_err!(
            AdminUtils::sudo_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            SubtensorError::<Test>::LiquidAlphaDisabled
        );
        // Correct scenario after error
        SubtensorModule::set_liquid_alpha_enabled(netuid, true); // Re-enable for further tests
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 2. Alpha high too low
        let alpha_high_too_low = (u16::MAX as u32 / 40) as u16 - 1; // One less than the minimum acceptable value
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low,
                alpha_high_too_low
            ),
            SubtensorError::<Test>::AlphaHighTooLow
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 3. Alpha low too low or too high
        let alpha_low_too_low = 0_u16;
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_low,
                alpha_high
            ),
            SubtensorError::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        let alpha_low_too_high = alpha_high + 1;
        assert_err!(
            AdminUtils::sudo_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_high,
                alpha_high
            ),
            SubtensorError::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(AdminUtils::sudo_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
    });
}

#[test]
fn test_sudo_set_coldkey_swap_schedule_duration() {
    new_test_ext().execute_with(|| {
        // Arrange
        let root = RuntimeOrigin::root();
        let non_root = RuntimeOrigin::signed(U256::from(1));
        let new_duration = 100u32.into();

        // Act & Assert: Non-root account should fail
        assert_noop!(
            AdminUtils::sudo_set_coldkey_swap_schedule_duration(non_root, new_duration),
            DispatchError::BadOrigin
        );

        // Act: Root account should succeed
        assert_ok!(AdminUtils::sudo_set_coldkey_swap_schedule_duration(
            root.clone(),
            new_duration
        ));

        // Assert: Check if the duration was actually set
        assert_eq!(
            pallet_subtensor::ColdkeySwapScheduleDuration::<Test>::get(),
            new_duration
        );

        // Act & Assert: Setting the same value again should succeed (idempotent operation)
        assert_ok!(AdminUtils::sudo_set_coldkey_swap_schedule_duration(
            root,
            new_duration
        ));

        // You might want to check for events here if your pallet emits them
        System::assert_last_event(Event::ColdkeySwapScheduleDurationSet(new_duration).into());
    });
}

#[test]
fn test_sudo_set_dissolve_network_schedule_duration() {
    new_test_ext().execute_with(|| {
        // Arrange
        let root = RuntimeOrigin::root();
        let non_root = RuntimeOrigin::signed(U256::from(1));
        let new_duration = 200u32.into();

        // Act & Assert: Non-root account should fail
        assert_noop!(
            AdminUtils::sudo_set_dissolve_network_schedule_duration(non_root, new_duration),
            DispatchError::BadOrigin
        );

        // Act: Root account should succeed
        assert_ok!(AdminUtils::sudo_set_dissolve_network_schedule_duration(
            root.clone(),
            new_duration
        ));

        // Assert: Check if the duration was actually set
        assert_eq!(
            pallet_subtensor::DissolveNetworkScheduleDuration::<Test>::get(),
            new_duration
        );

        // Act & Assert: Setting the same value again should succeed (idempotent operation)
        assert_ok!(AdminUtils::sudo_set_dissolve_network_schedule_duration(
            root,
            new_duration
        ));

        // You might want to check for events here if your pallet emits them
        System::assert_last_event(Event::DissolveNetworkScheduleDurationSet(new_duration).into());
    });
}

#[test]
fn sudo_set_commit_reveal_weights_interval() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 10);

        let too_high = 101;
        assert_err!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                too_high
            ),
            pallet_subtensor::Error::<Test>::RevealPeriodTooLarge
        );

        let to_be_set = 55;
        let init_value = SubtensorModule::get_reveal_period(netuid);

        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));

        assert!(init_value != to_be_set);
        assert_eq!(SubtensorModule::get_reveal_period(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_root_sets_evm_chain_id() {
    new_test_ext().execute_with(|| {
        let chain_id: u64 = 945;
        assert_eq!(pallet_evm_chain_id::ChainId::<Test>::get(), 0);

        assert_ok!(AdminUtils::sudo_set_evm_chain_id(
            <<Test as Config>::RuntimeOrigin>::root(),
            chain_id
        ));

        assert_eq!(pallet_evm_chain_id::ChainId::<Test>::get(), chain_id);
    });
}

#[test]
fn test_sudo_non_root_cannot_set_evm_chain_id() {
    new_test_ext().execute_with(|| {
        let chain_id: u64 = 945;
        assert_eq!(pallet_evm_chain_id::ChainId::<Test>::get(), 0);

        assert_eq!(
            AdminUtils::sudo_set_evm_chain_id(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                chain_id
            ),
            Err(DispatchError::BadOrigin)
        );

        assert_eq!(pallet_evm_chain_id::ChainId::<Test>::get(), 0);
    });
}

#[test]
fn test_schedule_grandpa_change() {
    new_test_ext().execute_with(|| {
        assert_eq!(Grandpa::grandpa_authorities(), vec![]);

        let bob: GrandpaId = ed25519::Pair::from_legacy_string("//Bob", None)
            .public()
            .into();

        assert_ok!(AdminUtils::schedule_grandpa_change(
            RuntimeOrigin::root(),
            vec![(bob.clone(), 1)],
            41,
            None
        ));

        Grandpa::on_finalize(42);

        assert_eq!(Grandpa::grandpa_authorities(), vec![(bob, 1)]);
    });
}

#[test]
fn test_sudo_toggle_evm_precompile() {
    new_test_ext().execute_with(|| {
        let precompile_id = crate::PrecompileEnum::BalanceTransfer;
        let initial_enabled = PrecompileEnable::<Test>::get(precompile_id);
        assert!(initial_enabled); // Assuming the default is true

        run_to_block(1);

        assert_eq!(
            AdminUtils::sudo_toggle_evm_precompile(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                precompile_id,
                false
            ),
            Err(DispatchError::BadOrigin)
        );

        assert_ok!(AdminUtils::sudo_toggle_evm_precompile(
            RuntimeOrigin::root(),
            precompile_id,
            false
        ));

        assert_eq!(
            System::events()
                .iter()
                .filter(|r| r.event
                    == RuntimeEvent::AdminUtils(crate::Event::PrecompileUpdated {
                        precompile_id,
                        enabled: false
                    }))
                .count(),
            1
        );

        let updated_enabled = PrecompileEnable::<Test>::get(precompile_id);
        assert!(!updated_enabled);

        run_to_block(2);

        assert_ok!(AdminUtils::sudo_toggle_evm_precompile(
            RuntimeOrigin::root(),
            precompile_id,
            false
        ));

        // no event without status change
        assert_eq!(
            System::events()
                .iter()
                .filter(|r| r.event
                    == RuntimeEvent::AdminUtils(crate::Event::PrecompileUpdated {
                        precompile_id,
                        enabled: false
                    }))
                .count(),
            0
        );

        assert_ok!(AdminUtils::sudo_toggle_evm_precompile(
            RuntimeOrigin::root(),
            precompile_id,
            true
        ));

        let final_enabled = PrecompileEnable::<Test>::get(precompile_id);
        assert!(final_enabled);
    });
}

#[test]
fn test_sudo_root_sets_subnet_moving_alpha() {
    new_test_ext().execute_with(|| {
        let alpha: I96F32 = I96F32::saturating_from_num(0.5);
        let initial = pallet_subtensor::SubnetMovingAlpha::<Test>::get();
        assert!(initial != alpha);

        assert_ok!(AdminUtils::sudo_set_subnet_moving_alpha(
            <<Test as Config>::RuntimeOrigin>::root(),
            alpha
        ));

        assert_eq!(pallet_subtensor::SubnetMovingAlpha::<Test>::get(), alpha);
    });
}

#[test]
fn test_sets_a_lower_value_clears_small_nominations() {
    new_test_ext().execute_with(|| {
        let hotkey: U256 = U256::from(3);
        let owner_coldkey: U256 = U256::from(1);
        let staker_coldkey: U256 = U256::from(2);

        let initial_nominator_min_required_stake = 10;
        let nominator_min_required_stake_0 = 5;
        let nominator_min_required_stake_1 = 20;

        assert!(nominator_min_required_stake_0 < nominator_min_required_stake_1);
        assert!(nominator_min_required_stake_0 < initial_nominator_min_required_stake);

        let to_stake = initial_nominator_min_required_stake + 1;

        assert!(to_stake > initial_nominator_min_required_stake);
        assert!(to_stake > nominator_min_required_stake_0); // Should stay when set
        assert!(to_stake < nominator_min_required_stake_1); // Should be removed when set

        // Create network
        let netuid = NetUid::from(2);
        add_network(netuid, 10);

        // Register a neuron
        register_ok_neuron(netuid, hotkey, owner_coldkey, 0);

        let default_min_stake = pallet_subtensor::DefaultMinStake::<Test>::get();
        assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
            RuntimeOrigin::root(),
            initial_nominator_min_required_stake
        ));
        assert_eq!(
            SubtensorModule::get_nominator_min_required_stake(),
            initial_nominator_min_required_stake * default_min_stake.to_u64() / 1_000_000
        );

        // Stake to the hotkey as staker_coldkey
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &staker_coldkey,
            netuid,
            to_stake.into(),
        );

        let default_min_stake = pallet_subtensor::DefaultMinStake::<Test>::get();
        assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
            RuntimeOrigin::root(),
            nominator_min_required_stake_0
        ));
        assert_eq!(
            SubtensorModule::get_nominator_min_required_stake(),
            nominator_min_required_stake_0 * default_min_stake.to_u64() / 1_000_000
        );

        // Check this nomination is not cleared
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &staker_coldkey,
                netuid
            ) > 0.into()
        );

        assert_ok!(AdminUtils::sudo_set_nominator_min_required_stake(
            RuntimeOrigin::root(),
            nominator_min_required_stake_1
        ));
        assert_eq!(
            SubtensorModule::get_nominator_min_required_stake(),
            nominator_min_required_stake_1 * default_min_stake.to_u64() / 1_000_000
        );

        // Check this nomination is cleared
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &staker_coldkey,
                netuid
            ),
            0.into()
        );
    });
}

// #[test]
// fn test_sudo_set_subnet_owner_hotkey() {
//     new_test_ext().execute_with(|| {
//         let netuid = NetUid::from(1);

//         let coldkey: U256 = U256::from(1);
//         let hotkey: U256 = U256::from(2);
//         let new_hotkey: U256 = U256::from(3);

//         let coldkey_origin = <<Test as Config>::RuntimeOrigin>::signed(coldkey);
//         let root = RuntimeOrigin::root();
//         let random_account = RuntimeOrigin::signed(U256::from(123456));

//         pallet_subtensor::SubnetOwner::<Test>::insert(netuid, coldkey);
//         pallet_subtensor::SubnetOwnerHotkey::<Test>::insert(netuid, hotkey);
//         assert_eq!(
//             pallet_subtensor::SubnetOwnerHotkey::<Test>::get(netuid),
//             hotkey
//         );

//         assert_ok!(AdminUtils::sudo_set_subnet_owner_hotkey(
//             coldkey_origin,
//             netuid,
//             new_hotkey
//         ));

//         assert_eq!(
//             pallet_subtensor::SubnetOwnerHotkey::<Test>::get(netuid),
//             new_hotkey
//         );

//         assert_noop!(
//             AdminUtils::sudo_set_subnet_owner_hotkey(random_account, netuid, new_hotkey),
//             DispatchError::BadOrigin
//         );

//         assert_noop!(
//             AdminUtils::sudo_set_subnet_owner_hotkey(root, netuid, new_hotkey),
//             DispatchError::BadOrigin
//         );
//     });
// }

// cargo test --package pallet-admin-utils --lib -- tests::test_sudo_set_ema_halving --exact --show-output
#[test]
fn test_sudo_set_ema_halving() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u64 = 10;
        add_network(netuid, 10);

        let value_before: u64 = pallet_subtensor::EMAPriceHalvingBlocks::<Test>::get(netuid);
        assert_eq!(
            AdminUtils::sudo_set_ema_price_halving_period(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        let value_after_0: u64 = pallet_subtensor::EMAPriceHalvingBlocks::<Test>::get(netuid);
        assert_eq!(value_after_0, value_before);

        let owner = U256::from(10);
        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, owner);
        assert_eq!(
            AdminUtils::sudo_set_ema_price_halving_period(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        let value_after_1: u64 = pallet_subtensor::EMAPriceHalvingBlocks::<Test>::get(netuid);
        assert_eq!(value_after_1, value_before);
        assert_ok!(AdminUtils::sudo_set_ema_price_halving_period(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        let value_after_2: u64 = pallet_subtensor::EMAPriceHalvingBlocks::<Test>::get(netuid);
        assert_eq!(value_after_2, to_be_set);
    });
}

// cargo test --package pallet-admin-utils --lib -- tests::test_set_sn_owner_hotkey --exact --show-output
#[test]
fn test_set_sn_owner_hotkey_owner() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey: U256 = U256::from(3);
        let bad_origin_coldkey: U256 = U256::from(4);
        add_network(netuid, 10);

        let owner = U256::from(10);
        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, owner);

        // Non-owner and non-root cannot set the sn owner hotkey
        assert_eq!(
            AdminUtils::sudo_set_sn_owner_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(bad_origin_coldkey),
                netuid,
                hotkey
            ),
            Err(DispatchError::BadOrigin)
        );

        // SN owner can set the hotkey
        assert_ok!(AdminUtils::sudo_set_sn_owner_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            hotkey
        ));

        // Check the value
        let actual_hotkey = pallet_subtensor::SubnetOwnerHotkey::<Test>::get(netuid);
        assert_eq!(actual_hotkey, hotkey);

        // Cannot set again (rate limited)
        assert_err!(
            AdminUtils::sudo_set_sn_owner_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                hotkey
            ),
            pallet_subtensor::Error::<Test>::TxRateLimitExceeded
        );
    });
}

// cargo test --package pallet-admin-utils --lib -- tests::test_set_sn_owner_hotkey_root --exact --show-output
#[test]
fn test_set_sn_owner_hotkey_root() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey: U256 = U256::from(3);
        add_network(netuid, 10);

        let owner = U256::from(10);
        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, owner);

        // Root can set the hotkey
        assert_ok!(AdminUtils::sudo_set_sn_owner_hotkey(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            hotkey
        ));

        // Check the value
        let actual_hotkey = pallet_subtensor::SubnetOwnerHotkey::<Test>::get(netuid);
        assert_eq!(actual_hotkey, hotkey);
    });
}

#[test]
fn test_sudo_set_bonds_reset_enabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: bool = true;
        let sn_owner = U256::from(1);
        add_network(netuid, 10);
        let init_value: bool = SubtensorModule::get_bonds_reset(netuid);

        assert_eq!(
            AdminUtils::sudo_set_bonds_reset_enabled(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );

        assert_ok!(AdminUtils::sudo_set_bonds_reset_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_reset(netuid), to_be_set);
        assert_ne!(SubtensorModule::get_bonds_reset(netuid), init_value);

        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, sn_owner);

        assert_ok!(AdminUtils::sudo_set_bonds_reset_enabled(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            !to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_reset(netuid), !to_be_set);
    });
}

#[test]
fn test_sudo_set_yuma3_enabled() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: bool = true;
        let sn_owner = U256::from(1);
        add_network(netuid, 10);
        let init_value: bool = SubtensorModule::get_yuma3_enabled(netuid);

        assert_eq!(
            AdminUtils::sudo_set_yuma3_enabled(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );

        assert_ok!(AdminUtils::sudo_set_yuma3_enabled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_yuma3_enabled(netuid), to_be_set);
        assert_ne!(SubtensorModule::get_yuma3_enabled(netuid), init_value);

        pallet_subtensor::SubnetOwner::<Test>::insert(netuid, sn_owner);

        assert_ok!(AdminUtils::sudo_set_yuma3_enabled(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            !to_be_set
        ));
        assert_eq!(SubtensorModule::get_yuma3_enabled(netuid), !to_be_set);
    });
}

#[test]
fn test_sudo_set_commit_reveal_version() {
    new_test_ext().execute_with(|| {
        add_network(NetUid::from(1), 10);

        let to_be_set: u16 = 5;
        let init_value: u16 = SubtensorModule::get_commit_reveal_weights_version();

        assert_ok!(AdminUtils::sudo_set_commit_reveal_version(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));

        assert!(init_value != to_be_set);
        assert_eq!(
            SubtensorModule::get_commit_reveal_weights_version(),
            to_be_set
        );
    });
}

#[test]
fn test_sudo_set_admin_freeze_window_and_rate() {
    new_test_ext().execute_with(|| {
        // Non-root fails
        assert_eq!(
            AdminUtils::sudo_set_admin_freeze_window(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                7
            ),
            Err(DispatchError::BadOrigin)
        );
        // Root succeeds
        assert_ok!(AdminUtils::sudo_set_admin_freeze_window(
            <<Test as Config>::RuntimeOrigin>::root(),
            7
        ));
        assert_eq!(pallet_subtensor::AdminFreezeWindow::<Test>::get(), 7);

        // Owner hyperparam tempos setter
        assert_eq!(
            AdminUtils::sudo_set_owner_hparam_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                5
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_ok!(AdminUtils::sudo_set_owner_hparam_rate_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            5
        ));
        assert_eq!(pallet_subtensor::OwnerHyperparamRateLimit::<Test>::get(), 5);
    });
}

#[test]
fn test_freeze_window_blocks_root_and_owner() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo = 10;
        // Create subnet with tempo 10
        add_network(netuid, tempo);
        // Set freeze window to 3 blocks
        assert_ok!(AdminUtils::sudo_set_admin_freeze_window(
            <<Test as Config>::RuntimeOrigin>::root(),
            3
        ));
        // Advance to a block where remaining < 3
        run_to_block((tempo - 2).into());

        // Root should be blocked during freeze window
        assert_noop!(
            AdminUtils::sudo_set_min_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                123.into()
            ),
            SubtensorError::<Test>::AdminActionProhibitedDuringWeightsWindow
        );

        // Owner should be blocked during freeze window as well
        // Set owner
        let owner: U256 = U256::from(9);
        SubnetOwner::<Test>::insert(netuid, owner);
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                77
            ),
            SubtensorError::<Test>::AdminActionProhibitedDuringWeightsWindow
        );
    });
}

#[test]
fn test_sudo_set_min_burn() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set = TaoCurrency::from(1_000_000);
        add_network(netuid, 10);
        let init_value = SubtensorModule::get_min_burn(netuid);

        // Simple case
        assert_ok!(AdminUtils::sudo_set_min_burn(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            TaoCurrency::from(to_be_set)
        ));
        assert_ne!(SubtensorModule::get_min_burn(netuid), init_value);
        assert_eq!(SubtensorModule::get_min_burn(netuid), to_be_set);

        // Unknown subnet
        assert_err!(
            AdminUtils::sudo_set_min_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                NetUid::from(42),
                TaoCurrency::from(to_be_set)
            ),
            Error::<Test>::SubnetDoesNotExist
        );

        // Non subnet owner
        assert_err!(
            AdminUtils::sudo_set_min_burn(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                TaoCurrency::from(to_be_set)
            ),
            DispatchError::BadOrigin
        );

        // Above upper bound
        assert_err!(
            AdminUtils::sudo_set_min_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                <Test as pallet_subtensor::Config>::MinBurnUpperBound::get() + 1.into()
            ),
            Error::<Test>::ValueNotInBounds
        );

        // Above max burn
        assert_err!(
            AdminUtils::sudo_set_min_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_max_burn(netuid) + 1.into()
            ),
            Error::<Test>::ValueNotInBounds
        );
    });
}

#[test]
fn test_owner_hyperparam_update_rate_limit_enforced() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 10);
        // Set owner
        let owner: U256 = U256::from(5);
        SubnetOwner::<Test>::insert(netuid, owner);

        // Set tempo to 1 so owner hyperparam RL = 2 tempos = 2 blocks
        SubtensorModule::set_tempo(netuid, 1);
        // Disable admin freeze window to avoid blocking on small tempo
        assert_ok!(AdminUtils::sudo_set_admin_freeze_window(
            <<Test as Config>::RuntimeOrigin>::root(),
            0
        ));

        // First update succeeds
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            11
        ));
        // Immediate second update fails due to TxRateLimitExceeded
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                12
            ),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // Advance less than limit still fails
        run_to_block(SubtensorModule::get_current_block_as_u64() + 1);
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                13
            ),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // Advance one more block to pass the limit; should succeed
        run_to_block(SubtensorModule::get_current_block_as_u64() + 1);
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            14
        ));
    });
}

// Verifies that owner hyperparameter rate limit is enforced based on tempo (2 tempos).
#[test]
fn test_hyperparam_rate_limit_enforced_by_tempo() {
    new_test_ext().execute_with(|| {
        // Setup subnet and owner
        let netuid = NetUid::from(42);
        add_network(netuid, 10);
        let owner: U256 = U256::from(77);
        SubnetOwner::<Test>::insert(netuid, owner);

        // Set tempo to 1 so RL = 2 blocks
        SubtensorModule::set_tempo(netuid, 1);
        // Disable admin freeze window to avoid blocking on small tempo
        assert_ok!(AdminUtils::sudo_set_admin_freeze_window(
            <<Test as Config>::RuntimeOrigin>::root(),
            0
        ));

        // First owner update should succeed
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            1
        ));

        // Immediate second update should fail due to tempo-based RL
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                2
            ),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // Advance 2 blocks (2 tempos with tempo=1) then succeed
        run_to_block(SubtensorModule::get_current_block_as_u64() + 2);
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            3
        ));
    });
}

// Verifies owner hyperparameters are rate-limited independently per parameter.
// Setting one hyperparameter should not block setting a different hyperparameter
// during the same rate-limit window, but it should still block itself.
#[test]
fn test_owner_hyperparam_rate_limit_independent_per_param() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(7);
        add_network(netuid, 10);

        // Set subnet owner
        let owner: U256 = U256::from(123);
        SubnetOwner::<Test>::insert(netuid, owner);

        // Use small tempo to make RL short and deterministic (2 blocks when tempo=1)
        SubtensorModule::set_tempo(netuid, 1);
        // Disable admin freeze window so it doesn't interfere with small tempo
        assert_ok!(AdminUtils::sudo_set_admin_freeze_window(
            <<Test as Config>::RuntimeOrigin>::root(),
            0
        ));

        // First update to kappa should succeed
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            10
        ));

        // Immediate second update to the SAME param (kappa) should be blocked by RL
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                11
            ),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // Updating a DIFFERENT param (rho) should pass immediately — independent RL key
        assert_ok!(AdminUtils::sudo_set_rho(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            5
        ));

        // kappa should still be blocked until its own RL window passes
        assert_noop!(
            AdminUtils::sudo_set_commit_reveal_weights_interval(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                12
            ),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // rho should also be blocked for itself immediately after being set
        assert_noop!(
            AdminUtils::sudo_set_rho(<<Test as Config>::RuntimeOrigin>::signed(owner), netuid, 6),
            SubtensorError::<Test>::TxRateLimitExceeded
        );

        // Advance enough blocks to pass the RL window (2 blocks when tempo=1 and default epochs=2)
        run_to_block(SubtensorModule::get_current_block_as_u64() + 2);

        // Now both hyperparameters can be updated again
        assert_ok!(AdminUtils::sudo_set_commit_reveal_weights_interval(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            13
        ));
        assert_ok!(AdminUtils::sudo_set_rho(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            7
        ));
    });
}

#[test]
fn test_sudo_set_max_burn() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set = TaoCurrency::from(100_000_001);
        add_network(netuid, 10);
        let init_value = SubtensorModule::get_max_burn(netuid);

        // Simple case
        assert_ok!(AdminUtils::sudo_set_max_burn(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            TaoCurrency::from(to_be_set)
        ));
        assert_ne!(SubtensorModule::get_max_burn(netuid), init_value);
        assert_eq!(SubtensorModule::get_max_burn(netuid), to_be_set);

        // Unknown subnet
        assert_err!(
            AdminUtils::sudo_set_max_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                NetUid::from(42),
                TaoCurrency::from(to_be_set)
            ),
            Error::<Test>::SubnetDoesNotExist
        );

        // Non subnet owner
        assert_err!(
            AdminUtils::sudo_set_max_burn(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                TaoCurrency::from(to_be_set)
            ),
            DispatchError::BadOrigin
        );

        // Below lower bound
        assert_err!(
            AdminUtils::sudo_set_max_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                <Test as pallet_subtensor::Config>::MaxBurnLowerBound::get() - 1.into()
            ),
            Error::<Test>::ValueNotInBounds
        );

        // Below min burn
        assert_err!(
            AdminUtils::sudo_set_max_burn(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_min_burn(netuid) - 1.into()
            ),
            Error::<Test>::ValueNotInBounds
        );
    });
}

#[test]
fn test_sudo_set_mechanism_count() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let ss_count_ok = MaxMechanismCount::<Test>::get();
        let ss_count_bad = MechId::from(u8::from(ss_count_ok) + 1);

        let sn_owner = U256::from(1324);
        add_network(netuid, 10);
        // Set the Subnet Owner
        SubnetOwner::<Test>::insert(netuid, sn_owner);

        assert_eq!(
            AdminUtils::sudo_set_mechanism_count(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                ss_count_ok
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_noop!(
            AdminUtils::sudo_set_mechanism_count(RuntimeOrigin::root(), netuid, ss_count_bad),
            pallet_subtensor::Error::<Test>::InvalidValue
        );

        assert_ok!(AdminUtils::sudo_set_mechanism_count(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            ss_count_ok
        ));

        assert_ok!(AdminUtils::sudo_set_mechanism_count(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            ss_count_ok
        ));
    });
}

// cargo test --package pallet-admin-utils --lib -- tests::test_sudo_set_mechanism_count_and_emissions --exact --show-output
#[test]
fn test_sudo_set_mechanism_count_and_emissions() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let ss_count_ok = MechId::from(2);

        let sn_owner = U256::from(1324);
        add_network(netuid, 10);
        // Set the Subnet Owner
        SubnetOwner::<Test>::insert(netuid, sn_owner);

        assert_ok!(AdminUtils::sudo_set_mechanism_count(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            ss_count_ok
        ));

        // Cannot set emission split with wrong number of entries
        // With two mechanisms the size of the split vector should be 2, not 3
        assert_noop!(
            AdminUtils::sudo_set_mechanism_emission_split(
                <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
                netuid,
                Some(vec![0xFFFF / 5 * 2, 0xFFFF / 5 * 2, 0xFFFF / 5])
            ),
            pallet_subtensor::Error::<Test>::InvalidValue
        );

        // Cannot set emission split with wrong total of entries
        // Split vector entries should sum up to exactly 0xFFFF
        assert_noop!(
            AdminUtils::sudo_set_mechanism_emission_split(
                <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
                netuid,
                Some(vec![0xFFFF / 5 * 4, 0xFFFF / 5 - 1])
            ),
            pallet_subtensor::Error::<Test>::InvalidValue
        );

        // Can set good split ok
        // We also verify here that it can happen in the same block as setting mechanism counts
        // or soon, without rate limiting
        assert_ok!(AdminUtils::sudo_set_mechanism_emission_split(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            Some(vec![0xFFFF / 5, 0xFFFF / 5 * 4])
        ));

        // Cannot set it again due to rate limits
        assert_noop!(
            AdminUtils::sudo_set_mechanism_emission_split(
                <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
                netuid,
                Some(vec![0xFFFF / 5 * 4, 0xFFFF / 5])
            ),
            pallet_subtensor::Error::<Test>::TxRateLimitExceeded
        );
    });
}

#[test]
fn test_trim_to_max_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let sn_owner = U256::from(1);
        let sn_owner_hotkey1 = U256::from(2);
        let sn_owner_hotkey2 = U256::from(3);
        add_network(netuid, 10);
        SubnetOwner::<Test>::insert(netuid, sn_owner);
        SubnetOwnerHotkey::<Test>::insert(netuid, sn_owner_hotkey1);
        MaxRegistrationsPerBlock::<Test>::insert(netuid, 256);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 256);
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 2);
        // We set a low value here to make testing easier
        MinAllowedUids::<Test>::set(netuid, 4);
        // We define 4 mechanisms
        let mechanism_count = MechId::from(4);
        MechanismCountCurrent::<Test>::insert(netuid, mechanism_count);

        // Add some neurons
        let max_n = 16;
        for i in 1..=max_n {
            let n = i * 1000;
            register_ok_neuron(netuid, U256::from(n), U256::from(n + i), 0);
        }

        // Run some block to ensure stake weights are set and that we are past the immunity period
        // for all neurons
        run_to_block((ImmunityPeriod::<Test>::get(netuid) + 1).into());

        // Set some randomized values that we can keep track of
        let values = vec![
            17u16, 42u16, 8u16, 56u16, 23u16, 91u16, 34u16, // owner owned
            77u16, // temporally immune
            12u16, 65u16, 3u16, 88u16, // owner owned
            29u16, 51u16, 74u16, // temporally immune
            39u16,
        ];
        let bool_values = vec![
            false, false, false, true, false, true, true, // owner owned
            true, // temporally immune
            false, true, false, true, // owner owned
            false, true, true, // temporally immune
            false,
        ];
        let alpha_values = values.iter().map(|&v| (v as u64).into()).collect();
        let u64_values: Vec<u64> = values.iter().map(|&v| v as u64).collect();

        Emission::<Test>::set(netuid, alpha_values);
        Rank::<Test>::insert(netuid, values.clone());
        Trust::<Test>::insert(netuid, values.clone());
        Consensus::<Test>::insert(netuid, values.clone());
        Dividends::<Test>::insert(netuid, values.clone());
        PruningScores::<Test>::insert(netuid, values.clone());
        ValidatorTrust::<Test>::insert(netuid, values.clone());
        StakeWeight::<Test>::insert(netuid, values.clone());
        ValidatorPermit::<Test>::insert(netuid, bool_values.clone());
        Active::<Test>::insert(netuid, bool_values);

        for mecid in 0..mechanism_count.into() {
            let netuid_index =
                SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(mecid));
            Incentive::<Test>::insert(netuid_index, values.clone());
            LastUpdate::<Test>::insert(netuid_index, u64_values.clone());
        }

        // We set some owner immune uids
        let now = frame_system::Pallet::<Test>::block_number();
        BlockAtRegistration::<Test>::set(netuid, 6, now);
        BlockAtRegistration::<Test>::set(netuid, 11, now);

        // And some temporally immune uids
        Keys::<Test>::insert(netuid, 7, sn_owner_hotkey1);
        Uids::<Test>::insert(netuid, sn_owner_hotkey1, 7);
        Keys::<Test>::insert(netuid, 14, sn_owner_hotkey2);
        Uids::<Test>::insert(netuid, sn_owner_hotkey2, 14);

        // Set some evm addresses
        AssociatedEvmAddress::<Test>::insert(
            netuid,
            6,
            (sp_core::H160::from_slice(b"12345678901234567891"), now),
        );
        AssociatedEvmAddress::<Test>::insert(
            netuid,
            10,
            (sp_core::H160::from_slice(b"12345678901234567892"), now),
        );
        AssociatedEvmAddress::<Test>::insert(
            netuid,
            12,
            (sp_core::H160::from_slice(b"12345678901234567893"), now),
        );
        AssociatedEvmAddress::<Test>::insert(
            netuid,
            14,
            (sp_core::H160::from_slice(b"12345678901234567894"), now),
        );

        // Populate Weights and Bonds storage items to test trimming
        // Create weights and bonds that span across the range that will be trimmed
        for uid in 0..max_n {
            let mut weights = Vec::new();
            let mut bonds = Vec::new();

            // Add connections to all other uids, including those that will be trimmed
            for target_uid in 0..max_n {
                if target_uid != uid {
                    // Use some non-zero values to make the test more meaningful
                    let weight_value = (uid + target_uid) % 1000;
                    let bond_value = (uid * target_uid) % 1000;
                    weights.push((target_uid, weight_value));
                    bonds.push((target_uid, bond_value));
                }
            }

            for mecid in 0..mechanism_count.into() {
                let netuid_index =
                    SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(mecid));
                Weights::<Test>::insert(netuid_index, uid, weights.clone());
                Bonds::<Test>::insert(netuid_index, uid, bonds.clone());
            }
        }

        // Normal case
        let new_max_n = 8;
        assert_ok!(AdminUtils::sudo_trim_to_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            new_max_n
        ));

        // Ensure the max allowed uids has been set correctly
        assert_eq!(MaxAllowedUids::<Test>::get(netuid), new_max_n);

        // Ensure the emission has been trimmed correctly, keeping the highest emitters
        // and immune and compressed to the left
        assert_eq!(
            Emission::<Test>::get(netuid),
            vec![
                56.into(),
                91.into(),
                34.into(),
                77.into(),
                65.into(),
                88.into(),
                51.into(),
                74.into()
            ]
        );
        // Ensure rest of storage has been trimmed correctly
        let expected_values = vec![56, 91, 34, 77, 65, 88, 51, 74];
        let expected_bools = vec![true, true, true, true, true, true, true, true];
        let expected_u64_values = vec![56, 91, 34, 77, 65, 88, 51, 74];
        assert_eq!(Rank::<Test>::get(netuid), expected_values);
        assert_eq!(Trust::<Test>::get(netuid), expected_values);
        assert_eq!(Active::<Test>::get(netuid), expected_bools);
        assert_eq!(Consensus::<Test>::get(netuid), expected_values);
        assert_eq!(Dividends::<Test>::get(netuid), expected_values);
        assert_eq!(PruningScores::<Test>::get(netuid), expected_values);
        assert_eq!(ValidatorTrust::<Test>::get(netuid), expected_values);
        assert_eq!(ValidatorPermit::<Test>::get(netuid), expected_bools);
        assert_eq!(StakeWeight::<Test>::get(netuid), expected_values);

        for mecid in 0..mechanism_count.into() {
            let netuid_index =
                SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(mecid));
            assert_eq!(Incentive::<Test>::get(netuid_index), expected_values);
            assert_eq!(LastUpdate::<Test>::get(netuid_index), expected_u64_values);
        }

        // Ensure trimmed uids related storage has been cleared
        for uid in new_max_n..max_n {
            assert!(!Keys::<Test>::contains_key(netuid, uid));
            assert!(!BlockAtRegistration::<Test>::contains_key(netuid, uid));
            assert!(!AssociatedEvmAddress::<Test>::contains_key(netuid, uid));
            for mecid in 0..mechanism_count.into() {
                let netuid_index =
                    SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(mecid));
                assert!(!Weights::<Test>::contains_key(netuid_index, uid));
                assert!(!Bonds::<Test>::contains_key(netuid_index, uid));
            }
        }

        // Ensure trimmed uids hotkey related storage has been cleared
        let trimmed_hotkeys = vec![
            U256::from(1000),
            U256::from(2000),
            U256::from(3000),
            U256::from(5000),
            U256::from(9000),
            U256::from(11000),
            U256::from(13000),
            U256::from(16000),
        ];
        for hotkey in trimmed_hotkeys {
            assert!(!Uids::<Test>::contains_key(netuid, hotkey));
            assert!(!IsNetworkMember::<Test>::contains_key(hotkey, netuid));
            assert!(!LastHotkeyEmissionOnNetuid::<Test>::contains_key(
                hotkey, netuid
            ));
            assert!(!AlphaDividendsPerSubnet::<Test>::contains_key(
                netuid, hotkey
            ));
            assert!(!TaoDividendsPerSubnet::<Test>::contains_key(netuid, hotkey));
            assert!(!Axons::<Test>::contains_key(netuid, hotkey));
            assert!(!NeuronCertificates::<Test>::contains_key(netuid, hotkey));
            assert!(!Prometheus::<Test>::contains_key(netuid, hotkey));
        }

        // Ensure trimmed uids weights and bonds connections have been trimmed correctly
        for uid in 0..new_max_n {
            for mecid in 0..mechanism_count.into() {
                let netuid_index =
                    SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(mecid));
                assert!(
                    Weights::<Test>::get(netuid_index, uid)
                        .iter()
                        .all(|(target_uid, _)| *target_uid < new_max_n),
                    "Found a weight with target_uid >= new_max_n"
                );
                assert!(
                    Bonds::<Test>::get(netuid_index, uid)
                        .iter()
                        .all(|(target_uid, _)| *target_uid < new_max_n),
                    "Found a bond with target_uid >= new_max_n"
                );
            }
        }

        // Actual number of neurons on the network updated after trimming
        assert_eq!(SubnetworkN::<Test>::get(netuid), new_max_n);

        // Uids match enumeration order
        for i in 0..new_max_n.into() {
            let hotkey = Keys::<Test>::get(netuid, i);
            let uid = Uids::<Test>::get(netuid, hotkey);
            assert_eq!(uid, Some(i));
        }

        // EVM association have been remapped correctly (uids: 7 -> 2, 14 -> 7)
        assert_eq!(
            AssociatedEvmAddress::<Test>::get(netuid, 2),
            Some((sp_core::H160::from_slice(b"12345678901234567891"), now))
        );
        assert_eq!(
            AssociatedEvmAddress::<Test>::get(netuid, 7),
            Some((sp_core::H160::from_slice(b"12345678901234567894"), now))
        );

        // Non existent subnet
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                NetUid::from(42),
                new_max_n
            ),
            pallet_subtensor::Error::<Test>::SubnetNotExists
        );

        // New max n less than lower bound
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                2
            ),
            pallet_subtensor::Error::<Test>::InvalidValue
        );

        // New max n greater than upper bound
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_max_allowed_uids(netuid) + 1
            ),
            pallet_subtensor::Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn test_trim_to_max_allowed_uids_too_many_immune() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let sn_owner = U256::from(1);
        add_network(netuid, 10);
        SubnetOwner::<Test>::insert(netuid, sn_owner);
        MaxRegistrationsPerBlock::<Test>::insert(netuid, 256);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 256);
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 2);
        MinAllowedUids::<Test>::set(netuid, 2);

        // Add 5 neurons
        let max_n = 5;
        for i in 1..=max_n {
            let n = i * 1000;
            register_ok_neuron(netuid, U256::from(n), U256::from(n + i), 0);
        }

        // Run some blocks to ensure stake weights are set
        run_to_block((ImmunityPeriod::<Test>::get(netuid) + 1).into());

        // Set owner immune uids (2 UIDs) by adding them to OwnedHotkeys
        let owner_hotkey1 = U256::from(1000);
        let owner_hotkey2 = U256::from(2000);
        OwnedHotkeys::<Test>::insert(sn_owner, vec![owner_hotkey1, owner_hotkey2]);
        Keys::<Test>::insert(netuid, 0, owner_hotkey1);
        Uids::<Test>::insert(netuid, owner_hotkey1, 0);
        Keys::<Test>::insert(netuid, 1, owner_hotkey2);
        Uids::<Test>::insert(netuid, owner_hotkey2, 1);

        // Set temporally immune uids (2 UIDs) to make total immune count 4 out of 5 (80%)
        // Set their registration block to current block to make them temporally immune
        let current_block = frame_system::Pallet::<Test>::block_number();
        for uid in 2..4 {
            let hotkey = U256::from(uid * 1000 + 1000);
            Keys::<Test>::insert(netuid, uid, hotkey);
            Uids::<Test>::insert(netuid, hotkey, uid);
            BlockAtRegistration::<Test>::insert(netuid, uid, current_block);
        }

        // Try to trim to 4 UIDs - this should fail because 4/4 = 100% immune (>= 80%)
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                4
            ),
            pallet_subtensor::Error::<Test>::TrimmingWouldExceedMaxImmunePercentage
        );

        // Try to trim to 3 UIDs - this should also fail because 4/3 > 80% immune (>= 80%)
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                3
            ),
            pallet_subtensor::Error::<Test>::TrimmingWouldExceedMaxImmunePercentage
        );

        // Now test a scenario where trimming should succeed
        // Remove one immune UID to make it 3 immune out of 4 total
        let uid_to_remove = 3;
        let hotkey_to_remove = U256::from(uid_to_remove * 1000 + 1000);
        #[allow(unknown_lints)]
        Keys::<Test>::remove(netuid, uid_to_remove);
        Uids::<Test>::remove(netuid, hotkey_to_remove);
        BlockAtRegistration::<Test>::remove(netuid, uid_to_remove);

        // Remove another immune UID to make it 2 immune out of 3 total
        let uid_to_remove2 = 2;
        let hotkey_to_remove2 = U256::from(uid_to_remove2 * 1000 + 1000);
        #[allow(unknown_lints)]
        Keys::<Test>::remove(netuid, uid_to_remove2);
        Uids::<Test>::remove(netuid, hotkey_to_remove2);
        BlockAtRegistration::<Test>::remove(netuid, uid_to_remove2);

        // Now we have 2 immune out of 2 total UIDs
        // Try to trim to 1 UID - this should fail because 2/1 is impossible, but the check prevents it
        assert_err!(
            AdminUtils::sudo_trim_to_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                1
            ),
            pallet_subtensor::Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn test_sudo_set_min_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let to_be_set: u16 = 8;
        add_network(netuid, 10);
        MaxRegistrationsPerBlock::<Test>::insert(netuid, 256);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 256);

        // Register some neurons
        for i in 0..=16 {
            register_ok_neuron(netuid, U256::from(i * 1000), U256::from(i * 1000 + i), 0);
        }

        // Normal case
        assert_ok!(AdminUtils::sudo_set_min_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_min_allowed_uids(netuid), to_be_set);

        // Non root
        assert_err!(
            AdminUtils::sudo_set_min_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            DispatchError::BadOrigin
        );

        // Non existent subnet
        assert_err!(
            AdminUtils::sudo_set_min_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                NetUid::from(42),
                to_be_set
            ),
            Error::<Test>::SubnetDoesNotExist
        );

        // Min allowed uids greater than max allowed uids
        assert_err!(
            AdminUtils::sudo_set_min_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_max_allowed_uids(netuid) + 1
            ),
            Error::<Test>::MinAllowedUidsGreaterThanMaxAllowedUids
        );

        // Min allowed uids greater than current uids
        assert_err!(
            AdminUtils::sudo_set_min_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid,
                SubtensorModule::get_subnetwork_n(netuid) + 1
            ),
            Error::<Test>::MinAllowedUidsGreaterThanCurrentUids
        );
    });
}

#[test]
fn test_get_validator_cut() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);
        let expected_cut: u64 = u64::MAX / 2; // 50% cut

        // Set up a network
        add_network(netuid, 10);

        // Set a validator cut value
        assert_ok!(SubtensorModule::set_validator_cut(netuid, expected_cut));

        // Test that we can retrieve the value
        let retrieved_cut = SubtensorModule::get_validator_cut(netuid);
        assert_eq!(retrieved_cut, expected_cut);
    });
}

#[test]
fn test_set_validator_cut() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(2);
        let initial_cut: u64 = pallet_subtensor::DefaultValidatorCut::<Test>::get();
        let new_cut: u64 = u64::MAX / 3; // 33% cut

        // Set up a network
        add_network(netuid, 10);

        // Verify initial value
        assert_eq!(SubtensorModule::get_validator_cut(netuid), initial_cut);

        // Set new validator cut
        assert_ok!(SubtensorModule::set_validator_cut(netuid, new_cut));

        // Verify the value was set correctly
        assert_eq!(SubtensorModule::get_validator_cut(netuid), new_cut);
    });
}

#[test]
fn test_sudo_set_validator_cut() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(3);
        let to_be_set: u64 = u64::MAX / 3;

        // Set up a network
        add_network(netuid, 10);

        let sn_owner = U256::from(1324);
        // Set the Subnet Owner
        SubnetOwner::<Test>::insert(netuid, sn_owner);

        let init_value = SubtensorModule::get_validator_cut(netuid);

        // Test that non-authorized origin fails (using a regular signed origin)
        assert_eq!(
            AdminUtils::sudo_set_validator_cut(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        // Value should remain unchangeds
        assert_eq!(SubtensorModule::get_validator_cut(netuid), init_value);

        assert_ok!(AdminUtils::sudo_set_validator_cut(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner),
            netuid,
            to_be_set
        ));

        // Verify the value was set correctly
        assert_eq!(SubtensorModule::get_validator_cut(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_validator_cut_root() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(4);
        let to_be_set: u64 = u64::MAX / 3;

        // Set up a network
        add_network(netuid, 10);

        // Test that root can set the validator cut successfully
        assert_ok!(AdminUtils::sudo_set_validator_cut(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));

        // Verify the value was set correctly
        assert_eq!(SubtensorModule::get_validator_cut(netuid), to_be_set);
    });
}

#[test]
fn test_validator_cut_bounds() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(5);
        let min_cut: u64 = 0; // 0% cut
        let max_cut: u64 = u64::MAX; // 100% cut

        // Set up a network
        add_network(netuid, 10);

        // Test minimum value
        assert_err!(
            SubtensorModule::set_validator_cut(netuid, min_cut),
            DispatchError::from(pallet_subtensor::Error::<Test>::InvalidValidatorCut)
        );

        // Test maximum value
        assert_err!(
            SubtensorModule::set_validator_cut(netuid, max_cut),
            DispatchError::from(pallet_subtensor::Error::<Test>::InvalidValidatorCut)
        );
        assert_eq!(
            SubtensorModule::get_validator_cut(netuid),
            DefaultValidatorCut::<Test>::get()
        );
    });
}
