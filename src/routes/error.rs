use gemini_rs::error::Error as GeminiError;
use std::error::Error;
use std::fmt::Display;
use tracing::error;

use axum::response::IntoResponse;

use crate::ai::language_model::AILanguageModelError;

#[derive(Debug)]
pub enum ApplicationError {
    LanguageModelError(AILanguageModelError),
    GeminiError(GeminiError),
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::LanguageModelError(err) => write!(f, "{}", err),
            ApplicationError::GeminiError(err) => write!(f, "{}", err),
        }
    }
}

impl From<AILanguageModelError> for ApplicationError {
    fn from(err: AILanguageModelError) -> Self {
        ApplicationError::LanguageModelError(err)
    }
}

impl From<GeminiError> for ApplicationError {
    fn from(err: GeminiError) -> Self {
        ApplicationError::GeminiError(err)
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApplicationError::LanguageModelError(err) => {
                let status_code = match err {
                    AILanguageModelError::SystemPromptError(_) => {
                        axum::http::StatusCode::BAD_REQUEST
                    }
                    AILanguageModelError::PromptInputError(_) => {
                        axum::http::StatusCode::BAD_REQUEST
                    }
                };
                (status_code, err.to_string()).into_response()
            }
            ApplicationError::GeminiError(err) => {
                error!("Gemini error: {}", err);
                let status_code = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                (status_code, "Internal Server Error").into_response()
            }
        }
    }
}
