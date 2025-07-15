use crate::ethereum::EthConfiguration;
use sc_cli::RunCmd;

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
    #[arg(long, value_enum, ignore_case = true)]
    pub initial_consensus: Option<InitialConsensus>,

    #[command(flatten)]
    pub eth: EthConfiguration,
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

/// Avaliable initial consensus types.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum InitialConsensus {
    /// Babe
    Babe,
    /// Aura
    #[default]
    Aura,
}
