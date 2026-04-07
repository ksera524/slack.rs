use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderName, HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::Response,
};

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

pub struct PutObjectBase64Request {
    pub bucket: String,
    pub key: String,
    pub file_data_base64: String,
    pub content_type: Option<String>,
}

pub struct GetObjectRequest {
    pub bucket: String,
    pub key: String,
}

pub struct HeadObjectRequest {
    pub bucket: String,
    pub key: String,
}

pub struct DeleteObjectRequest {
    pub bucket: String,
    pub key: String,
}

pub struct ListObjectsV2Request {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub continuation_token: Option<String>,
    pub start_after: Option<String>,
}

pub struct CreateMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub content_type: Option<String>,
}

pub struct UploadPartBase64Request {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number: i32,
    pub part_data_base64: String,
}

pub struct CompletePartRequest {
    pub part_number: i32,
    pub e_tag: String,
}

pub struct CompleteMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<CompletePartRequest>,
}

pub struct AbortMultipartUploadRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

pub struct ListPartsRequest {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub max_parts: Option<i32>,
    pub part_number_marker: Option<i32>,
}

pub struct ListMultipartUploadsRequest {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_uploads: Option<i32>,
    pub key_marker: Option<String>,
    pub upload_id_marker: Option<String>,
}

pub struct PresignedObjectRequest {
    pub bucket: String,
    pub key: String,
    pub expires_in_secs: Option<u64>,
}

pub struct BucketRequest {
    pub bucket: String,
}

fn parse_json_body(body: &str) -> Result<nojson::RawJson<'_>, ApiError> {
    nojson::RawJson::parse(body).map_err(|e| ApiError::BadRequest(format!("Invalid JSON: {e}")))
}

fn get_required_string(root: nojson::RawJsonValue<'_, '_>, name: &str) -> Result<String, ApiError> {
    root.to_member(name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .required()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .try_into()
        .map_err(|e| ApiError::BadRequest(format!("Invalid '{name}': {e}")))
}

fn get_optional_string(
    root: nojson::RawJsonValue<'_, '_>,
    name: &str,
) -> Result<Option<String>, ApiError> {
    let Some(value) = root
        .to_member(name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .optional()
    else {
        return Ok(None);
    };
    let parsed = String::try_from(value)
        .map_err(|e| ApiError::BadRequest(format!("Invalid '{name}': {e}")))?;
    Ok(Some(parsed))
}

fn get_optional_i32(
    root: nojson::RawJsonValue<'_, '_>,
    name: &str,
) -> Result<Option<i32>, ApiError> {
    let Some(value) = root
        .to_member(name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .optional()
    else {
        return Ok(None);
    };
    let parsed =
        i32::try_from(value).map_err(|e| ApiError::BadRequest(format!("Invalid '{name}': {e}")))?;
    Ok(Some(parsed))
}

fn get_optional_u64(
    root: nojson::RawJsonValue<'_, '_>,
    name: &str,
) -> Result<Option<u64>, ApiError> {
    let Some(value) = root
        .to_member(name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .optional()
    else {
        return Ok(None);
    };
    let parsed =
        u64::try_from(value).map_err(|e| ApiError::BadRequest(format!("Invalid '{name}': {e}")))?;
    Ok(Some(parsed))
}

fn json_response(body: String) -> Response {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = StatusCode::OK;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

fn parse_put_object_base64_request(body: &str) -> Result<PutObjectBase64Request, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(PutObjectBase64Request {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        file_data_base64: get_required_string(root, "file_data_base64")?,
        content_type: get_optional_string(root, "content_type")?,
    })
}

fn parse_get_object_request(body: &str) -> Result<GetObjectRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(GetObjectRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
    })
}

fn parse_head_object_request(body: &str) -> Result<HeadObjectRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(HeadObjectRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
    })
}

fn parse_delete_object_request(body: &str) -> Result<DeleteObjectRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(DeleteObjectRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
    })
}

fn parse_list_objects_v2_request(body: &str) -> Result<ListObjectsV2Request, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(ListObjectsV2Request {
        bucket: get_required_string(root, "bucket")?,
        prefix: get_optional_string(root, "prefix")?,
        delimiter: get_optional_string(root, "delimiter")?,
        max_keys: get_optional_i32(root, "max_keys")?,
        continuation_token: get_optional_string(root, "continuation_token")?,
        start_after: get_optional_string(root, "start_after")?,
    })
}

fn parse_create_multipart_upload_request(
    body: &str,
) -> Result<CreateMultipartUploadRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(CreateMultipartUploadRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        content_type: get_optional_string(root, "content_type")?,
    })
}

fn parse_upload_part_base64_request(body: &str) -> Result<UploadPartBase64Request, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(UploadPartBase64Request {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        upload_id: get_required_string(root, "upload_id")?,
        part_number: get_required_i32(root, "part_number")?,
        part_data_base64: get_required_string(root, "part_data_base64")?,
    })
}

fn get_required_i32(root: nojson::RawJsonValue<'_, '_>, name: &str) -> Result<i32, ApiError> {
    root.to_member(name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .required()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .try_into()
        .map_err(|e| ApiError::BadRequest(format!("Invalid '{name}': {e}")))
}

fn parse_complete_multipart_upload_request(
    body: &str,
) -> Result<CompleteMultipartUploadRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    let parts_raw = root
        .to_member("parts")
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .required()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let mut parts = Vec::new();
    for part in parts_raw
        .to_array()
        .map_err(|e| ApiError::BadRequest(format!("Invalid 'parts': {e}")))?
    {
        parts.push(CompletePartRequest {
            part_number: get_required_i32(part, "part_number")?,
            e_tag: get_required_string(part, "e_tag")?,
        });
    }

    Ok(CompleteMultipartUploadRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        upload_id: get_required_string(root, "upload_id")?,
        parts,
    })
}

fn parse_abort_multipart_upload_request(
    body: &str,
) -> Result<AbortMultipartUploadRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(AbortMultipartUploadRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        upload_id: get_required_string(root, "upload_id")?,
    })
}

fn parse_list_parts_request(body: &str) -> Result<ListPartsRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(ListPartsRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        upload_id: get_required_string(root, "upload_id")?,
        max_parts: get_optional_i32(root, "max_parts")?,
        part_number_marker: get_optional_i32(root, "part_number_marker")?,
    })
}

fn parse_list_multipart_uploads_request(
    body: &str,
) -> Result<ListMultipartUploadsRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(ListMultipartUploadsRequest {
        bucket: get_required_string(root, "bucket")?,
        prefix: get_optional_string(root, "prefix")?,
        delimiter: get_optional_string(root, "delimiter")?,
        max_uploads: get_optional_i32(root, "max_uploads")?,
        key_marker: get_optional_string(root, "key_marker")?,
        upload_id_marker: get_optional_string(root, "upload_id_marker")?,
    })
}

fn parse_presigned_object_request(body: &str) -> Result<PresignedObjectRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(PresignedObjectRequest {
        bucket: get_required_string(root, "bucket")?,
        key: get_required_string(root, "key")?,
        expires_in_secs: get_optional_u64(root, "expires_in_secs")?,
    })
}

fn parse_bucket_request(body: &str) -> Result<BucketRequest, ApiError> {
    let json = parse_json_body(body)?;
    let root = json.value();
    Ok(BucketRequest {
        bucket: get_required_string(root, "bucket")?,
    })
}

pub async fn put_object_base64(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_put_object_base64_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn get_object_base64(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_get_object_request(&body)?;
    let result = s3_service::get_object(
        &app_state.client,
        &app_state.settings,
        GetObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(json_response(result))
}

pub async fn head_object(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_head_object_request(&body)?;
    let result = s3_service::head_object(
        &app_state.client,
        &app_state.settings,
        HeadObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(json_response(result))
}

pub async fn delete_object(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_delete_object_request(&body)?;
    let result = s3_service::delete_object(
        &app_state.client,
        &app_state.settings,
        DeleteObjectInput {
            bucket: payload.bucket,
            key: payload.key,
        },
    )
    .await?;
    Ok(json_response(result))
}

pub async fn list_objects_v2(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_list_objects_v2_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn create_multipart_upload(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_create_multipart_upload_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn upload_part_base64(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_upload_part_base64_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn complete_multipart_upload(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_complete_multipart_upload_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn abort_multipart_upload(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_abort_multipart_upload_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn list_parts(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_list_parts_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn list_multipart_uploads(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_list_multipart_uploads_request(&body)?;
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
    Ok(json_response(result))
}

pub async fn presigned_get_object(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_presigned_object_request(&body)?;
    let result = s3_service::presigned_get(
        &app_state.settings,
        PresignedObjectInput {
            bucket: payload.bucket,
            key: payload.key,
            expires_in_secs: payload.expires_in_secs.unwrap_or(900),
        },
    )?;
    Ok(json_response(result))
}

pub async fn presigned_put_object(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_presigned_object_request(&body)?;
    let result = s3_service::presigned_put(
        &app_state.settings,
        PresignedObjectInput {
            bucket: payload.bucket,
            key: payload.key,
            expires_in_secs: payload.expires_in_secs.unwrap_or(900),
        },
    )?;
    Ok(json_response(result))
}

pub async fn list_buckets(State(app_state): State<AppState>) -> Result<Response, ApiError> {
    let result = s3_service::list_buckets(&app_state.client, &app_state.settings).await?;
    Ok(json_response(result))
}

pub async fn create_bucket(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_bucket_request(&body)?;
    let result = s3_service::create_bucket(
        &app_state.client,
        &app_state.settings,
        CreateBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(json_response(result))
}

pub async fn head_bucket(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_bucket_request(&body)?;
    let result = s3_service::head_bucket(
        &app_state.client,
        &app_state.settings,
        HeadBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(json_response(result))
}

pub async fn delete_bucket(
    State(app_state): State<AppState>,
    body: String,
) -> Result<Response, ApiError> {
    let payload = parse_bucket_request(&body)?;
    let result = s3_service::delete_bucket(
        &app_state.client,
        &app_state.settings,
        DeleteBucketInput {
            bucket: payload.bucket,
        },
    )
    .await?;
    Ok(json_response(result))
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
