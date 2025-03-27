use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Result},
    routing::post,
};
use serde::Deserialize;
use tracing::info;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/prompt", post(prompt))
        .route("/count-tokens", post(count_tokens))
}

#[derive(Debug, Deserialize)]
pub struct LanguageModelPromptRequest {}

pub async fn count_tokens(
    State(_app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse> {
    info!(request = ?request, "prompt request");
    Ok("Hello")
}

pub async fn prompt(
    State(_app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse> {
    info!(request = ?request, "prompt request");

    Ok("Hello")
}
