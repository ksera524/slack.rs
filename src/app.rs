use crate::config::state::AppState;
use crate::handlers::health_handler::health;
use crate::middleware::request_tracing::request_tracing_middleware;
use crate::routes;
use axum::{extract::DefaultBodyLimit, middleware, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(routes::slack::create_slack_routes())
        .layer(
            ServiceBuilder::new()
                .layer(DefaultBodyLimit::disable()) // Axumレベルでの制限無効化
                .layer(middleware::from_fn(request_tracing_middleware))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(app_state)
}
