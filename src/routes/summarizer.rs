use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Result},
    routing::method_routing::post,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    AppState,
    ai::summarizer::{SummarizerOptions, build_system_prompt},
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/prompt", post(prompt))
}

#[derive(Debug, Deserialize)]
pub struct PromptRequest {
    pub input: String,
    pub options: SummarizerOptions,
}

pub async fn prompt(
    State(_app_state): State<AppState>,
    Json(request): Json<PromptRequest>,
) -> Result<impl IntoResponse> {
    info!(request = ?request, "prompt request");

    let system_prompt = build_system_prompt(&request.input, &request.options);
    info!(system_prompt = system_prompt, "system prompt");

    Ok("Hello")
}
