//! Вспомогательные макросы.

/// Макрос для автоматической реализации IntoResponse через Json.
///
/// # Пример
/// ```
/// #[derive(Serialize)]
/// struct MyResponse {
///     data: String,
/// }
///
/// impl_json_response!(MyResponse);
/// ```
#[macro_export]
macro_rules! impl_json_response {
    ($type:ty) => {
        impl axum::response::IntoResponse for $type {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    };
}