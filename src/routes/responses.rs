use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
        .into_response()
}

pub fn tags_not_found_response(ids: &[u32]) -> Response {
    let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    error_response(
        StatusCode::NOT_FOUND,
        format!("tags not found: {}", ids.join(", ")),
    )
}
