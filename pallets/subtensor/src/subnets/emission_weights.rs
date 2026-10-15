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
//! | 3    | validator | `≤ 50%`, and must keep root's share       |
//! | 4    | miner     | `≥ 33%`                                   |
//!
//! Root holders are paid out of the validator slot in proportion to their
//! TAO-weighted stake. So that a subnet cannot cut root's income by shrinking
//! the validator slot, every subnet carries its own effective TAO weight
//! `w_s` ([`SubnetTaoWeight`]). It starts equal to the global `TaoWeight`.
//! Whenever the validator share changes, `w_s` is solved **once**, at set
//! time, so that root's share of the subnet's emission is exactly what it was
//! the block before; afterwards it is left alone and everything scales with
//! stake and issuance as usual. The call is rejected when no `w_s ≤ 1` can
//! preserve root's share.
use super::*;
use frame_support::{BoundedVec, traits::ConstU32};
use safe_math::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid};

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

/// Distinct storage items [`Pallet::read_subnet_emission_weights_view`] reads:
/// `SubnetEmissionWeights` and `SubnetCreatedViaLease`.
pub const EMISSION_WEIGHTS_VIEW_READS: u64 = 2;

/// A TAO weight in u64 parts of [`EMISSION_WEIGHT_ONE`] as a fraction in `[0, 1]`.
pub fn normalize_tao_weight(parts: u64) -> U96F32 {
    U96F32::saturating_from_num(parts).safe_div(U96F32::saturating_from_num(u64::MAX))
}

/// A TAO weight in `[0, 1]` as u64 parts of [`EMISSION_WEIGHT_ONE`].
pub fn tao_weight_to_parts(weight: U96F32) -> u64 {
    weight
        .saturating_mul(U96F32::saturating_from_num(EMISSION_WEIGHT_ONE))
        .saturating_to_num::<u64>()
}

/// Root's share of a subnet's emission: `V · x / (x + A)` with
/// `x = root_tao · tao_weight` and `A` the subnet's alpha issuance. This is
/// the shape `consensus_root_proportion` applies to the validator slot each
/// block.
pub fn root_emission_share(
    validator: U96F32,
    tao_weight: U96F32,
    root_tao: U96F32,
    alpha_issuance: U96F32,
) -> U96F32 {
    let weighted_root_tao = root_tao.saturating_mul(tao_weight);
    weighted_root_tao
        .checked_div(weighted_root_tao.saturating_add(alpha_issuance))
        .unwrap_or(U96F32::saturating_from_num(0))
        .saturating_mul(validator)
}

/// Solve the TAO weight that gives root exactly the share `root_share` of
/// the emission once the validator slot is `validator`:
/// `x = r · A / (V − r)`, `w = x / root_tao`. `None` when no `w ≤ 1` exists,
/// i.e. `V ≤ r` or root TAO would have to count more than 1:1.
pub fn solve_tao_weight_for_root_share(
    root_share: U96F32,
    validator: U96F32,
    root_tao: U96F32,
    alpha_issuance: U96F32,
) -> Option<U96F32> {
    if validator <= root_share {
        return None;
    }
    let headroom = validator.saturating_sub(root_share);
    let weighted_root_tao = root_share
        .saturating_mul(alpha_issuance)
        .checked_div(headroom)?;
    let tao_weight = weighted_root_tao.checked_div(root_tao)?;
    (tao_weight <= U96F32::saturating_from_num(1)).then_some(tao_weight)
}

/// Snapshot of the storage items behind a subnet's emission weight view, each
/// read once. A caller that has paid for [`EMISSION_WEIGHTS_VIEW_READS`]
/// reads owes nothing more.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmissionWeightsView {
    /// `[burn, recycle, owner, validator, miner]`.
    pub weights: EmissionWeights,
    /// Whether the subnet was registered through a lease.
    pub created_via_lease: bool,
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

    /// Effective TAO weight `w_s` on `netuid` as a `[0, 1]` fraction. Consensus-side
    /// only: it feeds the root carve-out of the validator slot, dividend and
    /// child/parent contribution splits, and the stake weighting in Yuma
    /// Consensus. The pool side (`tao_in`, `alpha_in`, the injection cap,
    /// `RootProp`) uses the global [`Pallet::get_tao_weight`] and must never
    /// read this. A subnet that has never changed its validator share reads
    /// the global `TaoWeight`.
    pub fn get_subnet_tao_weight(netuid: NetUid) -> U96F32 {
        normalize_tao_weight(Self::get_subnet_tao_weight_parts(netuid))
    }

    /// [`Self::get_subnet_tao_weight`] in u64 parts, for RPC. Falls back to the
    /// global `TaoWeight` while the subnet has no frozen weight of its own.
    pub fn get_subnet_tao_weight_parts(netuid: NetUid) -> u64 {
        SubnetTaoWeight::<T>::get(netuid).unwrap_or_else(TaoWeight::<T>::get)
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
            created_via_lease: Self::get_subnet_created_via_lease(netuid),
        }
    }

    /// Root's current share of `netuid`'s emission under its stored state:
    /// `V · x / (x + A)`, see [`root_emission_share`].
    pub fn get_root_emission_share(netuid: NetUid) -> U96F32 {
        root_emission_share(
            Self::get_float_subnet_validator_emission_cut(netuid),
            Self::get_subnet_tao_weight(netuid),
            U96F32::saturating_from_num(Self::get_subnet_tao(NetUid::ROOT)),
            U96F32::saturating_from_num(Self::get_alpha_issuance(netuid)),
        )
    }

    // ---------------------------------------------------------------------
    // Writes
    // ---------------------------------------------------------------------

    /// Seed a freshly registered subnet with the canonical vector and reset any
    /// per-subnet state a previous occupant of this netuid may have left
    /// (effective TAO weight, lease marker, accrued owner cut). With no
    /// `SubnetTaoWeight` entry the subnet follows the global `TaoWeight`. The
    /// accrued owner cut is written as an explicit zero so that an *absent*
    /// entry always means "pre-dates the seed migration" (see
    /// [`Self::owner_cut_alpha_for_lock_refund`]). Lease registration sets the
    /// marker after this runs.
    pub fn seed_subnet_emission_weights(netuid: NetUid) {
        if netuid.is_root() {
            return;
        }
        let weights = Self::default_emission_weights();
        SubnetEmissionWeights::<T>::insert(netuid, Self::emission_weights_to_vec(weights));
        SubnetTaoWeight::<T>::remove(netuid);
        SubnetCreatedViaLease::<T>::remove(netuid);
        SubnetOwnerCutAccrued::<T>::insert(netuid, AlphaBalance::ZERO);
    }

    /// The owner α the pre-emission-weights lock refund deducted:
    /// `alpha_issuance × legacy_owner_cut / u16::MAX`, floored. `legacy_owner_cut`
    /// is the frozen global `SubnetOwnerCut` (its setter is deprecated).
    pub fn legacy_owner_cut_accrued(netuid: NetUid, legacy_owner_cut: u16) -> AlphaBalance {
        let issuance: u64 = Self::get_alpha_issuance(netuid).to_u64();
        let fraction = U96F32::saturating_from_num(legacy_owner_cut)
            .safe_div(U96F32::saturating_from_num(u16::MAX));
        U96F32::saturating_from_num(issuance)
            .saturating_mul(fraction)
            .floor()
            .saturating_to_num::<u64>()
            .into()
    }

    /// Owner α to deduct from a legacy lock refund. The accrued history
    /// (`SubnetOwnerCutAccrued`) when the subnet has an entry; otherwise the
    /// pre-upgrade formula. Every subnet registered or migrated after the seed
    /// migration has an entry (possibly zero), so the fallback only fires for a
    /// subnet that was already dissolving when the migration ran and was not
    /// seeded. It can never make the deduction silently zero.
    pub fn owner_cut_alpha_for_lock_refund(netuid: NetUid) -> AlphaBalance {
        match SubnetOwnerCutAccrued::<T>::try_get(netuid) {
            Ok(accrued) => accrued,
            Err(()) => Self::legacy_owner_cut_accrued(netuid, Self::get_subnet_owner_cut()),
        }
    }

    /// Write a vector verbatim: no rule checks, no TAO-weight solve, no
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

    /// The effective TAO weight `netuid` must store so that root's share of
    /// emission is unchanged when the validator slot moves from
    /// `current_validator` to `new_validator`, using live chain values at this
    /// block. `Ok(None)` means "keep the stored weight": the validator share
    /// is unchanged, or there is nothing to preserve (no root TAO, no alpha
    /// issuance, or a validator slot that was zero). Fails with
    /// [`Error::ValidatorEmissionTooLowToPreserveRoot`] when `V_new ≤ r` or the
    /// solved weight would exceed one (root TAO never counts more than 1:1).
    pub fn tao_weight_preserving_root(
        netuid: NetUid,
        current_validator: u64,
        new_validator: u64,
    ) -> Result<Option<u64>, Error<T>> {
        if new_validator == current_validator {
            return Ok(None);
        }
        let root_tao = U96F32::saturating_from_num(Self::get_subnet_tao(NetUid::ROOT));
        let alpha_issuance = U96F32::saturating_from_num(Self::get_alpha_issuance(netuid));
        let zero = U96F32::saturating_from_num(0);
        if root_tao == zero || alpha_issuance == zero || current_validator == 0 {
            return Ok(None);
        }
        let root_share = root_emission_share(
            Self::emission_weight_to_fixed(current_validator),
            Self::get_subnet_tao_weight(netuid),
            root_tao,
            alpha_issuance,
        );
        let tao_weight = solve_tao_weight_for_root_share(
            root_share,
            Self::emission_weight_to_fixed(new_validator),
            root_tao,
            alpha_issuance,
        )
        .ok_or(Error::<T>::ValidatorEmissionTooLowToPreserveRoot)?;
        Ok(Some(tao_weight_to_parts(tao_weight)))
    }

    /// Merge, validate and store a sparse emission weight update, re-solve the
    /// subnet's effective TAO weight when the validator slot changed, and emit
    /// [`Event::SubnetEmissionWeightsSet`].
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

        let validator_slot = |weights: &EmissionWeights| {
            weights
                .get(EmissionIndex::Validator.position())
                .copied()
                .unwrap_or(0)
        };
        let tao_weight = Self::tao_weight_preserving_root(
            netuid,
            validator_slot(&current),
            validator_slot(&merged),
        )?;

        SubnetEmissionWeights::<T>::insert(netuid, Self::emission_weights_to_vec(merged));
        if let Some(parts) = tao_weight {
            SubnetTaoWeight::<T>::insert(netuid, parts);
        }

        Self::deposit_event(Event::SubnetEmissionWeightsSet {
            netuid,
            weights: merged.to_vec(),
        });
        Ok(())
    }
}
