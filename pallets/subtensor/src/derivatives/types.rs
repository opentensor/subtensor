use codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use substrate_fixed::types::I64F64;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};

/// A merged covered short position for one `(coldkey, netuid)` (spec §2.2).
///
/// `C`, `N`, `λ_eff`, and `ϕ` are open-time derivations and are deliberately not
/// persisted. `r/e/b_stored` are the values at the last materialization; current
/// values are recovered by multiplying by `exp(-(Ω_S − omega_entry))`.
#[freeze_struct("43ae40d25be019c8")]
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortPosition<AccountId> {
    /// Hotkey the liability alpha is repaid from on close.
    pub hotkey: AccountId,
    /// Non-decaying TAO floor supplied by the trader (spec `P`).
    pub p_floor: TaoBalance,
    /// Fixed alpha liability (spec `Q`); changes only on close/default/dereg.
    pub q_liability: AlphaBalance,
    /// Retained-proceeds buffer at last materialization (spec `R`).
    pub r_stored: TaoBalance,
    /// Linked TAO escrow at last materialization (spec `E`).
    pub e_stored: TaoBalance,
    /// Utilization footprint at last materialization (spec `B = λC`).
    pub b_stored: TaoBalance,
    /// Value of `Ω_S` at last materialization (spec `Ω_entry`).
    pub omega_entry: I64F64,
    /// Block of the last owner action (open / merge / top-up). Permissionless
    /// default is gated to `last_active + grace`, so an owner always has a
    /// window to top up before a third party can default them.
    pub last_active: u64,
}

/// Per-subnet short-side aggregate and decay accumulator (spec §2.4, §6.3).
#[freeze_struct("376a8ccf882d6dea")]
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortAgg {
    /// Σ current retained buffer.
    pub r_sigma: TaoBalance,
    /// Σ current escrow.
    pub e_sigma: TaoBalance,
    /// Σ current footprint == active utilization `S`.
    pub b_sigma: TaoBalance,
    /// Σ fixed alpha liability (open interest `Q_Σ`).
    pub q_sigma: AlphaBalance,
    /// Cumulative monotone decay accumulator `Ω_S` (`Ω ← Ω − ln g`).
    pub omega: I64F64,
}

impl ShortAgg {
    /// Empty short-side aggregate.
    pub fn zero() -> Self {
        Self {
            r_sigma: TaoBalance::ZERO,
            e_sigma: TaoBalance::ZERO,
            b_sigma: TaoBalance::ZERO,
            q_sigma: AlphaBalance::ZERO,
            omega: I64F64::from_num(0),
        }
    }
}

/// Pre-open trader quote (spec §1.2). Pure derivation, no state change.
#[freeze_struct("54beac46977b1ec5")]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortOpenQuote {
    /// Gross open-time collateral `C = P + N`.
    pub gross_collateral: TaoBalance,
    /// Retained proceeds `N` (becomes the initial buffer `R0`).
    pub retained_proceeds: TaoBalance,
    /// Fixed alpha liability `Q`.
    pub alpha_liability: AlphaBalance,
    /// Linked TAO escrow `E`.
    pub escrow: TaoBalance,
    /// Effective LTV `λ_eff`, scaled by 1e9.
    pub effective_ltv: u64,
    /// Current daily decay/carry rate, scaled by 1e9.
    pub daily_decay: u64,
    /// Estimated TAO cost to repay `Q` at the current pool (slippage-aware).
    pub est_close_cost: TaoBalance,
}

/// Live, materialized view of a trader's short position (decayed to the current
/// block) plus the health metrics a client needs to manage it.
#[freeze_struct("9f6810752569e314")]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortPositionInfo<AccountId> {
    pub netuid: NetUid,
    pub hotkey: AccountId,
    /// Non-decaying floor `P`.
    pub floor: TaoBalance,
    /// Fixed alpha liability `Q`.
    pub alpha_liability: AlphaBalance,
    /// Current retained buffer `R(t)` after decay.
    pub buffer: TaoBalance,
    /// Current linked escrow `E(t)` after decay.
    pub escrow: TaoBalance,
    /// Current TAO collateral claim `C = P + R(t)`.
    pub collateral_claim: TaoBalance,
    /// Current daily carry/decay rate, scaled by 1e9.
    pub daily_decay: u64,
    /// Estimated blocks until `R` decays to dust at the current rate
    /// (`u64::MAX` if decay is effectively zero).
    pub blocks_to_dust: u64,
    /// Whether the position can be defaulted right now.
    pub default_eligible: bool,
    /// Earliest block a third party could default once dusted (`last_active + grace`).
    pub defaultable_at_block: u64,
    /// Slippage-aware TAO cost to repay the full liability `Q` now.
    pub est_close_cost: TaoBalance,
    /// Alpha already staked at the position hotkey (counts toward `Q`).
    pub alpha_held: AlphaBalance,
    /// Incremental alpha still to acquire before a full close (spec §1.6).
    pub alpha_needed: AlphaBalance,
}

/// Per-subnet short market state for sizing and capacity decisions.
#[freeze_struct("b87648108ccb15")]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortMarketInfo {
    pub shorts_enabled: bool,
    /// Base LTV `λ`, scaled by 1e9.
    pub base_ltv: u64,
    /// Footprint-cap factor `κ_S`, scaled by 1e9.
    pub kappa: u64,
    /// Daily decay bounds, scaled by 1e9.
    pub decay_min: u64,
    pub decay_max: u64,
    /// Current daily decay at the live utilization, scaled by 1e9.
    pub current_daily_decay: u64,
    /// Conservative TAO reference `T_ref`.
    pub t_ref: TaoBalance,
    /// Active footprint `S` (used capacity).
    pub footprint_used: TaoBalance,
    /// Footprint cap `κ_S · T_ref`.
    pub footprint_cap: TaoBalance,
    /// Remaining openable footprint.
    pub footprint_remaining: TaoBalance,
    /// Aggregate fixed alpha liability (open interest).
    pub open_interest_alpha: AlphaBalance,
    /// Aggregate retained buffer and escrow.
    pub buffer_total: TaoBalance,
    pub escrow_total: TaoBalance,
    /// Dust threshold, minimum input, and default grace.
    pub dust_threshold: TaoBalance,
    pub min_input: TaoBalance,
    pub default_grace: u64,
}

/// Pre-close quote for a fraction of a position (spec §1.5–1.6).
#[freeze_struct("e5828d301fddd1a1")]
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct CloseShortQuote {
    /// Alpha that must be repaid for this close fraction.
    pub repay_alpha: AlphaBalance,
    /// TAO returned to the trader (floor + buffer fraction).
    pub returned_tao: TaoBalance,
    /// Escrow settled back into the pool.
    pub escrow_settled: TaoBalance,
    /// Slippage-aware TAO cost to acquire `repay_alpha` now.
    pub est_buyback_cost: TaoBalance,
    /// Alpha already held toward the repayment.
    pub alpha_held: AlphaBalance,
    /// Incremental alpha still to acquire (`max(0, repay_alpha − held)`).
    pub alpha_needed: AlphaBalance,
}
