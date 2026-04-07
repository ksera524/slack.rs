use axum::{
    Json,
    body::Bytes,
    extract::{RawQuery, State},
    http::{HeaderMap, header::CONTENT_TYPE},
    response::IntoResponse,
};
use shiguredo_http11::uri::percent_decode;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

use crate::{config::state::AppState, errors::api_error::ApiError, service::slack_service};

pub struct SlackMessageRequest {
    pub channel: String,
    pub text: String,
}

struct UploadQuery {
    pub channel: String,
    pub file_name: Option<String>,
}

fn parse_message_request(body: &str) -> Result<SlackMessageRequest, ApiError> {
    let json = nojson::RawJson::parse(body)
        .map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {e}")))?;
    let root = json.value();
    let channel: String = root
        .to_member("channel")
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .required()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .try_into()
        .map_err(|e| ApiError::BadRequest(format!("Invalid 'channel': {e}")))?;
    let text: String = root
        .to_member("text")
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .required()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .try_into()
        .map_err(|e| ApiError::BadRequest(format!("Invalid 'text': {e}")))?;

    Ok(SlackMessageRequest { channel, text })
}

fn parse_upload_query(raw_query: Option<&str>) -> Result<UploadQuery, ApiError> {
    let query = raw_query.unwrap_or_default();
    let mut channel = None;
    let mut file_name = None;

    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_query_component(raw_key)?;
        let value = decode_query_component(raw_value)?;

        match key.as_ref() {
            "channel" => channel = Some(value),
            "file_name" => file_name = Some(value),
            _ => {}
        }
    }

    let channel = channel.filter(|c| !c.trim().is_empty()).ok_or_else(|| {
        ApiError::BadRequest("Missing required query parameter 'channel'".to_string())
    })?;

    let file_name = file_name.and_then(|name| {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    Ok(UploadQuery { channel, file_name })
}

fn decode_query_component(value: &str) -> Result<String, ApiError> {
    let replaced = value.replace('+', " ");
    percent_decode(&replaced).map_err(|_| ApiError::BadRequest("Invalid query string".to_string()))
}

fn content_type(headers: &HeaderMap) -> Result<String, ApiError> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or_else(|| ApiError::BadRequest("Missing Content-Type header".to_string()))?
        .to_str()
        .map_err(|_| ApiError::BadRequest("Invalid Content-Type header".to_string()))?;

    let normalized = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();

    Ok(normalized)
}

fn looks_like_pdf(data: &[u8]) -> bool {
    data.starts_with(b"%PDF-")
}

fn looks_like_png(data: &[u8]) -> bool {
    data.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'])
}

fn looks_like_jpeg(data: &[u8]) -> bool {
    data.starts_with(&[0xFF, 0xD8, 0xFF])
}

fn looks_like_gif(data: &[u8]) -> bool {
    data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")
}

fn looks_like_webp(data: &[u8]) -> bool {
    data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP"
}

fn ensure_image_content(content_type: &str, data: &[u8]) -> Result<&'static str, ApiError> {
    let extension = match content_type {
        "image/png" => {
            if !looks_like_png(data) {
                return Err(ApiError::BadRequest(
                    "Body is not a valid PNG image".to_string(),
                ));
            }
            "png"
        }
        "image/jpeg" => {
            if !looks_like_jpeg(data) {
                return Err(ApiError::BadRequest(
                    "Body is not a valid JPEG image".to_string(),
                ));
            }
            "jpg"
        }
        "image/gif" => {
            if !looks_like_gif(data) {
                return Err(ApiError::BadRequest(
                    "Body is not a valid GIF image".to_string(),
                ));
            }
            "gif"
        }
        "image/webp" => {
            if !looks_like_webp(data) {
                return Err(ApiError::BadRequest(
                    "Body is not a valid WEBP image".to_string(),
                ));
            }
            "webp"
        }
        _ => {
            return Err(ApiError::BadRequest(
                "Unsupported image Content-Type. Use image/png, image/jpeg, image/gif, or image/webp"
                    .to_string(),
            ));
        }
    };

    Ok(extension)
}

fn build_default_name(prefix: &str, extension: &str) -> String {
    format!("{prefix}.{extension}")
}

fn validate_file_not_empty(file_data: &[u8]) -> Result<(), ApiError> {
    if file_data.is_empty() {
        return Err(ApiError::BadRequest(
            "Request body must not be empty".to_string(),
        ));
    }
    Ok(())
}

#[instrument(skip(app_state, body), fields(request_id = tracing::field::Empty))]
pub async fn post_message(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Json<String>, ApiError> {
    let payload = parse_message_request(&body)?;

    debug!(
        channel = %payload.channel,
        text_length = payload.text.len(),
        "Processing Slack message request"
    );

    let start = Instant::now();

    let response_text = slack_service::post_message(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &app_state.settings.slack_api_base_url,
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

#[instrument(skip(app_state, headers, body), fields(request_id = tracing::field::Empty))]
pub async fn upload_image_raw(
    State(app_state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let payload = parse_upload_query(raw_query.as_deref())?;
    let content_type = content_type(&headers)?;
    validate_file_not_empty(&body)?;
    let extension = ensure_image_content(&content_type, &body)?;
    let file_name = payload
        .file_name
        .unwrap_or_else(|| build_default_name("image-upload", extension));

    debug!(
        file_name = %file_name,
        channel = %payload.channel,
        content_type = %content_type,
        file_size = body.len(),
        "Processing raw image upload request"
    );

    let start = Instant::now();

    let response_text = slack_service::send_single_file_to_slack(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &app_state.settings.slack_api_base_url,
        &body,
        &file_name,
        &payload.channel,
    )
    .await
    .map_err(|e| {
        error!(
            error = %e,
            file_name = %file_name,
            channel = %payload.channel,
            "Failed to upload image to Slack"
        );
        ApiError::InternalServerError(e.to_string())
    })?;

    let duration = start.elapsed();
    info!(
        file_name = %file_name,
        channel = %payload.channel,
        file_size = body.len(),
        duration_ms = duration.as_millis() as u64,
        "Successfully uploaded image to Slack"
    );

    Ok(Json(response_text))
}

#[instrument(skip(app_state, headers, body), fields(request_id = tracing::field::Empty))]
pub async fn upload_pdf_raw(
    State(app_state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let payload = parse_upload_query(raw_query.as_deref())?;
    let content_type = content_type(&headers)?;
    validate_file_not_empty(&body)?;

    if content_type != "application/pdf" {
        return Err(ApiError::BadRequest(
            "Unsupported Content-Type. Use application/pdf".to_string(),
        ));
    }

    if !looks_like_pdf(&body) {
        warn!(channel = %payload.channel, "PDF signature check failed");
        return Err(ApiError::BadRequest(
            "Body is not a valid PDF document".to_string(),
        ));
    }

    let file_name = payload
        .file_name
        .unwrap_or_else(|| build_default_name("document", "pdf"));

    debug!(
        file_name = %file_name,
        channel = %payload.channel,
        file_size = body.len(),
        "Processing raw PDF upload request"
    );

    let start = Instant::now();

    let response_text = slack_service::send_single_file_to_slack(
        &app_state.client,
        &app_state.settings.slack_bot_token,
        &app_state.settings.slack_api_base_url,
        &body,
        &file_name,
        &payload.channel,
    )
    .await
    .map_err(|e| {
        error!(
            error = %e,
            file_name = %file_name,
            channel = %payload.channel,
            "Failed to upload PDF to Slack"
        );
        ApiError::InternalServerError(e.to_string())
    })?;

    let duration = start.elapsed();
    info!(
        file_name = %file_name,
        channel = %payload.channel,
        file_size = body.len(),
        duration_ms = duration.as_millis() as u64,
        "Successfully uploaded PDF to Slack"
    );

    Ok(Json(response_text))
}
