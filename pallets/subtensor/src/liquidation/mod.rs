pub mod types;

pub mod cleanup;
pub mod emergency;
pub mod freeze;
pub mod processor;
pub mod snapshot;

/// Weight constants (placeholders — must be determined via benchmarking)
pub const WEIGHT_PER_SNAPSHOT: u64 = 50_000;
pub const WEIGHT_PER_DISTRIBUTION: u64 = 100_000;
pub const WEIGHT_PER_MATRIX_ENTRY: u64 = 10_000;
pub const WEIGHT_PER_NEURON_CLEAR: u64 = 200_000;
pub const WEIGHT_PER_HYPERPARAM: u64 = 10_000;
pub const MIN_LIQUIDATION_WEIGHT: u64 = 1_000_000;
pub const FIXED_OVERHEAD: u64 = 500_000;
pub const MIN_SNAPSHOT_ALPHA: u64 = 1_000;
pub const NETUID_COOLDOWN_BLOCKS: u64 = 100;
pub const MIN_PHASE_WEIGHT: u64 = 500_000;
/// When this fraction of tempo has elapsed, double the per-block weight budget.
pub const BUDGET_DOUBLE_THRESHOLD_NUMER: u64 = 9;
pub const BUDGET_DOUBLE_THRESHOLD_DENOM: u64 = 10;
pub const MAX_LIQUIDATION_BLOCKS: u64 = 7_200;
pub const MIN_LIQUIDATION_BLOCKS: u64 = 10;
pub const STAKERS_PER_NEURON_ESTIMATE: u32 = 10;
/// Number of hyperparameter storage items cleared in `clear_hyperparams`.
pub const HYPERPARAM_COUNT: u64 = 66;
/// Number of two-key maps cleared per neuron (TwoKeyMap variants, excluding Done).
pub const TWO_KEY_MAP_COUNT: u64 = (types::TwoKeyMap::LAST_IDX as u64) + 1;
