pub mod ai;
mod routes;

use std::{env, error::Error};

use axum::Router;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let app_state = AppState {};

    // Create Router.
    let app = Router::new()
        .fallback_service(ServeDir::new("static"))
        .merge(routes::routes())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
