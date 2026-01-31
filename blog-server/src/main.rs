mod domain;
mod infrastructure;
mod presentation;
mod application;
mod data;

use std::sync::Arc;
use infrastructure::database::{create_pool, run_migrations};
use infrastructure::logging::init_logging;
use infrastructure::jwt;
use tower_http::cors::{CorsLayer, Any};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use std::time::Duration;
use tokio::net::TcpListener;
use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::infrastructure::jwt::JwtService;
use crate::presentation::{create_router, AppState};
use tower::ServiceBuilder;

use clap::Parser;
use std::net::SocketAddr;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;

mod blog_grpc {
    tonic::include_proto!("blog");
}

/// Система блога.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Адрес для прослушивания.
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: SocketAddr,

    /// Уровень логирования.
    ///
    /// Доступные варианты: "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE".
    #[arg(long, value_name = "LEVEL", default_value = "INFO")]
    log_level: String,
}

fn create_cors_layer() -> CorsLayer {
    use axum::http::Method;

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    #[cfg(debug_assertions)]
    let cors = cors.allow_origin(Any);

    #[cfg(not(debug_assertions))]
    let cors = {
        let whitelist = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse::<axum::http::HeaderValue>().ok())
            .collect::<Vec<_>>();

        cors.allow_origin(whitelist)
    };

    cors
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let args = Args::parse();

    dotenvy::dotenv()?;

    init_logging(&args.log_level);

    tracing::info!("Starting server..");

    let pool = create_pool().await?;
    run_migrations(&pool).await?;

    let jwt_secret= jwt::load_secret()?;

    let jwt_service = Arc::new(JwtService::new(&jwt_secret));

    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let post_repository = Arc::new(PostRepository::new(pool.clone()));

    let auth_service = Arc::new(AuthService::new(jwt_service.clone(), user_repository.clone()));
    let blog_service = Arc::new(BlogService::new(post_repository.clone()));

    let app = AppState::new(auth_service.clone(), blog_service.clone(), jwt_service.clone());

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .finish().ok_or(anyhow::anyhow!("Failed to prepare rate limiter"))?;

    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(GovernorLayer::new(governor_conf))
        .concurrency_limit(20)
        .layer(create_cors_layer())
        .layer(TimeoutLayer::with_status_code(axum::http::StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)));

    let router = create_router(app, middleware);

    let listener = TcpListener::bind(args.addr).await?;

    tracing::info!("Listening on {}", args.addr);

    axum::serve(listener, router).await?;

    Ok(())
}
