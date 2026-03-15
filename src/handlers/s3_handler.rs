use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    config::state::AppState,
    errors::api_error::ApiError,
    service::s3_service::{
        self, AbortMultipartUploadInput, CompleteMultipartUploadInput, CompletePartInput,
        CreateBucketInput, CreateMultipartUploadInput, DeleteBucketInput, DeleteObjectInput,
        GetObjectInput, HeadBucketInput, HeadObjectInput, ListMultipartUploadsInput,
        ListObjectsV2Input, ListPartsInput, PresignedObjectInput, PutObjectInput, UploadPartInput,
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
