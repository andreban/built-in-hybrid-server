mod error;

mod language_model;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().nest("/language-model", language_model::routes())
}
