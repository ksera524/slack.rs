use std::net::SocketAddr;
use slack::{app, config, logging};
use tracing::{error, info, info_span};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    dotenvy::dotenv().ok();

    // ログシステムの初期化
    let log_config = logging::LogConfig::default();
    logging::init_tracing(log_config);

    let main_span = info_span!("application", service = "slack-rs", version = env!("CARGO_PKG_VERSION"));
    let _enter = main_span.enter();

    info!(
        service = "slack-rs",
        version = env!("CARGO_PKG_VERSION"),
        "Starting application"
    );

    let settings = match config::settings::Settings::new() {
        Ok(s) => {
            info!(
                config_loaded = true,
                "Configuration loaded successfully"
            );
            s
        },
        Err(e) => {
            error!(
                error = %e,
                config_loaded = false,
                "Failed to load settings"
            );
            std::process::exit(1);
        }
    };

    let client = reqwest::Client::new();

    let app_state = config::state::AppState { settings, client };

    let app = app::create_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!(
        addr = %addr,
        port = addr.port(),
        "Starting HTTP server"
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            l
        },
        Err(e) => {
            error!(
                error = %e,
                address = %addr,
                port = addr.port(),
                "Failed to bind to address"
            );
            std::process::exit(1);
        }
    };

    info!(
        addr = %addr,
        "Server successfully bound to address"
    );

    // ARC/DinD環境対応: Graceful shutdown with signal handling
    // Hyperのボディサイズ制限も解除
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        error!(
            error = %e,
            "Server error occurred"
        );
        std::process::exit(1);
    }

    info!("Server shutdown complete");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!(signal = "SIGINT", "Received shutdown signal");
        },
        _ = terminate => {
            info!(signal = "SIGTERM", "Received shutdown signal");
        },
    }
}
