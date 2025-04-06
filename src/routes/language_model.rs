use std::convert::Infallible;

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::header,
    response::{AppendHeaders, IntoResponse, Result},
    routing::post,
};

use serde::Deserialize;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tracing::{error, info};

use crate::AppState;
use built_in_hybrid_server::ai::language_model::{
    AILanguageModel, AILanguageModelCreateOptions, AILanguageModelPrompt, CountTokens, Prompt,
    PromptTreaming, providers::GeminiProvider,
};

use super::error::ApplicationError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/prompt", post(prompt))
        .route("/prompt-streaming", post(prompt_streaming))
        .route("/count-tokens", post(count_tokens))
        .route("/capabilities", post(capabilities))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelPromptRequest {
    pub create_options: AILanguageModelCreateOptions,
    pub inputs: Vec<AILanguageModelPrompt>,
}

#[axum::debug_handler]
pub async fn prompt(
    State(app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!(request = ?request, "prompt request");

    let provider = GeminiProvider::new(
        app_state.gemini_client.clone(),
        request.create_options.clone(),
    );

    let text = provider.prompt(&request.inputs).await?;

    Ok(text)
}

#[axum::debug_handler]
pub async fn prompt_streaming(
    State(app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> impl IntoResponse {
    info!(request = ?request, "prompt streaming request");

    let (tx, rx) = mpsc::channel::<Result<String, Infallible>>(2);
    tokio::spawn(stream_response(tx, app_state, request));
    let body = Body::from_stream(ReceiverStream::new(rx));

    let headers = AppendHeaders([
        (header::CONTENT_TYPE, "text/event-stream"),
        (header::CACHE_CONTROL, "no-cache"),
        (header::CONNECTION, "keep-alive"),
    ]);

    (headers, body)
}

pub async fn stream_response(
    tx: Sender<Result<String, Infallible>>,
    app_state: AppState,
    request: LanguageModelPromptRequest,
) {
    let provider = GeminiProvider::new(
        app_state.gemini_client.clone(),
        request.create_options.clone(),
    );

    let mut stream = provider.prompt_streaming(&request.inputs).await.unwrap();
    while let Some(response) = stream.next().await {
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                error!("Gemini streaming response error: {}", e);
                break;
            }
        };

        let _ = tx.send(Ok(response)).await;
    }
}

#[axum::debug_handler]
async fn capabilities() -> impl IntoResponse {
    Json(serde_json::to_value(GeminiProvider::capabilities()).unwrap())
}

#[axum::debug_handler]
async fn count_tokens(
    State(app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!(request = ?request, "count tokens request");

    let provider = GeminiProvider::new(
        app_state.gemini_client.clone(),
        request.create_options.clone(),
    );

    let total_tokens = provider.count_tokens(&request.inputs)?;

    Ok(total_tokens.to_string())
}
