use blueprint::{TangleTaskManager, TASK_MANAGER_ADDRESS};
use blueprint_sdk::alloy::primitives::{address, Address, U256};
use blueprint_sdk::logging::{info, warn};
use blueprint_sdk::macros::main;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::utils::evm::get_provider_http;

use my_eigenlayer_avs_1 as blueprint;
use my_eigenlayer_avs_1::actix_server;
use structopt::StructOpt;

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
    let say_hello_job = blueprint::SayHelloEventHandler::new(contract, context.clone());

    // Spawn a task to create a task - this is just for testing/example purposes
    info!("Spawning a task to create a task on the contract...");
    blueprint_sdk::tokio::spawn(async move {
        let provider = get_provider_http(&rpc_endpoint);
        let contract = TangleTaskManager::new(*TASK_MANAGER_ADDRESS, provider);
        loop {
            blueprint_sdk::tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            // We use the Anvil Account #4 as the Task generator address
            let task = contract
                .createNewTask(U256::from(5), 100u32, vec![0].into())
                .from(address!("15d34AAf54267DB7D7c367839AAf71A00a2C6A65"));
            let receipt = task.send().await.unwrap().get_receipt().await.unwrap();
            if receipt.status() {
                info!("Task created successfully");
            } else {
                warn!("Task creation failed");
            }
        }
    });

    info!("Starting the event watcher ...");
    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    BlueprintRunner::new(eigen_config, env)
        .job(say_hello_job)
        .run()
        .await?;

    info!("Exiting...");
    Ok(())
}
