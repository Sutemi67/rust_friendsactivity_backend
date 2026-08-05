use std::net::SocketAddr;

/// Конфигурация приложения, загружаемая из переменных окружения.
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("BIND_ADDR")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0.0.0.0:6655".to_string())
            .parse()
            .expect("BIND_ADDR должен быть валидным адресом вида host:port");

        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "sqlite://friends_activity.db".to_string());

        Self {
            bind_addr,
            database_url,
        }
    }
}
