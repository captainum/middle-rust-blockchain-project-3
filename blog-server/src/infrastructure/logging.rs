//! Модуль логгирования.

use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Инициализация логгера с указанием уровня логгирования.
pub(crate) fn init_logging(log_level: &str) {
    let filter = EnvFilter::new(format!(
        "blog_server={},axum=info,tower=info,tower_http=info",
        log_level.to_lowercase()
    ));

    let format = tracing_subscriber::fmt::layer().with_timer(ChronoUtc::rfc_3339());

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();
}
