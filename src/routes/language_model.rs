use std::{convert::Infallible, vec};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::header,
    response::{AppendHeaders, IntoResponse, Result},
    routing::post,
};
use gemini_rs::prelude::{Content, GenerateContentRequest, GenerationConfig, Role};
use serde::Deserialize;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

use crate::{
    AppState,
    ai::language_model::{
        self, AILanguageModelCreateOptions, AILanguageModelError, AILanguageModelPrompt,
        AILanguageModelPromptRole,
    },
};

static GEMINI_MODEL: &str = "gemini-2.0-flash-lite-001";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/prompt", post(prompt))
        .route("/prompt-streaming", post(prompt_streaming))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelPromptRequest {
    pub create_options: AILanguageModelCreateOptions,
    pub inputs: Vec<AILanguageModelPrompt>,
}

impl TryInto<GenerateContentRequest> for LanguageModelPromptRequest {
    type Error = language_model::AILanguageModelError;

    fn try_into(self) -> std::result::Result<GenerateContentRequest, Self::Error> {
        let generation_config = GenerationConfig::builder()
            .temperature(self.create_options.temperature)
            .top_k(self.create_options.top_k as i32)
            .build();

        let mut request_builder =
            GenerateContentRequest::builder().generation_config(generation_config);

        // Set the System Prompt.
        if let Some(system_prompt) = self.create_options.system_prompt_text()? {
            request_builder = request_builder
                .system_instruction(Content::builder().add_text_part(system_prompt).build());
        }

        // Set the User / Assistant Prompts.
        let mut contents: Vec<Content> = vec![];

        // Concatenate the initial prompts with the request prompts.
        let all_inputs = self
            .create_options
            .initial_prompts
            .iter()
            .chain(self.inputs.iter());

        for input in all_inputs {
            match input {
                AILanguageModelPrompt::Text { role, content } => {
                    let role = match role {
                        AILanguageModelPromptRole::User => Role::User,
                        AILanguageModelPromptRole::Assistant => Role::Model,
                        _ => continue, // AIlanguageModelPromptRole::System only applies for the system prompt.
                    };

                    contents.push(Content::builder().role(role).add_text_part(content).build());
                }
                _ => {
                    return Err(AILanguageModelError::PromptInputError(
                        "Unsupported input type",
                    ));
                }
            }
        }
        request_builder = request_builder.contents(contents);

        Ok(request_builder.build())
    }
}

#[axum::debug_handler]
pub async fn prompt(
    State(app_state): State<AppState>,
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse> {
    info!(request = ?request, "prompt request");

    let gemini_request: GenerateContentRequest = request.try_into().unwrap();
    let mut gemini_response = app_state
        .gemini_client
        .generate_content(&gemini_request, GEMINI_MODEL)
        .await
        .unwrap();
    info!(response = ?gemini_response, "gemini response");

    let text = gemini_response
        .candidates
        .pop()
        .unwrap()
        .get_text()
        .unwrap_or_default();
    info!(text);

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
    let gemini_request: GenerateContentRequest = request.try_into().unwrap();
    let gemini_response = app_state
        .gemini_client
        .stream_generate_content(&gemini_request, GEMINI_MODEL)
        .await;

    while let Some(Ok(mut response)) = gemini_response.pop().await {
        let Some(candidate) = response.candidates.pop() else {
            break;
        };

        if let Some(text) = candidate.get_text() {
            let _ = tx.send(Ok(text)).await;
        }

        if candidate.finish_reason.is_some() {
            break;
        }
    }
}
