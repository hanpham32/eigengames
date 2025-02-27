use crate::gaia_manager::GaiaNodeManager;
use crate::types::GaiaNodeConfig;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state - contains the Gaia node manager
pub struct AppState {
    pub node_manager: Arc<GaiaNodeManager>,
}

#[derive(Serialize, Deserialize)]
pub struct StartNodeRequest {
    pub network: Option<String>,
    pub data_dir: Option<String>,
}

#[get("/status")]
async fn get_status(data: web::Data<AppState>) -> impl Responder {
    let status = data.node_manager.get_status().await;
    HttpResponse::Ok().json(status)
}

// #[get("/info")]
// async fn get_info(data: web::Data<AppState>) -> impl Responder {
//     match data.node_manager.get_info().await {
//         Ok(info) => HttpResponse::Ok().json(info),
//         Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
//             "error": format!("Failed to get node info: {}", e)
//         })),
//     }
// }

#[post("/start")]
async fn start_node(data: web::Data<AppState>, req: web::Json<StartNodeRequest>) -> impl Responder {
    let mut config = GaiaNodeConfig::default();

    if let Some(network) = &req.network {
        config.network = network.clone();
    }

    if let Some(data_dir) = &req.data_dir {
        config.data_dir = data_dir.clone();
    }

    match data.node_manager.start(config).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "started",
            "message": "GaiaNet node started successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to start node: {}", e)
        })),
    }
}

#[post("/stop")]
async fn stop_node(data: web::Data<AppState>) -> impl Responder {
    match data.node_manager.stop().await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "stopped",
            "message": "GaiaNet node stopped successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to stop node: {}", e)
        })),
    }
}

pub async fn run_server(
    node_manager: Arc<GaiaNodeManager>,
    bind_address: &str,
) -> std::io::Result<()> {
    blueprint_sdk::logging::info!("Starting Gaia Node API server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                node_manager: Arc::clone(&node_manager),
            }))
            .service(get_status)
            .service(start_node)
            .service(stop_node)
        // .service(get_info)
    })
    .bind(bind_address)?
    .run()
    .await
}
