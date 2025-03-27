mod language_model;
mod summarizer;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().nest("/summarize", summarizer::routes())
}
