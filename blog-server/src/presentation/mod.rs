mod http_handlers;
pub mod middleware;

use http_handlers::api;

use std::sync::Arc;
use axum::Router;
use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub blog_service: Arc<BlogService>,
    pub jwt_service: Arc<JwtService>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api(state))
}