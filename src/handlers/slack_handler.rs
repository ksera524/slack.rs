use axum::{Json, extract::State, response::IntoResponse};
use base64::Engine;
use serde::Deserialize;
use tracing::info;

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

pub async fn post_message(
    State(app_state): State<AppState>,
    Json(payload): Json<SlackMessageRequest>,
) -> Result<Json<String>, ApiError> {
    info!(text = %payload.text, channel = %payload.channel,"Received a message to post to Slack");

    let response_text = slack_service::post_message(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &payload.channel,
        &payload.text,
    )
    .await
    .map_err(|e| {
        info!("Failed to post message to Slack: {:?}", e);
        ApiError::InternalServerError(e.to_string())
    })?;

    Ok(Json(response_text))
}

pub async fn upload_file_base64(
    State(app_state): State<AppState>,
    Json(payload): Json<SlackFileUploadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!(text = %payload.file_name, channel = %payload.channel,"Received a message to post to Slack");

    let file_data = BASE64_STANDARD
        .decode(&payload.file_data_base64)
        .map_err(|e| {
            info!("Failed to decode base64 file data: {:?}", e);
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
        info!("Failed to upload file to Slack: {:?}", e);
        ApiError::InternalServerError(e.to_string())
    })?;

    Ok(Json(response_text))
}
