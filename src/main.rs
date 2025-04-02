pub mod ai;
mod middleware;
mod routes;

use std::{collections::HashSet, env, error::Error, sync::Arc};

use axum::{
    Router,
    http::{HeaderValue, Method},
    middleware::from_fn_with_state,
};
use gcp_auth::TokenProvider;
use gemini_rs::prelude::GeminiClient;
use middleware::allowed_origins::allowed_origins_middelware;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
pub struct AppState {
    pub gemini_client: GeminiClient<Arc<dyn TokenProvider>>,
    pub accepted_origins: Arc<HashSet<HeaderValue>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let api_endpoint = env::var("API_ENDPOINT")?;
    let project_id = env::var("PROJECT_ID")?;
    let location_id = env::var("LOCATION_ID")?;
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let accepted_origins = env::var("ALLOWED_ORIGINS")?
        .split(";")
        .filter_map(|header| HeaderValue::from_str(header).ok())
        .collect::<Vec<_>>();

    let authentication_manager = gcp_auth::provider().await?;
    tracing::info!("GCP AuthenticationManager initialized.");

    let gemini_client = GeminiClient::new(
        authentication_manager,
        api_endpoint,
        project_id,
        location_id,
    );
    tracing::info!("GeminiClient initialized.");

    let app_state = AppState {
        gemini_client,
        accepted_origins: Arc::new(HashSet::from_iter(accepted_origins.clone().into_iter())),
    };

    // Sets up a compression layer that supports brotli, deflate, gzip, and zstd.
    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(accepted_origins);

    // Create Router.
    let app = Router::new()
        .fallback_service(ServeDir::new("static"))
        .merge(routes::routes())
        .layer(cors)
        .layer(compression_layer)
        .layer(from_fn_with_state(
            app_state.clone(),
            allowed_origins_middelware,
        ))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
