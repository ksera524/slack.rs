use crate::{
    config::state::AppState,
    handlers::slack_handler::{post_message, upload_image_raw, upload_pdf_raw},
};
use axum::{Router, routing::post};

pub fn create_slack_routes() -> Router<AppState> {
    Router::new()
        .route("/slack/message", post(post_message))
        .route("/slack/upload/image", post(upload_image_raw))
        .route("/slack/upload/pdf", post(upload_pdf_raw))
}
