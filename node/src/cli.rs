use crate::{
    client::{FullBackend, FullClient},
    consensus::{AuraConsensus, BabeConsensus},
    ethereum::{EthConfiguration, FrontierBackend},
    service::new_chain_ops,
};
use node_subtensor_runtime::opaque::Block;
use sc_cli::RunCmd;
use sc_consensus::BasicQueue;
use sc_service::{Configuration, TaskManager};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,

    /// Choose sealing method.
    #[arg(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    /// Whether to try Aura or Babe consensus on first start.
    ///
    /// After starting, the consensus used by the node will automatically
    /// switch to whatever is required to continue validating / syncing.
    ///
    /// TODO: Remove this after the Babe transition has settled.
    #[arg(long, value_enum, ignore_case = true, default_value_t=SupportedConsensusMechanism::default())]
    pub initial_consensus: SupportedConsensusMechanism,

    #[command(flatten)]
    pub eth: EthConfiguration,

    /// Skip creating historical gap-backfill during initial/catch-up sync.
    ///
    /// This reduces sync time/disk usage but historical block data may be incomplete.
    #[arg(long, default_value_t = false)]
    pub skip_history_backfill: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    // Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    // Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    // Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    // Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    // Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    // Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    // Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    // Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    // Sub-commands concerned with benchmarking.
    #[cfg(feature = "runtime-benchmarks")]
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    // Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),

    // Build a patched test clone chainspec from synced network state.
    #[command(name = "build-test-clone")]
    CloneState(CloneStateCmd),
}

/// Build a patched clone chainspec by syncing state, exporting raw state, and applying test patch.
#[derive(Debug, Clone, clap::Args)]
pub struct CloneStateCmd {
    /// Chain spec identifier or path (same semantics as `--chain`).
    #[arg(long, value_name = "CHAIN")]
    pub chain: String,

    /// Base path used for syncing and state export.
    #[arg(long, value_name = "PATH")]
    pub base_path: PathBuf,

    /// Output file path for the final patched chainspec JSON.
    #[arg(long, value_name = "FILE")]
    pub output: PathBuf,

    /// Sync mode for the temporary sync node.
    #[arg(long, value_enum, default_value_t = CloneSyncMode::Warp)]
    pub sync: CloneSyncMode,

    /// Database backend for the temporary sync/export node.
    #[arg(long, value_enum, default_value_t = CloneDatabase::ParityDb)]
    pub database: CloneDatabase,

    /// Whether to keep or skip history backfill after state sync.
    #[arg(long, value_enum, default_value_t = CloneHistoryBackfill::Skip)]
    pub history_backfill: CloneHistoryBackfill,

    /// RPC port used by the temporary sync node.
    #[arg(long, default_value_t = 9966)]
    pub rpc_port: u16,

    /// P2P port used by the temporary sync node.
    #[arg(long, default_value_t = 30466)]
    pub port: u16,

    /// Maximum time to wait for sync completion.
    #[arg(long, default_value_t = 7200)]
    pub sync_timeout_sec: u64,

    /// Accept sync completion when current is within this many blocks of highest.
    #[arg(long, default_value_t = 8)]
    pub sync_lag_blocks: u64,

    /// Optional bootnodes for the sync step. Repeatable.
    #[arg(long, value_name = "BOOTNODE")]
    pub bootnodes: Vec<String>,

    /// Include Alice in patched validator authorities.
    #[arg(long, default_value_t = false)]
    pub alice: bool,

    /// Include Bob in patched validator authorities.
    #[arg(long, default_value_t = false)]
    pub bob: bool,

    /// Include Charlie in patched validator authorities.
    #[arg(long, default_value_t = false)]
    pub charlie: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CloneSyncMode {
    Warp,
    Full,
}

impl AsRef<str> for CloneSyncMode {
    fn as_ref(&self) -> &str {
        match self {
            CloneSyncMode::Warp => "warp",
            CloneSyncMode::Full => "full",
        }
    }
}

impl fmt::Display for CloneSyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CloneDatabase {
    #[value(name = "auto")]
    Auto,
    #[value(name = "rocksdb")]
    RocksDb,
    #[value(name = "paritydb")]
    ParityDb,
}

impl AsRef<str> for CloneDatabase {
    fn as_ref(&self) -> &str {
        match self {
            CloneDatabase::Auto => "auto",
            CloneDatabase::RocksDb => "rocksdb",
            CloneDatabase::ParityDb => "paritydb",
        }
    }
}

impl fmt::Display for CloneDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum CloneHistoryBackfill {
    Keep,
    #[default]
    Skip,
}

impl AsRef<str> for CloneHistoryBackfill {
    fn as_ref(&self) -> &str {
        match self {
            CloneHistoryBackfill::Keep => "keep",
            CloneHistoryBackfill::Skip => "skip",
        }
    }
}

impl fmt::Display for CloneHistoryBackfill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// Available Sealing methods.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum Sealing {
    /// Seal using rpc method.
    #[default]
    Manual,
    /// Seal when transaction is executed.
    Instant,
}

/// Supported consensus mechanisms.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum SupportedConsensusMechanism {
    // Babe
    Babe,
    /// Aura
    #[default]
    Aura,
}

// Convinience methods for static dispatch of different service methods with
// different consensus mechanisms.
impl SupportedConsensusMechanism {
    pub fn new_chain_ops(
        &self,
        config: &mut Configuration,
        eth_config: &EthConfiguration,
    ) -> Result<
        (
            Arc<FullClient>,
            Arc<FullBackend>,
            BasicQueue<Block>,
            TaskManager,
            FrontierBackend,
        ),
        sc_service::Error,
    > {
        match self {
            SupportedConsensusMechanism::Aura => new_chain_ops::<AuraConsensus>(config, eth_config),
            SupportedConsensusMechanism::Babe => new_chain_ops::<BabeConsensus>(config, eth_config),
        }
    }
}
