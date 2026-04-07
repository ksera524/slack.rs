use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::Response,
};

const OPENAPI_JSON: &str = include_str!("../../openapi.json");

pub async fn openapi_json() -> Response {
    let mut response = Response::new(Body::from(OPENAPI_JSON));
    *response.status_mut() = StatusCode::OK;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}
