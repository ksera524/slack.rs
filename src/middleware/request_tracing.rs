use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{error, info, info_span, warn, Instrument};
use uuid::Uuid;

/// リクエストIDヘッダー名
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// リクエストトレーシングミドルウェア
pub async fn request_tracing_middleware(
    request: Request,
    next: Next,
) -> Response {
    // リクエストIDの取得または生成
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().map(String::from);
    let version = format!("{:?}", request.version());

    // クライアント情報の取得
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
        });

    // リクエストのサイズ
    let content_length = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    // トレーシングスパンの作成
    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
        query = ?query,
        version = %version,
        user_agent = ?user_agent,
        client_ip = ?client_ip,
        content_length = ?content_length,
        status = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
        error = tracing::field::Empty,
    );

    let start = Instant::now();

    // リクエストログ
    info!(
        target: "http::request",
        request_id = %request_id,
        method = %method,
        path = %path,
        query = ?query,
        user_agent = ?user_agent,
        client_ip = ?client_ip,
        "Incoming request"
    );

    // リクエストにIDヘッダーを追加
    let mut request = request;
    request.headers_mut().insert(
        HeaderName::from_static(REQUEST_ID_HEADER),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    // リクエスト処理
    let response = next.run(request).instrument(span.clone()).await;

    let latency = start.elapsed();
    let latency_ms = latency.as_millis() as u64;
    let status = response.status();

    // スパンに情報を記録
    span.record("status", status.as_u16());
    span.record("latency_ms", latency_ms);

    // レスポンスログ
    match status {
        s if s.is_success() => {
            info!(
                target: "http::response",
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status.as_u16(),
                latency_ms = latency_ms,
                "Request completed"
            );
        }
        s if s.is_redirection() => {
            info!(
                target: "http::response",
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status.as_u16(),
                latency_ms = latency_ms,
                "Request redirected"
            );
        }
        s if s.is_client_error() => {
            warn!(
                target: "http::response",
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status.as_u16(),
                latency_ms = latency_ms,
                "Client error"
            );
        }
        s if s.is_server_error() => {
            error!(
                target: "http::response",
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status.as_u16(),
                latency_ms = latency_ms,
                "Server error"
            );
            span.record("error", "server_error");
        }
        _ => {
            warn!(
                target: "http::response",
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status.as_u16(),
                latency_ms = latency_ms,
                "Unexpected status"
            );
        }
    }

    // レスポンスにリクエストIDヘッダーを追加
    let mut response = response;
    response.headers_mut().insert(
        HeaderName::from_static(REQUEST_ID_HEADER),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

