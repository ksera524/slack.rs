use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, head, post, put},
};
use base64::Engine;
use serde_json::{Value, json};
use slack::{
    app,
    config::{settings::Settings, state::AppState},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::{net::TcpListener, sync::oneshot};

#[derive(Clone)]
struct MockSlackState {
    public_base_url: String,
}

#[derive(Clone)]
struct MockS3State {
    buckets: Arc<Mutex<HashMap<String, HashMap<String, MockS3Object>>>>,
}

#[derive(Clone)]
struct MockS3Object {
    body: Vec<u8>,
    content_type: String,
}

pub struct TestServer {
    pub base_url: String,
    shutdown: oneshot::Sender<()>,
}

impl TestServer {
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
    }
}

pub async fn start_mock_slack() -> tanu::eyre::Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let public_base_url = format!("http://{}", addr);
    let api_base_url = format!("{}/api", public_base_url);
    let state = MockSlackState {
        public_base_url: public_base_url.clone(),
    };

    let router = Router::new()
        .route("/api/chat.postMessage", post(mock_chat_post_message))
        .route("/api/files.getUploadURLExternal", get(mock_get_upload_url))
        .route(
            "/api/files.completeUploadExternal",
            post(mock_complete_upload),
        )
        .route("/upload/{id}", post(mock_upload_content))
        .with_state(state);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
    });

    tokio::spawn(async move {
        let _ = server.await;
    });

    Ok(TestServer {
        base_url: api_base_url,
        shutdown: shutdown_tx,
    })
}

pub async fn start_mock_s3() -> tanu::eyre::Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let state = MockS3State {
        buckets: Arc::new(Mutex::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/", get(mock_s3_list_buckets))
        .route("/{bucket}", put(mock_s3_create_bucket))
        .route("/{bucket}", get(mock_s3_list_or_get_bucket))
        .route("/{bucket}", head(mock_s3_head_bucket))
        .route("/{bucket}", delete(mock_s3_delete_bucket))
        .route("/{bucket}/{*key}", put(mock_s3_put_object))
        .route("/{bucket}/{*key}", get(mock_s3_get_object))
        .route("/{bucket}/{*key}", head(mock_s3_head_object))
        .route("/{bucket}/{*key}", delete(mock_s3_delete_object))
        .with_state(state);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
    });

    tokio::spawn(async move {
        let _ = server.await;
    });

    Ok(TestServer {
        base_url: format!("http://{}", addr),
        shutdown: shutdown_tx,
    })
}

pub async fn start_app(
    slack_api_base_url: String,
    s3_endpoint: String,
) -> tanu::eyre::Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{}", addr);

    let settings = Settings {
        slack_bot_token: "test-token".to_string(),
        slack_api_base_url,
        s3_access_key_id: "test-access-key".to_string(),
        s3_secret_access_key: "test-secret-key".to_string(),
        s3_region: "us-east-1".to_string(),
        s3_endpoint: Some(s3_endpoint),
        s3_use_path_style: true,
        s3_ignore_cert_check: false,
        s3_session_token: None,
    };
    let client = reqwest::Client::new();
    let app_state = AppState { settings, client };
    let app = app::create_app(app_state);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
    });

    tokio::spawn(async move {
        let _ = server.await;
    });

    Ok(TestServer {
        base_url,
        shutdown: shutdown_tx,
    })
}

async fn mock_chat_post_message(Json(payload): Json<Value>) -> Json<Value> {
    let channel = payload.get("channel").cloned().unwrap_or_else(|| json!(""));
    Json(json!({
        "ok": true,
        "channel": channel,
        "ts": "1700000000.000000"
    }))
}

async fn mock_get_upload_url(State(state): State<MockSlackState>) -> Json<Value> {
    let file_id = "F123456";
    let upload_url = format!("{}/upload/{}", state.public_base_url, file_id);
    Json(json!({
        "ok": true,
        "file_id": file_id,
        "upload_url": upload_url
    }))
}

async fn mock_upload_content(Path(_id): Path<String>, body: axum::body::Bytes) -> StatusCode {
    let _ = body;
    StatusCode::OK
}

async fn mock_complete_upload(Json(_payload): Json<Value>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "files": []
    }))
}

async fn mock_s3_create_bucket(
    Path(bucket): Path<String>,
    State(state): State<MockS3State>,
) -> (StatusCode, axum::http::HeaderMap) {
    let mut guard = state.buckets.lock().await;
    guard.entry(bucket.clone()).or_insert_with(HashMap::new);

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "location",
        format!("/{}", bucket)
            .parse()
            .expect("valid location header"),
    );
    (StatusCode::OK, headers)
}

async fn mock_s3_head_bucket(
    Path(bucket): Path<String>,
    State(state): State<MockS3State>,
) -> (StatusCode, axum::http::HeaderMap) {
    let guard = state.buckets.lock().await;
    if !guard.contains_key(&bucket) {
        return (StatusCode::NOT_FOUND, axum::http::HeaderMap::new());
    }

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "x-amz-bucket-region",
        "us-east-1".parse().expect("valid header"),
    );
    (StatusCode::OK, headers)
}

async fn mock_s3_delete_bucket(
    Path(bucket): Path<String>,
    State(state): State<MockS3State>,
) -> StatusCode {
    let mut guard = state.buckets.lock().await;
    if let Some(objects) = guard.get(&bucket)
        && !objects.is_empty()
    {
        return StatusCode::CONFLICT;
    }
    guard.remove(&bucket);
    StatusCode::NO_CONTENT
}

async fn mock_s3_list_buckets(State(state): State<MockS3State>) -> (StatusCode, String) {
    let guard = state.buckets.lock().await;
    let buckets_xml = guard
        .keys()
        .map(|name| {
            format!(
                "<Bucket><Name>{name}</Name><CreationDate>2026-01-01T00:00:00.000Z</CreationDate></Bucket>"
            )
        })
        .collect::<String>();

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListAllMyBucketsResult><Buckets>{buckets_xml}</Buckets></ListAllMyBucketsResult>"
    );

    (StatusCode::OK, body)
}

async fn mock_s3_list_or_get_bucket(
    Path(bucket): Path<String>,
    State(state): State<MockS3State>,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, String) {
    if query.get("list-type").map(String::as_str) == Some("2") {
        let prefix = query.get("prefix").cloned().unwrap_or_default();
        return mock_s3_list_objects_v2(bucket, state, prefix).await;
    }

    (StatusCode::BAD_REQUEST, "".to_string())
}

async fn mock_s3_list_objects_v2(
    bucket: String,
    state: MockS3State,
    prefix: String,
) -> (StatusCode, String) {
    let guard = state.buckets.lock().await;
    let Some(objects) = guard.get(&bucket) else {
        return (StatusCode::NOT_FOUND, "".to_string());
    };

    let contents_xml = objects
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(key, value)| {
            format!(
                "<Contents><Key>{key}</Key><LastModified>2026-01-01T00:00:00.000Z</LastModified><ETag>\"mock-etag\"</ETag><Size>{}</Size><StorageClass>STANDARD</StorageClass></Contents>",
                value.body.len()
            )
        })
        .collect::<String>();

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult><Name>{bucket}</Name><Prefix>{prefix}</Prefix><MaxKeys>1000</MaxKeys><KeyCount>{}</KeyCount><IsTruncated>false</IsTruncated>{contents_xml}</ListBucketResult>",
        contents_xml.matches("<Contents>").count()
    );

    (StatusCode::OK, body)
}

async fn mock_s3_put_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<MockS3State>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, axum::http::HeaderMap) {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let mut guard = state.buckets.lock().await;
    let bucket_map = guard.entry(bucket).or_insert_with(HashMap::new);
    bucket_map.insert(
        key,
        MockS3Object {
            body: body.to_vec(),
            content_type,
        },
    );

    let mut response_headers = axum::http::HeaderMap::new();
    response_headers.insert("etag", "\"mock-etag\"".parse().expect("valid header"));
    (StatusCode::OK, response_headers)
}

async fn mock_s3_get_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<MockS3State>,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let guard = state.buckets.lock().await;
    let Some(bucket_map) = guard.get(&bucket) else {
        return (
            StatusCode::NOT_FOUND,
            axum::http::HeaderMap::new(),
            Vec::new(),
        );
    };
    let Some(object) = bucket_map.get(&key) else {
        return (
            StatusCode::NOT_FOUND,
            axum::http::HeaderMap::new(),
            Vec::new(),
        );
    };

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "content-type",
        object.content_type.parse().expect("valid content type"),
    );
    headers.insert(
        "content-length",
        object
            .body
            .len()
            .to_string()
            .parse()
            .expect("valid content length"),
    );
    headers.insert("etag", "\"mock-etag\"".parse().expect("valid etag"));
    (StatusCode::OK, headers, object.body.clone())
}

async fn mock_s3_head_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<MockS3State>,
) -> (StatusCode, axum::http::HeaderMap) {
    let guard = state.buckets.lock().await;
    let Some(bucket_map) = guard.get(&bucket) else {
        return (StatusCode::NOT_FOUND, axum::http::HeaderMap::new());
    };
    let Some(object) = bucket_map.get(&key) else {
        return (StatusCode::NOT_FOUND, axum::http::HeaderMap::new());
    };

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "content-type",
        object.content_type.parse().expect("valid content type"),
    );
    headers.insert(
        "content-length",
        object
            .body
            .len()
            .to_string()
            .parse()
            .expect("valid content length"),
    );
    headers.insert("etag", "\"mock-etag\"".parse().expect("valid etag"));
    (StatusCode::OK, headers)
}

async fn mock_s3_delete_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<MockS3State>,
) -> StatusCode {
    let mut guard = state.buckets.lock().await;
    if let Some(bucket_map) = guard.get_mut(&bucket) {
        bucket_map.remove(&key);
    }
    StatusCode::NO_CONTENT
}

pub fn encode_base64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}
