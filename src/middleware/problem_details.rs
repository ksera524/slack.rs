use axum::{
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::Next,
    response::Response,
};

use crate::errors::api_error::problem_details_response;

pub async fn problem_details_middleware(
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    let status = response.status();

    if !(status.is_client_error() || status.is_server_error()) {
        return response;
    }

    let is_problem_details = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.starts_with("application/problem+json"))
        .unwrap_or(false);

    if is_problem_details {
        return response;
    }

    let detail = status.canonical_reason().unwrap_or("Unknown Error");
    problem_details_response(status, detail)
}
