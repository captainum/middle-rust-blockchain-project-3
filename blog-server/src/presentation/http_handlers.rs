use axum::extract::State;
use axum::{Json, Router};
use axum::response::{IntoResponse, Result};
use axum::routing::post;
use crate::domain::user::{CreateUserRequest, CreateUserResponse, LoginUserRequest, LoginUserResponse};
use crate::presentation::AppState;

pub fn api(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth(state.clone()))
}

fn auth(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state)
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<CreateUserResponse> {
    Ok(state.auth_service.register(request).await?)
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginUserRequest>,
) -> Result<LoginUserResponse> {
    Ok(state.auth_service.login(request).await?)
}
