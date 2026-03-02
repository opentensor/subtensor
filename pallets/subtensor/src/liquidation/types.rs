use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::weights::Weight;
use frame_support::{BoundedVec, pallet_prelude::RuntimeDebug};
use scale_info::TypeInfo;
use sp_core::ConstU32;

/// Map index for `ClearNeuronData` phase. Converts from stored `u8`.
/// Out-of-range values map to `VectorStorage` (the final cleanup step).
pub enum NeuronDataMap {
    BlockAtRegistration,
    Axons,
    NeuronCertificates,
    Prometheus,
    AlphaDividendsPerSubnet,
    PendingChildKeys,
    AssociatedEvmAddress,
    Uids,
    Keys,
    LastHotkeySwapOnNetuid,
    VectorStorage,
}

impl From<u8> for NeuronDataMap {
    fn from(idx: u8) -> Self {
        match idx {
            0 => Self::BlockAtRegistration,
            1 => Self::Axons,
            2 => Self::NeuronCertificates,
            3 => Self::Prometheus,
            4 => Self::AlphaDividendsPerSubnet,
            5 => Self::PendingChildKeys,
            6 => Self::AssociatedEvmAddress,
            7 => Self::Uids,
            8 => Self::Keys,
            9 => Self::LastHotkeySwapOnNetuid,
            _ => Self::VectorStorage,
        }
    }
}

/// Map index for `ClearMatrices` phase (per mechanism). Converts from stored `u8`.
/// Out-of-range values map to `NextMechanism`.
pub enum MatrixMap {
    WeightCommits,
    TimelockedWeightCommits,
    CRV3WeightCommits,
    CRV3WeightCommitsV2,
    Bonds,
    Weights,
    NextMechanism,
}

impl MatrixMap {
    pub const LAST_IDX: u8 = 5;
}

impl From<u8> for MatrixMap {
    fn from(idx: u8) -> Self {
        match idx {
            0 => Self::WeightCommits,
            1 => Self::TimelockedWeightCommits,
            2 => Self::CRV3WeightCommits,
            3 => Self::CRV3WeightCommitsV2,
            4 => Self::Bonds,
            5 => Self::Weights,
            _ => Self::NextMechanism,
        }
    }
}

/// Map index for `ClearTwoKeyMaps` phase. Converts from stored `u8`.
/// Out-of-range values map to `Done`.
pub enum TwoKeyMap {
    ChildkeyTake,
    ChildKeys,
    ParentKeys,
    LastHotkeyEmissionOnNetuid,
    TotalHotkeyAlphaLastEpoch,
    IsNetworkMember,
    HotkeyAlphaAndShares,
    LeasesAndIdentity,
    Done,
}

impl TwoKeyMap {
    pub const LAST_IDX: u8 = 7;
}

impl From<u8> for TwoKeyMap {
    fn from(idx: u8) -> Self {
        match idx {
            0 => Self::ChildkeyTake,
            1 => Self::ChildKeys,
            2 => Self::ParentKeys,
            3 => Self::LastHotkeyEmissionOnNetuid,
            4 => Self::TotalHotkeyAlphaLastEpoch,
            5 => Self::IsNetworkMember,
            6 => Self::HotkeyAlphaAndShares,
            7 => Self::LeasesAndIdentity,
            _ => Self::Done,
        }
    }
}

/// Cursor for resumable iteration.
/// Alpha storage keys are 130 bytes: twox_128(pallet) + twox_128(storage) = 32,
/// Blake2_128Concat(AccountId) × 2 = 96, Identity(NetUid) = 2. Total = 130.
/// Use 256 to accommodate this with safety margin for other StorageNMap cursors.
pub type CursorBytes = BoundedVec<u8, ConstU32<256>>;

/// Result of processing a single chunk of work within a liquidation phase.
pub enum ChunkResult {
    /// Phase completed all its work.
    Complete(Weight),
    /// Phase needs more blocks. Contains the updated phase to resume from.
    Incomplete {
        weight_used: Weight,
        phase: LiquidationPhase,
    },
}

impl ChunkResult {
    pub fn weight_used(&self) -> Weight {
        match self {
            Self::Complete(w) | Self::Incomplete { weight_used: w, .. } => *w,
        }
    }
}

/// The phase of a subnet liquidation process.
/// Each variant tracks any cursor/state needed to resume that phase across blocks.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LiquidationPhase {
    Freeze,
    SnapshotStakers {
        cursor: Option<CursorBytes>,
    },
    ClearHyperparams,
    ClearNeuronData {
        map_idx: u8,
        cursor: Option<CursorBytes>,
    },
    ClearRootWeights {
        uid_cursor: u16,
    },
    FinalizeRootDividends {
        cursor: Option<CursorBytes>,
    },
    DistributeAlpha {
        cursor_idx: u32,
    },
    DissolveUserLPs {
        cursor: Option<u32>,
    },
    ClearProtocolLPs,
    ClearMatrices {
        mechanism_idx: u8,
        map_idx: u8,
        cursor: Option<CursorBytes>,
    },
    ClearTwoKeyMaps {
        map_idx: u8,
        cursor: Option<CursorBytes>,
    },
    FinalCleanup,
}

impl LiquidationPhase {
    /// Get a simple tag for event emission (no cursor data).
    pub fn tag(&self) -> LiquidationPhaseTag {
        match self {
            Self::Freeze => LiquidationPhaseTag::Freeze,
            Self::SnapshotStakers { .. } => LiquidationPhaseTag::SnapshotStakers,
            Self::ClearHyperparams => LiquidationPhaseTag::ClearHyperparams,
            Self::ClearNeuronData { .. } => LiquidationPhaseTag::ClearNeuronData,
            Self::ClearRootWeights { .. } => LiquidationPhaseTag::ClearRootWeights,
            Self::FinalizeRootDividends { .. } => LiquidationPhaseTag::FinalizeRootDividends,
            Self::DistributeAlpha { .. } => LiquidationPhaseTag::DistributeAlpha,
            Self::DissolveUserLPs { .. } => LiquidationPhaseTag::DissolveUserLPs,
            Self::ClearProtocolLPs => LiquidationPhaseTag::ClearProtocolLPs,
            Self::ClearMatrices { .. } => LiquidationPhaseTag::ClearMatrices,
            Self::ClearTwoKeyMaps { .. } => LiquidationPhaseTag::ClearTwoKeyMaps,
            Self::FinalCleanup => LiquidationPhaseTag::FinalCleanup,
        }
    }

    /// Get the next phase. Delegates to `LiquidationPhaseTag::next_phase`.
    pub fn next_phase(&self) -> Option<LiquidationPhase> {
        self.tag().next_phase()
    }
}

/// Simple tag enum for event emission — no cursor data.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LiquidationPhaseTag {
    Freeze,
    SnapshotStakers,
    ClearHyperparams,
    ClearNeuronData,
    ClearRootWeights,
    FinalizeRootDividends,
    DistributeAlpha,
    DissolveUserLPs,
    ClearProtocolLPs,
    ClearMatrices,
    ClearTwoKeyMaps,
    FinalCleanup,
}

impl LiquidationPhaseTag {
    /// Get the next phase with default (initial) cursor values.
    ///
    /// Phase order: Freeze → DissolveUserLPs → ClearProtocolLPs → SnapshotStakers →
    /// ClearHyperparams → ClearNeuronData → ClearRootWeights → FinalizeRootDividends →
    /// DistributeAlpha → ClearMatrices → ClearTwoKeyMaps → FinalCleanup
    ///
    /// LP dissolution runs BEFORE snapshot so that LP-derived alpha is captured
    /// in the snapshot and included in the TAO distribution.
    pub fn next_phase(&self) -> Option<LiquidationPhase> {
        match self {
            Self::Freeze => Some(LiquidationPhase::DissolveUserLPs { cursor: None }),
            Self::DissolveUserLPs => Some(LiquidationPhase::ClearProtocolLPs),
            Self::ClearProtocolLPs => Some(LiquidationPhase::SnapshotStakers { cursor: None }),
            Self::SnapshotStakers => Some(LiquidationPhase::ClearHyperparams),
            Self::ClearHyperparams => Some(LiquidationPhase::ClearNeuronData {
                map_idx: 0,
                cursor: None,
            }),
            Self::ClearNeuronData => Some(LiquidationPhase::ClearRootWeights { uid_cursor: 0 }),
            Self::ClearRootWeights => {
                Some(LiquidationPhase::FinalizeRootDividends { cursor: None })
            }
            Self::FinalizeRootDividends => {
                Some(LiquidationPhase::DistributeAlpha { cursor_idx: 0 })
            }
            Self::DistributeAlpha => Some(LiquidationPhase::ClearMatrices {
                mechanism_idx: 0,
                map_idx: 0,
                cursor: None,
            }),
            Self::ClearMatrices => Some(LiquidationPhase::ClearTwoKeyMaps {
                map_idx: 0,
                cursor: None,
            }),
            Self::ClearTwoKeyMaps => Some(LiquidationPhase::FinalCleanup),
            Self::FinalCleanup => None,
        }
    }
}

/// Full liquidation state stored per subnet.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct LiquidationState<BlockNumber> {
    /// Block when liquidation started
    pub started_at: BlockNumber,
    /// Maximum block by which liquidation must complete
    pub max_completion_block: BlockNumber,
    /// Current phase
    pub phase: LiquidationPhase,
    /// Pre-calculated weight budget per block
    pub weight_per_block: Weight,
    /// Snapshot of work to do (for progress tracking)
    pub total_stakers: u32,
    pub total_neurons: u16,
    pub mechanism_count: u8,
    /// TAO pot snapshot at freeze time
    pub tao_pot: u64,
    /// Total alpha value — accumulated during SnapshotStakers phase.
    /// Equals exactly sum(alpha_for_dist) across all snapshot entries.
    /// Used as denominator in calculate_share().
    pub total_alpha_value: u128,
    /// Number of stakers in snapshot. Single source of truth.
    /// Updated during SnapshotStakers; read during DistributeAlpha.
    pub snapshot_count: u32,
    /// Cumulative TAO distributed (for invariant checking)
    pub tao_distributed: u64,
}

/// Warnings emitted during liquidation (non-fatal).
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LiquidationWarning {
    /// Cursor exceeded maximum size, skipped to next phase
    CursorOverflow,
    /// Weight budget exceeded in single operation
    WeightBudgetExceeded,
    /// Dust amount burned (rounding remainder)
    DistributionDust(u64),
    /// Phase skipped due to error
    PhaseSkipped(LiquidationPhaseTag),
    /// Emergency finalization burned undistributed TAO
    EmergencyBurn(u64),
    /// User LP dissolution failed
    LpDissolutionFailed,
    /// Protocol LP clearing failed
    ProtocolLpClearFailed,
}

/// Pending registration that will auto-complete when liquidation finishes.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct PendingRegistration<AccountId> {
    pub coldkey: AccountId,
    pub hotkey: AccountId,
    pub mechid: u16,
    /// Lock/burn cost paid upfront (for refund on emergency failure)
    pub cost_paid: u64,
}
