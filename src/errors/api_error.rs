use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    InternalServerError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(message) => write!(f, "Bad Request: {message}"),
            Self::InternalServerError(_) => write!(f, "Internal Server Error"),
        }
    }
}

impl std::error::Error for ApiError {}

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
