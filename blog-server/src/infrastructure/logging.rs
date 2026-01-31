use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_subscriber::fmt::time::ChronoUtc;

pub fn init_logging(log_level: &str) {
    let filter = EnvFilter::new(format!(
        "server={},axum=info,tower=info,tower_http=info",
        log_level.to_lowercase()
    ));

    let format = tracing_subscriber::fmt::layer()
        .with_timer(ChronoUtc::rfc_3339());

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();
}