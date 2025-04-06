mod routes;

use std::{env, error::Error, sync::Arc};

use axum::Router;
use gcp_auth::TokenProvider;
use gemini_rs::prelude::GeminiClient;
use tower_http::{compression::CompressionLayer, services::ServeDir};

#[derive(Clone)]
pub struct AppState {
    pub gemini_client: GeminiClient<Arc<dyn TokenProvider>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let api_endpoint = env::var("API_ENDPOINT")?;
    let project_id = env::var("PROJECT_ID")?;
    let location_id = env::var("LOCATION_ID")?;
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let authentication_manager = gcp_auth::provider().await?;
    tracing::info!("GCP AuthenticationManager initialized.");

    let gemini_client = GeminiClient::new(
        authentication_manager,
        api_endpoint,
        project_id,
        location_id,
    );
    tracing::info!("GeminiClient initialized.");

    let app_state = AppState { gemini_client };

    // Sets up a compression layer that supports brotli, deflate, gzip, and zstd.
    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    // Create Router.
    let app = Router::new()
        .fallback_service(ServeDir::new("static"))
        .merge(routes::routes())
        .layer(compression_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
