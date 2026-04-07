use shiguredo_http11::Response;

const OPENAPI_JSON: &str = include_str!("../../openapi.json");

pub fn openapi_json() -> Response {
    Response::new(200, "OK")
        .header("Content-Type", "application/json")
        .body(OPENAPI_JSON.as_bytes().to_vec())
}
