use crate::{
    config::state::AppState,
    handlers::slack_handler::{post_message, upload_file_base64},
};
use axum::{Router, routing::post};

pub fn create_slack_routes() -> Router<AppState> {
    Router::new()
        .route("/slack/message", post(post_message))
        .route("/slack/upload_base64", post(upload_file_base64))
}
