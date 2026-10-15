//! Claim-based miner (server) incentive emissions.
//!
//! # Why
//!
//! Epoch distribution previously staked every miner's incentive into the alpha share
//! pool every tempo. That path writes several share-pool keys per earner
//! (`TotalHotkeyAlpha`, share denominator, `AlphaV2` row, optional `StakingHotkeys`,
//! collateral indexes). On Null consensus with thousands of UIDs that dominates
//! epoch weight.
//!
//! Claim-based mode keeps validator dividends on the live stake path, and only
//! **accrues** miner incentive into a per-hotkey pending balance (two writes:
//! the row and a subnet total). Miners (or forced deregister / dissolve payout)
//! later claim into stake.
//!
//! # Why not a global reward-index
//!
//! A lazy per-share reward index works when every member accrues in proportion to
//! a fixed share of one pool. Miner incentives are **not** proportional to stake:
//! each UID gets an arbitrary amount from the weight matrix every epoch. The
//! cheapest correct design is therefore O(1) storage work **per earner**
//! (one pending mutate), not O(1) for the whole subnet.
//!
//! # Fee
//!
//! `claim_miner_emission` charges an alpha fee proportional to the extrinsic's
//! declared weight:
//!
//! ```text
//! fee_rao = weight.ref_time() * MinerClaimAlphaFeePerWeightSecond
//!         / WEIGHT_REF_TIME_PER_SECOND
//! ```
//!
//! Justification: the chain cost of a claim is the claim's weight, not the alpha
//! amount. Pricing fee ∝ weight discourages claiming every epoch on large Null
//! subnets (which would recreate the write load we removed) without punishing
//! large one-shot claims. The fee is **recycled** (SubnetAlphaOut decreases):
//! the alpha was minted into outstanding issuance but never staked, so recycling
//! correctly un-mints it. Burning would leave SubnetAlphaOut inflated relative
//! to live alpha.

use super::*;
use crate::weights::WeightInfo;
use frame_support::weights::Weight;
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND;
use subtensor_runtime_common::{AlphaBalance, NetUid};

impl<T: Config> Pallet<T> {
    /// Whether this subnet accrues miner incentive instead of staking it every epoch.
    pub fn get_claim_based_miner_emissions(netuid: NetUid) -> bool {
        ClaimBasedMinerEmissions::<T>::get(netuid)
    }

    /// Root/sudo toggle. New subnets and the enablement migration default this on.
    pub fn set_claim_based_miner_emissions(netuid: NetUid, enabled: bool) {
        ClaimBasedMinerEmissions::<T>::insert(netuid, enabled);
        Self::deposit_event(Event::ClaimBasedMinerEmissionsSet { netuid, enabled });
    }

    /// Pending (unclaimed) miner incentive for `hotkey` on `netuid`.
    pub fn get_pending_miner_emission(hotkey: &T::AccountId, netuid: NetUid) -> AlphaBalance {
        PendingMinerAlpha::<T>::get(netuid, hotkey)
    }

    /// Subnet-wide sum of pending miner incentive (issuance that is outstanding but
    /// not yet in `TotalHotkeyAlpha`).
    pub fn get_pending_miner_emission_total(netuid: NetUid) -> AlphaBalance {
        PendingMinerAlphaTotal::<T>::get(netuid)
    }

    /// Alpha fee charged for a claim of the given weight envelope.
    pub fn miner_claim_alpha_fee(weight: Weight) -> AlphaBalance {
        let per_second = MinerClaimAlphaFeePerWeightSecond::<T>::get();
        if per_second == 0 || weight.ref_time() == 0 {
            return AlphaBalance::ZERO;
        }
        let fee = (weight.ref_time() as u128)
            .saturating_mul(per_second as u128)
            .checked_div(WEIGHT_REF_TIME_PER_SECOND as u128)
            .unwrap_or(0);
        AlphaBalance::from(fee.min(u128::from(u64::MAX)) as u64)
    }

    /// Accrue `amount` of miner incentive into the pending map (epoch hot path).
    pub fn accrue_pending_miner_alpha(netuid: NetUid, hotkey: &T::AccountId, amount: AlphaBalance) {
        if amount.is_zero() {
            return;
        }
        PendingMinerAlpha::<T>::mutate(netuid, hotkey, |pending| {
            *pending = pending.saturating_add(amount);
        });
        PendingMinerAlphaTotal::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(amount);
        });
    }

    /// Move pending accrual from `old_hotkey` to `new_hotkey` on `netuid` (hotkey swap).
    pub fn transfer_pending_miner_alpha(
        netuid: NetUid,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
    ) {
        let amount = PendingMinerAlpha::<T>::take(netuid, old_hotkey);
        if amount.is_zero() {
            return;
        }
        PendingMinerAlpha::<T>::mutate(netuid, new_hotkey, |pending| {
            *pending = pending.saturating_add(amount);
        });
    }

    /// Stake (or recycle-as-fee) all pending miner incentive for `hotkey` on `netuid`.
    ///
    /// When `charge_fee` is true, an alpha fee proportional to `claim_weight` is
    /// recycled from the pending balance before the remainder is staked. Forced
    /// payouts (UID replace / prune / dissolve) pass `charge_fee = false`.
    ///
    /// Returns `(staked, fee)`.
    pub fn payout_pending_miner_alpha(
        netuid: NetUid,
        hotkey: &T::AccountId,
        charge_fee: bool,
        claim_weight: Weight,
    ) -> (AlphaBalance, AlphaBalance) {
        let pending = PendingMinerAlpha::<T>::take(netuid, hotkey);
        if pending.is_zero() {
            return (AlphaBalance::ZERO, AlphaBalance::ZERO);
        }

        PendingMinerAlphaTotal::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(pending);
        });

        let fee = if charge_fee {
            Self::miner_claim_alpha_fee(claim_weight).min(pending)
        } else {
            AlphaBalance::ZERO
        };
        if !fee.is_zero() {
            Self::recycle_subnet_alpha(netuid, fee);
        }

        let liquid = pending.saturating_sub(fee);
        if liquid.is_zero() {
            if charge_fee {
                Self::deposit_event(Event::MinerIncentiveClaimed {
                    netuid,
                    hotkey: hotkey.clone(),
                    owner: Owner::<T>::get(hotkey),
                    destination: hotkey.clone(),
                    staked: AlphaBalance::ZERO,
                    fee,
                });
            }
            return (AlphaBalance::ZERO, fee);
        }

        let owner: T::AccountId = Owner::<T>::get(hotkey);
        let maybe_dest = AutoStakeDestination::<T>::get(&owner, netuid);
        let destination = maybe_dest.clone().unwrap_or_else(|| hotkey.clone());

        if let Some(dest) = maybe_dest {
            Self::deposit_event(Event::AutoStakeAdded {
                netuid,
                destination: dest,
                hotkey: hotkey.clone(),
                owner: owner.clone(),
                incentive: liquid,
            });
        }

        Self::credit_epoch_owner_emission(&destination, &owner, netuid, liquid);

        Self::deposit_event(Event::MinerIncentiveClaimed {
            netuid,
            hotkey: hotkey.clone(),
            owner,
            destination,
            staked: liquid,
            fee,
        });

        (liquid, fee)
    }

    /// Forced fee-free payout used on UID replace / prune / dissolve.
    pub fn force_payout_pending_miner_alpha(netuid: NetUid, hotkey: &T::AccountId) {
        let _ = Self::payout_pending_miner_alpha(netuid, hotkey, false, Weight::zero());
    }

    /// Signed claim extrinsic body.
    pub fn do_claim_miner_emission(
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            !netuid.is_root(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(
            Self::get_claim_based_miner_emissions(netuid),
            Error::<T>::ClaimBasedMinerEmissionsDisabled
        );
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        let pending = Self::get_pending_miner_emission(&hotkey, netuid);
        ensure!(!pending.is_zero(), Error::<T>::AmountTooLow);

        let weight = <T as Config>::WeightInfo::claim_miner_emission();
        let _ = Self::payout_pending_miner_alpha(netuid, &hotkey, true, weight);
        Ok(())
    }

    /// Worst-case weight for one fee-free dissolve payout of a pending miner row.
    ///
    /// Covers iterator progress, pending take / total mutate, Owner + AutoStake
    /// reads, and a full `credit_epoch_owner_emission` share-pool transition
    /// (including a large `StakingHotkeys` read). Proof-size allowance matches
    /// the measured `add_stake` envelope. Charged *before* each row so cleanup
    /// cannot exceed the remaining block weight meter.
    pub fn dissolve_pending_miner_alpha_row_weight() -> Weight {
        <T as Config>::WeightInfo::claim_miner_emission()
            .saturating_add(T::DbWeight::get().reads_writes(4, 2))
            .saturating_add(Weight::from_parts(0, 8_192))
    }

    /// Resumable dissolve helper: stake every pending row on `netuid` (fee-free)
    /// so later AlphaInOut settlement can convert it to TAO with the rest of stake.
    ///
    /// Always resumes from the front of the remaining prefix. Paid rows are
    /// removed via `PendingMinerAlpha::take`, so progress does not need a hashed
    /// cursor — and a mid-cleanup hotkey swap that rekeys an unpaid row cannot
    /// hide it behind a previous `iter_prefix_from` cursor.
    pub fn dissolve_pending_miner_alpha_to_stake(
        netuid: NetUid,
        weight_meter: &mut WeightMeter,
        _last_key: Option<Vec<u8>>,
    ) -> (bool, Option<Vec<u8>>) {
        let row_weight = Self::dissolve_pending_miner_alpha_row_weight();
        // Final total-map removal after the last row.
        let finish_weight = T::DbWeight::get().writes(1);
        let mut done = true;

        for (hotkey, _amount) in PendingMinerAlpha::<T>::iter_prefix(netuid) {
            if !weight_meter.can_consume(row_weight) {
                done = false;
                break;
            }
            weight_meter.consume(row_weight);
            Self::force_payout_pending_miner_alpha(netuid, &hotkey);
        }

        if done {
            if !weight_meter.can_consume(finish_weight) {
                // Rows are drained but the total removal must wait for a later
                // block; resume with no cursor (prefix restart).
                return (false, None);
            }
            weight_meter.consume(finish_weight);
            PendingMinerAlphaTotal::<T>::remove(netuid);
            return (true, None);
        }
        (false, None)
    }
}
