use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use slack::{
    app,
    config::{settings::Settings, state::AppState},
};
use tokio::{net::TcpListener, sync::oneshot};

#[derive(Clone)]
struct MockSlackState {
    public_base_url: String,
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

pub async fn start_app(slack_api_base_url: String) -> tanu::eyre::Result<TestServer> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{}", addr);

    let settings = Settings {
        slack_bot_token: "test-token".to_string(),
        slack_api_base_url,
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

fn json_response(body: String) -> Response {
    (StatusCode::OK, [("content-type", "application/json")], body).into_response()
}

async fn mock_chat_post_message(body: String) -> Response {
    let channel = nojson::RawJson::parse(&body)
        .ok()
        .and_then(|json| {
            json.value()
                .to_member("channel")
                .ok()
                .and_then(|member| member.optional())
                .and_then(|value| String::try_from(value).ok())
        })
        .unwrap_or_default();

    json_response(
        nojson::json(|f| {
            f.object(|f| {
                f.member("ok", true)?;
                f.member("channel", &channel)?;
                f.member("ts", "1700000000.000000")
            })
        })
        .to_string(),
    )
}

async fn mock_get_upload_url(State(state): State<MockSlackState>) -> Response {
    let file_id = "F123456";
    let upload_url = format!("{}/upload/{}", state.public_base_url, file_id);
    json_response(
        nojson::json(|f| {
            f.object(|f| {
                f.member("ok", true)?;
                f.member("file_id", file_id)?;
                f.member("upload_url", &upload_url)
            })
        })
        .to_string(),
    )
}

async fn mock_upload_content(Path(_id): Path<String>, body: axum::body::Bytes) -> StatusCode {
    let _ = body;
    StatusCode::OK
}

async fn mock_complete_upload(_body: String) -> Response {
    json_response(
        nojson::json(|f| {
            f.object(|f| {
                f.member("ok", true)?;
                f.member("files", nojson::array(|_| Ok(())))
            })
        })
        .to_string(),
    )
}
