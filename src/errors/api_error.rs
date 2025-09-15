use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Server Error")]
    InternalServerError(String),
}

#[derive(Serialize)]
struct ErrorResponseBody {
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(ref message) => {
                error!(
                    error_type = "bad_request",
                    message = %message,
                    status = 400,
                    "API error occurred"
                );
                let body = ErrorResponseBody { message: message.clone() };
                (StatusCode::BAD_REQUEST, Json(&body)).into_response()
            }
            ApiError::InternalServerError(ref details) => {
                error!(
                    error_type = "internal_server_error",
                    details = %details,
                    status = 500,
                    "API error occurred"
                );
                let body = ErrorResponseBody {
                    message: self.to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(&body)).into_response()
            }
        }
    }
}
