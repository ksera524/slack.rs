use crate::{
    config::state::AppState,
    handlers::s3_handler::{
        abort_multipart_upload, complete_multipart_upload, create_bucket, create_multipart_upload,
        delete_bucket, delete_object, get_object_base64, head_bucket, head_object, list_buckets,
        list_multipart_uploads, list_objects_v2, list_parts, presigned_get_object,
        presigned_put_object, preview_object, put_object_base64, upload_part_base64,
    },
};
use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    routing::{get, options, post},
};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn create_s3_routes() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static(
            "https://hitomi-upload-viewer.internal.qroksera.com",
        ))
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-request-id"),
        ])
        .max_age(Duration::from_secs(600));

    Router::new()
        .route("/s3/preview/{bucket}/{*key}", get(preview_object))
        .route("/s3/put_object_base64", post(put_object_base64))
        .route("/s3/get_object_base64", post(get_object_base64))
        .route("/s3/head_object", post(head_object))
        .route("/s3/delete_object", post(delete_object))
        .route(
            "/s3/list_objects_v2",
            post(list_objects_v2).options(options(s3_preflight)),
        )
        .route("/s3/create_multipart_upload", post(create_multipart_upload))
        .route("/s3/upload_part_base64", post(upload_part_base64))
        .route(
            "/s3/complete_multipart_upload",
            post(complete_multipart_upload),
        )
        .route("/s3/abort_multipart_upload", post(abort_multipart_upload))
        .route("/s3/list_parts", post(list_parts))
        .route("/s3/list_multipart_uploads", post(list_multipart_uploads))
        .route(
            "/s3/presigned_get_object",
            post(presigned_get_object).options(options(s3_preflight)),
        )
        .route("/s3/presigned_put_object", post(presigned_put_object))
        .route("/s3/list_buckets", post(list_buckets))
        .route("/s3/create_bucket", post(create_bucket))
        .route("/s3/head_bucket", post(head_bucket))
        .route("/s3/delete_bucket", post(delete_bucket))
        .layer(cors)
}

async fn s3_preflight() -> StatusCode {
    StatusCode::NO_CONTENT
}
