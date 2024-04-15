use frame_support::assert_ok;
use frame_support::sp_runtime::DispatchError;
use frame_system::Config;
use pallet_admin_utils::Error;
use pallet_subtensor::Event;
use sp_core::U256;

mod mock;
use mock::*;

pub fn add_network(netuid: u16, tempo: u16) {
    SubtensorModule::init_new_network(netuid, tempo);
    SubtensorModule::set_network_registration_allowed(netuid, true);
    SubtensorModule::set_network_pow_registration_allowed(netuid, true);
}

#[test]
fn test_sudo_set_default_take() {
    new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = SubtensorModule::get_default_take();
        assert_eq!(
            AdminUtils::sudo_set_default_take(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_default_take(), init_value);
        assert_ok!(AdminUtils::sudo_set_default_take(
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
fn test_sudo_set_weights_set_rate_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
fn test_sudo_set_max_weight_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_weight_limit(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_weight_limit(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_weight_limit(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_weight_limit(
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
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
        add_network(netuid, 10);
        let init_value: u16 = SubtensorModule::get_max_allowed_uids(netuid);
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(0)),
                netuid,
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(
            AdminUtils::sudo_set_max_allowed_uids(
                <<Test as Config>::RuntimeOrigin>::root(),
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_ok!(AdminUtils::sudo_set_max_allowed_uids(
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
        let netuid: u16 = 1;
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(SubtensorModule::get_difficulty_as_u64(netuid), init_value);
        assert_ok!(AdminUtils::sudo_set_difficulty(
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
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
fn test_sudo_set_weights_min_stake() {
    new_test_ext().execute_with(|| {
        let to_be_set: u64 = 10;
        let init_value: u64 = SubtensorModule::get_weights_min_stake();
        assert_eq!(
            AdminUtils::sudo_set_weights_min_stake(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_weights_min_stake(), init_value);
        assert_ok!(AdminUtils::sudo_set_weights_min_stake(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_weights_min_stake(), to_be_set);
    });
}

#[test]
fn test_sudo_set_bonds_moving_average() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_bonds_moving_average(netuid);
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
                netuid + 1,
                to_be_set
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
        assert_eq!(
            SubtensorModule::get_bonds_moving_average(netuid),
            init_value
        );
        assert_ok!(AdminUtils::sudo_set_bonds_moving_average(
            <<Test as Config>::RuntimeOrigin>::root(),
            netuid,
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_bonds_moving_average(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_rao_recycled() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        add_network(netuid, 10);
        let init_value: u64 = SubtensorModule::get_rao_recycled(netuid);

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
fn test_sudo_set_subnet_limit() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        add_network(netuid, 10);

        let init_value: u16 = SubtensorModule::get_max_subnets();
        assert_eq!(
            AdminUtils::sudo_set_subnet_limit(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(1)),
                to_be_set
            ),
            Err(DispatchError::BadOrigin)
        );
        assert_eq!(SubtensorModule::get_max_subnets(), init_value);
        assert_ok!(AdminUtils::sudo_set_subnet_limit(
            <<Test as Config>::RuntimeOrigin>::root(),
            to_be_set
        ));
        assert_eq!(SubtensorModule::get_max_subnets(), to_be_set);
    });
}

#[test]
fn test_sudo_set_network_lock_reduction_interval() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
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
        let netuid: u16 = 1;
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
