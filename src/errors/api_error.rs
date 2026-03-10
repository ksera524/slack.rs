use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
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

fn problem_details_json(status: StatusCode, detail: impl Into<String>) -> String {
    let title = status
        .canonical_reason()
        .unwrap_or("Unknown Error")
        .to_string();
    let detail = detail.into();
    nojson::json(|f| {
        f.object(|f| {
            f.member("type", "about:blank")?;
            f.member("title", &title)?;
            f.member("status", status.as_u16())?;
            f.member("detail", &detail)
        })
    })
    .to_string()
}

pub fn problem_details_response(status: StatusCode, detail: impl Into<String>) -> Response {
    let body = problem_details_json(status, detail);
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
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
