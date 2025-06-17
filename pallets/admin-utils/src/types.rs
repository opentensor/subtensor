
use crate::{AuthorityList, Config};
use frame_support::pallet_prelude::{Decode, Encode};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::NetUid;

/// Enum for specifying the type of precompile operation.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Copy)]
pub enum PrecompileEnum {
    /// Enum for balance transfer precompile
    BalanceTransfer,
    /// Enum for staking precompile
    Staking,
    /// Enum for subnet precompile
    Subnet,
    /// Enum for metagraph precompile
    Metagraph,
    /// Enum for neuron precompile
    Neuron,
    /// Enum for UID lookup precompile
    UidLookup,
    /// Enum for alpha precompile
    Alpha,
}

/// Unified payload for `sudo_set_hyperparameter`.
///
/// * **Tuple-like variants** change *network-wide* parameters
/// * **Struct-like variants** target a *specific subnet* or require
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum HyperParam<T: Config> {
    /*─────────── NETWORK-WIDE ───────────*/
    /// Default delegate-take applied (in basis-points) when a delegator
    /// first bonds to a hotkey.
    DefaultTake(u16),

    /// Global, per-second extrinsic rate-limit
    /// (`tx · s⁻¹ × 10³` to preserve precision).
    TxRateLimit(u64),

    /// Chain-wide operations limit (applied per block).
    NetworkRateLimit(u64),

    /// Manually override the total TAO issuance in circulation.
    TotalIssuance(u64),

    /// Default immunity period (in blocks) given to all future subnets.
    NetworkImmunityPeriod(u64),

    /// Minimum TAO that must be locked to spin-up a subnet.
    NetworkMinLockCost(u64),

    /// Hard cap on the number of subnets that can exist at once.
    SubnetLimit(u16),

    /// Interval (in blocks) at which locked TAO is linearly released.
    LockReductionInterval(u64),

    /// Minimum self-stake a miner must hold before it can emit weights.
    StakeThreshold(u64),

    /// Minimum stake a nominator must supply to back a hotkey.
    NominatorMinRequiredStake(u64),

    /// Cool-down (in blocks) between successive `delegate_take` calls.
    TxDelegateTakeRateLimit(u64),

    /// Network-wide lower bound for delegate-take (basis-points).
    MinDelegateTake(u16),

    /// EVM chain-ID reported by the precompile layer.
    EvmChainId(u64),

    /// Share of subnet emissions reserved for the subnet owner (b.p.).
    SubnetOwnerCut(u16),

    /// Exponential moving-average α used for KPI smoothing.
    SubnetMovingAlpha(I96F32),

    /// Duration (in blocks) of the coldkey-swap unbonding schedule.
    ColdkeySwapScheduleDuration(BlockNumberFor<T>),

    /// Duration (in blocks) of the dissolve-network schedule.
    DissolveNetworkScheduleDuration(BlockNumberFor<T>),

    /*──────────── CONSENSUS ────────────*/
    /// Schedule a GRANDPA authority-set change.
    ScheduleGrandpaChange {
        /// New weighted authority list.
        next_authorities: AuthorityList,
        /// Delay (in blocks) before the change becomes active.
        in_blocks: BlockNumberFor<T>,
        /// Median last-finalised block for a *forced* replacement.
        forced: Option<BlockNumberFor<T>>,
    },

    /*──────── EVM PRECOMPILE GATE ───────*/
    /// Enable or disable a specific EVM precompile.
    ToggleEvmPrecompile {
        /// Target precompile identifier.
        precompile_id: PrecompileEnum,
        /// `true` to enable, `false` to disable.
        enabled: bool,
    },

    /*────────── SUBNET-SPECIFIC ─────────*/
    /// Set the serving-rate limit (queries · block⁻¹).
    ServingRateLimit {
        /// Subnet identifier.
        netuid: NetUid,
        /// Maximum queries allowed per block.
        limit: u64,
    },

    /// Minimum PoW difficulty accepted during registration.
    MinDifficulty {
        /// Subnet identifier.
        netuid: NetUid,
        /// New minimum difficulty.
        value: u64,
    },

    /// Maximum PoW difficulty enforced during registration.
    MaxDifficulty {
        /// Subnet identifier.
        netuid: NetUid,
        /// New maximum difficulty.
        value: u64,
    },

    /// Bump the weights-version key to invalidate all cached weights.
    WeightsVersionKey {
        /// Subnet identifier.
        netuid: NetUid,
        /// New version key.
        key: u64,
    },

    /// Cool-down (blocks) between weight-set extrinsics.
    WeightsSetRateLimit {
        /// Subnet identifier.
        netuid: NetUid,
        /// Minimum blocks between calls.
        limit: u64,
    },

    /// Epoch count between automatic difficulty adjustments.
    AdjustmentInterval {
        /// Subnet identifier.
        netuid: NetUid,
        /// Interval in epochs.
        interval: u16,
    },

    /// α parameter for exponential difficulty adjustment.
    AdjustmentAlpha {
        /// Subnet identifier.
        netuid: NetUid,
        /// Alpha value (fixed-point, scaled ×10⁶).
        alpha: u64,
    },

    /// Hard cap on non-zero weights per submission.
    MaxWeightLimit {
        /// Subnet identifier.
        netuid: NetUid,
        /// Maximum allowed weights.
        limit: u16,
    },

    /// Immunity period (epochs) before pruning can occur.
    ImmunityPeriod {
        /// Subnet identifier.
        netuid: NetUid,
        /// Period length.
        period: u16,
    },

    /// Minimum number of non-zero weights required in a submission.
    MinAllowedWeights {
        /// Subnet identifier.
        netuid: NetUid,
        /// Minimum weights.
        min: u16,
    },

    /// Maximum UID capacity of the subnet.
    MaxAllowedUids {
        /// Subnet identifier.
        netuid: NetUid,
        /// New UID cap.
        max: u16,
    },

    /// κ (kappa) parameter for incentive-decay curve.
    Kappa {
        /// Subnet identifier.
        netuid: NetUid,
        /// Kappa value.
        kappa: u16,
    },

    /// ρ (rho) parameter for incentive-decay curve.
    Rho {
        /// Subnet identifier.
        netuid: NetUid,
        /// Rho value.
        rho: u16,
    },

    /// Block-age threshold after which miners are considered inactive.
    ActivityCutoff {
        /// Subnet identifier.
        netuid: NetUid,
        /// Cut-off (blocks).
        cutoff: u16,
    },

    /// Toggle extrinsic-based (signature) registration.
    NetworkRegistrationAllowed {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether registration is allowed.
        allowed: bool,
    },

    /// Toggle PoW-based registration.
    NetworkPowRegistrationAllowed {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether registration is allowed.
        allowed: bool,
    },

    /// Target number of registrations per difficulty interval.
    TargetRegistrationsPerInterval {
        /// Subnet identifier.
        netuid: NetUid,
        /// Target registrations.
        target: u16,
    },

    /// Minimum TAO to burn during registration.
    MinBurn {
        /// Subnet identifier.
        netuid: NetUid,
        /// Minimum burn.
        min: u64,
    },

    /// Maximum TAO that can be burned.
    MaxBurn {
        /// Subnet identifier.
        netuid: NetUid,
        /// Maximum burn.
        max: u64,
    },

    /// Directly set subnet PoW difficulty.
    Difficulty {
        /// Subnet identifier.
        netuid: NetUid,
        /// New difficulty.
        value: u64,
    },

    /// Cap on validator seats in the subnet.
    MaxAllowedValidators {
        /// Subnet identifier.
        netuid: NetUid,
        /// Seat limit.
        max: u16,
    },

    /// Window size (blocks) for bonds moving-average.
    BondsMovingAverage {
        /// Subnet identifier.
        netuid: NetUid,
        /// Window length.
        ma: u64,
    },

    /// Penalty factor (%) applied to stale bonds.
    BondsPenalty {
        /// Subnet identifier.
        netuid: NetUid,
        /// Penalty percentage.
        penalty: u16,
    },

    /// Hard cap on registrations per block.
    MaxRegistrationsPerBlock {
        /// Subnet identifier.
        netuid: NetUid,
        /// Maximum per block.
        max: u16,
    },

    /// Epoch length (tempo) of the subnet, in blocks.
    Tempo {
        /// Subnet identifier.
        netuid: NetUid,
        /// Tempo (blocks).
        tempo: u16,
    },

    /// TAO recycled back into rewards pool.
    RaoRecycled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Recycled amount.
        recycled: u64,
    },

    /// Enable or disable commit-reveal weights scheme.
    CommitRevealWeightsEnabled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether the feature is enabled.
        enabled: bool,
    },

    /// Enable or disable Liquid Alpha staking.
    LiquidAlphaEnabled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether Liquid Alpha is enabled.
        enabled: bool,
    },

    /// Lower and upper α bounds for Liquid Alpha sigmoid.
    AlphaValues {
        /// Subnet identifier.
        netuid: NetUid,
        /// Lower bound.
        low: u16,
        /// Upper bound.
        high: u16,
    },

    /// Maximum stake (RAO) a miner can bond.
    NetworkMaxStake {
        /// Subnet identifier.
        netuid: NetUid,
        /// Stake cap (RAO).
        max_stake: u64,
    },

    /// Reveal period (in blocks) for committed weights.
    CommitRevealWeightsInterval {
        /// Subnet identifier.
        netuid: NetUid,
        /// Reveal window length.
        interval: u64,
    },

    /// Enable or block α transfers between miners.
    ToggleTransfer {
        /// Subnet identifier.
        netuid: NetUid,
        /// Transfer enabled?
        enabled: bool,
    },

    /// Force-set the owner hotkey (emergency only).
    SubnetOwnerHotkey {
        /// Subnet identifier.
        netuid: NetUid,
        /// New owner hotkey account.
        hotkey: T::AccountId,
    },

    /// Same as `SubnetOwnerHotkey` but rate-limited.
    SNOwnerHotkey {
        /// Subnet identifier.
        netuid: NetUid,
        /// New owner hotkey account.
        hotkey: T::AccountId,
    },

    /// EMA price-halving period (blocks).
    EMAPriceHalvingPeriod {
        /// Subnet identifier.
        netuid: NetUid,
        /// Halving period.
        period: u64,
    },

    /// Steepness of the α-sigmoid emission curve.
    AlphaSigmoidSteepness {
        /// Subnet identifier.
        netuid: NetUid,
        /// Sigmoid steepness.
        steepness: u16,
    },

    /// Enable or disable Yuma3 staking.
    Yuma3Enabled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether Yuma3 is enabled.
        enabled: bool,
    },

    /// Enable or disable automatic bonds reset.
    BondsResetEnabled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether reset is enabled.
        enabled: bool,
    },

    /// Enable or disable the internal sub-token marketplace.
    SubtokenEnabled {
        /// Subnet identifier.
        netuid: NetUid,
        /// Whether trading is enabled.
        enabled: bool,
    },
}
