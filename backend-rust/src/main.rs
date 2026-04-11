mod config;
mod error;

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, Method},
    routing::{get, put},
    Json, Router,
};
use chrono::{Duration, Utc};
use config::AppConfig;
use error::AppError;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    db_pool: sqlx::PgPool,
    http_client: Client,
}

const REQUIRED_READY_TABLES: &[&str] = &[
    "workspaces",
    "oauth_state_tokens",
    "users",
    "oauth_identities",
    "user_profiles",
    "user_sessions",
    "role_definitions",
    "permission_definitions",
    "role_permissions",
    "user_roles",
    "event_store",
    "audit_chain",
];
const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000001";

fn db_read_failed(
    operation: &'static str,
    message: &'static str,
    trace_id: &str,
    database_url: &str,
    err: &sqlx::Error,
) -> AppError {
    error!(
        trace_id = %trace_id,
        operation = operation,
        status = "failed",
        error_code = "DB_READ_FAILED",
        error = %err,
        database_url = %database_url,
        "{message}"
    );
    AppError::internal("DB_READ_FAILED", message, trace_id.to_string())
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: String,
}

#[derive(Debug, Deserialize)]
struct CreateWorkspaceRequest {
    workspace_code: Option<String>,
    name: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceDto {
    workspace_id: Uuid,
    workspace_code: String,
    name: String,
    status: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct AuthStartResponse {
    auth_url: String,
    state: String,
    trace_id: String,
    workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthCallbackResponse {
    session_token: String,
    expires_at: String,
    trace_id: String,
    workspace_id: Uuid,
    user: UserProfileDto,
}

#[derive(Debug, Serialize)]
struct UserProfileDto {
    user_id: Uuid,
    workspace_id: Uuid,
    email: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    locale: String,
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProfileUpdateRequest {
    display_name: Option<String>,
    avatar_url: Option<String>,
    locale: String,
    reason_code: String,
}

#[derive(Debug, Serialize)]
struct ProfileUpdateResponse {
    profile: UserProfileDto,
    trace_id: String,
}

#[derive(Debug, Deserialize)]
struct RoleUpdateRequest {
    role_codes: Vec<String>,
    reason_code: String,
}

#[derive(Debug, Serialize)]
struct RoleUpdateResponse {
    workspace_id: Uuid,
    user_id: Uuid,
    role_codes: Vec<String>,
    trace_id: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env().expect("failed to load config from environment");

    let db_pool = PgPoolOptions::new()
        .max_connections(config.max_db_connections)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        http_client: Client::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/v1/workspaces", get(list_workspaces))
        .route("/v1/workspaces", axum::routing::post(create_workspace))
        .route("/auth/google/start", get(auth_google_start))
        .route("/auth/google/callback", get(auth_google_callback))
        .route("/v1/me/profile", get(get_me_profile))
        .route("/v1/me/profile", put(update_me_profile))
        .route(
            "/v1/workspaces/:workspace_id/users/:user_id/roles",
            put(update_user_roles),
        )
        .layer(cors)
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
                trace_id: trace_id.clone(),
            }
        })?;

    ensure_required_schema_ready(&state.db_pool, &trace_id).await?;

    Ok(Json(HealthResponse {
        status: "ok",
        service: state.config.service_name.clone(),
    }))
}

async fn list_workspaces(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkspaceDto>>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    let rows = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>)>(
        "SELECT id, workspace_code, name, status, created_at
         FROM workspaces
         ORDER BY created_at ASC",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|err| {
        db_read_failed(
            "list_workspaces",
            "failed to list workspaces",
            &trace_id,
            &state.config.database_url,
            &err,
        )
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|(workspace_id, workspace_code, name, status, created_at)| WorkspaceDto {
                workspace_id,
                workspace_code,
                name,
                status,
                created_at: created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

async fn create_workspace(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceDto>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request(
            "WORKSPACE_NAME_REQUIRED",
            "workspace name is required",
            trace_id,
        ));
    }

    let workspace_code = req
        .workspace_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            let suffix: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            format!("ws-{}", suffix.to_ascii_lowercase())
        });

    let workspace_id = Uuid::new_v4();
    let created_at =
        sqlx::query_scalar::<_, chrono::DateTime<Utc>>(
            "INSERT INTO workspaces (id, workspace_code, name, status)
             VALUES ($1, $2, $3, 'active')
             RETURNING created_at",
        )
        .bind(workspace_id)
        .bind(&workspace_code)
        .bind(name)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|err| {
            let message = if err.to_string().contains("workspaces_workspace_code_key") {
                "workspace_code already exists"
            } else {
                "failed to create workspace"
            };
            AppError::bad_request("WORKSPACE_CREATE_FAILED", message, trace_id.clone())
        })?;

    Ok(Json(WorkspaceDto {
        workspace_id,
        workspace_code,
        name: name.to_string(),
        status: "active".to_string(),
        created_at: created_at.to_rfc3339(),
    }))
}

async fn auth_google_start(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AuthStartResponse>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    let workspace_id = parse_workspace_id(&params, &trace_id)?;
    ensure_workspace_exists(
        &state.db_pool,
        workspace_id,
        &trace_id,
        &state.config.database_url,
    )
    .await?;
    let client_id = state.config.google_client_id.clone().ok_or_else(|| {
        AppError::service_unavailable(
            "OAUTH_NOT_CONFIGURED",
            "google oauth not configured",
            trace_id.clone(),
        )
    })?;
    let redirect_uri = state.config.google_redirect_uri.clone().ok_or_else(|| {
        AppError::service_unavailable(
            "OAUTH_NOT_CONFIGURED",
            "google oauth not configured",
            trace_id.clone(),
        )
    })?;

    info!(
        trace_id = %trace_id,
        operation = "auth_google_start",
        workspace_id = %workspace_id,
        status = "resolved",
        "resolved workspace for oauth start"
    );

    let state_token = Uuid::new_v4().to_string();
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query(
        "INSERT INTO oauth_state_tokens (id, workspace_id, state_token, nonce, trace_id, expires_at)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(&state_token)
    .bind(&nonce)
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .map_err(|err| {
        error!(trace_id=%trace_id, operation="auth_google_start", workspace_id=%workspace_id, status="failed", error_code="DB_WRITE_FAILED", error=%err, "failed to store oauth state");
        AppError::internal("DB_WRITE_FAILED", "failed to store oauth state", trace_id.clone())
    })?;

    let mut auth_url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|_| AppError::internal("GOOGLE_AUTH_URL_INVALID", "failed to build google auth url", trace_id.clone()))?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &state.config.google_oauth_scope)
        .append_pair("state", &state_token)
        .append_pair("access_type", "offline")
        .append_pair("include_granted_scopes", "true")
        .append_pair("prompt", "consent");
    let auth_url = auth_url.into_string();

    info!(
        trace_id = %trace_id,
        operation = "auth_google_start",
        workspace_id = %workspace_id,
        client_id = %client_id,
        scope = %state.config.google_oauth_scope,
        redirect_uri = %redirect_uri,
        auth_url = %auth_url,
        status = "generated",
        "generated google auth url"
    );

    Ok(Json(AuthStartResponse {
        auth_url,
        state: state_token,
        trace_id,
        workspace_id,
    }))
}

async fn auth_google_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallbackQuery>,
) -> Result<Json<AuthCallbackResponse>, AppError> {
    let trace_id = Uuid::new_v4().to_string();

    let oauth_row = sqlx::query_as::<_, (Uuid, Uuid, bool)>(
        "SELECT id, workspace_id, used FROM oauth_state_tokens
         WHERE state_token = $1 AND expires_at > now()",
    )
    .bind(&query.state)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|err| {
        db_read_failed(
            "auth_google_callback",
            "failed to read oauth state",
            &trace_id,
            &state.config.database_url,
            &err,
        )
    })?
    .ok_or_else(|| {
        AppError::bad_request(
            "INVALID_OAUTH_STATE",
            "oauth state invalid or expired",
            trace_id.clone(),
        )
    })?;

    info!(
        trace_id = %trace_id,
        operation = "auth_google_callback",
        workspace_id = %oauth_row.1,
        status = "resolved",
        "resolved workspace for oauth callback"
    );

    if oauth_row.2 {
        return Err(AppError::bad_request(
            "OAUTH_STATE_ALREADY_USED",
            "oauth state already used",
            trace_id,
        ));
    }

    let client_id = state.config.google_client_id.clone().ok_or_else(|| {
        AppError::service_unavailable(
            "OAUTH_NOT_CONFIGURED",
            "google oauth not configured",
            trace_id.clone(),
        )
    })?;
    let client_secret = state.config.google_client_secret.clone().ok_or_else(|| {
        AppError::service_unavailable(
            "OAUTH_NOT_CONFIGURED",
            "google oauth not configured",
            trace_id.clone(),
        )
    })?;
    let redirect_uri = state.config.google_redirect_uri.clone().ok_or_else(|| {
        AppError::service_unavailable(
            "OAUTH_NOT_CONFIGURED",
            "google oauth not configured",
            trace_id.clone(),
        )
    })?;

    let token_resp = state
        .http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", query.code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable(
                "GOOGLE_TOKEN_UNAVAILABLE",
                "google token endpoint unavailable",
                trace_id.clone(),
            )
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::bad_request(
                "GOOGLE_CODE_EXCHANGE_FAILED",
                "google code exchange failed",
                trace_id.clone(),
            )
        })?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|_| {
            AppError::internal(
                "GOOGLE_TOKEN_PARSE_FAILED",
                "google token response parse failed",
                trace_id.clone(),
            )
        })?;

    let userinfo = state
        .http_client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(token_resp.access_token)
        .send()
        .await
        .map_err(|_| {
            AppError::service_unavailable(
                "GOOGLE_USERINFO_UNAVAILABLE",
                "google userinfo unavailable",
                trace_id.clone(),
            )
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::bad_request(
                "GOOGLE_USERINFO_FAILED",
                "google userinfo failed",
                trace_id.clone(),
            )
        })?
        .json::<GoogleUserInfo>()
        .await
        .map_err(|_| {
            AppError::internal(
                "GOOGLE_USERINFO_PARSE_FAILED",
                "google userinfo parse failed",
                trace_id.clone(),
            )
        })?;

    let session_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let session_expires_at = Utc::now() + Duration::hours(state.config.session_ttl_hours);

    let mut tx = state.db_pool.begin().await.map_err(|_| {
        AppError::internal(
            "DB_TX_START_FAILED",
            "failed to start transaction",
            trace_id.clone(),
        )
    })?;

    let user_id = Uuid::new_v4();
    let identity_id = Uuid::new_v4();
    let profile_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let audit_id = Uuid::new_v4();

    let existing_user = sqlx::query_as::<_, (Uuid,)>(
        "SELECT user_id FROM oauth_identities WHERE provider = 'google' AND provider_sub = $1",
    )
    .bind(&userinfo.sub)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|err| {
        db_read_failed(
            "auth_google_callback",
            "failed to read oauth identity",
            &trace_id,
            &state.config.database_url,
            &err,
        )
    })?;

    let resolved_user_id = if let Some((existing_user_id,)) = existing_user {
        existing_user_id
    } else {
        sqlx::query(
            "INSERT INTO users (id, workspace_id, email, status) VALUES ($1,$2,$3,'active')",
        )
        .bind(user_id)
        .bind(oauth_row.1)
        .bind(&userinfo.email)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            AppError::internal("DB_WRITE_FAILED", "failed to create user", trace_id.clone())
        })?;

        sqlx::query("INSERT INTO oauth_identities (id, user_id, provider, provider_sub, email) VALUES ($1,$2,'google',$3,$4)")
            .bind(identity_id)
            .bind(user_id)
            .bind(&userinfo.sub)
            .bind(&userinfo.email)
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to create oauth identity", trace_id.clone()))?;

        sqlx::query("INSERT INTO user_profiles (id, user_id, display_name, avatar_url, locale) VALUES ($1,$2,$3,$4,$5)")
            .bind(profile_id)
            .bind(user_id)
            .bind(userinfo.name.as_deref())
            .bind(userinfo.picture.as_deref())
            .bind("zh-TW")
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to create user profile", trace_id.clone()))?;

        sqlx::query("INSERT INTO user_roles (id, workspace_id, user_id, role_code, assigned_by, reason_code, trace_id) VALUES ($1,$2,$3,'workspace_member',$4,'AUTH_FIRST_LOGIN',$5)")
            .bind(Uuid::new_v4())
            .bind(oauth_row.1)
            .bind(user_id)
            .bind(user_id)
            .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to assign default role", trace_id.clone()))?;

        user_id
    };

    sqlx::query("UPDATE oauth_state_tokens SET used = true, used_at = now() WHERE id = $1")
        .bind(oauth_row.0)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            AppError::internal(
                "DB_WRITE_FAILED",
                "failed to consume oauth state",
                trace_id.clone(),
            )
        })?;

    sqlx::query("INSERT INTO user_sessions (id, user_id, workspace_id, session_token, trace_id, expires_at) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(session_id)
        .bind(resolved_user_id)
        .bind(oauth_row.1)
        .bind(&session_token)
        .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
        .bind(session_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to create session", trace_id.clone()))?;

    sqlx::query(
        "INSERT INTO event_store (id, workspace_id, aggregate_type, aggregate_id, event_type, event_version, reason_code, trace_id, payload)
         VALUES ($1,$2,'user', $3, 'UserLoggedInByGoogleOAuth', 1, 'AUTH_LOGIN_SUCCESS', $4, $5)",
    )
    .bind(event_id)
    .bind(oauth_row.1)
    .bind(resolved_user_id.to_string())
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(json!({
        "provider": "google",
        "email": userinfo.email,
    }))
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to write event", trace_id.clone()))?;

    sqlx::query(
        "INSERT INTO audit_chain (id, workspace_id, actor_type, actor_id, operation_name, aggregate_type, aggregate_id, reason_code, trace_id, event_id, prev_hash, entry_hash, payload)
         VALUES ($1,$2,'user',$3,'AuthGoogleCallback','user',$4,'AUTH_LOGIN_SUCCESS',$5,$6,NULL,$7,$8)",
    )
    .bind(audit_id)
    .bind(oauth_row.1)
    .bind(resolved_user_id.to_string())
    .bind(resolved_user_id.to_string())
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(event_id)
    .bind(vec![1_u8; 32])
    .bind(json!({"session_id": session_id.to_string()}))
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to write audit", trace_id.clone()))?;

    tx.commit().await.map_err(|_| {
        AppError::internal(
            "DB_TX_COMMIT_FAILED",
            "failed to commit auth flow",
            trace_id.clone(),
        )
    })?;

    let profile = fetch_user_profile(
        &state.db_pool,
        resolved_user_id,
        oauth_row.1,
        &trace_id,
        &state.config.database_url,
    )
    .await?;

    Ok(Json(AuthCallbackResponse {
        session_token,
        expires_at: session_expires_at.to_rfc3339(),
        trace_id,
        workspace_id: oauth_row.1,
        user: profile,
    }))
}

async fn get_me_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserProfileDto>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    let (user_id, workspace_id) =
        resolve_session_identity(&state.db_pool, &headers, &trace_id, &state.config.database_url)
            .await?;
    let profile = fetch_user_profile(
        &state.db_pool,
        user_id,
        workspace_id,
        &trace_id,
        &state.config.database_url,
    )
    .await?;
    Ok(Json(profile))
}

async fn update_user_roles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<RoleUpdateRequest>,
) -> Result<Json<RoleUpdateResponse>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    if req.role_codes.is_empty() {
        return Err(AppError::bad_request(
            "ROLE_CODES_EMPTY",
            "role_codes must not be empty",
            trace_id,
        ));
    }

    if req.reason_code.trim().is_empty() {
        return Err(AppError::bad_request(
            "REASON_CODE_REQUIRED",
            "reason_code is required",
            trace_id,
        ));
    }

    let (actor_user_id, actor_workspace_id) =
        resolve_session_identity(&state.db_pool, &headers, &trace_id, &state.config.database_url)
            .await?;

    if actor_workspace_id != workspace_id {
        return Err(AppError::forbidden(
            "WORKSPACE_SCOPE_MISMATCH",
            "session workspace scope mismatch",
            trace_id,
        ));
    }

    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM user_roles
            WHERE workspace_id = $1 AND user_id = $2 AND role_code = 'workspace_admin' AND revoked_at IS NULL
        )",
    )
    .bind(workspace_id)
    .bind(actor_user_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|err| {
        db_read_failed(
            "update_user_roles",
            "failed to validate actor role",
            &trace_id,
            &state.config.database_url,
            &err,
        )
    })?;

    if !is_admin {
        return Err(AppError::forbidden(
            "INSUFFICIENT_ROLE_PERMISSION",
            "workspace_admin role required",
            trace_id,
        ));
    }

    let mut tx = state.db_pool.begin().await.map_err(|_| {
        AppError::internal(
            "DB_TX_START_FAILED",
            "failed to start transaction",
            trace_id.clone(),
        )
    })?;

    sqlx::query(
        "UPDATE user_roles
         SET revoked_at = now(), revoked_by = $1, revoke_reason_code = $2
         WHERE workspace_id = $3 AND user_id = $4 AND revoked_at IS NULL",
    )
    .bind(actor_user_id)
    .bind(&req.reason_code)
    .bind(workspace_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| {
        AppError::internal(
            "DB_WRITE_FAILED",
            "failed to revoke previous roles",
            trace_id.clone(),
        )
    })?;

    for role_code in &req.role_codes {
        sqlx::query(
            "INSERT INTO user_roles (id, workspace_id, user_id, role_code, assigned_by, reason_code, trace_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(user_id)
        .bind(role_code)
        .bind(actor_user_id)
        .bind(&req.reason_code)
        .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to assign roles", trace_id.clone()))?;
    }

    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO event_store (id, workspace_id, aggregate_type, aggregate_id, event_type, event_version, reason_code, trace_id, payload)
         VALUES ($1,$2,'user',$3,'UserRolesUpdated',1,$4,$5,$6)",
    )
    .bind(event_id)
    .bind(workspace_id)
    .bind(user_id.to_string())
    .bind(&req.reason_code)
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(json!({"roles": req.role_codes}))
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to write role update event", trace_id.clone()))?;

    sqlx::query(
        "INSERT INTO audit_chain (id, workspace_id, actor_type, actor_id, operation_name, aggregate_type, aggregate_id, reason_code, trace_id, event_id, prev_hash, entry_hash, payload)
         VALUES ($1,$2,'user',$3,'UpdateUserRoles','user',$4,$5,$6,$7,NULL,$8,$9)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(actor_user_id.to_string())
    .bind(user_id.to_string())
    .bind(&req.reason_code)
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(event_id)
    .bind(vec![2_u8; 32])
    .bind(json!({"role_codes": req.role_codes}))
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::internal("DB_WRITE_FAILED", "failed to write role update audit", trace_id.clone()))?;

    tx.commit().await.map_err(|_| {
        AppError::internal(
            "DB_TX_COMMIT_FAILED",
            "failed to commit role update",
            trace_id.clone(),
        )
    })?;

    Ok(Json(RoleUpdateResponse {
        workspace_id,
        user_id,
        role_codes: req.role_codes,
        trace_id,
    }))
}

async fn update_me_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ProfileUpdateRequest>,
) -> Result<Json<ProfileUpdateResponse>, AppError> {
    let trace_id = Uuid::new_v4().to_string();
    let (user_id, workspace_id) =
        resolve_session_identity(&state.db_pool, &headers, &trace_id, &state.config.database_url)
            .await?;

    let locale = normalize_supported_locale(&req.locale, &trace_id)?;
    let reason_code = req.reason_code.trim();
    if reason_code.is_empty() {
        return Err(AppError::bad_request(
            "REASON_CODE_REQUIRED",
            "reason_code is required",
            trace_id,
        ));
    }

    let display_name = req
        .display_name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let avatar_url = req
        .avatar_url
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let mut tx = state.db_pool.begin().await.map_err(|_| {
        AppError::internal(
            "DB_TX_START_FAILED",
            "failed to start profile update transaction",
            trace_id.clone(),
        )
    })?;

    sqlx::query(
        "INSERT INTO user_profiles (id, user_id, display_name, avatar_url, locale, updated_at)
         VALUES ($1,$2,$3,$4,$5,now())
         ON CONFLICT (user_id)
         DO UPDATE SET display_name = EXCLUDED.display_name, avatar_url = EXCLUDED.avatar_url, locale = EXCLUDED.locale, updated_at = now()",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(display_name.as_deref())
    .bind(avatar_url.as_deref())
    .bind(locale)
    .execute(&mut *tx)
    .await
    .map_err(|_| {
        AppError::internal(
            "DB_WRITE_FAILED",
            "failed to upsert user profile",
            trace_id.clone(),
        )
    })?;

    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO event_store (id, workspace_id, aggregate_type, aggregate_id, event_type, event_version, reason_code, trace_id, payload)
         VALUES ($1,$2,'user',$3,'UserProfileUpdated',1,$4,$5,$6)",
    )
    .bind(event_id)
    .bind(workspace_id)
    .bind(user_id.to_string())
    .bind(reason_code)
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(json!({"display_name": display_name, "avatar_url": avatar_url, "locale": locale}))
    .execute(&mut *tx)
    .await
    .map_err(|_| {
        AppError::internal(
            "DB_WRITE_FAILED",
            "failed to write profile update event",
            trace_id.clone(),
        )
    })?;

    sqlx::query(
        "INSERT INTO audit_chain (id, workspace_id, actor_type, actor_id, operation_name, aggregate_type, aggregate_id, reason_code, trace_id, event_id, prev_hash, entry_hash, payload)
         VALUES ($1,$2,'user',$3,'UpdateMeProfile','user',$4,$5,$6,$7,NULL,$8,$9)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(user_id.to_string())
    .bind(user_id.to_string())
    .bind(reason_code)
    .bind(Uuid::parse_str(&trace_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(event_id)
    .bind(vec![3_u8; 32])
    .bind(json!({"display_name": display_name, "avatar_url": avatar_url, "locale": locale}))
    .execute(&mut *tx)
    .await
    .map_err(|_| {
        AppError::internal(
            "DB_WRITE_FAILED",
            "failed to write profile update audit",
            trace_id.clone(),
        )
    })?;

    tx.commit().await.map_err(|_| {
        AppError::internal(
            "DB_TX_COMMIT_FAILED",
            "failed to commit profile update",
            trace_id.clone(),
        )
    })?;

    let profile = fetch_user_profile(
        &state.db_pool,
        user_id,
        workspace_id,
        &trace_id,
        &state.config.database_url,
    )
    .await?;
    Ok(Json(ProfileUpdateResponse { profile, trace_id }))
}

fn parse_workspace_id(params: &HashMap<String, String>, trace_id: &str) -> Result<Uuid, AppError> {
    let fallback =
        Uuid::parse_str(DEFAULT_WORKSPACE_ID).expect("default workspace id must be valid");

    match params.get("workspace_id").map(|value| value.trim()) {
        None | Some("") => Ok(fallback),
        Some(value) => Uuid::parse_str(value).map_err(|_| {
            AppError::bad_request(
                "WORKSPACE_ID_INVALID",
                "query param workspace_id is invalid uuid",
                trace_id.to_string(),
            )
        }),
    }
}

async fn ensure_workspace_exists(
    pool: &sqlx::PgPool,
    workspace_id: Uuid,
    trace_id: &str,
    database_url: &str,
) -> Result<(), AppError> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1
            FROM workspaces
            WHERE id = $1 AND status = 'active'
        )",
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        error!(
            trace_id = %trace_id,
            operation = "ensure_workspace_exists",
            workspace_id = %workspace_id,
            status = "failed",
            error_code = "DB_READ_FAILED",
            error = %err,
            database_url = %database_url,
            "failed to validate workspace"
        );
        AppError::internal(
            "DB_READ_FAILED",
            "failed to validate workspace",
            trace_id.to_string(),
        )
    })?;

    if !exists {
        return Err(AppError::bad_request(
            "WORKSPACE_NOT_FOUND",
            "workspace_id does not exist or is not active",
            trace_id.to_string(),
        ));
    }

    Ok(())
}

async fn ensure_required_schema_ready(
    pool: &sqlx::PgPool,
    trace_id: &str,
) -> Result<(), AppError> {
    for table_name in REQUIRED_READY_TABLES {
        let regclass_name = format!("public.{table_name}");
        let exists = sqlx::query_scalar::<_, Option<String>>("SELECT to_regclass($1)::text")
            .bind(&regclass_name)
            .fetch_one(pool)
            .await
            .map_err(|_| AppError::ReadinessFailed {
                error_code: "DB_SCHEMA_CHECK_FAILED",
                message: "failed to verify required schema".to_string(),
                trace_id: trace_id.to_string(),
            })?;

        if exists.is_none() {
            return Err(AppError::ReadinessFailed {
                error_code: "DB_SCHEMA_NOT_READY",
                message: format!("missing required table: {table_name}"),
                trace_id: trace_id.to_string(),
            });
        }
    }

    let default_workspace_ready = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1
            FROM workspaces
            WHERE id = '00000000-0000-0000-0000-000000000001'
              AND workspace_code = 'default'
              AND status = 'active'
        )",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::ReadinessFailed {
        error_code: "DB_SCHEMA_CHECK_FAILED",
        message: "failed to verify default workspace bootstrap".to_string(),
        trace_id: trace_id.to_string(),
    })?;

    if !default_workspace_ready {
        return Err(AppError::ReadinessFailed {
            error_code: "WORKSPACE_BOOTSTRAP_MISSING",
            message: "default workspace bootstrap record is missing".to_string(),
            trace_id: trace_id.to_string(),
        });
    }

    Ok(())
}

async fn resolve_session_identity(
    pool: &sqlx::PgPool,
    headers: &HeaderMap,
    trace_id: &str,
    database_url: &str,
) -> Result<(Uuid, Uuid), AppError> {
    let auth = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            AppError::unauthorized(
                "SESSION_REQUIRED",
                "missing authorization header",
                trace_id.to_string(),
            )
        })?;

    let token = auth.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::unauthorized(
            "SESSION_REQUIRED",
            "authorization must be Bearer token",
            trace_id.to_string(),
        )
    })?;

    sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT user_id, workspace_id FROM user_sessions
         WHERE session_token = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|err| db_read_failed(
        "resolve_session_identity",
        "failed to read session",
        trace_id,
        database_url,
        &err,
    ))?
    .ok_or_else(|| {
        AppError::unauthorized(
            "SESSION_INVALID",
            "session invalid or expired",
            trace_id.to_string(),
        )
    })
}

async fn fetch_user_profile(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    workspace_id: Uuid,
    trace_id: &str,
    database_url: &str,
) -> Result<UserProfileDto, AppError> {
    let (email, display_name, avatar_url, locale) =
        sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>)>(
            "SELECT u.email, p.display_name, p.avatar_url, p.locale
         FROM users u
         LEFT JOIN user_profiles p ON p.user_id = u.id
         WHERE u.id = $1 AND u.workspace_id = $2",
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
        .map_err(|err| db_read_failed(
            "fetch_user_profile",
            "failed to read user profile",
            trace_id,
            database_url,
            &err,
        ))?
        .ok_or_else(|| {
            AppError::bad_request("USER_NOT_FOUND", "user not found", trace_id.to_string())
        })?;

    let roles = sqlx::query_scalar::<_, String>(
        "SELECT role_code FROM user_roles WHERE workspace_id = $1 AND user_id = $2 AND revoked_at IS NULL",
    )
    .bind(workspace_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        db_read_failed(
            "fetch_user_profile",
            "failed to read user roles",
            trace_id,
            database_url,
            &err,
        )
    })?;

    Ok(UserProfileDto {
        user_id,
        workspace_id,
        email,
        display_name,
        avatar_url,
        locale: locale.unwrap_or_else(|| "zh-TW".to_string()),
        roles,
    })
}

fn normalize_supported_locale(locale: &str, trace_id: &str) -> Result<&'static str, AppError> {
    match locale.trim() {
        "zh-TW" => Ok("zh-TW"),
        "zh-CN" => Ok("zh-CN"),
        "en" => Ok("en"),
        "es" => Ok("es"),
        "tr" => Ok("tr"),
        _ => Err(AppError::bad_request(
            "LOCALE_NOT_SUPPORTED",
            "locale must be one of zh-TW, zh-CN, en, es, tr",
            trace_id.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_supported_locale;

    #[test]
    fn accepts_supported_locale() {
        let locale = normalize_supported_locale("zh-TW", "trace-id").expect("locale should pass");
        assert_eq!(locale, "zh-TW");
    }

    #[test]
    fn rejects_unsupported_locale() {
        let err = normalize_supported_locale("fr", "trace-id")
            .expect_err("unsupported locale should return error");
        let body = format!("{err:?}");
        assert!(body.contains("LOCALE_NOT_SUPPORTED"));
    }
}
