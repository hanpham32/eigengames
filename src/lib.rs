use anyhow::Error;
use blueprint_sdk::alloy::primitives::{address, Address};
use blueprint_sdk::alloy::rpc::types::Log;
use blueprint_sdk::alloy::sol;
use blueprint_sdk::config::GadgetConfiguration;
use blueprint_sdk::event_listeners::evm::EvmContractEventListener;
use blueprint_sdk::job;
use blueprint_sdk::macros::load_abi;
use blueprint_sdk::std::sync::LazyLock;
use serde::{Deserialize, Serialize};

pub mod actix_server;
pub mod manager;
pub mod runner;
pub mod types;

type ProcessorError =
    blueprint_sdk::event_listeners::core::Error<blueprint_sdk::event_listeners::evm::error::Error>;

//// Generate Rust bindings for your smart contracts
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    TangleTaskManager,
    "contracts/out/TangleTaskManager.sol/TangleTaskManager.json"
);
load_abi!(
    TANGLE_TASK_MANAGER_ABI_STRING,
    "contracts/out/TangleTaskManager.sol/TangleTaskManager.json"
);

pub static TASK_MANAGER_ADDRESS: LazyLock<Address> = LazyLock::new(|| {
    std::env::var("TASK_MANAGER_ADDRESS")
        .map(|addr| addr.parse().expect("Invalid TASK_MANAGER_ADDRESS"))
        .unwrap_or_else(|_| address!("0000000000000000000000000000000000000000"))
});

#[derive(Clone)]
pub struct ExampleContext {
    pub config: GadgetConfiguration,
}

//// JOB DEFINITION TO HANDLE EVENTS

// Add a start_gaia_node job that starts the Gaia node using our manager
#[job(
    id = 1,
    params(network, data_dir),
    event_listener(
        listener = EvmContractEventListener<ExampleContext, TangleTaskManager::GaiaNodeStarted>,
        instance = TangleTaskManager,
        abi = TANGLE_TASK_MANAGER_ABI_STRING,
        pre_processor = start_gaia_pre_processor,
    ),
)]
pub fn start_gaia_node(
    _context: ExampleContext,
    network: Option<String>,
    data_dir: Option<String>,
) -> Result<(), Error> {
    blueprint_sdk::logging::info!("Received request to start Gaia node");

    let gaia_config = types::GaiaNodeConfig {
        network: network.unwrap(),
        data_dir: data_dir.unwrap(),
        verbose: true,
    };

    let gaia_node_manager =
        manager::GaiaNodeManager::new().expect("Failed to start Gaia Node Manager");

    gaia_node_manager.start(gaia_config);
    Ok(())
}

/// Pre-processor for the start_gaia_node job
async fn start_gaia_pre_processor(
    (event, _log): (TangleTaskManager::GaiaNodeStarted, Log),
) -> Result<Option<(Option<String>, Option<String>)>, ProcessorError> {
    // Extract network and data_dir from the event
    let network = event.network.clone();
    let data_dir = event.dataDir.clone();

    // Return the extracted values
    Ok(Some((Some(network), Some(data_dir))))
}

#[job(
    id = 1,
    params(who),
    event_listener(
        listener = EvmContractEventListener<ExampleContext, TangleTaskManager::GaiaNodeStopped>,
        instance = TangleTaskManager,
        abi = TANGLE_TASK_MANAGER_ABI_STRING,
        pre_processor = stop_gaia_pre_processor,
    ),
)]
pub async fn stop_gaia_node(context: ExampleContext, who: String) -> Result<String, Error> {
    blueprint_sdk::logging::info!("Received request to stop Gaia node");

    let gaia_node_manager =
        manager::GaiaNodeManager::new().expect("Failed to initialize Gaia Node Manager");

    // Stop the Gaia node
    gaia_node_manager
        .stop()
        .await
        .expect("Failed to stop gaia node");

    Ok(("Successfully stopped Gaia node").to_string())
}

/// Example pre-processor for handling inbound events
async fn stop_gaia_pre_processor(
    (_event, log): (TangleTaskManager::GaiaNodeStopped, Log),
) -> Result<Option<(String,)>, ProcessorError> {
    let who = log.address();
    Ok(Some((who.to_string(),)))
}
