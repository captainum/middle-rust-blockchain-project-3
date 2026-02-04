//! Логика взаимодействия пользовательского интерфейса и бекенда.

#![deny(unreachable_pub)]

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::window;

#[derive(Debug, Deserialize)]
/// Ответ сервера с JWT-токеном при авторизации.
pub struct AuthResponse {
    /// JWT-токен для последующих запросов.
    token: String,

    /// Авторизованный пользователь.
    user: User,
}

/// Информация о пользователе.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Идентификатор пользователя.
    pub id: i64,

    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    pub email: String,

    /// Время создания пользователя.
    pub created_at: String,
}

/// Информация о посте.
#[derive(Deserialize, Serialize)]
pub struct Post {
    /// Идентификатор поста.
    pub id: i64,

    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,

    /// Идентификатор пользователя-автора поста.
    pub author_id: i64,

    /// Время создания поста.
    pub created_at: String,

    /// Время последнего обновления поста.
    pub updated_at: String,
}

/// Клиентское приложение блога для взаимодействия с сервером.
#[wasm_bindgen]
pub struct BlogApp {
    /// URL-адрес сервера.
    server: String,

    /// JWT-токен авторизации.
    token: Option<String>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(server: String) -> Self {
        Self {
            server,
            token: None,
        }
    }

    /// Регистрация пользователя.
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/auth/register", self.server);

        let payload = serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
        });

        let response = Request::post(&url)
            .json(&payload)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать запрос: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            let msg = match response.status() {
                400 => "Некорректные данные для регистрации!".to_string(),
                409 => "Пользователь уже существует!".to_string(),
                status => {
                    format!("Регистрация не удалась, код: {}", status)
                }
            };

            return Err(JsValue::from_str(&msg));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось обработать ответ: {}", e)))?;

        self.token = Some(auth_response.token.clone());

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "user": auth_response.user,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }

    /// Авторизация пользователя.
    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/auth/login", self.server);

        let payload = serde_json::json! ({
            "username": username,
            "password": password,
        });

        let response = Request::post(&url)
            .json(&payload)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сформировать запрос: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            let msg = match response.status() {
                401 => "Неверные логин или пароль".to_string(),
                404 => "Пользователь не найден".to_string(),
                status => {
                    format!("Ошибка авторизации, код: {}", status)
                }
            };

            return Err(JsValue::from_str(&msg));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось обработать ответ: {}", e)))?;

        self.token = Some(auth_response.token.clone());

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "user": auth_response.user,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }

    /// Загрузить посты (с пагинацией).
    pub async fn load_posts(&self, limit: i64, offset: i64) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts", self.server);

        let response = Request::get(&url)
            .query([("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Не удалось загрузить посты, код: {}",
                response.status()
            )));
        }

        let posts: Vec<Post> = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось обработать ответ: {}", e)))?;

        serde_wasm_bindgen::to_value(&posts)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать посты: {}", e)))
    }

    /// Создать пост.
    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Не авторизован"))?;

        let url = format!("{}/api/posts", self.server);

        let payload = serde_json::json!({
            "title": title,
            "content": content,
        });

        let response = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&payload)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать запрос: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Не удалось создать пост, код: {}",
                response.status()
            )));
        }

        let post: Post = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось обработать ответ: {}", e)))?;

        serde_wasm_bindgen::to_value(&post)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать пост: {}", e)))
    }

    /// Обновить пост.
    pub async fn update_post(
        &self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Пользователь не авторизован"))?;

        let url = format!("{}/api/posts/{}", self.server, id);

        let payload = serde_json::json!({
            "title": title,
            "content": content,
        });

        let response = Request::put(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&payload)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать запрос: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Не удалось обновить пост, код: {}",
                response.status()
            )));
        }

        let post: Post = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось обработать ответ: {}", e)))?;

        serde_wasm_bindgen::to_value(&post)
            .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать пост: {}", e)))
    }

    /// Удалить пост.
    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Пользователь не авторизован"))?;

        let url = format!("{}/api/posts/{}", self.server, id);

        let response = Request::delete(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Не удалось отправить запрос: {}", e)))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Не удалось удалить пост, код: {}",
                response.status()
            )));
        }

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "success": true,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }

    /// Проверка наличия токена.
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Получить токен.
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    /// Установить токен.
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Сохранение JWT-токена в localStorage под ключом "blog_token".
    pub fn save_token_to_storage(&self) -> Result<JsValue, JsValue> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Нет токена для сохранения"))?;

        let window =
            window().ok_or_else(|| JsValue::from_str("Не удалось получить объект window"))?;

        let storage = window
            .local_storage()
            .map_err(|_| JsValue::from_str("Не удалось получить доступ к localStorage"))?
            .ok_or_else(|| JsValue::from_str("localStorage недоступен"))?;

        storage
            .set_item("blog_token", token)
            .map_err(|_| JsValue::from_str("Не удалось сохранить токен в localStorage"))?;

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "success": true,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }

    /// Загрузка токена из localStorage.
    pub fn get_token_from_storage(&mut self) -> Result<JsValue, JsValue> {
        let window =
            window().ok_or_else(|| JsValue::from_str("Не удалось получить объект window"))?;

        let storage = window
            .local_storage()
            .map_err(|_| JsValue::from_str("Не удалось получить доступ к localStorage"))?
            .ok_or_else(|| JsValue::from_str("localStorage недоступен"))?;

        let token = storage
            .get_item("blog_token")
            .map_err(|_| JsValue::from_str("Не удалось прочитать токен из localStorage"))?;

        match token {
            Some(token) if !token.is_empty() => {
                self.token = Some(token);
                serde_wasm_bindgen::to_value(&serde_json::json!({
                    "success": true,
                }))
                .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
            }
            _ => Err(JsValue::from_str("Токен не найден в localStorage")),
        }
    }

    /// Удаление токена из localStorage.
    pub fn remove_token_from_storage(&mut self) -> Result<JsValue, JsValue> {
        let window =
            window().ok_or_else(|| JsValue::from_str("Не удалось получить объект window"))?;

        let storage = window
            .local_storage()
            .map_err(|_| JsValue::from_str("Не удалось получить доступ к localStorage"))?
            .ok_or_else(|| JsValue::from_str("localStorage недоступен"))?;

        storage
            .remove_item("blog_token")
            .map_err(|_| JsValue::from_str("Не удалось удалить токен из localStorage"))?;

        self.token = None;

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "success": true,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }

    /// Показать уведомление с информацией о пользователе.
    pub fn show_user_notification(&self, user: JsValue) -> Result<JsValue, JsValue> {
        let window =
            window().ok_or_else(|| JsValue::from_str("Не удалось получить объект window"))?;

        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("Не удалось получить объект document"))?;

        let notification = document
            .create_element("div")
            .map_err(|_| JsValue::from_str("Не удалось создать элемент"))?;

        notification.set_id("user-notification");
        notification.set_class_name("user-notification-container");

        let inner_html = r#"<div style="background-color: #4CAF50; color: white; padding: 20px; margin: 10px 0; border-radius: 4px; border-left: 4px solid #45a049; box-shadow: 0 2px 5px rgba(0,0,0,0.2);">
            <div style="font-size: 18px; font-weight: bold; margin-bottom: 10px;">✅ Добро пожаловать!</div>
            <div style="font-size: 14px; line-height: 1.6;">
                <strong>ID:</strong> <span id="user-id"></span><br>
                <strong>Имя:</strong> <span id="user-name"></span><br>
                <strong>Email:</strong> <span id="user-email"></span><br>
                <strong>Зарегистрирован:</strong> <span id="user-created"></span>
            </div>
        </div>"#;

        notification.set_inner_html(inner_html);

        let body = document
            .body()
            .ok_or_else(|| JsValue::from_str("Не удалось получить body"))?;

        body.append_child(&notification)
            .map_err(|_| JsValue::from_str("Не удалось добавить уведомление на страницу"))?;

        // Заполняем данные пользователя через JavaScript
        let user_obj: User = serde_wasm_bindgen::from_value(user.clone()).map_err(|e| {
            JsValue::from_str(&format!("Не удалось обработать данные пользователя: {}", e))
        })?;

        if let Some(elem) = document.get_element_by_id("user-id") {
            elem.set_inner_html(&user_obj.id.to_string());
        }

        if let Some(elem) = document.get_element_by_id("user-name") {
            elem.set_inner_html(&user_obj.username);
        }

        if let Some(elem) = document.get_element_by_id("user-email") {
            elem.set_inner_html(&user_obj.email);
        }

        if let Some(elem) = document.get_element_by_id("user-created") {
            elem.set_inner_html(&user_obj.created_at);
        }

        serde_wasm_bindgen::to_value(&serde_json::json!({
            "success": true,
        }))
        .map_err(|e| JsValue::from_str(&format!("Не удалось сериализовать ответ: {}", e)))
    }
}
