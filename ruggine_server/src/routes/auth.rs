use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Importa i nostri moduli!
use crate::error::AppError;
use crate::services::auth_service::{hash_password, verify_password};
use crate::state::AppState;

// --- Struct per JSON ---
#[derive(Deserialize)]
pub struct AuthPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    message: String,
}

// --- Handlers ---
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::debug!("Tentativo di registrazione per: {}", payload.username);

    let password_hash = hash_password(&payload.password)?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&payload.username)
        .bind(&password_hash)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok(Json(AuthResponse {
            message: "Utente registrato con successo".to_string(),
        })),
        Err(e) => {
            tracing::warn!("Errore registrazione: {:?}", e);
            Err(AppError::SqlxError(e))
        }
    }
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::debug!("Tentativo di login per: {}", payload.username);

    let user = sqlx::query_as::<_, (String,)>(
        "SELECT password_hash FROM users WHERE username = ?",
    )
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    if let Some((hash,)) = user {
        if verify_password(&payload.password, &hash)? {
            tracing::debug!("Login riuscito per: {}", payload.username);
            Ok(Json(AuthResponse {
                message: format!("Login riuscito per {}", payload.username),
            }))
        } else {
            tracing::debug!("Password errata per: {}", payload.username);
            Ok(Json(AuthResponse {
                message: "Password errata".to_string(),
            }))
        }
    } else {
        Ok(Json(AuthResponse {
            message: "Utente non trovato".to_string(),
        }))
    }
}