use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use reqwest::{Client, header::HeaderName};
use serde_json::{Value, json};
use shiguredo_s3::{
    Credential, S3Client, S3Config, S3Request, S3Response,
    types::{CompletedMultipartUpload, CompletedPart},
};

use crate::{config::settings::Settings, errors::api_error::ApiError};

pub struct PutObjectInput {
    pub bucket: String,
    pub key: String,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

pub struct GetObjectInput {
    pub bucket: String,
    pub key: String,
}

pub struct HeadObjectInput {
    pub bucket: String,
    pub key: String,
}

pub struct DeleteObjectInput {
    pub bucket: String,
    pub key: String,
}

pub struct ListObjectsV2Input {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
}

pub struct CreateMultipartUploadInput {
    pub bucket: String,
    pub key: String,
    pub content_type: Option<String>,
}

pub struct UploadPartInput {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number: i32,
    pub body: Vec<u8>,
}

pub struct CompleteMultipartUploadInput {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<CompletePartInput>,
}

pub struct CompletePartInput {
    pub part_number: i32,
    pub e_tag: String,
}

pub struct AbortMultipartUploadInput {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

pub struct ListPartsInput {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub max_parts: Option<i32>,
    pub part_number_marker: Option<i32>,
}

pub struct ListMultipartUploadsInput {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_uploads: Option<i32>,
    pub key_marker: Option<String>,
    pub upload_id_marker: Option<String>,
}

pub struct PresignedObjectInput {
    pub bucket: String,
    pub key: String,
    pub expires_in_secs: u64,
}

pub struct CreateBucketInput {
    pub bucket: String,
}

pub struct HeadBucketInput {
    pub bucket: String,
}

pub struct DeleteBucketInput {
    pub bucket: String,
}

pub async fn put_object(
    http_client: &Client,
    settings: &Settings,
    input: PutObjectInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let mut req = s3
        .put_object()
        .bucket(input.bucket)
        .key(input.key)
        .body(input.body);
    if let Some(content_type) = input.content_type {
        req = req.content_type(content_type);
    }

    let request = req
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::PutObjectFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "e_tag": output.e_tag,
        "version_id": output.version_id,
    }))
}

pub async fn get_object(
    http_client: &Client,
    settings: &Settings,
    input: GetObjectInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .get_object()
        .bucket(input.bucket)
        .key(input.key)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::GetObjectFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "file_data_base64": BASE64_STANDARD.encode(&output.body),
        "content_type": output.content_type,
        "content_length": output.content_length,
        "e_tag": output.e_tag,
        "last_modified": output.last_modified,
        "version_id": output.version_id,
        "metadata": output.metadata,
    }))
}

pub async fn head_object(
    http_client: &Client,
    settings: &Settings,
    input: HeadObjectInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .head_object()
        .bucket(input.bucket)
        .key(input.key)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::HeadObjectFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "content_type": output.content_type,
        "content_length": output.content_length,
        "e_tag": output.e_tag,
        "last_modified": output.last_modified,
        "version_id": output.version_id,
        "metadata": output.metadata,
    }))
}

pub async fn delete_object(
    http_client: &Client,
    settings: &Settings,
    input: DeleteObjectInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .delete_object()
        .bucket(input.bucket)
        .key(input.key)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "delete_marker": output.delete_marker,
        "version_id": output.version_id,
    }))
}

pub async fn list_objects_v2(
    http_client: &Client,
    settings: &Settings,
    input: ListObjectsV2Input,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let mut req = s3.list_objects_v2().bucket(input.bucket);
    if let Some(prefix) = input.prefix {
        req = req.prefix(prefix);
    }
    if let Some(delimiter) = input.delimiter {
        req = req.delimiter(delimiter);
    }
    if let Some(max_keys) = input.max_keys {
        req = req.max_keys(max_keys);
    }
    if let Some(continuation_token) = input.continuation_token {
        req = req.continuation_token(continuation_token);
    }
    if let Some(start_after) = input.start_after {
        req = req.start_after(start_after);
    }

    let request = req
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    let contents = output.contents.unwrap_or_default();
    let common_prefixes = output.common_prefixes.unwrap_or_default();

    Ok(json!({
        "is_truncated": output.is_truncated,
        "name": output.name,
        "prefix": output.prefix,
        "delimiter": output.delimiter,
        "max_keys": output.max_keys,
        "key_count": output.key_count,
        "continuation_token": output.continuation_token,
        "next_continuation_token": output.next_continuation_token,
        "start_after": output.start_after,
        "contents": contents.into_iter().map(|obj| {
            json!({
                "key": obj.key,
                "last_modified": obj.last_modified,
                "e_tag": obj.e_tag,
                "size": obj.size,
                "storage_class": obj.storage_class,
            })
        }).collect::<Vec<_>>(),
        "common_prefixes": common_prefixes.into_iter().map(|prefix| {
            json!({ "prefix": prefix.prefix })
        }).collect::<Vec<_>>(),
    }))
}

pub async fn create_multipart_upload(
    http_client: &Client,
    settings: &Settings,
    input: CreateMultipartUploadInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let mut req = s3
        .create_multipart_upload()
        .bucket(input.bucket)
        .key(input.key);
    if let Some(content_type) = input.content_type {
        req = req.content_type(content_type);
    }

    let request = req
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "bucket": output.bucket,
        "key": output.key,
        "upload_id": output.upload_id,
    }))
}

pub async fn upload_part(
    http_client: &Client,
    settings: &Settings,
    input: UploadPartInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .upload_part()
        .bucket(input.bucket)
        .key(input.key)
        .upload_id(input.upload_id)
        .part_number(input.part_number)
        .body(input.body)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::UploadPartFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({ "e_tag": output.e_tag }))
}

pub async fn complete_multipart_upload(
    http_client: &Client,
    settings: &Settings,
    input: CompleteMultipartUploadInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let multipart_upload = CompletedMultipartUpload {
        parts: Some(
            input
                .parts
                .into_iter()
                .map(|part| CompletedPart {
                    part_number: Some(part.part_number),
                    e_tag: Some(part.e_tag),
                })
                .collect(),
        ),
    };

    let request = s3
        .complete_multipart_upload()
        .bucket(input.bucket)
        .key(input.key)
        .upload_id(input.upload_id)
        .multipart_upload(multipart_upload)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "location": output.location,
        "bucket": output.bucket,
        "key": output.key,
        "e_tag": output.e_tag,
        "version_id": output.version_id,
    }))
}

pub async fn abort_multipart_upload(
    http_client: &Client,
    settings: &Settings,
    input: AbortMultipartUploadInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .abort_multipart_upload()
        .bucket(input.bucket)
        .key(input.key)
        .upload_id(input.upload_id)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({ "aborted": true }))
}

pub async fn list_parts(
    http_client: &Client,
    settings: &Settings,
    input: ListPartsInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let mut req = s3
        .list_parts()
        .bucket(input.bucket)
        .key(input.key)
        .upload_id(input.upload_id);
    if let Some(max_parts) = input.max_parts {
        req = req.max_parts(max_parts);
    }
    if let Some(part_number_marker) = input.part_number_marker {
        req = req.part_number_marker(part_number_marker);
    }

    let request = req
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::ListPartsFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;
    let parts = output.parts.unwrap_or_default();

    Ok(json!({
        "bucket": output.bucket,
        "key": output.key,
        "upload_id": output.upload_id,
        "part_number_marker": output.part_number_marker,
        "next_part_number_marker": output.next_part_number_marker,
        "max_parts": output.max_parts,
        "is_truncated": output.is_truncated,
        "storage_class": output.storage_class,
        "parts": parts.into_iter().map(|part| {
            json!({
                "part_number": part.part_number,
                "last_modified": part.last_modified,
                "e_tag": part.e_tag,
                "size": part.size,
            })
        }).collect::<Vec<_>>(),
    }))
}

pub async fn list_multipart_uploads(
    http_client: &Client,
    settings: &Settings,
    input: ListMultipartUploadsInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let mut req = s3.list_multipart_uploads().bucket(input.bucket);
    if let Some(prefix) = input.prefix {
        req = req.prefix(prefix);
    }
    if let Some(delimiter) = input.delimiter {
        req = req.delimiter(delimiter);
    }
    if let Some(max_uploads) = input.max_uploads {
        req = req.max_uploads(max_uploads);
    }
    if let Some(key_marker) = input.key_marker {
        req = req.key_marker(key_marker);
    }
    if let Some(upload_id_marker) = input.upload_id_marker {
        req = req.upload_id_marker(upload_id_marker);
    }

    let request = req
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::ListMultipartUploadsFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;
    let uploads = output.uploads.unwrap_or_default();
    let common_prefixes = output.common_prefixes.unwrap_or_default();

    Ok(json!({
        "bucket": output.bucket,
        "key_marker": output.key_marker,
        "upload_id_marker": output.upload_id_marker,
        "next_key_marker": output.next_key_marker,
        "next_upload_id_marker": output.next_upload_id_marker,
        "prefix": output.prefix,
        "delimiter": output.delimiter,
        "max_uploads": output.max_uploads,
        "is_truncated": output.is_truncated,
        "uploads": uploads.into_iter().map(|upload| {
            json!({
                "upload_id": upload.upload_id,
                "key": upload.key,
                "initiated": upload.initiated,
                "storage_class": upload.storage_class,
            })
        }).collect::<Vec<_>>(),
        "common_prefixes": common_prefixes.into_iter().map(|prefix| {
            json!({ "prefix": prefix.prefix })
        }).collect::<Vec<_>>(),
    }))
}

pub fn presigned_get(settings: &Settings, input: PresignedObjectInput) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let output = s3
        .get_object()
        .bucket(input.bucket)
        .key(input.key)
        .presigned(input.expires_in_secs)
        .map_err(map_s3_input_error_to_api_error)?;

    Ok(json!({
        "url": output.url,
        "method": output.method,
        "headers": output.headers,
    }))
}

pub fn presigned_put(settings: &Settings, input: PresignedObjectInput) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let output = s3
        .put_object()
        .bucket(input.bucket)
        .key(input.key)
        .presigned(input.expires_in_secs)
        .map_err(map_s3_input_error_to_api_error)?;

    Ok(json!({
        "url": output.url,
        "method": output.method,
        "headers": output.headers,
    }))
}

pub async fn list_buckets(http_client: &Client, settings: &Settings) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .list_buckets()
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::ListBucketsFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({
        "continuation_token": output.continuation_token,
        "prefix": output.prefix,
        "buckets": output.buckets.into_iter().map(|bucket| {
            json!({
                "name": bucket.name,
                "creation_date": bucket.creation_date,
                "bucket_region": bucket.bucket_region,
                "bucket_arn": bucket.bucket_arn,
            })
        }).collect::<Vec<_>>(),
    }))
}

pub async fn create_bucket(
    http_client: &Client,
    settings: &Settings,
    input: CreateBucketInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .create_bucket()
        .bucket(input.bucket)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::CreateBucketFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({ "location": output.location }))
}

pub async fn head_bucket(
    http_client: &Client,
    settings: &Settings,
    input: HeadBucketInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .head_bucket()
        .bucket(input.bucket)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    let output = shiguredo_s3::api::HeadBucketFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({ "bucket_region": output.bucket_region }))
}

pub async fn delete_bucket(
    http_client: &Client,
    settings: &Settings,
    input: DeleteBucketInput,
) -> Result<Value, ApiError> {
    let s3 = create_s3_client(settings)?;
    let request = s3
        .delete_bucket()
        .bucket(input.bucket)
        .build_request()
        .map_err(map_s3_input_error_to_api_error)?;
    let response = execute_s3(http_client, request).await?;
    shiguredo_s3::api::DeleteBucketFluentBuilder::parse_response(&response)
        .map_err(map_s3_runtime_error_to_api_error)?;

    Ok(json!({ "deleted": true }))
}

pub fn decode_base64_payload(payload: &str) -> Result<Vec<u8>, ApiError> {
    BASE64_STANDARD
        .decode(payload)
        .map_err(|_| ApiError::BadRequest("Failed to decode base64 data".to_string()))
}

fn create_s3_client(settings: &Settings) -> Result<S3Client, ApiError> {
    let credential = if let Some(session_token) = &settings.s3_session_token {
        Credential::with_session_token(
            settings.s3_access_key_id.clone(),
            settings.s3_secret_access_key.clone(),
            session_token.clone(),
        )
    } else {
        Credential::new(
            settings.s3_access_key_id.clone(),
            settings.s3_secret_access_key.clone(),
        )
    };

    let mut config_builder = S3Config::builder()
        .region(settings.s3_region.clone())
        .credential(credential)
        .use_path_style(settings.s3_use_path_style)
        .ignore_cert_check(settings.s3_ignore_cert_check);
    if let Some(endpoint) = &settings.s3_endpoint {
        config_builder = config_builder.endpoint(endpoint.clone());
    }

    let config = config_builder
        .build()
        .map_err(map_s3_input_error_to_api_error)?;

    Ok(S3Client::new(config))
}

async fn execute_s3(http_client: &Client, request: S3Request) -> Result<S3Response, ApiError> {
    let url = build_s3_url(&request)?;
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|e| ApiError::InternalServerError(format!("Invalid HTTP method: {e}")))?;

    let mut req_builder = http_client.request(method, url);

    for (name, value) in &request.headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| ApiError::InternalServerError(format!("Invalid request header: {e}")))?;
        req_builder = req_builder.header(header_name, value);
    }

    req_builder = req_builder.body(request.body);

    let response = req_builder
        .send()
        .await
        .map_err(|e| ApiError::InternalServerError(format!("S3 HTTP request failed: {e}")))?;

    let status_code = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value
                    .to_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|_| String::from_utf8_lossy(value.as_bytes()).to_string()),
            )
        })
        .collect::<Vec<_>>();
    let body = response
        .bytes()
        .await
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to read S3 response body: {e}"))
        })?
        .to_vec();

    Ok(S3Response {
        status_code,
        headers,
        body,
    })
}

fn build_s3_url(request: &S3Request) -> Result<String, ApiError> {
    let scheme = if request.https { "https" } else { "http" };
    let uri = if request.uri.starts_with('/') {
        request.uri.as_str().to_owned()
    } else {
        format!("/{}", request.uri)
    };
    let url = format!("{scheme}://{}:{}{}", request.host, request.port, uri);

    reqwest::Url::parse(&url)
        .map(|_| url)
        .map_err(|e| ApiError::InternalServerError(format!("Invalid S3 URL: {e}")))
}

fn map_s3_input_error_to_api_error(error: shiguredo_s3::Error) -> ApiError {
    match error {
        shiguredo_s3::Error::InvalidInput(message) => ApiError::BadRequest(message),
        other => ApiError::InternalServerError(other.to_string()),
    }
}

fn map_s3_runtime_error_to_api_error(error: shiguredo_s3::Error) -> ApiError {
    ApiError::InternalServerError(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_s3_url, decode_base64_payload};
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
    use proptest::prop_assert_eq;
    use proptest::{
        prelude::{ProptestConfig, any},
        proptest,
        strategy::Strategy,
    };
    use shiguredo_s3::S3Response;

    fn arbitrary_path_string() -> impl proptest::strategy::Strategy<Value = String> {
        "[A-Za-z0-9_/-]{1,64}".prop_map(|s| format!("/{s}"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn base64_roundtrip_is_lossless(payload in any::<Vec<u8>>()) {
            let encoded = BASE64_STANDARD.encode(&payload);
            let decoded = decode_base64_payload(&encoded).expect("base64 decode should succeed");
            prop_assert_eq!(decoded, payload);
        }

        #[test]
        fn build_s3_url_preserves_scheme_host_port_and_uri(
            host in "[a-z]{1,8}(?:\\.[a-z]{1,8}){0,2}",
            port in any::<u16>(),
            https in any::<bool>(),
            uri in arbitrary_path_string(),
        ) {
            let request = shiguredo_s3::S3Request {
                method: "GET".to_string(),
                uri: uri.clone(),
                headers: Vec::new(),
                body: Vec::new(),
                host: host.clone(),
                port,
                https,
                ignore_cert_check: false,
                expect_no_body: false,
            };

            let built = build_s3_url(&request).expect("url should be valid");
            let parsed = reqwest::Url::parse(&built).expect("built URL should parse");

            let expected_scheme = if https { "https" } else { "http" };
            prop_assert_eq!(parsed.scheme(), expected_scheme);
            prop_assert_eq!(parsed.host_str(), Some(host.as_str()));
            prop_assert_eq!(parsed.port_or_known_default(), Some(port));
            prop_assert_eq!(parsed.path(), uri);
        }
    }

    #[test]
    fn invalid_base64_is_rejected() {
        let err = decode_base64_payload("not_base64").expect_err("invalid base64 should fail");
        assert_eq!(err.to_string(), "Bad Request: Failed to decode base64 data");
    }

    #[test]
    fn s3_response_header_lookup_is_case_insensitive() {
        let response = S3Response {
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: Vec::new(),
        };
        assert_eq!(response.get_header("content-type"), Some("text/plain"));
        assert_eq!(response.get_header("CONTENT-TYPE"), Some("text/plain"));
    }
}
