use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error_code: &'static str,
    pub message: String,
    pub trace_id: String,
}

#[derive(Debug)]
pub enum AppError {
    ReadinessFailed {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::ReadinessFailed {
                error_code,
                message,
                trace_id,
            } => {
                let body = ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                };
                (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
            }
        }
    }
}
