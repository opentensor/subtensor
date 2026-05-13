#![allow(clippy::expect_used)]
use frame_support::{assert_noop, assert_ok};
use frame_system::Config;
use sp_core::U256;
use subtensor_runtime_common::NetUid;

use super::mock::*;
use crate::{
    AdminFreezeWindow, CommitRevealWeightsEnabled, Error, LastEpochBlock, PendingEpochAt,
    SubnetOwner, SubtokenEnabled, Tempo,
};

const DEFAULT_TEMPO: u16 = 360;
const NEW_TEMPO: u16 = 720;

fn setup_subnet(owner: U256) -> NetUid {
    let netuid = NetUid::from(1);
    add_network(netuid, DEFAULT_TEMPO, 0);
    SubnetOwner::<Test>::insert(netuid, owner);
    SubtokenEnabled::<Test>::insert(netuid, true);
    crate::Pallet::<Test>::set_admin_freeze_window(0);
    netuid
}

#[test]
fn do_set_tempo_blocked_when_commit_reveal_enabled() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        // Default for `CommitRevealWeightsEnabled` is `true` (DefaultCommitRevealWeightsEnabled).
        assert!(CommitRevealWeightsEnabled::<Test>::get(netuid));

        assert_noop!(
            crate::Pallet::<Test>::do_set_tempo(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
                NEW_TEMPO,
            ),
            Error::<Test>::DynamicTempoBlockedByCommitReveal
        );

        // Tempo unchanged.
        assert_eq!(Tempo::<Test>::get(netuid), DEFAULT_TEMPO);
    });
}

#[test]
fn do_set_tempo_passes_when_commit_reveal_disabled() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        CommitRevealWeightsEnabled::<Test>::insert(netuid, false);

        assert_ok!(crate::Pallet::<Test>::do_set_tempo(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            NEW_TEMPO,
        ));

        assert_eq!(Tempo::<Test>::get(netuid), NEW_TEMPO);
    });
}

#[test]
fn do_trigger_epoch_blocked_when_commit_reveal_enabled() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        assert!(CommitRevealWeightsEnabled::<Test>::get(netuid));

        assert_noop!(
            crate::Pallet::<Test>::do_trigger_epoch(
                <<Test as Config>::RuntimeOrigin>::signed(owner),
                netuid,
            ),
            Error::<Test>::DynamicTempoBlockedByCommitReveal
        );

        // No pending trigger recorded.
        assert_eq!(PendingEpochAt::<Test>::get(netuid), 0);
    });
}

#[test]
fn do_trigger_epoch_passes_when_commit_reveal_disabled() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        CommitRevealWeightsEnabled::<Test>::insert(netuid, false);
        AdminFreezeWindow::<Test>::set(5);

        assert_ok!(crate::Pallet::<Test>::do_trigger_epoch(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
        ));

        let now = crate::Pallet::<Test>::get_current_block_as_u64();
        assert_eq!(PendingEpochAt::<Test>::get(netuid), now + 5);
    });
}

#[test]
fn get_next_epoch_start_block_returns_none_when_tempo_zero() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        Tempo::<Test>::insert(netuid, 0);

        assert_eq!(
            crate::Pallet::<Test>::get_next_epoch_start_block(netuid),
            None
        );
    });
}

#[test]
fn get_next_epoch_start_block_uses_last_epoch_block_plus_tempo_plus_one() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        LastEpochBlock::<Test>::insert(netuid, 100u64);
        Tempo::<Test>::insert(netuid, 50u16);
        PendingEpochAt::<Test>::insert(netuid, 0u64);

        // last (100) + tempo (50) + 1 = 151
        assert_eq!(
            crate::Pallet::<Test>::get_next_epoch_start_block(netuid),
            Some(151)
        );
    });
}

#[test]
fn get_next_epoch_start_block_returns_pending_when_pending_is_earlier() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        LastEpochBlock::<Test>::insert(netuid, 100u64);
        Tempo::<Test>::insert(netuid, 50u16);
        // Owner-triggered manual fire scheduled before automatic next.
        PendingEpochAt::<Test>::insert(netuid, 120u64);

        // min(151, 120) = 120
        assert_eq!(
            crate::Pallet::<Test>::get_next_epoch_start_block(netuid),
            Some(120)
        );
    });
}

#[test]
fn get_next_epoch_start_block_ignores_pending_when_auto_is_earlier() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        LastEpochBlock::<Test>::insert(netuid, 100u64);
        Tempo::<Test>::insert(netuid, 50u16);
        // Pending scheduled after the next automatic fire.
        PendingEpochAt::<Test>::insert(netuid, 200u64);

        // min(151, 200) = 151
        assert_eq!(
            crate::Pallet::<Test>::get_next_epoch_start_block(netuid),
            Some(151)
        );
    });
}

#[test]
fn get_next_epoch_start_block_reflects_set_tempo_cycle_reset() {
    new_test_ext(1).execute_with(|| {
        let owner = U256::from(1);
        let netuid = setup_subnet(owner);

        // CR off so do_set_tempo is allowed.
        CommitRevealWeightsEnabled::<Test>::insert(netuid, false);

        run_to_block(10);
        let new_tempo: u16 = 720;

        assert_ok!(crate::Pallet::<Test>::do_set_tempo(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            netuid,
            new_tempo,
        ));

        let now = crate::Pallet::<Test>::get_current_block_as_u64();
        // apply_tempo_with_cycle_reset sets LastEpochBlock = now;
        // next fire is now + tempo + 1.
        assert_eq!(
            crate::Pallet::<Test>::get_next_epoch_start_block(netuid),
            Some(now + new_tempo as u64 + 1)
        );
    });
}
