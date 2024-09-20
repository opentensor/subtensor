use crate::{
    chain_spec,
    cli::{Cli, Subcommand},
    service,
};

#[cfg(feature = "runtime-benchmarks")]
pub use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder};
#[cfg(feature = "runtime-benchmarks")]
pub use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
#[cfg(feature = "runtime-benchmarks")]
pub use node_subtensor_runtime::EXISTENTIAL_DEPOSIT;
#[cfg(feature = "runtime-benchmarks")]
pub use sp_keyring::Sr25519Keyring;

use node_subtensor_runtime::Block;
use sc_cli::SubstrateCli;
use sc_service::{
    config::{ExecutorConfiguration, RpcConfiguration},
    Configuration, PartialComponents,
};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Subtensor Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "local" => Box::new(chain_spec::localnet::localnet_config()?),
            "finney" => Box::new(chain_spec::finney::finney_mainnet_config()?),
            "" | "test_finney" => Box::new(chain_spec::testnet::finney_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config)?;
                let aux_revert = Box::new(|client, _, blocks| {
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err(
                                "Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
                                    .into(),
                            );
                        }

                        cmd.run_with_spec::<sp_runtime::traits::HashingFor<Block>, ()>(Some(
                            config.chain_spec,
                        ))
                    }
                    BenchmarkCmd::Block(cmd) => {
                        let PartialComponents { client, .. } = service::new_partial(&config)?;
                        cmd.run(client)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Storage(_) => Err(
                        "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                            .into(),
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Storage(cmd) => {
                        let PartialComponents {
                            client, backend, ..
                        } = service::new_partial(&config)?;
                        let db = backend.expose_db();
                        let storage = backend.expose_storage();

                        cmd.run(config, client, db, storage)
                    }
                    BenchmarkCmd::Overhead(cmd) => {
                        let PartialComponents { client, .. } = service::new_partial(&config)?;
                        let ext_builder = RemarkBuilder::new(client.clone());

                        cmd.run(
                            config,
                            client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                        )
                    }
                    BenchmarkCmd::Extrinsic(cmd) => {
                        let PartialComponents { client, .. } = service::new_partial(&config)?;
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::new(client.clone())),
                            Box::new(TransferKeepAliveBuilder::new(
                                client.clone(),
                                Sr25519Keyring::Alice.to_account_id(),
                                EXISTENTIAL_DEPOSIT,
                            )),
                        ]);

                        cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
                    }
                    BenchmarkCmd::Machine(cmd) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                }
            })
        }
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                let config = override_default_heap_pages(config, 60_000);
                match config.network.network_backend {
					sc_network::config::NetworkBackendType::Libp2p => service::new_full::<
						sc_network::NetworkWorker<
							node_subtensor_runtime::opaque::Block,
							<node_subtensor_runtime::opaque::Block as sp_runtime::traits::Block>::Hash,
						>,
					>(config)
					.map_err(sc_cli::Error::Service),
					sc_network::config::NetworkBackendType::Litep2p =>
						service::new_full::<sc_network::Litep2pNetworkBackend>(config)
							.map_err(sc_cli::Error::Service),
				}
            })
        }
    }
}

/// Override default heap pages
fn override_default_heap_pages(config: Configuration, pages: u64) -> Configuration {
    Configuration {
        impl_name: config.impl_name,
        impl_version: config.impl_version,
        role: config.role,
        tokio_handle: config.tokio_handle,
        transaction_pool: config.transaction_pool,
        network: config.network,
        keystore: config.keystore,
        database: config.database,
        trie_cache_maximum_size: config.trie_cache_maximum_size,
        state_pruning: config.state_pruning,
        blocks_pruning: config.blocks_pruning,
        chain_spec: config.chain_spec,
        wasm_runtime_overrides: config.wasm_runtime_overrides,
        prometheus_config: config.prometheus_config,
        telemetry_endpoints: config.telemetry_endpoints,
        offchain_worker: config.offchain_worker,
        force_authoring: config.force_authoring,
        disable_grandpa: config.disable_grandpa,
        dev_key_seed: config.dev_key_seed,
        tracing_targets: config.tracing_targets,
        tracing_receiver: config.tracing_receiver,
        announce_block: config.announce_block,
        data_path: config.data_path,
        base_path: config.base_path,
        executor: ExecutorConfiguration {
            default_heap_pages: Some(pages),
            wasm_method: config.executor.wasm_method,
            max_runtime_instances: config.executor.max_runtime_instances,
            runtime_cache_size: config.executor.runtime_cache_size,
        },
        rpc: RpcConfiguration {
            addr: config.rpc.addr,
            max_connections: config.rpc.max_connections,
            cors: config.rpc.cors,
            methods: config.rpc.methods,
            max_request_size: config.rpc.max_request_size,
            max_response_size: config.rpc.max_response_size,
            id_provider: config.rpc.id_provider,
            max_subs_per_conn: config.rpc.max_subs_per_conn,
            port: config.rpc.port,
            message_buffer_capacity: config.rpc.message_buffer_capacity,
            batch_config: config.rpc.batch_config,
            rate_limit: config.rpc.rate_limit,
            rate_limit_whitelisted_ips: config.rpc.rate_limit_whitelisted_ips,
            rate_limit_trust_proxy_headers: config.rpc.rate_limit_trust_proxy_headers,
        },
    }
}
