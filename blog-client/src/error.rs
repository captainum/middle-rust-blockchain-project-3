use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Grpc(#[from] tonic::Status),

    #[error(transparent)]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("Внутренний клиент не инициализирован!")]
    InvalidInnerState,

    #[error("Передан некорректный JWT-токен")]
    InvalidToken,

    #[error("JWT-токен не установлен")]
    TokenNotFound,

    #[error("Пост не найден")]
    PostNotFound,

    #[error("Некорректное содержимое поста")]
    InvalidPostContent,
}
