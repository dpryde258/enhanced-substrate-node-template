use crate::cli::{Cli, Subcommand};
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use service::{chain_spec, IdentifyVariant};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Substrate Node".into()
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
		2022
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			#[cfg(feature = "with-devnet-runtime")]
			"devnet-dev" => Box::new(chain_spec::devnet::development_config()?),
			#[cfg(feature = "with-devnet-runtime")]
			"devnet-local" => Box::new(chain_spec::devnet::local_testnet_config()?),
			#[cfg(feature = "with-devnet-runtime")]
			"devnet-prod-sample" => Box::new(chain_spec::devnet::production_sample_config()?),

			#[cfg(feature = "with-mainnet-runtime")]
			"mainnet-dev" => Box::new(chain_spec::mainnet::development_config()?),
			#[cfg(feature = "with-mainnet-runtime")]
			"mainnet-local" => Box::new(chain_spec::mainnet::local_testnet_config()?),
			#[cfg(feature = "with-mainnet-runtime")]
			"mainnet-prod-sample" => Box::new(chain_spec::mainnet::production_sample_config()?),

			path => {
				let path = std::path::PathBuf::from(path);
				let chain_spec =
					Box::new(service::chain_spec::DummyChainSpec::from_json_file(path.clone())?)
						as Box<dyn sc_service::ChainSpec>;

				if chain_spec.is_mainnet() {
					#[cfg(feature = "with-mainnet-runtime")]
					{
						Box::new(chain_spec::mainnet::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-mainnet-runtime"))]
					return Err(service::MAINNET_RUNTIME_NOT_AVAILABLE.into());
				} else {
					#[cfg(feature = "with-devnet-runtime")]
					{
						Box::new(chain_spec::devnet::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-devnet-runtime"))]
					return Err(service::DEVNET_RUNTIME_NOT_AVAILABLE.into());
				}
			},
		})
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_mainnet() {
			#[cfg(feature = "with-mainnet-runtime")]
			return &service::mainnet_runtime::VERSION;
			#[cfg(not(feature = "with-mainnet-runtime"))]
			panic!("{}", service::MAINNET_RUNTIME_NOT_AVAILABLE);
		} else {
			#[cfg(feature = "with-devnet-runtime")]
			return &service::devnet_runtime::VERSION;
			#[cfg(not(feature = "with-devnet-runtime"))]
			panic!("{}", service::DEVNET_RUNTIME_NOT_AVAILABLE);
		}
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				let chain_spec = &config.chain_spec;

				if chain_spec.is_mainnet() {
					#[cfg(feature = "with-mainnet-runtime")]
					{
						return service::new_full::<
							service::mainnet_runtime::RuntimeApi,
							service::MainnetExecutor,
						>(config)
						.map_err(sc_cli::Error::Service);
					}
					#[cfg(not(feature = "with-mainnet-runtime"))]
					return Err(service::MAINNET_RUNTIME_NOT_AVAILABLE.into());
				} else {
					#[cfg(feature = "with-devnet-runtime")]
					{
						return service::new_full::<
							service::devnet_runtime::RuntimeApi,
							service::DevnetExecutor,
						>(config)
						.map_err(sc_cli::Error::Service);
					}
					#[cfg(not(feature = "with-devnet-runtime"))]
					return Err(service::DEVNET_RUNTIME_NOT_AVAILABLE.into());
				}
			})
		},
		Some(Subcommand::Inspect(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				let chain_spec = &config.chain_spec;

				if chain_spec.is_mainnet() {
					#[cfg(feature = "with-mainnet-runtime")]
					{
						return cmd.run::<service::mainnet_runtime::Block, service::mainnet_runtime::RuntimeApi, service::MainnetExecutor>(
							config,
						);
					}
					#[cfg(not(feature = "with-mainnet-runtime"))]
					return Err(service::MAINNET_RUNTIME_NOT_AVAILABLE.into());
				}else {
					#[cfg(feature = "with-devnet-runtime")]
					{
						return cmd.run::<service::devnet_runtime::Block, service::devnet_runtime::RuntimeApi, service::DevnetExecutor>(
							config,
						);
					}
					#[cfg(not(feature = "with-devnet-runtime"))]
					return Err(service::DEVNET_RUNTIME_NOT_AVAILABLE.into());
				}
			})
		},
		Some(Subcommand::Benchmark(cmd)) =>
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;

				runner.sync_run(|config| {
					let chain_spec = &config.chain_spec;

					if chain_spec.is_mainnet() {
						#[cfg(feature = "with-mainnet-runtime")]
						{
							return cmd
								.run::<service::mainnet_runtime::Block, service::MainnetExecutor>(
									config,
								);
						}
						#[cfg(not(feature = "with-mainnet-runtime"))]
						return Err(service::MAINNET_RUNTIME_NOT_AVAILABLE.into());
					} else {
						#[cfg(feature = "with-devnet-runtime")]
						{
							return cmd
								.run::<service::devnet_runtime::Block, service::DevnetExecutor>(
									config,
								);
						}
						#[cfg(not(feature = "with-devnet-runtime"))]
						return Err(service::DEVNET_RUNTIME_NOT_AVAILABLE.into());
					}
				})
			} else {
				Err("Benchmarking wasn't enabled when building the node. You can enable it with \
				 `--features runtime-benchmarks`."
					.into())
			},
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|config| {
				let chain_spec = &config.chain_spec;

				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

				if chain_spec.is_mainnet() {
					#[cfg(feature = "with-mainnet-runtime")]
					return Ok((
						cmd.run::<service::mainnet_runtime::Block, service::MainnetExecutor>(config),
						task_manager,
					));
					#[cfg(not(feature = "with-mainnet-runtime"))]
					return Err(service::MAINNET_RUNTIME_NOT_AVAILABLE.into());
				} else {
					#[cfg(feature = "with-devnet-runtime")]
					return Ok((
						cmd.run::<service::devnet_runtime::Block, service::DevnetExecutor>(config),
						task_manager,
					));
					#[cfg(not(feature = "with-devnet-runtime"))]
					return Err(service::DEVNET_RUNTIME_NOT_AVAILABLE.into());
				}
			})
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
					You can enable it with `--features try-runtime`."
			.into()),
	}
}
