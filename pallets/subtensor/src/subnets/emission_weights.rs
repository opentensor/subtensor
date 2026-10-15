//! Per-subnet emission weight vector ("etao").
//!
//! Every subnet carries a five-slot vector of `u64` parts out of
//! [`EMISSION_WEIGHT_ONE`] that sums to exactly one and says how each block's
//! alpha emission is divided:
//!
//! | slot | role      | v1 rule                                   |
//! |-----:|-----------|-------------------------------------------|
//! | 0    | burn      | must be 0 (not yet enabled)               |
//! | 1    | recycle   | must be 0 (not yet enabled)               |
//! | 2    | owner     | `0 ≤ owner ≤ 33%`; leased subnets `≥ 18%` |
//! | 3    | validator | `V₀·TaoWeight ≤ validator ≤ 50%`          |
//! | 4    | miner     | `≥ 33%`                                   |
//!
//! The validator slot also drives the subnet's TAO weight. Lowering the
//! validator share from the subnet's base share `V₀` to `V` multiplies the
//! global `TaoWeight` by `V₀ / V` on that subnet, so the alpha routed to root
//! holders (and TAO's weight in consensus) is not reduced by shrinking the
//! validator cut. The multiplier is written once per call and never
//! recomputed per block.
use super::*;
use frame_support::{BoundedVec, traits::ConstU32};
use safe_math::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::NetUid;

/// Number of slots in the emission weight vector.
pub const EMISSION_WEIGHT_SLOTS: u32 = 5;

/// Denominator of every emission weight: weights are `u64` parts out of `u64::MAX`.
pub const EMISSION_WEIGHT_ONE: u64 = u64::MAX;

/// Storage shape of a subnet's emission weight vector.
pub type EmissionWeightsVec = BoundedVec<u64, ConstU32<EMISSION_WEIGHT_SLOTS>>;

/// In-memory shape of a subnet's emission weight vector.
pub type EmissionWeights = [u64; EMISSION_WEIGHT_SLOTS as usize];

/// `num / den` expressed in parts of [`EMISSION_WEIGHT_ONE`].
pub const fn emission_fraction(num: u64, den: u64) -> u64 {
    match (EMISSION_WEIGHT_ONE as u128)
        .saturating_mul(num as u128)
        .checked_div(den as u128)
    {
        Some(parts) => parts as u64,
        None => 0,
    }
}

/// Owner slot may be at most 33% of emission.
pub const MAX_OWNER_EMISSION: u64 = emission_fraction(33, 100);
/// Validator slot may be at most 50% of emission.
pub const MAX_VALIDATOR_EMISSION: u64 = emission_fraction(50, 100);
/// Miner slot must be at least 33% of emission.
pub const MIN_MINER_EMISSION: u64 = emission_fraction(33, 100);

/// Fixed-point scale of the TAO-weight multiplier in RPC and EVM views
/// (`1.0` is reported as `1_000_000_000`).
pub const TAO_WEIGHT_MULTIPLIER_SCALE: u64 = 1_000_000_000;

/// Distinct storage items [`Pallet::read_subnet_emission_weights_view`] reads:
/// `SubnetEmissionWeights`, `BaseValidatorEmissionShare`, `TaoWeight`,
/// `SubnetTaoWeightMultiplier` and `SubnetCreatedViaLease`.
pub const EMISSION_WEIGHTS_VIEW_READS: u64 = 5;

/// Global `TaoWeight` (u64 parts) as a fraction in `[0, 1]`.
pub fn normalize_tao_weight(parts: u64) -> U96F32 {
    U96F32::saturating_from_num(parts).safe_div(U96F32::saturating_from_num(u64::MAX))
}

/// `V₀ · TaoWeight` in u64 parts, never below one part (see
/// [`Pallet::min_validator_emission_for_tao_weight`]).
pub fn min_validator_emission_from(base_validator: u64, tao_weight: u64) -> u64 {
    let floor = (base_validator as u128)
        .saturating_mul(tao_weight as u128)
        .safe_div(EMISSION_WEIGHT_ONE as u128) as u64;
    floor.max(1)
}

/// A TAO-weight multiplier scaled by [`TAO_WEIGHT_MULTIPLIER_SCALE`].
pub fn scale_tao_weight_multiplier(multiplier: U96F32) -> u64 {
    multiplier
        .saturating_mul(U96F32::saturating_from_num(TAO_WEIGHT_MULTIPLIER_SCALE))
        .saturating_to_num::<u64>()
}

/// `global · multiplier`, saturating at one.
pub fn effective_tao_weight(global: U96F32, multiplier: U96F32) -> U96F32 {
    let one = U96F32::saturating_from_num(1);
    global.saturating_mul(multiplier).min(one)
}

/// A TAO weight in `[0, 1]` as u64 parts of [`EMISSION_WEIGHT_ONE`].
pub fn tao_weight_to_parts(weight: U96F32) -> u64 {
    weight
        .saturating_mul(U96F32::saturating_from_num(EMISSION_WEIGHT_ONE))
        .saturating_to_num::<u64>()
}

/// Snapshot of the five storage items behind a subnet's emission weight
/// view. Derived fields are computed from the snapshot, so a caller that has
/// paid for [`EMISSION_WEIGHTS_VIEW_READS`] reads owes nothing more.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmissionWeightsView {
    /// `[burn, recycle, owner, validator, miner]`.
    pub weights: EmissionWeights,
    /// Base validator share `V₀`.
    pub base_validator: u64,
    /// Global `TaoWeight` in u64 parts.
    pub tao_weight: u64,
    /// Current multiplier `V₀ / V`.
    pub multiplier: U96F32,
    /// Whether the subnet was registered through a lease.
    pub created_via_lease: bool,
}

impl EmissionWeightsView {
    /// Lowest validator share currently allowed on this subnet.
    pub fn min_validator(&self) -> u64 {
        min_validator_emission_from(self.base_validator, self.tao_weight)
    }

    /// Multiplier scaled by [`TAO_WEIGHT_MULTIPLIER_SCALE`].
    pub fn multiplier_scaled(&self) -> u64 {
        scale_tao_weight_multiplier(self.multiplier)
    }

    /// Effective TAO weight on this subnet in u64 parts.
    pub fn effective_tao_weight_parts(&self) -> u64 {
        tao_weight_to_parts(effective_tao_weight(
            normalize_tao_weight(self.tao_weight),
            self.multiplier,
        ))
    }
}

/// Named slot of the emission weight vector.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u16)]
pub enum EmissionIndex {
    /// Alpha destroyed outright. Reserved: must be zero in v1.
    Burn = 0,
    /// Alpha burned and un-minted from issuance. Reserved: must be zero in v1.
    Recycle = 1,
    /// Subnet owner cut.
    Owner = 2,
    /// Validator dividends (root alpha is carved out of this slot).
    Validator = 3,
    /// Miner incentive (split across mechanisms by `MechanismEmissionSplit`).
    Miner = 4,
}

impl EmissionIndex {
    /// Every slot, in vector order.
    pub const ALL: [EmissionIndex; EMISSION_WEIGHT_SLOTS as usize] = [
        EmissionIndex::Burn,
        EmissionIndex::Recycle,
        EmissionIndex::Owner,
        EmissionIndex::Validator,
        EmissionIndex::Miner,
    ];

    /// Decode a wire index; `None` for reserved / unknown slots.
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Burn),
            1 => Some(Self::Recycle),
            2 => Some(Self::Owner),
            3 => Some(Self::Validator),
            4 => Some(Self::Miner),
            _ => None,
        }
    }

    /// Position of this slot in the vector.
    pub const fn position(self) -> usize {
        self as u16 as usize
    }
}

impl<T: Config> Pallet<T> {
    // ---------------------------------------------------------------------
    // Defaults
    // ---------------------------------------------------------------------

    /// Scale a legacy `u16 / u16::MAX` fraction to `u64 / u64::MAX` parts.
    pub fn scale_u16_fraction_to_emission_weight(value: u16) -> u64 {
        (value as u128)
            .saturating_mul(EMISSION_WEIGHT_ONE as u128)
            .safe_div(u16::MAX as u128) as u64
    }

    /// Canonical owner share for subnets that never set a vector: the legacy
    /// `InitialSubnetOwnerCut` (18% on mainnet) scaled to u64 parts.
    pub fn default_owner_emission() -> u64 {
        Self::scale_u16_fraction_to_emission_weight(T::InitialSubnetOwnerCut::get())
    }

    /// Canonical validator share: half of what remains after the owner cut
    /// (41% on mainnet), matching the pre-etao 50/50 split.
    pub fn default_validator_emission() -> u64 {
        EMISSION_WEIGHT_ONE.saturating_sub(Self::default_owner_emission()) / 2
    }

    /// Lowest owner share a lease-created subnet may set: the canonical
    /// default, so lease dividends can never be squeezed below launch terms.
    pub fn leased_subnet_min_owner_emission() -> u64 {
        Self::default_owner_emission()
    }

    /// Build the canonical vector `[0, 0, owner, validator, miner]`; the miner
    /// slot absorbs any rounding so the vector sums to exactly one.
    pub fn default_emission_weights() -> EmissionWeights {
        let owner = Self::default_owner_emission();
        let validator = Self::default_validator_emission();
        let miner = EMISSION_WEIGHT_ONE
            .saturating_sub(owner)
            .saturating_sub(validator);
        [0, 0, owner, validator, miner]
    }

    /// `[0, 0, owner, validator, miner]` with the non-owner remainder split
    /// evenly between validators and miners (miner absorbs rounding).
    pub fn emission_weights_with_owner(owner: u64) -> EmissionWeights {
        let validator = EMISSION_WEIGHT_ONE.saturating_sub(owner) / 2;
        let miner = EMISSION_WEIGHT_ONE
            .saturating_sub(owner)
            .saturating_sub(validator);
        [0, 0, owner, validator, miner]
    }

    pub(crate) fn emission_weights_to_vec(weights: EmissionWeights) -> EmissionWeightsVec {
        EmissionWeightsVec::truncate_from(weights.to_vec())
    }

    // ---------------------------------------------------------------------
    // Reads
    // ---------------------------------------------------------------------

    /// The subnet's full emission weight vector.
    pub fn get_subnet_emission_weights(netuid: NetUid) -> EmissionWeights {
        let stored = SubnetEmissionWeights::<T>::get(netuid);
        let mut weights = [0u64; EMISSION_WEIGHT_SLOTS as usize];
        for (slot, value) in weights.iter_mut().zip(stored.iter()) {
            *slot = *value;
        }
        weights
    }

    /// One slot of the subnet's emission weight vector.
    pub fn get_subnet_emission_weight(netuid: NetUid, index: EmissionIndex) -> u64 {
        Self::get_subnet_emission_weights(netuid)
            .get(index.position())
            .copied()
            .unwrap_or_default()
    }

    /// Burn share (slot 0). Always zero in v1.
    pub fn get_subnet_burn_emission(netuid: NetUid) -> u64 {
        Self::get_subnet_emission_weight(netuid, EmissionIndex::Burn)
    }

    /// Recycle share (slot 1). Always zero in v1.
    pub fn get_subnet_recycle_emission(netuid: NetUid) -> u64 {
        Self::get_subnet_emission_weight(netuid, EmissionIndex::Recycle)
    }

    /// Owner cut (slot 2) in u64 parts.
    pub fn get_subnet_owner_emission_cut(netuid: NetUid) -> u64 {
        Self::get_subnet_emission_weight(netuid, EmissionIndex::Owner)
    }

    /// Validator share (slot 3) in u64 parts.
    pub fn get_subnet_validator_emission_cut(netuid: NetUid) -> u64 {
        Self::get_subnet_emission_weight(netuid, EmissionIndex::Validator)
    }

    /// Miner share (slot 4) in u64 parts.
    pub fn get_subnet_miner_emission_cut(netuid: NetUid) -> u64 {
        Self::get_subnet_emission_weight(netuid, EmissionIndex::Miner)
    }

    /// Convert u64 parts to a `[0, 1]` fixed-point fraction.
    pub fn emission_weight_to_fixed(weight: u64) -> U96F32 {
        U96F32::saturating_from_num(weight)
            .safe_div(U96F32::saturating_from_num(EMISSION_WEIGHT_ONE))
    }

    /// Owner cut as a `[0, 1]` fraction.
    pub fn get_float_subnet_owner_emission_cut(netuid: NetUid) -> U96F32 {
        Self::emission_weight_to_fixed(Self::get_subnet_owner_emission_cut(netuid))
    }

    /// Validator share as a `[0, 1]` fraction.
    pub fn get_float_subnet_validator_emission_cut(netuid: NetUid) -> U96F32 {
        Self::emission_weight_to_fixed(Self::get_subnet_validator_emission_cut(netuid))
    }

    /// Miner share as a `[0, 1]` fraction.
    pub fn get_float_subnet_miner_emission_cut(netuid: NetUid) -> U96F32 {
        Self::emission_weight_to_fixed(Self::get_subnet_miner_emission_cut(netuid))
    }

    /// Whether the owner receives any emission cut (legacy `OwnerCutEnabled`
    /// view: true iff the owner slot is non-zero).
    pub fn get_owner_cut_enabled(netuid: NetUid) -> bool {
        Self::get_subnet_owner_emission_cut(netuid) > 0
    }

    /// Base validator share `V₀` the TAO-weight multiplier is measured against.
    pub fn get_base_validator_emission_share(netuid: NetUid) -> u64 {
        BaseValidatorEmissionShare::<T>::get(netuid)
    }

    /// Current TAO-weight multiplier `V₀ / V` for the subnet.
    pub fn get_subnet_tao_weight_multiplier(netuid: NetUid) -> U96F32 {
        SubnetTaoWeightMultiplier::<T>::get(netuid)
    }

    /// [`Self::get_subnet_tao_weight_multiplier`] scaled by
    /// [`TAO_WEIGHT_MULTIPLIER_SCALE`] for RPC and EVM views. The multiplier
    /// can exceed one (up to `1 / TaoWeight`), so it cannot share the
    /// `u64::MAX` scale the weights use.
    pub fn get_subnet_tao_weight_multiplier_scaled(netuid: NetUid) -> u64 {
        scale_tao_weight_multiplier(Self::get_subnet_tao_weight_multiplier(netuid))
    }

    /// `V₀ / V`; `V = 0` is unrepresentable and yields `MAX`.
    pub fn compute_tao_weight_multiplier(base_validator: u64, validator: u64) -> U96F32 {
        if validator == 0 {
            return U96F32::max_value();
        }
        U96F32::saturating_from_num(base_validator).safe_div(U96F32::saturating_from_num(validator))
    }

    /// Lowest validator share that keeps `TaoWeight · V₀ / V ≤ 1`, i.e.
    /// `V₀ · TaoWeight` in u64 parts. Never below one part so `V = 0` is
    /// always rejected (it would imply an infinite TAO weight).
    pub fn min_validator_emission_for_tao_weight(netuid: NetUid) -> u64 {
        min_validator_emission_from(
            Self::get_base_validator_emission_share(netuid),
            TaoWeight::<T>::get(),
        )
    }

    /// Effective TAO weight on `netuid`: the global weight scaled by the
    /// subnet multiplier, saturating at one. Consensus-side only: it feeds the
    /// root carve-out of the validator slot, dividend and child/parent
    /// contribution splits, and the stake weighting in Yuma Consensus. The
    /// pool side (`tao_in`, `alpha_in`, the injection cap, `RootProp`) uses the
    /// global [`Pallet::get_tao_weight`] and must never read this.
    pub fn get_subnet_tao_weight(netuid: NetUid) -> U96F32 {
        effective_tao_weight(
            Self::get_tao_weight(),
            Self::get_subnet_tao_weight_multiplier(netuid),
        )
    }

    /// [`Self::get_subnet_tao_weight`] in u64 parts, for events and RPC.
    pub fn get_subnet_tao_weight_parts(netuid: NetUid) -> u64 {
        tao_weight_to_parts(Self::get_subnet_tao_weight(netuid))
    }

    /// Whether the subnet was registered through a lease (persists after the
    /// lease ends).
    pub fn get_subnet_created_via_lease(netuid: NetUid) -> bool {
        SubnetCreatedViaLease::<T>::get(netuid)
    }

    /// Every storage item behind the emission weight view, each read exactly
    /// once. Views that meter storage (the EVM precompile) charge
    /// [`EMISSION_WEIGHTS_VIEW_READS`] reads and derive the rest from this.
    pub fn read_subnet_emission_weights_view(netuid: NetUid) -> EmissionWeightsView {
        EmissionWeightsView {
            weights: Self::get_subnet_emission_weights(netuid),
            base_validator: Self::get_base_validator_emission_share(netuid),
            tao_weight: TaoWeight::<T>::get(),
            multiplier: Self::get_subnet_tao_weight_multiplier(netuid),
            created_via_lease: Self::get_subnet_created_via_lease(netuid),
        }
    }

    // ---------------------------------------------------------------------
    // Writes
    // ---------------------------------------------------------------------

    /// Seed a freshly registered subnet with the canonical vector, base
    /// validator share and a unit multiplier, and clear any per-subnet state a
    /// previous occupant of this netuid may have left (lease marker, accrued
    /// owner cut). Lease registration sets the marker after this runs.
    pub fn seed_subnet_emission_weights(netuid: NetUid) {
        if netuid.is_root() {
            return;
        }
        let weights = Self::default_emission_weights();
        SubnetEmissionWeights::<T>::insert(netuid, Self::emission_weights_to_vec(weights));
        BaseValidatorEmissionShare::<T>::insert(netuid, Self::default_validator_emission());
        SubnetTaoWeightMultiplier::<T>::insert(netuid, U96F32::saturating_from_num(1));
        SubnetCreatedViaLease::<T>::remove(netuid);
        SubnetOwnerCutAccrued::<T>::remove(netuid);
    }

    /// Write a vector verbatim: no rule checks, no multiplier update, no
    /// event. Migration and test plumbing only; extrinsics must go through
    /// [`Self::do_set_subnet_emission_weights`].
    pub fn set_subnet_emission_weights_unchecked(netuid: NetUid, weights: EmissionWeights) {
        SubnetEmissionWeights::<T>::insert(netuid, Self::emission_weights_to_vec(weights));
    }

    /// Apply a sparse `(uids, values)` update on top of `current`. Omitted
    /// slots keep their current value; burn and recycle are always treated as
    /// zero when omitted.
    pub fn merge_emission_weight_updates(
        current: EmissionWeights,
        uids: &[u16],
        values: &[u64],
    ) -> Result<EmissionWeights, Error<T>> {
        ensure!(
            uids.len() == values.len(),
            Error::<T>::EmissionWeightsLengthMismatch
        );
        let mut merged = current;
        for reserved in [EmissionIndex::Burn, EmissionIndex::Recycle] {
            if let Some(target) = merged.get_mut(reserved.position()) {
                *target = 0;
            }
        }
        let mut seen = [false; EMISSION_WEIGHT_SLOTS as usize];
        for (uid, value) in uids.iter().zip(values.iter()) {
            let index =
                EmissionIndex::from_u16(*uid).ok_or(Error::<T>::EmissionWeightIndexOutOfRange)?;
            let slot = index.position();
            ensure!(
                !seen.get(slot).copied().unwrap_or(true),
                Error::<T>::EmissionWeightIndexDuplicated
            );
            if let Some(flag) = seen.get_mut(slot) {
                *flag = true;
            }
            if let Some(target) = merged.get_mut(slot) {
                *target = *value;
            }
        }
        Ok(merged)
    }

    /// Check every v1 rule against a full vector. Each failure names the rule
    /// so callers learn the constraints from the error alone.
    pub fn validate_subnet_emission_weights(
        netuid: NetUid,
        weights: &EmissionWeights,
    ) -> DispatchResult {
        let slot = |index: EmissionIndex| weights.get(index.position()).copied().unwrap_or(0);

        ensure!(
            slot(EmissionIndex::Burn) == 0,
            Error::<T>::BurnEmissionMustBeZero
        );
        ensure!(
            slot(EmissionIndex::Recycle) == 0,
            Error::<T>::RecycleEmissionMustBeZero
        );

        let owner = slot(EmissionIndex::Owner);
        ensure!(
            owner <= MAX_OWNER_EMISSION,
            Error::<T>::OwnerEmissionAboveMaximum
        );
        if Self::get_subnet_created_via_lease(netuid) {
            ensure!(
                owner >= Self::leased_subnet_min_owner_emission(),
                Error::<T>::LeasedSubnetOwnerCutBelowMinimum
            );
        }

        let validator = slot(EmissionIndex::Validator);
        ensure!(
            validator <= MAX_VALIDATOR_EMISSION,
            Error::<T>::ValidatorEmissionAboveMaximum
        );
        ensure!(
            validator >= Self::min_validator_emission_for_tao_weight(netuid),
            Error::<T>::ValidatorEmissionBelowTaoWeightFloor
        );

        ensure!(
            slot(EmissionIndex::Miner) >= MIN_MINER_EMISSION,
            Error::<T>::MinerEmissionBelowMinimum
        );

        let total: u128 = weights.iter().map(|w| *w as u128).sum();
        ensure!(
            total == EMISSION_WEIGHT_ONE as u128,
            Error::<T>::EmissionWeightsDoNotSumToOne
        );
        Ok(())
    }

    /// Merge, validate and store a sparse emission weight update, refresh the
    /// TAO-weight multiplier from the (possibly unchanged) validator slot and
    /// emit [`Event::SubnetEmissionWeightsSet`].
    pub fn do_set_subnet_emission_weights(
        netuid: NetUid,
        uids: Vec<u16>,
        values: Vec<u64>,
    ) -> DispatchResult {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            !netuid.is_root(),
            Error::<T>::EmissionWeightsNotSettableOnRoot
        );

        let current = Self::get_subnet_emission_weights(netuid);
        let merged = Self::merge_emission_weight_updates(current, &uids, &values)?;
        Self::validate_subnet_emission_weights(netuid, &merged)?;

        let base_validator = Self::get_base_validator_emission_share(netuid);
        let validator = merged
            .get(EmissionIndex::Validator.position())
            .copied()
            .unwrap_or(0);
        let multiplier = Self::compute_tao_weight_multiplier(base_validator, validator);

        SubnetEmissionWeights::<T>::insert(netuid, Self::emission_weights_to_vec(merged));
        SubnetTaoWeightMultiplier::<T>::insert(netuid, multiplier);

        Self::deposit_event(Event::SubnetEmissionWeightsSet {
            netuid,
            weights: merged.to_vec(),
            tao_weight_multiplier: multiplier,
            effective_tao_weight: Self::get_subnet_tao_weight_parts(netuid),
        });
        Ok(())
    }
}
