use frame_support::assert_ok;
use frame_system::Config;
mod mock;
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::{Error, Event};
use sp_core::U256;

#[test]
fn test_sudo_registration() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 0, 0);
        SubtensorModule::set_max_allowed_uids(0, 10);
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(0),
            U256::from(0),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(1),
            U256::from(1),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(2),
            U256::from(2),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(3),
            U256::from(3),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(4),
            U256::from(4),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(5),
            U256::from(5),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(6),
            U256::from(6),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(7),
            U256::from(7),
            10,
            11
        ));
        assert_ok!(SubtensorModule::sudo_register(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            U256::from(8),
            U256::from(8),
            10,
            11
        ));
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(0)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(1)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(2)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(3)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(4)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(5)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(6)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(7)), 11);
        assert_eq!(SubtensorModule::get_coldkey_balance(&U256::from(8)), 11);
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 0).unwrap(),
            U256::from(0)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 1).unwrap(),
            U256::from(1)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 2).unwrap(),
            U256::from(2)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 3).unwrap(),
            U256::from(3)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 4).unwrap(),
            U256::from(4)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 5).unwrap(),
            U256::from(5)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 6).unwrap(),
            U256::from(6)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 7).unwrap(),
            U256::from(7)
        );
        assert_eq!(
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, 8).unwrap(),
            U256::from(8)
        );
        assert_eq!(SubtensorModule::get_total_stake(), 90);
        assert!(SubtensorModule::coldkey_owns_hotkey(
            &U256::from(0),
            &U256::from(0)
        ));
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&U256::from(0)),
            U256::from(0)
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&U256::from(0), &U256::from(0)),
            10
        );
    });
}

#[test]
fn test_sudo_set_default_take() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_default_take();
        assert_eq!(
            SubtensorModule::sudo_set_default_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(SubtensorModule::get_default_take(), init_value);
        assert_ok!(SubtensorModule::sudo_set_default_take(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_default_take(), to_be_set);
    });
}

#[test]
fn test_sudo_set_serving_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 3;
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_serving_rate_limit(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_serving_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(SubtensorModule::get_serving_rate_limit(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_serving_rate_limit(
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
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_min_difficulty(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_min_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_difficulty(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_min_difficulty(
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
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_max_difficulty(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_max_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_difficulty(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_max_difficulty(
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
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_weights_version_key(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_weights_version_key(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_weights_version_key(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_version_key(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_set_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_weights_set_rate_limit(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_weights_set_rate_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_weights_set_rate_limit(netuid),
            init_value
        );
        assert_ok!(SubtensorModule::sudo_set_weights_set_rate_limit(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_adjustment_interval(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_adjustment_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_interval(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_adjustment_interval(
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
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_adjustment_alpha(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_adjustment_alpha(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_adjustment_alpha(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_adjustment_alpha(
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
            SubtensorModule::sudo_set_subnet_owner_cut(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), init_value);
        assert_ok!(SubtensorModule::sudo_set_subnet_owner_cut(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_subnet_owner_cut(), to_be_set);
    });
}

#[test]
fn test_sudo_validator_prune_len() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_validator_prune_len(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_validator_prune_len(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_validator_prune_len(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_validator_prune_len(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_validator_prune_len(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_validator_prune_len(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_scaling_law_power() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 50;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_scaling_law_power(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_scaling_law_power(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_scaling_law_power(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_scaling_law_power(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_scaling_law_power(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_scaling_law_power(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_weight_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_max_weight_limit(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_max_weight_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_issuance() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        assert_eq!(
            SubtensorModule::sudo_set_total_issuance(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_ok!(SubtensorModule::sudo_set_total_issuance(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_total_issuance(), to_be_set);
    });
}

#[test]
fn test_sudo_set_immunity_period() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_immunity_period(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_immunity_period(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_immunity_period(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_immunity_period(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_min_allowed_weights(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_min_allowed_weights(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_min_allowed_weights(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_min_allowed_weights(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_and_decrease_max_allowed_uids() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_ok!(SubtensorModule::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set - 1
        ));
    });
}

#[test]
fn test_sudo_set_kappa() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_kappa(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_kappa(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_kappa(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_kappa(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_rho(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_rho(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_rho(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_rho(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_activity_cutoff(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_activity_cutoff(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_activity_cutoff(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_activity_cutoff(
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_target_registrations_per_interval(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_target_registrations_per_interval(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_target_registrations_per_interval(netuid),
            init_value
        );
        assert_ok!(SubtensorModule::sudo_set_target_registrations_per_interval(
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
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_difficulty(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), init_value);
        assert_ok!(SubtensorModule::sudo_set_difficulty(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_allowed_validators() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10, 0);
        let init_value: u16 = SubtensorModule::get_max_allowed_validators(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_max_allowed_validators(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_max_allowed_validators(netuid),
            init_value
        );
        assert_ok!(SubtensorModule::sudo_set_max_allowed_validators(
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
fn test_sudo_set_bonds_moving_average() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_bonds_moving_average(netuid);
        assert_eq!(
            SubtensorModule::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_bonds_moving_average(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_bonds_moving_average(netuid),
            init_value
        );
        assert_ok!(SubtensorModule::sudo_set_bonds_moving_average(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_moving_average(netuid), to_be_set);
    });
}

// DEPRECATED #[test]
// fn test_sudo_set_network_connection_requirement() {
//     new_test_ext().execute_with(|| {
//         let netuid_a: u16 = 1;
//         let netuid_b: u16 = 2;
//         let requirement: u16 = u16::MAX;
//         assert_eq!(
//             SubtensorModule::sudo_add_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//                 netuid_a,
//                 netuid_b,
//                 requirement
//             ),
//             Err(DispatchError::BadOrigin.into())
//         );
//         assert_eq!(
//             SubtensorModule::sudo_add_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::root(),
//                 netuid_a,
//                 netuid_b,
//                 requirement
//             ),
//             Err(Error::<Test>::NetworkDoesNotExist.into())
//         );
//         add_network(netuid_a, 10, 0);
//         assert_eq!(
//             SubtensorModule::sudo_add_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::root(),
//                 netuid_a,
//                 netuid_a,
//                 requirement
//             ),
//             Err(Error::<Test>::InvalidConnectionRequirement.into())
//         );
//         assert_eq!(
//             SubtensorModule::sudo_add_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::root(),
//                 netuid_a,
//                 netuid_b,
//                 requirement
//             ),
//             Err(Error::<Test>::NetworkDoesNotExist.into())
//         );
//         add_network(netuid_b, 10, 0);
//         assert_ok!(SubtensorModule::sudo_add_network_connection_requirement(
//             <<Test as Config>::RuntimeOrigin>::root(),
//             netuid_a,
//             netuid_b,
//             requirement
//         ));
//         assert_eq!(
//             SubtensorModule::get_network_connection_requirement(netuid_a, netuid_b),
//             requirement
//         );
//         assert_eq!(
//             SubtensorModule::sudo_remove_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
//                 netuid_a,
//                 netuid_b
//             ),
//             Err(DispatchError::BadOrigin.into())
//         );
//         assert_eq!(
//             SubtensorModule::sudo_remove_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::root(),
//                 5 as u16,
//                 5 as u16
//             ),
//             Err(Error::<Test>::NetworkDoesNotExist.into())
//         );
//         assert_eq!(
//             SubtensorModule::sudo_remove_network_connection_requirement(
//                 <<Test as Config>::RuntimeOrigin>::root(),
//                 netuid_a,
//                 5 as u16
//             ),
//             Err(Error::<Test>::NetworkDoesNotExist.into())
//         );
//         assert_ok!(SubtensorModule::sudo_remove_network_connection_requirement(
//             <<Test as Config>::RuntimeOrigin>::root(),
//             netuid_a,
//             netuid_b
//         ));
//         assert_eq!(
//             SubtensorModule::network_connection_requirement_exists(netuid_a, netuid_b),
//             false
//         );
//     });
// }

#[test]
fn test_sudo_set_rao_recycled() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10, 0);
        let init_value: u64 = SubtensorModule::get_rao_recycled(netuid);

        // Need to run from genesis block
        run_to_block(1);

        assert_eq!(
            SubtensorModule::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin.into())
        );
        assert_eq!(
            SubtensorModule::sudo_set_rao_recycled(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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

        assert_ok!(SubtensorModule::sudo_set_rao_recycled(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid), to_be_set);

        // Verify event emitted with correct values
        assert_eq!(
            System::events()
                .last()
                .expect(
                    format!(
                        "Expected there to be events: {:?}",
                        System::events().to_vec()
                    )
                    .as_str()
                )
                .event,
            RuntimeEvent::SubtensorModule(Event::RAORecycledForRegistrationSet(netuid, to_be_set))
        );
    });
}

// -------- tests for PendingEmissionValues --------
#[test]
fn test_sudo_test_tempo_pending_emissions_ok() {
    new_test_ext().execute_with(|| {
        let netuid0: u16 = 1;
        let netuid1: u16 = 2;
        let netuid2: u16 = 3;
        let netuid3: u16 = 5;
        let tempo0: u16 = 1;
        let tempo1: u16 = 2;
        let tempo2: u16 = 3;
        let tempo3: u16 = 5;
        add_network(netuid0, tempo0, 0);
        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);
        add_network(netuid3, tempo3, 0);
        assert_eq!(SubtensorModule::get_tempo(netuid0), tempo0);
        assert_eq!(SubtensorModule::get_tempo(netuid1), tempo1);
        assert_eq!(SubtensorModule::get_tempo(netuid2), tempo2);
        assert_eq!(SubtensorModule::get_tempo(netuid3), tempo3);
        assert_eq!(SubtensorModule::get_emission_value(netuid0), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid1), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid2), 0);
        assert_eq!(SubtensorModule::get_emission_value(netuid3), 0);
        let netuids: Vec<u16> = vec![1, 2, 3, 5];
        let emission: Vec<u64> = vec![100000000, 400000000, 200000000, 300000000];
        SubtensorModule::set_emission_values( &netuids, emission.clone() );
        assert_eq!(SubtensorModule::get_emission_value(netuid0), 100000000);
        assert_eq!(SubtensorModule::get_emission_value(netuid1), 400000000);
        assert_eq!(SubtensorModule::get_emission_value(netuid2), 200000000);
        assert_eq!(SubtensorModule::get_emission_value(netuid3), 300000000);
        assert_eq!(SubtensorModule::get_pending_emission(netuid0), 0);
        assert_eq!(SubtensorModule::get_pending_emission(netuid1), 0);
        assert_eq!(SubtensorModule::get_pending_emission(netuid2), 0);
        assert_eq!(SubtensorModule::get_pending_emission(netuid3), 0);
    });
}

#[test]
pub fn test_sudo_test_pending_emission_ok() {
    new_test_ext().execute_with(|| {
		let netuid1: u16 = 1;
        let tempo1: u16 = 5;

        let netuid2: u16 = 2;
        let tempo2: u16 = 7;

        let netuids: Vec<u16> = vec![1, 2];
        let emission: Vec<u64> = vec![250000000, 750000000];

        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);

        assert_ok!(SubtensorModule::sudo_set_emission_values(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuids,
            emission
        ));
        assert_eq!(SubtensorModule::get_emission_value(netuid1), 250000000);

		// Need to register at least one UID per network or no emission will be produced
		register_ok_neuron(netuid1, hotkey_account_id, coldkey_account_id, 0);

		// We leave netuid2 empty. It should receive no emissions
		//register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 0)

		step_block(2);

		assert_eq!(SubtensorModule::get_pending_emission(netuid1), 250_000_000 * 2 ); // ONLY it's portion of the emission for 2 blocks
		assert_eq!(SubtensorModule::get_pending_emission(netuid2), 0); // Empty networks get no emissions

        step_block(1); // Block == 3

        assert_eq!(SubtensorModule::get_pending_emission(netuid1), 0); // emission drained at block 3 for tempo 5
        assert_eq!(SubtensorModule::get_pending_emission(netuid2), 0); // Empty networks get no emissions

		// Step to avoid tempo for netuid 2
		step_block(1); // Block == 4

		// Register to netuid2 -- No longer empty
		register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 0);

		step_block(1); // Block == 5

		assert_eq!(SubtensorModule::get_pending_emission(netuid2), 750_000_000 * 1); // Gets 1 block of emissions
	}); 
}

#[test]
pub fn test_sudo_test_pending_emission_ok_empty_network() {
    new_test_ext().execute_with(|| {
		let netuid1: u16 = 1;
        let tempo1: u16 = 5;

        let netuid2: u16 = 2;
        let tempo2: u16 = 7;

        let netuids: Vec<u16> = vec![1, 2];
        let emission: Vec<u64> = vec![250000000, 750000000];

		let hotkey_account_id: U256 = U256::from(1); // Can share hotkey account ID between networks
		let coldkey_account_id: U256 = U256::from(2); // Can share coldkey account ID between networks

        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);

        assert_ok!(SubtensorModule::sudo_set_emission_values(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuids,
            emission
        ));
        assert_eq!(SubtensorModule::get_emission_value(netuid1), 250000000);

		// Need to register at least one UID per network or no emission will be produced
		register_ok_neuron(netuid1, hotkey_account_id, coldkey_account_id, 0);

		// We leave netuid2 empty. It should receive no emissions
		//register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 0);

		step_block(2);

		assert_eq!(SubtensorModule::get_pending_emission(netuid1), 250_000_000 * 2 ); // ONLY it's portion of the emission for 2 blocks
		assert_eq!(SubtensorModule::get_pending_emission(netuid2), 0); // Empty networks get no emissions

        step_block(1); // Block == 3

        assert_eq!(SubtensorModule::get_pending_emission(netuid1), 0); // emission drained at block 3 for tempo 5
        assert_eq!(SubtensorModule::get_pending_emission(netuid2), 0); // Empty networks get no emissions

		// Step to avoid tempo for netuid 2
		step_block(1); // Block == 4

		// Register to netuid2 -- No longer empty
		register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 0);

		step_block(1); // Block == 5

		assert_eq!(SubtensorModule::get_pending_emission(netuid2), 750_000_000 * 1); // Gets 1 block of emissions
	});
}
