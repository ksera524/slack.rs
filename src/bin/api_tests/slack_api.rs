use tanu::{check, check_eq, eyre, http::Client};

use crate::support;

fn json_body_with<K, V>(pairs: &[(K, V)]) -> String
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    nojson::json(|f| {
        f.object(|f| {
            for (key, value) in pairs {
                f.member(key.as_ref(), value.as_ref())?;
            }
            Ok(())
        })
    })
    .to_string()
}

fn get_bool(body: &str, key: &str) -> bool {
    nojson::RawJson::parse(body)
        .ok()
        .and_then(|json| {
            json.value()
                .to_member(key)
                .ok()
                .and_then(|member| member.optional())
                .and_then(|value| bool::try_from(value).ok())
        })
        .unwrap_or(false)
}

fn get_str(body: &str, key: &str) -> String {
    nojson::RawJson::parse(body)
        .ok()
        .and_then(|json| {
            json.value()
                .to_member(key)
                .ok()
                .and_then(|member| member.optional())
                .and_then(|value| String::try_from(value).ok())
        })
        .unwrap_or_default()
}

fn get_i64(body: &str, key: &str) -> i64 {
    nojson::RawJson::parse(body)
        .ok()
        .and_then(|json| {
            json.value()
                .to_member(key)
                .ok()
                .and_then(|member| member.optional())
                .and_then(|value| i64::try_from(value).ok())
        })
        .unwrap_or_default()
}

#[tanu::test]
async fn post_message_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/message", app.base_url))
        .header("content-type", "application/json")
        .body(json_body_with(&[("channel", "C123456"), ("text", "hello")]))
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.json::<String>().await?;
    check!(get_bool(&body, "ok"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn openapi_json_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .get(format!("{}/openapi.json", app.base_url))
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.text().await?;
    check_eq!("3.0.3", get_str(&body, "openapi"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_image_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/slack/upload/image?channel=C123456&file_name=hello.png",
            app.base_url
        ))
        .header("content-type", "image/png")
        .body(vec![
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n', 0x00,
        ])
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.json::<String>().await?;
    check!(get_bool(&body, "ok"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_pdf_ok() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/slack/upload/pdf?channel=C123456&file_name=doc.pdf",
            app.base_url
        ))
        .header("content-type", "application/pdf")
        .body(b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n".to_vec())
        .send()
        .await?;

    check_eq!(200, response.status().as_u16());

    let body = response.json::<String>().await?;
    check!(get_bool(&body, "ok"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_pdf_invalid_content_type() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/upload/pdf?channel=C123456", app.base_url))
        .header("content-type", "text/plain")
        .body(b"%PDF-1.7".to_vec())
        .send()
        .await?;

    check_eq!(400, response.status().as_u16());

    let body = response.text().await?;
    check_eq!("about:blank", get_str(&body, "type"));
    check_eq!("Bad Request", get_str(&body, "title"));
    check_eq!(400, get_i64(&body, "status"));
    check_eq!(
        "Unsupported Content-Type. Use application/pdf",
        get_str(&body, "detail")
    );

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_pdf_invalid_signature() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/upload/pdf?channel=C123456", app.base_url))
        .header("content-type", "application/pdf")
        .body(b"not_a_pdf".to_vec())
        .send()
        .await?;

    check_eq!(400, response.status().as_u16());

    let body = response.text().await?;
    check_eq!("about:blank", get_str(&body, "type"));
    check_eq!("Bad Request", get_str(&body, "title"));
    check_eq!(400, get_i64(&body, "status"));
    check_eq!("Body is not a valid PDF document", get_str(&body, "detail"));

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}

#[tanu::test]
async fn upload_pdf_missing_channel() -> eyre::Result<()> {
    let mock_slack = support::start_mock_slack().await?;
    let mock_s3 = support::start_mock_s3().await?;
    let app = support::start_app(mock_slack.base_url.clone(), mock_s3.base_url.clone()).await?;

    let client = Client::new();
    let response = client
        .post(format!("{}/slack/upload/pdf", app.base_url))
        .header("content-type", "application/pdf")
        .body(b"%PDF-1.7".to_vec())
        .send()
        .await?;

    check_eq!(400, response.status().as_u16());

    let body = response.text().await?;
    check_eq!("about:blank", get_str(&body, "type"));
    check_eq!("Bad Request", get_str(&body, "title"));
    check_eq!(400, get_i64(&body, "status"));
    check_eq!(
        "Missing required query parameter 'channel'",
        get_str(&body, "detail")
    );

    app.shutdown();
    mock_s3.shutdown();
    mock_slack.shutdown();
    Ok(())
}
