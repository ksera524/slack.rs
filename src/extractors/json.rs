use axum::{Json, extract::{FromRequest, Request}, extract::rejection::JsonRejection};

use crate::errors::api_error::ApiError;

pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            match Json::<T>::from_request(req, state).await {
                Ok(Json(value)) => Ok(Self(value)),
                Err(rejection) => Err(ApiError::BadRequest(rejection.body_text())),
            }
        }
    }
}
