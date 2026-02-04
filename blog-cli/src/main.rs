//! Консольный клиент для взаимодействия с системой блога.

#![deny(unreachable_pub)]

use clap::{Parser, Subcommand};
use std::borrow::Cow;

use blog_client::{BlogClient, Transport};

/// Взаимодействие с системой блога.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Доступные команды.
    #[clap(subcommand)]
    command: Commands,

    /// Взаимодействие через gRPC-протокол.
    #[arg(long)]
    grpc: bool,

    /// Адрес сервера (по-умолчанию localhost:3000 для HTTP или localhost:50051 для gRPC).
    #[arg(long)]
    server: Option<String>,
}

/// Доступные команды.
#[derive(Subcommand)]
enum Commands {
    /// Регистрация нового пользователя.
    Register {
        /// Имя пользователя.
        #[arg(long)]
        username: String,

        /// Email-адрес пользователя.
        #[arg(long)]
        email: String,

        /// Пароль пользователя.
        #[arg(long)]
        password: String,
    },

    /// Авторизация текущего пользователя.
    Login {
        /// Имя пользователя.
        #[arg(long)]
        username: String,

        /// Пароль пользователя.
        #[arg(long)]
        password: String,
    },

    /// Создать пост.
    CreatePost {
        /// Заголовок поста.
        #[arg(long)]
        title: String,

        /// Содержимое поста.
        #[arg(long)]
        content: String,
    },

    /// Получить пост.
    GetPost {
        /// Идентификатор поста.
        #[arg(long)]
        id: i64,
    },

    /// Получить посты.
    GetPosts {
        /// Максимальное количество постов в ответе.
        #[arg(default_value_t = 100)]
        limit: i64,

        /// Сдвиг от первого поста.
        #[arg(default_value_t = 0)]
        offset: i64,
    },

    /// Обновить пост.
    UpdatePost {
        /// Идентификатор поста.
        #[arg(long)]
        id: i64,

        /// Заголовок поста.
        #[arg(long)]
        title: Option<String>,

        /// Содержимое поста.
        #[arg(long)]
        content: Option<String>,
    },

    /// Удалить пост.
    DeletePost {
        /// Идентификатор поста.
        #[arg(long)]
        id: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let server = match args.server {
        Some(server) => server,
        None => {
            if args.grpc {
                "127.0.0.1:50051".to_string()
            } else {
                "127.0.0.1:3000".to_string()
            }
        }
    };

    let transport = if args.grpc {
        Transport::Grpc(server.parse()?)
    } else {
        Transport::Http(server.parse()?)
    };

    let token_path = ".blog_token";

    let token = if std::fs::exists(token_path)? {
        Cow::Owned(std::fs::read_to_string(".blog_token")?)
    } else {
        Cow::Borrowed("")
    };

    let mut client = BlogClient::new(transport).await?;

    if !token.is_empty() {
        client.set_token(token.to_string());
    }

    match args.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let user = client.register(&username, &email, &password).await?;

            tokio::fs::write(
                token_path,
                client.get_token().ok_or(anyhow::anyhow!(
                    "Токен не был установлен после успешной регистрации!"
                ))?,
            )
            .await?;

            println!("Зарегистрированный пользователь:");

            println!("{}", user);
        }
        Commands::Login { username, password } => {
            let user = client.login(&username, &password).await?;

            tokio::fs::write(
                token_path,
                client.get_token().ok_or(anyhow::anyhow!(
                    "Токен не был установлен после успешного логина!"
                ))?,
            )
            .await?;

            println!("Авторизованный пользователь:");

            println!("{}", user);
        }
        Commands::CreatePost { title, content } => {
            let post = client.create_post(&title, &content).await?;

            println!("Созданный пост:");

            println!("{}", post);
        }
        Commands::GetPost { id } => {
            let post = client.get_post(id).await?;

            println!("Полученный пост:");

            println!("{}", post);
        }
        Commands::GetPosts { limit, offset } => {
            let posts = client.get_posts(limit, offset).await?;

            println!("Полученные посты:\n");

            for post in posts {
                println!("{}\n", post);
                println!("-----------");
            }
        }
        Commands::UpdatePost { id, title, content } => {
            let post = client.update_post(id, title, content).await?;

            println!("Обновленный пост:");

            println!("{}", post);
        }
        Commands::DeletePost { id } => {
            client.delete_post(id).await?;

            println!("Пост удален!")
        }
    }

    Ok(())
}
