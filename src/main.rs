use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan, prelude::*};
mod app;
mod config;
mod errors;
mod handlers;
mod routes;
mod service;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    dotenvy::dotenv().ok();

    let fmt_layer = fmt::layer().json().with_span_events(FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();

    let settings = match config::settings::Settings::new() {
        Ok(s) => {
            s
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to load settings");
            std::process::exit(1);
        }
    };

    let client = reqwest::Client::new();

    let app_state = config::state::AppState { settings, client };

    let app = app::create_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            l
        },
        Err(e) => {
            tracing::error!(error = %e, address = %addr, "Failed to bind to address");
            std::process::exit(1);
        }
    };
    
    tracing::info!("Server bound to address, starting to serve...");
    
    // ARC/DinD環境対応: Graceful shutdown with signal handling
    let server = axum::serve(listener, app);
    
    
    // Run server with timeout and signal handling
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };
    
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!(error = %e, "Server error");
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            tracing::info!("Shutdown signal received, gracefully shutting down");
        }
    }
    
}
