use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use std::error::Error as StdError;
use tracing::{debug, error, info, instrument, warn};

fn get_required_string(
    root: nojson::RawJsonValue<'_, '_>,
    name: &str,
) -> Result<String, Box<dyn StdError>> {
    let value = root
        .to_member(name)
        .map_err(Box::<dyn StdError>::from)?
        .required()
        .map_err(Box::<dyn StdError>::from)?;
    let converted = String::try_from(value).map_err(Box::<dyn StdError>::from)?;
    Ok(converted)
}

fn get_required_bool(
    root: nojson::RawJsonValue<'_, '_>,
    name: &str,
) -> Result<bool, Box<dyn StdError>> {
    let value = root
        .to_member(name)
        .map_err(Box::<dyn StdError>::from)?
        .required()
        .map_err(Box::<dyn StdError>::from)?;
    let converted = bool::try_from(value).map_err(Box::<dyn StdError>::from)?;
    Ok(converted)
}

fn get_optional_string(root: nojson::RawJsonValue<'_, '_>, name: &str) -> Option<String> {
    let value = root.to_member(name).ok()?.optional()?;
    String::try_from(value).ok()
}

#[instrument(skip(client, slack_bot_token, text), fields(channel = %channel))]
pub async fn post_message(
    client: &Client,
    slack_bot_token: &str,
    slack_api_base_url: &str,
    channel: &str,
    text: &str,
) -> Result<String, Box<dyn StdError>> {
    let url = format!("{}/chat.postMessage", slack_api_base_url);

    let payload = nojson::json(|f| {
        f.object(|f| {
            f.member("channel", channel)?;
            f.member("text", text)
        })
    });

    debug!(
        api_endpoint = "chat.postMessage",
        channel = %channel,
        "Calling Slack API"
    );

    let response = client
        .post(url)
        .bearer_auth(slack_bot_token)
        .header(CONTENT_TYPE, "application/json")
        .body(payload.to_string())
        .send()
        .await?
        .text()
        .await?;

    let parsed = nojson::RawJson::parse(&response)?;
    let root = parsed.value();
    let ok = get_required_bool(root, "ok")?;

    if !ok {
        let error_message =
            get_optional_string(root, "error").unwrap_or_else(|| "unknown_error".to_string());
        warn!(
            error = %error_message,
            channel = %channel,
            "Slack API returned error response"
        );
        return Err(Box::new(std::io::Error::other(error_message)));
    }

    debug!(
        channel = %channel,
        "Slack API call successful"
    );

    Ok(response)
}

#[instrument(skip(client, slack_bot_token, file_data), fields(file_name = %file_name, file_size = file_data.len()))]
pub async fn upload_file(
    client: &Client,
    slack_bot_token: &str,
    slack_api_base_url: &str,
    file_name: &str,
    file_data: &[u8],
) -> Result<(String, String), Box<dyn StdError>> {
    let url = format!("{}/files.getUploadURLExternal", slack_api_base_url);
    let params = [
        ("filename", file_name),
        ("length", &file_data.len().to_string()),
    ];

    debug!(
        api_endpoint = "files.getUploadURLExternal",
        file_name = %file_name,
        file_size = file_data.len(),
        "Getting upload URL from Slack"
    );

    let response = client
        .get(url)
        .bearer_auth(slack_bot_token)
        .query(&params)
        .send()
        .await?
        .text()
        .await?;

    let parsed = nojson::RawJson::parse(&response)?;
    let root = parsed.value();
    let ok = get_required_bool(root, "ok")?;

    if !ok {
        let error_message =
            get_optional_string(root, "error").unwrap_or_else(|| "unknown_error".to_string());
        error!(
            error = %error_message,
            file_name = %file_name,
            "Failed to get upload URL from Slack"
        );
        return Err(Box::new(std::io::Error::other(error_message)));
    }

    let upload_url = get_required_string(root, "upload_url")?;
    let file_id = get_required_string(root, "file_id")?;

    debug!(
        file_id = %file_id,
        "Uploading file content to Slack"
    );

    client
        .post(&upload_url)
        .header("Content-Type", "application/octet-stream")
        .body(file_data.to_vec())
        .send()
        .await?;

    debug!(
        file_id = %file_id,
        "File upload completed"
    );

    Ok((file_id, upload_url))
}

#[instrument(skip(client, token, file_data), fields(file_name = %file_name, channel = %channel, file_size = file_data.len()))]
pub async fn send_single_file_to_slack(
    client: &Client,
    token: &str,
    slack_api_base_url: &str,
    file_data: &[u8],
    file_name: &str,
    channel: &str,
) -> Result<String, Box<dyn StdError>> {
    let (file_id, _upload_url) =
        upload_file(client, token, slack_api_base_url, file_name, file_data).await?;

    let url = format!("{}/files.completeUploadExternal", slack_api_base_url);

    let data = nojson::json(|f| {
        f.object(|f| {
            f.member(
                "files",
                nojson::array(|f| {
                    f.element(nojson::object(|f| {
                        f.member("id", &file_id)?;
                        f.member("title", file_name)
                    }))
                }),
            )?;
            f.member("channel_id", channel)
        })
    });

    debug!(
        api_endpoint = "files.completeUploadExternal",
        file_id = %file_id,
        channel = %channel,
        "Completing file upload to Slack"
    );

    let response_text = client
        .post(url)
        .bearer_auth(token)
        .header(CONTENT_TYPE, "application/json")
        .body(data.to_string())
        .send()
        .await?
        .text()
        .await?;

    let parsed = nojson::RawJson::parse(&response_text)?;
    let root = parsed.value();
    let ok = get_required_bool(root, "ok")?;

    if ok {
        info!(
            file_id = %file_id,
            file_name = %file_name,
            channel = %channel,
            "File successfully shared to Slack channel"
        );
        Ok(response_text)
    } else {
        error!(
            error = %get_optional_string(root, "error").unwrap_or_else(|| "unknown_error".to_string()),
            file_id = %file_id,
            channel = %channel,
            "Failed to complete file upload"
        );
        let error_message =
            get_optional_string(root, "error").unwrap_or_else(|| "unknown_error".to_string());
        Err(Box::new(std::io::Error::other(error_message)))
    }
}
