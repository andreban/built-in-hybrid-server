use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::info;

use crate::AppState;

pub async fn allowed_origins_middelware(
    State(app_state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if !req.uri().path().starts_with("/language-model") {
        return next.run(req).await;
    }

    let origin = req.headers().get("origin");
    if let Some(origin) = origin {
        if app_state.accepted_origins.contains(origin) {
            return next.run(req).await;
        }
    }

    info!(origin = ?origin, uri = ?req.uri(), "Forbidden origin for request.");
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}
