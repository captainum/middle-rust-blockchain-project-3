mod http_handlers;
pub mod middleware;
pub mod grpc_service;

use std::convert::Infallible;
use http_handlers::api;

use std::sync::Arc;
use axum::Router;
use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::infrastructure::jwt::JwtService;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use tonic::codegen::Service;
use tower::{Layer, ServiceBuilder};

#[derive(Debug, Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub blog_service: Arc<BlogService>,
    pub jwt_service: Arc<JwtService>,
}

impl AppState {
    pub fn new(auth_service: Arc<AuthService>, blog_service: Arc<BlogService>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }
}

pub fn create_router<L>(state: AppState, middleware: ServiceBuilder<L>) -> Router
where
    L: Layer<Route> + Clone + Send + Sync + 'static,
    L::Service: Service<Request> + Clone + Send + Sync + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
{
    Router::new()
        .nest("/api", api(state))
        .layer(middleware)
}