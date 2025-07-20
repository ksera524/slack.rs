use std::net::SocketAddr;
use std::io::Write;
use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan, prelude::*};
mod app;
mod config;
mod errors;
mod handlers;
mod routes;
mod service;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    println!("=== SLACK APP STARTING ===");
    std::io::stdout().flush().ok();
    dotenvy::dotenv().ok();
    println!("✓ dotenv loaded");
    std::io::stdout().flush().ok();

    let fmt_layer = fmt::layer().json().with_span_events(FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
    println!("✓ tracing initialized");

    println!("Loading settings...");
    std::io::stdout().flush().ok();
    let settings = match config::settings::Settings::new() {
        Ok(s) => {
            println!("✓ Settings loaded successfully");
            std::io::stdout().flush().ok();
            s
        },
        Err(e) => {
            println!("✗ Failed to load settings: {}", e);
            eprintln!("✗ Failed to load settings: {}", e);
            std::io::stdout().flush().ok();
            std::io::stderr().flush().ok();
            std::process::exit(1);
        }
    };

    println!("Creating HTTP client...");
    let client = reqwest::Client::new();
    println!("✓ HTTP client created");

    let app_state = config::state::AppState { settings, client };
    println!("✓ App state created");

    let app = app::create_app(app_state);
    println!("✓ App router created");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Binding to address: {}", addr);

    tracing::info!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            println!("✓ Successfully bound to address {}", addr);
            l
        },
        Err(e) => {
            println!("✗ Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    
    tracing::info!("Server bound to address, starting to serve...");
    println!("=== STARTING AXUM SERVER ===");
    
    // ARC/DinD環境対応: Graceful shutdown with signal handling
    let server = axum::serve(listener, app);
    
    println!("✓ Server started successfully, listening for connections...");
    
    // Run server with timeout and signal handling
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
        println!("Shutdown signal received");
    };
    
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                println!("✗ Server error: {}", e);
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            println!("Shutting down gracefully...");
        }
    }
    
    println!("=== SERVER STOPPED ===");
}
