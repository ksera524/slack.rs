use crate::{
    config::state::AppState,
    handlers::slack_handler::{post_message, upload_file_base64},
};
use axum::{extract::DefaultBodyLimit, routing::post, Router};

pub fn create_slack_routes() -> Router<AppState> {
    Router::new()
        .layer(DefaultBodyLimit::disable())
        .route("/slack/message", post(post_message))
        .route("/slack/upload_base64", post(upload_file_base64))
}
