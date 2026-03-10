use crate::config::state::AppState;
use crate::handlers::health_handler::health;
use crate::handlers::openapi_handler::openapi_json;
use crate::middleware::{
    problem_details::problem_details_middleware, request_tracing::request_tracing_middleware,
};
use crate::routes;
use axum::{Router, extract::DefaultBodyLimit, middleware, routing::get};

pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi_json))
        .merge(routes::slack::create_slack_routes())
        .layer(middleware::from_fn(request_tracing_middleware))
        .layer(middleware::from_fn(problem_details_middleware))
        .layer(DefaultBodyLimit::disable())
        .with_state(app_state)
}
