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
    let base64_size = payload.file_data_base64.len();
    debug!(
        file_name = %payload.file_name,
        channel = %payload.channel,
        base64_size = base64_size,
        "Processing file upload request"
    );

    // 大容量ファイルの場合の処理時間警告
    if base64_size > 50_000_000 {  // 50MB以上
        info!(
            file_name = %payload.file_name,
            base64_size = base64_size,
            "Processing large file upload (>50MB)"
        );
    }

    let start = Instant::now();

    let file_data = BASE64_STANDARD
        .decode(&payload.file_data_base64)
        .map_err(|e| {
            warn!(
                error = %e,
                file_name = %payload.file_name,
                base64_size = payload.file_data_base64.len(),
                "Failed to decode base64 file data"
            );
            ApiError::BadRequest("Failed to decode base64 file data".to_string())
        })?;

    let file_size = file_data.len();
    debug!(
        file_name = %payload.file_name,
        file_size = file_size,
        compression_ratio = (file_size as f64 / base64_size as f64),
        "File decoded successfully"
    );

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
