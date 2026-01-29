//! Описание ошибок при взаимодействии с данными.

use thiserror::Error;

/// Ошибка взаимодействия с данными пользователя.
#[derive(Debug, Error)]
enum UserError {
    #[error("Пользователь не найден!")]
    UserNotFound,

    #[error("Пользователь уже существует!")]
    UserAlreadyExists,

    #[error("Некорректные логин или пароль!")]
    InvalidCredentials
}

/// Ошибка взаимодействия с данными поста.
#[derive(Debug, Error)]
enum PostError {
    #[error("Пост не найден!")]
    PostNotFound,

    #[error("Запрещено взаимодействие с данным постом!")]
    Forbidden
}
