use crate::config::state::AppState;
use crate::routes;
use axum::Router;
pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        .merge(routes::slack::create_slack_routes())
        .with_state(app_state)
}
