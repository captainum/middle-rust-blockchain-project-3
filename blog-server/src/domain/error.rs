//! Описание ошибок при взаимодействии с данными.

use axum::response::IntoResponse;
use thiserror::Error;

use axum::http::StatusCode;

/// Ошибка взаимодействия с данными пользователя.
#[derive(Debug, Error)]
pub enum UserError {
    #[error("Пользователь не найден!")]
    UserNotFound,

    #[error("Пользователь уже существует!")]
    UserAlreadyExists,

    #[error("Некорректные логин или пароль!")]
    InvalidCredentials,

    #[error("Ошибка хеширования пароля")]
    PasswordHashing(#[from] argon2::password_hash::Error),

    #[error("Не удалось создать JWT-токен ({0})")]
    CreateJwtToken(String),

    #[error("Внутренняя ошибка со стороны базы данных ({0})")]
    Database(#[from] sqlx::Error)
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserAlreadyExists => StatusCode::CONFLICT,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::PasswordHashing(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::CreateJwtToken(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status_code.into_response()
    }
}

/// Ошибка взаимодействия с данными поста.
#[derive(Debug, Error)]
pub enum PostError {
    #[error("Пост не найден!")]
    PostNotFound,

    #[error("Запрещено взаимодействие с данным постом!")]
    Forbidden,

    #[error("Внутренняя ошибка со стороны базы данных ({0})")]
    Database(#[from] sqlx::Error)
}

impl IntoResponse for PostError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            PostError::PostNotFound => StatusCode::NOT_FOUND,
            PostError::Forbidden => StatusCode::FORBIDDEN,
            PostError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status_code.into_response()
    }
}
