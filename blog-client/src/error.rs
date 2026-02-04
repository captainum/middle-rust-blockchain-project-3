//! Описание ошибок при взаимодействии с сервером блога.

use thiserror::Error;

/// Ошибки взаимодействия с сервером блога.
#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("Передан некорректный JWT-токен")]
    InvalidToken,

    #[error("JWT-токен не установлен!")]
    TokenNotFound,

    #[error("Пользователь не найден!")]
    UserNotFound,

    #[error("Пользователь уже существует!")]
    UserAlreadyExists,

    #[error("Пользователь не авторизован!")]
    UserUnauthorized,

    #[error("Некорректные логин или пароль!")]
    InvalidCredentials,

    #[error("Некорректные данные для регистрации!")]
    InvalidRegistrationCredentials,

    #[error("Некорректное содержимое информации о пользователе!")]
    InvalidUser,

    #[error("Пост не найден!")]
    PostNotFound,

    #[error("Некорректное содержимое поста!")]
    InvalidPostContent,

    #[error("Запрещено взаимодействие с данным постом!")]
    Forbidden,

    #[error("Непредвиденная ошибка!")]
    Unexpected,

    #[error("Внутренняя ошибка HTTP протокола: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Внутренняя ошибка gRPC протокола: {0}")]
    Grpc(#[from] tonic::transport::Error),

    #[error("Непредвиденный статус gRPC ответа: {0}")]
    GrpcStatus(#[from] tonic::Status),
}
