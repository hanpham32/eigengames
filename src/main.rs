use blueprint::{TangleTaskManager, TASK_MANAGER_ADDRESS};
use blueprint_sdk::alloy::primitives::Address;
use blueprint_sdk::logging::info;
use blueprint_sdk::macros::main;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::utils::evm::get_provider_http;

use my_eigenlayer_avs_1::{self as blueprint};

#[main(env)]
async fn main() {
    // Create your service context
    // Here you can pass any configuration or context that your service needs.
    let context = blueprint::ExampleContext {
        config: env.clone(),
    };

    // Get the provider
    let rpc_endpoint = env.http_rpc_endpoint.clone();
    let provider = get_provider_http(&rpc_endpoint);

    // Create an instance of your task manager
    let contract = TangleTaskManager::new(*TASK_MANAGER_ADDRESS, provider);

    // Create the event handler from the job
    let start_gaia_node =
        blueprint::StartGaiaNodeEventHandler::new(contract.clone(), context.clone());
    let stop_gaia_node =
        blueprint::StopGaiaNodeEventHandler::new(contract.clone(), context.clone());

    info!("Starting the event watcher ...");

    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    BlueprintRunner::new(eigen_config, env)
        .job(start_gaia_node)
        .job(stop_gaia_node)
        .run()
        .await?;

    info!("Exiting...");

    Ok(())
}
