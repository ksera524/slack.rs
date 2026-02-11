use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
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
struct ProblemDetails {
    #[serde(rename = "type")]
    type_url: String,
    title: String,
    status: u16,
    detail: String,
}

fn problem_details(status: StatusCode, detail: impl Into<String>) -> ProblemDetails {
    let title = status
        .canonical_reason()
        .unwrap_or("Unknown Error")
        .to_string();
    ProblemDetails {
        type_url: "about:blank".to_string(),
        title,
        status: status.as_u16(),
        detail: detail.into(),
    }
}

pub fn problem_details_response(status: StatusCode, detail: impl Into<String>) -> Response {
    let body = problem_details(status, detail);
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
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
                problem_details_response(StatusCode::BAD_REQUEST, message.clone())
            }
            ApiError::InternalServerError(ref details) => {
                error!(
                    error_type = "internal_server_error",
                    details = %details,
                    status = 500,
                    "API error occurred"
                );
                problem_details_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        }
    }
}
