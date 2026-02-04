# Проектная работа модуля 3. Cистема блога

Для проекта используется [cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) для удобства
общей сборки сервера и клиента.

Проект представляет собой четыре крейта:

1. blog-server - бекенд системы блога, отвечающий за логику работы с пользователями и постами;
2. blog-client - библиотека для взаимодействия с бекендом системы блога. Представляет собой инструментарий для 
взаимодействия с бекендом посредством протоколов HTTP и gRPC.
3. blog-cli - консольный клиент для взаимодействия с системой блога. Использует инструментарий blog-client, 
дает пользователю набор доступных команд для взаимодействия, а также отображает результаты его взаимодействия;
4. blog-wasm - WASM-модуль для взаимодействия пользователя с системой блога посредством использования WEB-интерфейса.

## Необходимые зависимости

```
sudo apt-get install protobuf-compiler
cargo install sqlx-cli
cargo install wasm-pack
```

Необходимо установить postgres, а также создать БД, в которой в дальнейшем будут храниться необходимые для бекенда 
сущности.
Перед запуском серверной части необходимо заполнить переменную окружения DATABASE_URL в формате
```
DATABASE_URL=postgres://postgres:passw0rd@localhost/blog_project
```

Требуемые миграции БД будут выполнены автоматически при старте серверной части.

Также необходимо заполнить переменную окружения ключом для генерации JWT-токенов. Ключ - последовательность символов 
с длиной до 32 символов.

При запуске release сборки также необходимо заполнить переменную окружения CORS_ALLOWED_ORIGINS в формате

```
CORS_ALLOWED_ORIGINS="http://localhost:8080,http://127.0.0.1:8080"
```

## Сборка проекта

```
cargo build --workspace
```

## Cборка WASM-модуля

```
cd blog-wasm && ./build.sh && cd ..
```

## Запуск WASM-модуля

```
cd blog-wasm/www && python3 -m http.server 8080
```

Далее необходимо перейти в браузере на localhost:8080.

![img.png](readme/img.png)

## Запуск сервера

```
cargo run --bin blog-server
```

## Запуск консольного клиента

```
cargo run --bin blog-cli
```

Доступен help по командам и опциям:

```
cargo run --bin blog-cli -- --help

Взаимодействие с системой блога

Usage: blog-cli [OPTIONS] <COMMAND>

Commands:
  register     Регистрация нового пользователя
  login        Авторизация текущего пользователя
  create-post  Создать пост
  get-post     Получить пост
  get-posts    Получить посты
  update-post  Обновить пост
  delete-post  Удалить пост
  help         Print this message or the help of the given subcommand(s)

Options:
      --grpc             Взаимодействие через gRPC-протокол
      --server <SERVER>  Адрес сервера (по-умолчанию localhost:3000 для HTTP или localhost:50051 для gRPC)
  -h, --help             Print help
  -V, --version          Print version
```

### Пример команды

```
cargo run --bin blog-cli -- get-posts

Идентификатор поста: 20
Заголовок поста: заголовок
Содержимое поста: очень информативное содержимое
Идентификатор пользователя-автора поста: 2
Время создания поста: 2026-02-04 08:55:28.701115 UTC
Время последнего обновления поста: 2026-02-04 09:00:07.649539 UTC
```

или

```
curl -X GET "http://127.0.0.1:3000/api/posts" -v
Note: Unnecessary use of -X or --request, GET is already inferred.
*   Trying 127.0.0.1:3000...
* Connected to 127.0.0.1 (127.0.0.1) port 3000
> GET /api/posts HTTP/1.1
> Host: 127.0.0.1:3000
> User-Agent: curl/8.4.0
> Accept: */*
> 
< HTTP/1.1 200 OK
< content-type: application/json
< vary: origin, access-control-request-method, access-control-request-headers
< content-length: 142
< date: Wed, 04 Feb 2026 10:42:52 GMT
< 
* Connection #0 to host 127.0.0.1 left intact
[{"id":2,"title":"asfd","content":"sadf","author_id":1,"created_at":"2026-02-04T10:42:49.274383Z","updated_at":"2026-02-04T10:42:49.274383Z"}]
```
