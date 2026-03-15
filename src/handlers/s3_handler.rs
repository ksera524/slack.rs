use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderName, HeaderValue, StatusCode},
    response::Response,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    config::state::AppState,
    errors::api_error::ApiError,
    service::s3_service::{
        self, AbortMultipartUploadInput, CompleteMultipartUploadInput, CompletePartInput,
        CreateBucketInput, CreateMultipartUploadInput, DeleteBucketInput, DeleteObjectInput,
        GetObjectInput, HeadBucketInput, HeadObjectInput, ListMultipartUploadsInput,
        ListObjectsV2Input, ListPartsInput, PresignedObjectInput, ProxyObjectInput, PutObjectInput,
        UploadPartInput,
    },
};

#[derive(Deserialize)]
pub struct PutObjectBase64Request {
    pub bucket: String,
    pub key: String,
    pub file_data_base64: String,
    pub content_type: Option<String>,
}

#[derive(Deserialize)]
pub struct GetObjectRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct HeadObjectRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct DeleteObjectRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct ListObjectsV2Request {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub content_type: Option<String>,
}

#[derive(Deserialize)]
pub struct UploadPartBase64Request {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number: i32,
    pub part_data_base64: String,
}

#[derive(Deserialize)]
pub struct CompletePartRequest {
    pub part_number: i32,
    pub e_tag: String,
}

#[derive(Deserialize)]
pub struct CompleteMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<CompletePartRequest>,
}

#[derive(Deserialize)]
pub struct AbortMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

#[derive(Deserialize)]
pub struct ListPartsRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub max_parts: Option<i32>,
    pub part_number_marker: Option<i32>,
}

#[derive(Deserialize)]
pub struct ListMultipartUploadsRequest {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_uploads: Option<i32>,
    pub key_marker: Option<String>,
    pub upload_id_marker: Option<String>,
}

#[derive(Deserialize)]
pub struct PresignedObjectRequest {
    pub bucket: String,
    pub key: String,
    pub expires_in_secs: Option<u64>,
}

#[derive(Deserialize)]
pub struct BucketRequest {
    pub bucket: String,
}

pub async fn put_object_base64(
    State(app_state): State<AppState>,
    Json(payload): Json<PutObjectBase64Request>,
) -> Result<Json<Value>, ApiError> {
    let body = s3_service::decode_base64_payload(&payload.file_data_base64)?;
    let result = s3_service::put_object(
        &app_state.client,
        &app_state.settings,
        PutObjectInput {
            bucket: payload.bucket,
            key: payload.key,
            body,
            content_type: payload.content_type,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn get_object_base64(
    State(app_state): State<AppState>,
    Json(payload): Json<GetObjectRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::get_object(
        &app_state.client,
        &app_state.settings,
        GetObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn head_object(
    State(app_state): State<AppState>,
    Json(payload): Json<HeadObjectRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::head_object(
        &app_state.client,
        &app_state.settings,
        HeadObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn delete_object(
    State(app_state): State<AppState>,
    Json(payload): Json<DeleteObjectRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::delete_object(
        &app_state.client,
        &app_state.settings,
        DeleteObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn list_objects_v2(
    State(app_state): State<AppState>,
    Json(payload): Json<ListObjectsV2Request>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::list_objects_v2(
        &app_state.client,
        &app_state.settings,
        ListObjectsV2Input {
            bucket: payload.bucket,
            prefix: payload.prefix,
            delimiter: payload.delimiter,
            max_keys: payload.max_keys,
            continuation_token: payload.continuation_token,
            start_after: payload.start_after,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn create_multipart_upload(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateMultipartUploadRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::create_multipart_upload(
        &app_state.client,
        &app_state.settings,
        CreateMultipartUploadInput {
            bucket: payload.bucket,
            key: payload.key,
            content_type: payload.content_type,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn upload_part_base64(
    State(app_state): State<AppState>,
    Json(payload): Json<UploadPartBase64Request>,
) -> Result<Json<Value>, ApiError> {
    let body = s3_service::decode_base64_payload(&payload.part_data_base64)?;
    let result = s3_service::upload_part(
        &app_state.client,
        &app_state.settings,
        UploadPartInput {
            bucket: payload.bucket,
            key: payload.key,
            upload_id: payload.upload_id,
            part_number: payload.part_number,
            body,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn complete_multipart_upload(
    State(app_state): State<AppState>,
    Json(payload): Json<CompleteMultipartUploadRequest>,
) -> Result<Json<Value>, ApiError> {
    let parts = payload
        .parts
        .into_iter()
        .map(|part| CompletePartInput {
            part_number: part.part_number,
            e_tag: part.e_tag,
        })
        .collect::<Vec<_>>();

    let result = s3_service::complete_multipart_upload(
        &app_state.client,
        &app_state.settings,
        CompleteMultipartUploadInput {
            bucket: payload.bucket,
            key: payload.key,
            upload_id: payload.upload_id,
            parts,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn abort_multipart_upload(
    State(app_state): State<AppState>,
    Json(payload): Json<AbortMultipartUploadRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::abort_multipart_upload(
        &app_state.client,
        &app_state.settings,
        AbortMultipartUploadInput {
            bucket: payload.bucket,
            key: payload.key,
            upload_id: payload.upload_id,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn list_parts(
    State(app_state): State<AppState>,
    Json(payload): Json<ListPartsRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::list_parts(
        &app_state.client,
        &app_state.settings,
        ListPartsInput {
            bucket: payload.bucket,
            key: payload.key,
            upload_id: payload.upload_id,
            max_parts: payload.max_parts,
            part_number_marker: payload.part_number_marker,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn list_multipart_uploads(
    State(app_state): State<AppState>,
    Json(payload): Json<ListMultipartUploadsRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::list_multipart_uploads(
        &app_state.client,
        &app_state.settings,
        ListMultipartUploadsInput {
            bucket: payload.bucket,
            prefix: payload.prefix,
            delimiter: payload.delimiter,
            max_uploads: payload.max_uploads,
            key_marker: payload.key_marker,
            upload_id_marker: payload.upload_id_marker,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn presigned_get_object(
    State(app_state): State<AppState>,
    Json(payload): Json<PresignedObjectRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::presigned_get(
        &app_state.settings,
        PresignedObjectInput {
            bucket: payload.bucket,
            key: payload.key,
            expires_in_secs: payload.expires_in_secs.unwrap_or(900),
        },
    )?;
    Ok(Json(result))
}

pub async fn presigned_put_object(
    State(app_state): State<AppState>,
    Json(payload): Json<PresignedObjectRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::presigned_put(
        &app_state.settings,
        PresignedObjectInput {
            bucket: payload.bucket,
            key: payload.key,
            expires_in_secs: payload.expires_in_secs.unwrap_or(900),
        },
    )?;
    Ok(Json(result))
}

pub async fn list_buckets(State(app_state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = s3_service::list_buckets(&app_state.client, &app_state.settings).await?;
    Ok(Json(result))
}

pub async fn create_bucket(
    State(app_state): State<AppState>,
    Json(payload): Json<BucketRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::create_bucket(
        &app_state.client,
        &app_state.settings,
        CreateBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn head_bucket(
    State(app_state): State<AppState>,
    Json(payload): Json<BucketRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::head_bucket(
        &app_state.client,
        &app_state.settings,
        HeadBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn delete_bucket(
    State(app_state): State<AppState>,
    Json(payload): Json<BucketRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = s3_service::delete_bucket(
        &app_state.client,
        &app_state.settings,
        DeleteBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(Json(result))
}

pub async fn preview_object(
    State(app_state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let s3_response = s3_service::get_object_proxy(
        &app_state.client,
        &app_state.settings,
        ProxyObjectInput {
            bucket,
            key: key.clone(),
        },
    )
    .await?;

    let mut response = Response::new(Body::from(s3_response.body));
    *response.status_mut() =
        StatusCode::from_u16(s3_response.status_code).unwrap_or(StatusCode::BAD_GATEWAY);

    let pass_through_headers = [
        "content-type",
        "content-length",
        "etag",
        "last-modified",
        "cache-control",
        "content-range",
        "accept-ranges",
    ];

    for (name, value) in s3_response.headers {
        if !pass_through_headers
            .iter()
            .any(|allowed| name.eq_ignore_ascii_case(allowed))
        {
            continue;
        }

        let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) else {
            continue;
        };
        let Ok(header_value) = HeaderValue::from_str(&value) else {
            continue;
        };
        response.headers_mut().insert(header_name, header_value);
    }

    if should_force_pdf_preview(response.headers().get("content-type"), &key) {
        response.headers_mut().insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/pdf"),
        );

        let file_name = sanitize_pdf_filename(&key);
        if let Ok(content_disposition) =
            HeaderValue::from_str(&format!("inline; filename=\"{}\"", file_name))
        {
            response.headers_mut().insert(
                HeaderName::from_static("content-disposition"),
                content_disposition,
            );
        }
    }

    Ok(response)
}

fn should_force_pdf_preview(content_type: Option<&HeaderValue>, key: &str) -> bool {
    let is_pdf_key = key.to_ascii_lowercase().ends_with(".pdf");
    let is_pdf_content_type = content_type
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .map(str::trim)
                .is_some_and(|media_type| media_type.eq_ignore_ascii_case("application/pdf"))
        })
        .unwrap_or(false);

    is_pdf_key || is_pdf_content_type
}

fn sanitize_pdf_filename(key: &str) -> String {
    let candidate = key.rsplit('/').next().unwrap_or("file.pdf");
    let mut filename = candidate
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        .collect::<String>();

    if filename.is_empty() {
        filename = "file.pdf".to_string();
    }

    if !filename.to_ascii_lowercase().ends_with(".pdf") {
        filename.push_str(".pdf");
    }

    filename
}
