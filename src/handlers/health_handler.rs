use shiguredo_http11::Response;

pub fn health() -> Response {
    Response::new(200, "OK")
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(b"ok".to_vec())
}
