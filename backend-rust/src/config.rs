use std::{env, net::SocketAddr};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub max_db_connections: u32,
    pub service_name: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
    pub google_connector_redirect_uri: Option<String>,
    pub google_oauth_scope: String,
    pub session_ttl_hours: i64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var: {0}")]
    MissingEnv(&'static str),
    #[error("invalid env var {name}: {reason}")]
    InvalidEnv { name: &'static str, reason: String },
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env::var("APP_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .map_err(|err: std::net::AddrParseError| ConfigError::InvalidEnv {
                name: "APP_BIND_ADDR",
                reason: err.to_string(),
            })?;

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?;

        let max_db_connections = env::var("MAX_DB_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|err: std::num::ParseIntError| ConfigError::InvalidEnv {
                name: "MAX_DB_CONNECTIONS",
                reason: err.to_string(),
            })?;

        let service_name =
            env::var("SERVICE_NAME").unwrap_or_else(|_| "aegis-broker-backend".to_string());

        let google_oauth_scope =
            env::var("GOOGLE_OAUTH_SCOPE").unwrap_or_else(|_| "openid email profile".to_string());

        let session_ttl_hours = env::var("SESSION_TTL_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .map_err(|err: std::num::ParseIntError| ConfigError::InvalidEnv {
                name: "SESSION_TTL_HOURS",
                reason: err.to_string(),
            })?;

        Ok(Self {
            bind_addr,
            database_url,
            max_db_connections,
            service_name,
            google_client_id: env::var("GOOGLE_CLIENT_ID")
                .ok()
                .or_else(|| env::var("GOOGLE_OAUTH_CLIENT_ID").ok()),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .ok()
                .or_else(|| env::var("GOOGLE_OAUTH_CLIENT_SECRET").ok()),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")
                .ok()
                .or_else(|| env::var("GOOGLE_OAUTH_REDIRECT_URI").ok()),
            google_connector_redirect_uri: env::var("GOOGLE_CONNECTOR_REDIRECT_URI").ok(),
            google_oauth_scope,
            session_ttl_hours,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn fails_without_database_url() {
        std::env::remove_var("DATABASE_URL");
        std::env::set_var("APP_BIND_ADDR", "127.0.0.1:8080");

        let result = AppConfig::from_env();

        assert!(matches!(
            result,
            Err(ConfigError::MissingEnv("DATABASE_URL"))
        ));
    }

    #[test]
    #[serial]
    fn parses_connection_and_ttl() {
        std::env::set_var(
            "DATABASE_URL",
            "postgres://postgres:postgres@localhost:5432/aegisbroker",
        );
        std::env::set_var("SESSION_TTL_HOURS", "12");
        std::env::set_var("MAX_DB_CONNECTIONS", "7");

        let cfg = AppConfig::from_env().expect("config");

        assert_eq!(cfg.session_ttl_hours, 12);
        assert_eq!(cfg.max_db_connections, 7);
    }
}
