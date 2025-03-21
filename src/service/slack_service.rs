use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Error};
use serde_json::{Value, json};
use std::error::Error as StdError;

pub async fn post_message(
    client: &Client,
    slack_bot_token: &str,
    channel: &str,
    text: &str,
) -> Result<String, Error> {
    let url = "https://slack.com/api/chat.postMessage";

    let payload = json!({
        "channel": channel,
        "text": text,
    });

    let response = client
        .post(url)
        .bearer_auth(slack_bot_token)
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response.to_string())
}

pub async fn upload_file(
    client: &Client,
    slack_bot_token: &str,
    file_name: &str,
    file_data: &[u8],
) -> Result<(String, String), Box<dyn StdError>> {
    let url = "https://slack.com/api/files.getUploadURLExternal";
    let params = [
        ("filename", file_name),
        ("length", &file_data.len().to_string()),
    ];

    let response = client
        .get(url)
        .bearer_auth(slack_bot_token)
        .query(&params)
        .send()
        .await?
        .text()
        .await?;

    let response: Value = serde_json::from_str(&response)?;

    if !response["ok"].as_bool().unwrap_or(false) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            response["error"].to_string(),
        )));
    }

    let upload_url = response["upload_url"].as_str().unwrap().to_string();
    let file_id = response["file_id"].as_str().unwrap().to_string();

    client
        .post(&upload_url)
        .header("Content-Type", "application/octet-stream")
        .body(file_data.to_vec())
        .send()
        .await?;

    Ok((file_id, upload_url))
}

pub async fn send_single_file_to_slack(
    client: &Client,
    token: &str,
    file_data: &[u8],
    file_name: &str,
    channel: &str,
) -> Result<String, Box<dyn StdError>> {
    let (file_id, _upload_url) = upload_file(client, token, file_name, file_data).await?;

    let url = "https://slack.com/api/files.completeUploadExternal";

    let data = serde_json::json!({
        "files": [{
            "id": file_id,
            "title": file_name,
        }],
        "channel_id": channel,
    });

    let response_text = client
        .post(url)
        .bearer_auth(token)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&data)?)
        .send()
        .await?
        .text()
        .await?;

    let response: Value = serde_json::from_str(&response_text)?;

    if response["ok"].as_bool().unwrap_or(false) {
        Ok(response_text)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            response["error"].to_string(),
        )))
    }
}
