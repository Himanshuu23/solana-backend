use axum::{http::StatusCode, Json, response::IntoResponse};
use crate::models::ErrorResponse;

pub fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (
        status,
        Json(ErrorResponse {
            success: false,
            error: message.to_string(),
        }),
    )
}
