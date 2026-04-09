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
    BadRequest {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
    Unauthorized {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
    Forbidden {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
    Internal {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
    ServiceUnavailable {
        error_code: &'static str,
        message: String,
        trace_id: String,
    },
}

impl AppError {
    pub fn bad_request(
        error_code: &'static str,
        message: impl Into<String>,
        trace_id: String,
    ) -> Self {
        Self::BadRequest {
            error_code,
            message: message.into(),
            trace_id,
        }
    }

    pub fn unauthorized(
        error_code: &'static str,
        message: impl Into<String>,
        trace_id: String,
    ) -> Self {
        Self::Unauthorized {
            error_code,
            message: message.into(),
            trace_id,
        }
    }

    pub fn forbidden(
        error_code: &'static str,
        message: impl Into<String>,
        trace_id: String,
    ) -> Self {
        Self::Forbidden {
            error_code,
            message: message.into(),
            trace_id,
        }
    }

    pub fn internal(
        error_code: &'static str,
        message: impl Into<String>,
        trace_id: String,
    ) -> Self {
        Self::Internal {
            error_code,
            message: message.into(),
            trace_id,
        }
    }

    pub fn service_unavailable(
        error_code: &'static str,
        message: impl Into<String>,
        trace_id: String,
    ) -> Self {
        Self::ServiceUnavailable {
            error_code,
            message: message.into(),
            trace_id,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::ReadinessFailed {
                error_code,
                message,
                trace_id,
            }
            | AppError::ServiceUnavailable {
                error_code,
                message,
                trace_id,
            } => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                }),
            )
                .into_response(),
            AppError::BadRequest {
                error_code,
                message,
                trace_id,
            } => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                }),
            )
                .into_response(),
            AppError::Unauthorized {
                error_code,
                message,
                trace_id,
            } => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                }),
            )
                .into_response(),
            AppError::Forbidden {
                error_code,
                message,
                trace_id,
            } => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                }),
            )
                .into_response(),
            AppError::Internal {
                error_code,
                message,
                trace_id,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error_code,
                    message,
                    trace_id,
                }),
            )
                .into_response(),
        }
    }
}
