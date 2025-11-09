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

    // --- BLOCCO DI VALIDAZIONE AGGIUNTO ---
    if payload.username.trim().is_empty() {
        return Err(AppError::ValidationError("Il campo Username non può essere vuoto.".to_string()));
    }

    // Aggiungiamo un controllo anche sulla lunghezza minima
    if payload.username.len() < 3 {
        return Err(AppError::ValidationError("Il campo Username deve essere lungo almeno 3 caratteri.".to_string()));
    }

    if payload.password.is_empty() {
        return Err(AppError::ValidationError("Il campo Password non può essere vuoto".to_string()));
    }

    // Aggiungiamo un controllo anche sulla lunghezza minima
    if payload.password.len() < 8 {
        return Err(AppError::ValidationError("Il campo Password deve essere lungo almeno 8 caratteri.".to_string()));
    }
    // --- FINE BLOCCO DI VALIDAZIONE ---

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

    // --- VALIDAZIONE AGGIUNTA ---
    // Blocca password o username vuoti PRIMA di controllare il DB
    if payload.username.trim().is_empty() || payload.password.is_empty() {
        return Err(AppError::ValidationError("Username and password cannot be empty.".to_string()));
    }
    // --- FINE VALIDAZIONE ---

    // Cerca utente nel DB
    let user = sqlx::query_as::<_, (String,)>( // (String,) è solo per l'hash
                                               "SELECT password_hash FROM users WHERE username = ?",
    )
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    if let Some((hash,)) = user {
        // Verifica password
        if verify_password(&payload.password, &hash)? {
            // --- UNICO CASO DI SUCCESSO ---
            tracing::debug!("Login riuscito per: {}", payload.username);
            Ok(Json(AuthResponse {
                message: format!("Login riuscito per {}", payload.username),
            }))
        } else {
            // --- CASO DI FALLIMENTO 1 ---
            tracing::debug!("Password errata per: {}", payload.username);
            Err(AppError::ValidationError("Invalid username or password.".to_string()))
        }
    } else {
        // --- CASO DI FALLIMENTO 2 ---
        tracing::debug!("Utente non trovato: {}", payload.username);
        Err(AppError::ValidationError("Invalid username or password.".to_string()))
    }
}