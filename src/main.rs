use blueprint::{GadgetConfiguration, TangleTaskManager, TASK_MANAGER_ADDRESS};
use blueprint_sdk::alloy::primitives::{address, Address, U256};
use blueprint_sdk::config::ContextConfig;
use blueprint_sdk::logging::{info, warn};
use blueprint_sdk::macros::main;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::utils::evm::get_provider_http;

use color_eyre::eyre::Context;
use my_eigenlayer_avs_1 as blueprint;
use my_eigenlayer_avs_1::actix_server;
use structopt::StructOpt;

#[main(env)]
async fn main() {
    // Create your service context
    // Here you can pass any configuration or context that your service needs.

    let config = ContextConfig::from_arg_matches(matches);

    let (env, mut runner) = create_gadget_runner(config.clone()).await;

    // Register the operator if needed
    if env.should_run_registration() {
        // Execute any custom registration hook
        runner.register().await?;
    }

    let model = "llama".to_string();
    let service_id = env.service_id.unwrap_or_default();

    // Run the server and the gadget concurrently
    blueprint_sdk::tokio::select! {
        server_result = actix_server::server::run_server(service_id, model) => {
            if let Err(e) = server_result {
                eprintln!("Server error: {}", e);
            }
        }
        runner_result = runner.run() => {
            if let Err(e) = runner_result {
                eprintln!("Runner error: {}", e);
            }
        }
    }

    // Get the provider
    let rpc_endpoint = env.http_rpc_endpoint.clone();
    let provider = get_provider_http(&rpc_endpoint);

    // Create an instance of your task manager
    let contract = TangleTaskManager::new(*TASK_MANAGER_ADDRESS, provider);

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

    // Create the event handler from the job
    let say_hello_job = blueprint::SayHelloEventHandler::new(contract, context.clone());

    info!("Starting the event watcher ...");

    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    BlueprintRunner::new(eigen_config, env)
        .job(say_hello_job)
        .run()
        .await?;

    info!("Exiting...");

    Ok(())
}

struct TangleGadgetRunner {
    env: GadgetConfiguration<parking_lot::RawRwLock>,
}

#[async_trait::async_trait]
impl GadgetRunner for TangleGadgetRunner {
    type Error = color_eyre::eyre::Report;

    fn config(&self) -> &StdGadgetConfiguration {
        todo!()
    }

    async fn register(&mut self) -> Result<()> {
        // TODO: Use the function in blueprint-test-utils
        if self.env.test_mode {
            info!("Skipping registration in test mode");
            return Ok(());
        }

        let client = self.env.client().await.map_err(|e| eyre!(e))?;
        let signer = self
            .env
            .first_sr25519_signer()
            .map_err(|e| eyre!(e))
            .map_err(|e| eyre!(e))?;
        let ecdsa_pair = self.env.first_ecdsa_signer().map_err(|e| eyre!(e))?;

        let xt = api::tx().services().register(
            self.env.blueprint_id,
            services::OperatorPreferences {
                key: ecdsa::Public(ecdsa_pair.signer().public().0),
                approval: services::ApprovalPrefrence::None,
                price_targets: PriceTargets {
                    cpu: 0,
                    mem: 0,
                    storage_hdd: 0,
                    storage_ssd: 0,
                    storage_nvme: 0,
                },
            },
            Default::default(),
        );

        // send the tx to the tangle and exit.
        let result = tx::tangle::send(&client, &signer, &xt).await?;
        info!("Registered operator with hash: {:?}", result);
        Ok(())
    }

    async fn benchmark(&self) -> std::result::Result<(), Self::Error> {
        todo!()
    }

    async fn run(&self) -> Result<()> {
        let client = self.env.client().await.map_err(|e| eyre!(e))?;
        let signer = self.env.first_sr25519_signer().map_err(|e| eyre!(e))?;

        info!("Starting the event watcher for {} ...", signer.account_id());

        let start_job = blueprint::RunGaiaNodeJobEventHandler {
            service_id: self.env.service_id.unwrap(),
            signer: signer.clone(),
        };

        let stop_job = blueprint::StopGaiaNodeJobEventHandler {
            service_id: self.env.service_id.unwrap(),
            signer: signer.clone(),
        };

        let upgrade_job = blueprint::UpgradeGaiaNodeJobEventHandler {
            service_id: self.env.service_id.unwrap(),
            signer: signer.clone(),
        };

        let update_config_job = blueprint::UpdateGaiaConfigJobEventHandler {
            service_id: self.env.service_id.unwrap(),
            signer,
        };

        let program = TangleEventsWatcher {
            span: self.env.span.clone(),
            client,
            handlers: vec![
                Box::new(start_job),
                Box::new(stop_job),
                Box::new(upgrade_job),
                Box::new(update_config_job),
            ],
        };

        program.into_tangle_event_listener().execute().await;

        Ok(())
    }
}

async fn create_gadget_runner(
    config: ContextConfig,
) -> (
    GadgetConfiguration<parking_lot::RawRwLock>,
    Box<dyn GadgetRunner<Error = color_eyre::Report>>,
) {
    let env = gadget_sdk::config::load(config).expect("Failed to load environment");
    match env.protocol {
        Protocol::Tangle => (env.clone(), Box::new(TangleGadgetRunner { env })),
        _ => panic!("Unsupported protocol Eigenlayer. Gadget/Tangle need U256 support."),
    }
}
