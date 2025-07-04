//! Substrate Node Subtensor CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod aura_rpc;
mod aura_service;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod cli;
mod client;
mod command;
mod common;
mod conditional_evm_block_import;
mod ethereum;
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
