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
    service_id: u64,
}

async fn chat(
    app_state: web::Data<AppState>,
    chat_request: web::Json<ChatRequest>,
) -> impl Responder {
    handle_gaia_request(app_state, chat_request, |client, request| async move {
        let client = client.lock().await;
        client.chat(request.messages).await
    })
    .await
}

pub async fn run_server(service_id: u64, model: string) -> Result<()> {
    /// Arc allows multiple ownership of the client across threads
    /// Mutex allows safe concurrent access to the GaiaNodeClient
    /// This shared state is used by the Actix web server to handle variousendpoints
    /// endpoints
    let app_state = web::Data::new(AppState {
        gaia_client: Arc::new(Mutex::new(GaiaNodeClient::new(
            "https://YOUR-NODE-ID.us.gaianet.network/v1".to_string(),
            "".to_string(),
            model,
        ))),
        service_id,
    });

    info!(
        "Starting server with base URL: {} and service ID: {}",
        app_state.gaia_client.lock().await.base_url,
        app_state.service_id
    );

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/chat", web::post().to(chat))
            .route("/analyze_image", web::post().to(analyze_image))
            .route("/create_image", web::post().to(create_image))
            .route("/edit_image", web::post().to(edit_image))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
