mod language_model;
mod summarizer;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/summarize", summarizer::routes())
        .nest("/language-model", language_model::routes())
}
