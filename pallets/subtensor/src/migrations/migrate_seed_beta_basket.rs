use super::*;
use frame_support::pallet_prelude::Weight;
use scale_info::prelude::string::String;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid};

/// Seeds the beta-basket escrow model from pre-existing legacy `RootClaimable` state.
///
/// Before this feature, a validator's root dividends accrued as a per-subnet *rate*
/// (`RootClaimable[hotkey][netuid]`, alpha-per-root-stake) backed by unattributed
/// outstanding alpha in `SubnetAlphaOut`. The beta basket instead backs each slot with a
/// real escrow stake position `(hotkey, escrow, netuid)` and an outstanding-principal
/// counter `BasketPrincipal`, paying out `owed * (escrow_value / principal)`.
///
/// If legacy slots were left unseeded, two problems arise:
/// 1. Claims compute `payout = owed * E/P` with `P = 0` → payout `0` → legacy dividends strand.
/// 2. If a legacy slot later receives new accrual, the shared rate mixes legacy + new while
///    `E/P` only tracks the new portion, breaking the `SubnetAlphaOut` ↔ stake invariant.
///
/// This migration converts every legacy slot to the escrow model with `E = P = remaining`,
/// where `remaining = rate * total_root_stake - Σ already-claimed`. It stakes that remaining
/// (previously unattributed) outstanding alpha to the validator under the escrow coldkey and
/// records it as basket principal, leaving the rate and per-coldkey `RootClaimed` watermarks
/// intact so existing per-staker owed amounts pay out unchanged (`E/P = 1`), then compound.
///
/// NOTE: this scans `RootClaimed` per `(netuid, hotkey)` to total already-claimed amounts.
/// On a large state this is heavy; if it cannot fit a single block it should be converted to a
/// multi-block migration before mainnet deployment.
pub fn migrate_seed_beta_basket<T: Config>() -> Weight {
    let migration_name = b"migrate_seed_beta_basket".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    let escrow = Pallet::<T>::get_beta_escrow_account_id();
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    let hotkeys: Vec<T::AccountId> = RootClaimable::<T>::iter_keys().collect();
    weight.saturating_accrue(T::DbWeight::get().reads(hotkeys.len() as u64));

    let mut seeded_slots: u64 = 0;

    for hotkey in hotkeys.iter() {
        let total_root: I96F32 = I96F32::saturating_from_num(
            Pallet::<T>::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT),
        );
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        if total_root <= I96F32::saturating_from_num(0) {
            continue;
        }

        let claimable = RootClaimable::<T>::get(hotkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        for (netuid, rate) in claimable.iter() {
            if netuid.is_root() {
                continue;
            }

            // Gross credited principal = rate * total_root_stake.
            let gross: I96F32 = rate.saturating_mul(total_root);

            // Total already claimed by all coldkeys on this (netuid, hotkey).
            let mut claimed_sum: I96F32 = I96F32::saturating_from_num(0);
            for (_coldkey, claimed) in RootClaimed::<T>::iter_prefix((*netuid, hotkey)) {
                claimed_sum = claimed_sum.saturating_add(I96F32::saturating_from_num(claimed));
                weight.saturating_accrue(T::DbWeight::get().reads(1));
            }

            // Remaining unclaimed (still-outstanding) principal.
            let remaining_f: I96F32 = gross.saturating_sub(claimed_sum);
            let remaining: u64 = if remaining_f.is_negative() {
                0
            } else {
                remaining_f.saturating_to_num::<u64>()
            };
            if remaining == 0 {
                continue;
            }
            let remaining_alpha = AlphaBalance::from(remaining);

            // Attribute the previously-unattributed outstanding alpha to the validator under the
            // escrow coldkey (this becomes the basket), and record it as basket principal.
            Pallet::<T>::increase_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                &escrow,
                *netuid,
                remaining_alpha,
            );
            BasketPrincipal::<T>::insert(hotkey, *netuid, remaining_alpha);
            weight.saturating_accrue(T::DbWeight::get().writes(2));
            seeded_slots = seeded_slots.saturating_add(1);
        }
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    log::info!("Migration 'migrate_seed_beta_basket' completed. Seeded {seeded_slots} slots.");

    weight
}
