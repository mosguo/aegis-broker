use std::{env, net::SocketAddr};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub service_name: String,
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
        let service_name =
            env::var("SERVICE_NAME").unwrap_or_else(|_| "aegis-broker-backend".to_string());

        Ok(Self {
            bind_addr,
            database_url,
            service_name,
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
}
