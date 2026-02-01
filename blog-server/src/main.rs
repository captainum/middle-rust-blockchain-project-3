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

use tonic::transport::Server;
use crate::blog_grpc::blog_service_server::BlogServiceServer;
use crate::presentation::grpc_service::BlogGrpcService;

pub mod blog_grpc {
    tonic::include_proto!("blog");
}

/// Система блога.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Адрес для прослушивания входящих соединений.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Порт для прослушивания входящих HTTP-соединений.
    #[arg(long, default_value = "3000")]
    http_port: u16,

    /// Порт для прослушивания входящих GRPC-соединений.
    #[arg(long, default_value = "50051")]
    grpc_port: u16,

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

async fn prepare_http_serve(app: AppState, addr: SocketAddr) -> anyhow::Result<
    axum::serve::Serve<TcpListener, axum::routing::Router, axum::routing::Router>
> {
    tracing::info!("Listening HTTP connections on {}", addr);

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

    let listener = TcpListener::bind(addr).await?;

    Ok(axum::serve(listener, router))
}

async fn prepare_grpc_serve(app: AppState, addr: SocketAddr) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    tracing::info!("Listening GRPC connections on {}", addr);

    let grpc_service = BlogServiceServer::new(BlogGrpcService::new(app));

    Server::builder().add_service(grpc_service).serve(addr)
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

    let http_addr = format!("{}:{}", args.host, args.http_port).parse()?;
    let grpc_addr = format!("{}:{}", args.host, args.grpc_port).parse()?;

    let http_serve = prepare_http_serve(app.clone(), http_addr).await?;
    let grpc_serve = prepare_grpc_serve(app.clone(), grpc_addr).await;

    tokio::select! {
        result = http_serve => {
           result?
        },
        result = grpc_serve => {
            result?
        }
    }

    Ok(())
}
