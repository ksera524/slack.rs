use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

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
            ApiError::BadRequest(message) => {
                let body = ErrorResponseBody { message };
                (StatusCode::BAD_REQUEST, Json(&body)).into_response()
            }
            ApiError::InternalServerError(_) => {
                let body = ErrorResponseBody {
                    message: self.to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(&body)).into_response()
            }
        }
    }
}
