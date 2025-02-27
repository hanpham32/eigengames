use crate::actix_server;
use crate::gaia_manager::GaiaNodeManager;
use blueprint_sdk::logging::error;
use std::sync::Arc;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure the Gaia node manager
    let node_manager = match GaiaNodeManager::new() {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            error!("Failed to create Gaia node manager: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )));
        }
    };

    // Get the bind address from config or use default
    let bind_address =
        std::env::var("GAIA_API_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    // Start the API server
    actix_server::run_server(node_manager, &bind_address).await?;

    Ok(())
}
