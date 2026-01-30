mod http_handlers;

use http_handlers::api;

use std::sync::Arc;
use axum::Router;
use crate::application::auth_service::AuthService;

#[derive(Debug, Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api(state))
}