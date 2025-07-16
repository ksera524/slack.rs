use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan, prelude::*};
mod app;
mod config;
mod errors;
mod handlers;
mod routes;
mod service;
use std::env;

#[tokio::main]
async fn main() {
    println!("Starting application...");
    dotenvy::dotenv().ok();

    let fmt_layer = fmt::layer().json().with_span_events(FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();

    let settings = config::settings::Settings::new().expect("Failed to load settings");

    let client = reqwest::Client::new();

    let app_state = config::state::AppState { settings, client };

    let app = app::create_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
