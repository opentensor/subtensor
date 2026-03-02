use super::*;
use crate::liquidation::types::{LiquidationState, LiquidationWarning};
use crate::pallet::*;
use crate::{Config, Error, Event, Pallet};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;
use subtensor_runtime_common::{Currency, NetUid};

impl<T: Config> Pallet<T> {
    /// Set the cooldown timer on a freed netuid.
    pub fn set_netuid_cooldown(netuid: NetUid) {
        let cooldown_until = <frame_system::Pallet<T>>::block_number()
            .saturating_add(BlockNumberFor::<T>::from(NETUID_COOLDOWN_BLOCKS as u32));
        NetuidCooldown::<T>::insert(netuid, cooldown_until);
    }

    /// Clear the staker snapshot for a liquidated subnet.
    pub fn clear_staker_snapshot(netuid: NetUid, snapshot_count: u32) {
        let removal_limit = snapshot_count.max(1);
        let result = LiquidationStakerSnapshot::<T>::clear_prefix(netuid, removal_limit, None);
        if result.maybe_cursor.is_some() {
            log::warn!(
                "Staker snapshot clear_prefix incomplete for netuid {:?}",
                netuid,
            );
        }
        LiquidationSnapshotCount::<T>::remove(netuid);
    }

    /// Complete pending registration and set cooldown if the slot wasn't claimed.
    pub fn finalize_liquidation_slot(netuid: NetUid) {
        let _ = Self::complete_pending_registration(netuid);
        if !NetworksAdded::<T>::get(netuid) {
            Self::set_netuid_cooldown(netuid);
        }
    }

    /// Emergency finalization — burns remaining TAO, refunds pending reg, frees slot.
    pub fn emergency_finalize(netuid: NetUid, state: &LiquidationState<BlockNumberFor<T>>) {
        // 1. Handle remaining TAO
        let remaining_tao = state.tao_pot.saturating_sub(state.tao_distributed);
        if remaining_tao > 0 {
            TotalIssuance::<T>::mutate(|total| {
                let amount: subtensor_runtime_common::TaoCurrency = remaining_tao.into();
                *total = (*total).saturating_sub(amount);
            });

            Self::deposit_event(Event::LiquidationWarning {
                netuid,
                warning: LiquidationWarning::EmergencyBurn(remaining_tao),
            });
        }

        // 2. Refund pending registration if one exists
        if let Some(pending) = PendingSubnetRegistration::<T>::take() {
            Self::add_balance_to_coldkey_account(&pending.coldkey, pending.cost_paid);
            Self::deposit_event(Event::PendingRegistrationRefunded {
                coldkey: pending.coldkey,
                amount: pending.cost_paid,
            });
        }

        // 3. Free the subnet slot
        NetworksAdded::<T>::remove(netuid);
        TotalNetworks::<T>::mutate(|n: &mut u16| *n = n.saturating_sub(1));
        Self::set_netuid_cooldown(netuid);

        // 4. Clear snapshot
        Self::clear_staker_snapshot(netuid, state.snapshot_count);

        Self::deposit_event(Event::NetworkRemoved(netuid));
    }

    /// Force-complete all remaining liquidation phases in a single call.
    /// WARNING: May exceed normal block weight — root-only.
    pub fn force_complete_all_phases(
        netuid: NetUid,
        mut state: LiquidationState<BlockNumberFor<T>>,
    ) -> frame_support::dispatch::DispatchResult {
        const MAX_ITERATIONS: u32 = 10_000;
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        let mut iterations = 0u32;

        loop {
            iterations = iterations.saturating_add(1);
            if iterations > MAX_ITERATIONS {
                log::error!(
                    "force_complete_all_phases: exceeded {} iterations for netuid {:?}",
                    MAX_ITERATIONS,
                    netuid,
                );
                Self::emergency_finalize(netuid, &state);
                LiquidatingSubnets::<T>::remove(netuid);
                return Err(Error::<T>::LiquidationStuck.into());
            }

            let (_weight_used, updated_state, is_complete) =
                Self::process_liquidation_step(netuid, state, unlimited);

            if is_complete {
                LiquidatingSubnets::<T>::remove(netuid);
                Self::clear_staker_snapshot(netuid, updated_state.snapshot_count);
                Self::finalize_liquidation_slot(netuid);
                return Ok(());
            }

            state = updated_state;
        }
    }

    /// Complete a pending registration using a freed netuid.
    /// Re-validates hotkey ownership at completion time to prevent registration
    /// with a hotkey that was transferred during liquidation.
    pub fn complete_pending_registration(
        freed_netuid: NetUid,
    ) -> frame_support::dispatch::DispatchResult {
        if let Some(pending) = PendingSubnetRegistration::<T>::take() {
            // Re-validate hotkey: if it exists and is owned by a different
            // coldkey, refund instead of registering. A non-existent hotkey
            // is fine — do_register_network_inner will create it.
            if Self::hotkey_account_exists(&pending.hotkey)
                && !Self::coldkey_owns_hotkey(&pending.coldkey, &pending.hotkey)
            {
                log::warn!(
                    "Pending registration refunded: hotkey {:?} now owned by {:?}, not {:?}",
                    pending.hotkey,
                    Owner::<T>::get(&pending.hotkey),
                    pending.coldkey,
                );
                Self::add_balance_to_coldkey_account(&pending.coldkey, pending.cost_paid);
                Self::deposit_event(Event::PendingRegistrationRefunded {
                    coldkey: pending.coldkey,
                    amount: pending.cost_paid,
                });
                return Ok(());
            }

            let result = Self::do_register_network_inner(
                &pending.coldkey,
                &pending.hotkey,
                pending.mechid,
                freed_netuid,
                pending.cost_paid,
            );

            match result {
                Ok(()) => {
                    Self::deposit_event(Event::RegistrationCompleted {
                        coldkey: pending.coldkey,
                        hotkey: pending.hotkey,
                        netuid: freed_netuid,
                    });
                }
                Err(e) => {
                    log::error!(
                        "Pending registration failed for {:?}: {:?}",
                        pending.coldkey,
                        e
                    );
                    Self::add_balance_to_coldkey_account(&pending.coldkey, pending.cost_paid);
                    Self::deposit_event(Event::PendingRegistrationRefunded {
                        coldkey: pending.coldkey,
                        amount: pending.cost_paid,
                    });
                }
            }
        }

        Ok(())
    }
}
