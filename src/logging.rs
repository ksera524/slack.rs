use chrono::Local;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use uuid::Uuid;

/// カスタム時刻フォーマッター
struct LocalTime;

impl FormatTime for LocalTime {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}


/// ログ設定
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,
    pub enable_target: bool,
    pub enable_thread: bool,
    pub enable_line_number: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            enable_target: std::env::var("LOG_TARGET").unwrap_or_default() == "true",
            enable_thread: std::env::var("LOG_THREAD").unwrap_or_default() == "true",
            enable_line_number: std::env::var("LOG_LINE").unwrap_or_default() == "true",
        }
    }
}

/// トレーシングサブスクライバーの初期化（JSONL形式固定）
pub fn init_tracing(config: LogConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let jsonl_layer = fmt::layer()
        .json()
        .with_current_span(false)  // Flatten span info into main object
        .with_span_list(false)     // Don't include span list to keep each line independent
        .with_timer(LocalTime)
        .with_target(config.enable_target)
        .with_thread_ids(config.enable_thread)
        .with_thread_names(config.enable_thread)
        .with_line_number(config.enable_line_number)
        .with_file(config.enable_line_number)
        .with_span_events(FmtSpan::CLOSE)  // Only log on span close to reduce noise
        .flatten_event(true);      // Flatten fields for easier parsing

    tracing_subscriber::registry()
        .with(env_filter)
        .with(jsonl_layer)
        .init();
}

/// リクエストIDの生成
#[allow(dead_code)]
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// 構造化ログ用のマクロ
#[macro_export]
macro_rules! log_event {
    ($level:expr, $($key:ident = $value:expr),* $(,)?) => {
        match $level {
            tracing::Level::ERROR => tracing::error!($($key = $value),*),
            tracing::Level::WARN => tracing::warn!($($key = $value),*),
            tracing::Level::INFO => tracing::info!($($key = $value),*),
            tracing::Level::DEBUG => tracing::debug!($($key = $value),*),
            tracing::Level::TRACE => tracing::trace!($($key = $value),*),
        }
    };
}

/// パフォーマンス計測用マクロ
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $body:expr) => {{
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        tracing::debug!(
            operation = $name,
            duration_ms = duration.as_millis() as u64,
            "Operation completed"
        );
        result
    }};
}