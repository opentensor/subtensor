//! Substrate Node Subtensor CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod aura_babe_import_queue;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod cli;
mod client;
mod command;
mod conditional_evm_block_import;
mod ethereum;
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
