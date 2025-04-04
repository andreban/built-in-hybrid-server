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
use serde_json::json;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info, warn};

use crate::{
    AppState,
    ai::language_model::{
        self, AILanguageModelCreateOptions, AILanguageModelError, AILanguageModelPrompt,
        AILanguageModelPromptRole,
    },
};

use super::error::ApplicationError;

static GEMINI_MODEL: &str = "gemini-2.0-flash-lite-001";

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

impl LanguageModelPromptRequest {
    // Concatenate the initial prompts with the request prompts.
    pub fn all_inputs(&self) -> impl Iterator<Item = &AILanguageModelPrompt> {
        self.create_options
            .initial_prompts
            .iter()
            .chain(self.inputs.iter())
    }
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

        for input in self.all_inputs() {
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
) -> Result<impl IntoResponse, ApplicationError> {
    info!(request = ?request, "prompt request");

    let gemini_request: GenerateContentRequest = request.try_into().unwrap();
    let gemini_response = app_state
        .gemini_client
        .generate_content(&gemini_request, GEMINI_MODEL)
        .await?;

    let text = gemini_response
        .candidates
        .first()
        .ok_or(gemini_rs::error::Error::NoCandidatesError)?
        .get_text()
        .unwrap_or_default();

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

    while let Some(response) = gemini_response.pop().await {
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                error!("Gemini streaming response error: {}", e);
                break;
            }
        };

        let Some(candidate) = response.candidates.first() else {
            warn!("No candidates returned for the prompt");
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

#[axum::debug_handler]
async fn capabilities() -> impl IntoResponse {
    Json(json!({
        "maxTemperature": 2.0,
        "maxTopK":  40,
        "defaultTemperature": 1.0,
        "defaultTopK": 3,
        "defaultTopP": 0.95,
        "maxTokens": 1_048_576,
    }))
}

async fn count_tokens(
    Json(request): Json<LanguageModelPromptRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!(request = ?request, "count tokens request");

    let prompt = build_gemma_prompt(&request)?;
    let total_tokens = crate::ai::tokenizer::count_tokens(&prompt)
        .map_err(|e| ApplicationError::TokenizerError(e))?;

    Ok(total_tokens.to_string())
}

// See https://ai.google.dev/gemma/docs/core/prompt-structure
pub fn build_gemma_prompt(
    request: &LanguageModelPromptRequest,
) -> Result<String, ApplicationError> {
    static START_OF_TURN: &str = "<start_of_turn>";
    static END_OF_TURN: &str = "<end_of_turn>";
    static MODEL: &str = "model";
    static USER: &str = "user";

    let mut prompt = String::new();
    let mut system_prompt = request.create_options.system_prompt_text()?;

    for input in request.all_inputs() {
        let (user_or_model, content) = match input {
            AILanguageModelPrompt::Text { role, content } => match role {
                AILanguageModelPromptRole::User => (USER, content),
                AILanguageModelPromptRole::Assistant => (MODEL, content),
                _ => continue,
            },
            _ => continue,
        };
        prompt.push_str(START_OF_TURN);
        prompt.push_str(user_or_model);
        prompt.push_str("\n");
        if let Some(system) = system_prompt.take() {
            prompt.push_str(&system);
            prompt.push_str("\n\n");
            system_prompt = None;
        }
        prompt.push_str(content);
        prompt.push_str(END_OF_TURN);
        prompt.push_str("\n");
    }
    Ok(prompt)
}
