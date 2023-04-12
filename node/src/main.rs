//! Substrate Node Subtensor CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

fn main() -> sc_cli::Result<()> {
	command::run()
}
