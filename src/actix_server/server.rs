use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use color_eyre::Result;
use gadget_sdk::info;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    gaia_client::{APIError, GaiaNodeClient},
    types::ChatRequest,
};

struct AppState {
    gaia_client: Arc<Mutex<GaiaNodeClient>>,
}

pub async fn run_server(service_id: u64, model: string) -> Result<()> {
    let app_state = web::Data::new(AppState {});
}
