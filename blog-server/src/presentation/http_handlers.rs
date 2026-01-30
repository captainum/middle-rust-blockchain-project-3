use axum::extract::{Path, Query, State};
use axum::{Json, Router, middleware, Extension};
use axum::response::Result;
use axum::routing::{delete, get, post, put};
use serde::Deserialize;
use crate::domain::user::{CreateUserRequest, CreateUserResponse, LoginUserRequest, LoginUserResponse};
use crate::domain::post::{CreatePostRequest, Post, UpdatePostRequest};
use crate::infrastructure::jwt::Claims;
use crate::presentation::AppState;
use crate::presentation::middleware::jwt_validator;

pub fn api(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth(state.clone()))
        .nest("/posts", posts(state.clone()))
}

fn auth(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state)
}

fn posts(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/{id}", get(get_post))
        .route("/", get(get_posts));

    let protected_routes = Router::new()
        .route("/", post(create_post))
        .route("/{id}", put(update_post))
        .route("/{id}", delete(delete_post))
        .route_layer(middleware::from_fn_with_state(state.clone(), jwt_validator));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
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

#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    10
}

async fn create_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreatePostRequest>,
) -> Result<(axum::http::StatusCode, Post)> {
    Ok((axum::http::StatusCode::CREATED, state.blog_service.create_post(request, claims.user_id).await?))
}

async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Post> {
    Ok(state.blog_service.get_post(id).await?)
}

async fn get_posts(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Post>>> {
    Ok(Json(state.blog_service.get_posts(params.limit, params.offset).await?))
}

async fn update_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(mut request): Json<UpdatePostRequest>,
) -> Result<Post> {
    request.id = id;

    Ok(state.blog_service.update_post(request, claims.user_id).await?)
}

async fn delete_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode> {
    state.blog_service.delete_post(id, claims.user_id).await?;
    
    Ok(axum::http::StatusCode::NO_CONTENT)
}
