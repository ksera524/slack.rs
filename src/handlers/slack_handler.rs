use axum::{Json, extract::State, response::IntoResponse};
use base64::Engine;
use serde::Deserialize;
use tracing::{debug, error, info, instrument, warn};
use std::time::Instant;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use crate::{config::state::AppState, errors::api_error::ApiError, service::slack_service};

#[derive(Deserialize)]
pub struct SlackMessageRequest {
    pub channel: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct SlackFileUploadRequest {
    pub file_name: String,
    pub file_data_base64: String,
    pub channel: String,
}

#[instrument(skip(app_state, payload), fields(
    channel = %payload.channel,
    text_length = payload.text.len(),
    request_id = tracing::field::Empty
))]
pub async fn post_message(
    State(app_state): State<AppState>,
    Json(payload): Json<SlackMessageRequest>,
) -> Result<Json<String>, ApiError> {
    debug!(
        channel = %payload.channel,
        text_length = payload.text.len(),
        "Processing Slack message request"
    );

    let start = Instant::now();

    let response_text = slack_service::post_message(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &payload.channel,
        &payload.text,
    )
    .await
    .map_err(|e| {
        error!(
            error = %e,
            channel = %payload.channel,
            "Failed to post message to Slack"
        );
        ApiError::InternalServerError(e.to_string())
    })?;

    let duration = start.elapsed();
    info!(
        channel = %payload.channel,
        duration_ms = duration.as_millis() as u64,
        "Successfully posted message to Slack"
    );

    Ok(Json(response_text))
}

#[instrument(skip(app_state, payload), fields(
    channel = %payload.channel,
    file_name = %payload.file_name,
    file_size = payload.file_data_base64.len(),
    request_id = tracing::field::Empty
))]
pub async fn upload_file_base64(
    State(app_state): State<AppState>,
    Json(payload): Json<SlackFileUploadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        file_name = %payload.file_name,
        channel = %payload.channel,
        base64_size = payload.file_data_base64.len(),
        "Processing file upload request"
    );

    let start = Instant::now();

    let file_data = BASE64_STANDARD
        .decode(&payload.file_data_base64)
        .map_err(|e| {
            warn!(
                error = %e,
                file_name = %payload.file_name,
                "Failed to decode base64 file data"
            );
            ApiError::BadRequest("Failed to decode base64 file data".to_string())
        })?;

    let response_text = slack_service::send_single_file_to_slack(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &file_data,
        &payload.file_name,
        &payload.channel,
    )
    .await
    .map_err(|e| {
        error!(
            error = %e,
            file_name = %payload.file_name,
            channel = %payload.channel,
            "Failed to upload file to Slack"
        );
        ApiError::InternalServerError(e.to_string())
    })?;

    let duration = start.elapsed();
    info!(
        file_name = %payload.file_name,
        channel = %payload.channel,
        file_size = file_data.len(),
        duration_ms = duration.as_millis() as u64,
        "Successfully uploaded file to Slack"
    );

    Ok(Json(response_text))
}
