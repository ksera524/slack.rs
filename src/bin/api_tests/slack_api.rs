use serde_json::{Value, json};
use tanu::{check, check_eq, eyre, http::Client};

use crate::support;

#[tanu::test]
async fn post_message_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let app = support::start_app(mock_slack.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/message", app.base_url))
        .json(&json!({
            "channel": "C123456",
            "text": "hello"
        }))
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.json::<String>().await?;
    let slack_response: Value = serde_json::from_str(&body)?;
    check!(slack_response["ok"].as_bool().unwrap_or(false));

    app.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_base64_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let app = support::start_app(mock_slack.base_url.clone()).await?;

    let client = Client::new();
    let file_data = support::encode_base64(b"hello");
    let response = client
        .post(format!("{}/slack/upload_base64", app.base_url))
        .json(&json!({
            "file_name": "hello.txt",
            "file_data_base64": file_data,
            "channel": "C123456"
        }))
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.json::<String>().await?;
    let slack_response: Value = serde_json::from_str(&body)?;
    check!(slack_response["ok"].as_bool().unwrap_or(false));

    app.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_base64_invalid() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let app = support::start_app(mock_slack.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/upload_base64", app.base_url))
        .json(&json!({
            "file_name": "bad.txt",
            "file_data_base64": "not_base64",
            "channel": "C123456"
        }))
        .send()
        .await?;

    check_eq!(400, response.status().as_u16());

    let body = response.json::<Value>().await?;
    check_eq!("about:blank", body["type"].as_str().unwrap_or(""));
    check_eq!("Bad Request", body["title"].as_str().unwrap_or(""));
    check_eq!(400, body["status"].as_i64().unwrap_or_default());
    check_eq!(
        "Failed to decode base64 file data",
        body["detail"].as_str().unwrap_or("")
    );

    app.shutdown();
    mock_slack.shutdown();
    Ok(())
}
