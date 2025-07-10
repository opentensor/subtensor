//! Substrate Node Subtensor CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod babe_service;
mod aura_rpc;
mod aura_service;
mod aura_wrapped_import_queue;
mod babe_rpc;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod cli;
mod client;
mod command;
mod conditional_evm_block_import;
mod ethereum;

fn main() -> sc_cli::Result<()> {
    command::run()
}
