use serde_json::json;
use tanu::{check, check_eq, eyre, http::Client};

use crate::support;

#[tanu::test]
async fn s3_put_and_get_object_base64_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let file_data_base64 = support::encode_base64(b"hello-s3");

    let put_response = client
        .post(format!("{}/s3/put_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "docs/hello.txt",
            "file_data_base64": file_data_base64,
            "content_type": "text/plain"
        }))
        .send()
        .await?;
    check_eq!(200, put_response.status().as_u16());

    let get_response = client
        .post(format!("{}/s3/get_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "docs/hello.txt"
        }))
        .send()
        .await?;

    check_eq!(200, get_response.status().as_u16());
    let get_body = get_response.json::<serde_json::Value>().await?;
    check_eq!(
        support::encode_base64(b"hello-s3"),
        get_body["file_data_base64"].as_str().unwrap_or("")
    );
    check_eq!(
        "text/plain",
        get_body["content_type"].as_str().unwrap_or("")
    );

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn s3_list_objects_v2_returns_uploaded_object() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let file_data_base64 = support::encode_base64(b"listable");

    let _ = client
        .post(format!("{}/s3/put_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "prefix/list.txt",
            "file_data_base64": file_data_base64
        }))
        .send()
        .await?;

    let list_response = client
        .post(format!("{}/s3/list_objects_v2", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "prefix": "prefix/"
        }))
        .send()
        .await?;

    check_eq!(200, list_response.status().as_u16());
    let body = list_response.json::<serde_json::Value>().await?;
    let keys = body["contents"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry["key"].as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    check!(keys.contains(&"prefix/list.txt"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn s3_put_object_base64_invalid_returns_400() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/s3/put_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "docs/bad.txt",
            "file_data_base64": "invalid_base64"
        }))
        .send()
        .await?;

    check_eq!(400, response.status().as_u16());
    let body = response.json::<serde_json::Value>().await?;
    check_eq!("about:blank", body["type"].as_str().unwrap_or(""));
    check_eq!("Bad Request", body["title"].as_str().unwrap_or(""));
    check_eq!(
        "Failed to decode base64 data",
        body["detail"].as_str().unwrap_or("")
    );

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn s3_preview_object_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let file_data_base64 = support::encode_base64(b"preview-data");

    let _ = client
        .post(format!("{}/s3/put_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "preview/image.png",
            "file_data_base64": file_data_base64,
            "content_type": "image/png"
        }))
        .send()
        .await?;

    let preview_response = client
        .get(format!(
            "{}/s3/preview/team-a/preview/image.png",
            app.base_url
        ))
        .send()
        .await?;

    check_eq!(200, preview_response.status().as_u16());
    check_eq!(
        Some("image/png"),
        preview_response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
    );
    check_eq!("preview-data", preview_response.text().await?);

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn s3_preview_object_not_found() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let preview_response = client
        .get(format!(
            "{}/s3/preview/team-a/missing/file.png",
            app.base_url
        ))
        .send()
        .await?;

    check_eq!(404, preview_response.status().as_u16());

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn s3_preview_pdf_forces_inline_disposition() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let file_data_base64 = support::encode_base64(b"%PDF-1.7 mock");

    let _ = client
        .post(format!("{}/s3/put_object_base64", app.base_url))
        .json(&json!({
            "bucket": "team-a",
            "key": "preview/document.pdf",
            "file_data_base64": file_data_base64,
            "content_type": "application/octet-stream"
        }))
        .send()
        .await?;

    let preview_response = client
        .get(format!(
            "{}/s3/preview/team-a/preview/document.pdf",
            app.base_url
        ))
        .send()
        .await?;

    check_eq!(200, preview_response.status().as_u16());
    check_eq!(
        Some("application/pdf"),
        preview_response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
    );
    check_eq!(
        Some("inline; filename=\"document.pdf\""),
        preview_response
            .headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
    );
    check_eq!("%PDF-1.7 mock", preview_response.text().await?);

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}
