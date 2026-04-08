mod config;
mod error;

use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use config::AppConfig;
use error::AppError;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    db_pool: sqlx::PgPool,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env().expect("failed to load config from environment");

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
    });

    let app = Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .with_state(app_state);

    info!(
        operation = "server_start",
        service = %config.service_name,
        bind_addr = %config.bind_addr,
        "starting backend server"
    );

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .expect("failed to bind tcp listener");

    axum::serve(listener, app).await.expect("server failed");
}

async fn health_live(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name.clone(),
    })
}

async fn health_ready(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, AppError> {
    let trace_id = Uuid::new_v4().to_string();

    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|err| {
            error!(
                trace_id = %trace_id,
                operation = "health_ready",
                aggregate_type = "system",
                aggregate_id = "backend",
                status = "failed",
                error_code = "DB_NOT_READY",
                error = %err,
                "readiness check failed"
            );
            AppError::ReadinessFailed {
                error_code: "DB_NOT_READY",
                message: "database connectivity check failed".to_string(),
                trace_id,
            }
        })?;

    Ok(Json(HealthResponse {
        status: "ok",
        service: state.config.service_name.clone(),
    }))
}
