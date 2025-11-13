// src/services/user_service.rs
use sqlx::SqlitePool;
use crate::{
    error::AppError,
    repositories::user_repository,
};

/// Struct "Sessione" per un utente connesso via WebSocket.
/// Omette la password_hash.
#[derive(Debug, Clone)]
pub struct WsUser {
    pub id: i64,
    pub username: String,
}

/// Autentica un utente tramite il suo token (username)
/// e restituisce i suoi dati di sessione.
pub async fn authenticate_ws_user(
    db: &SqlitePool,
    token: &str // Il token è l'username
) -> Result<WsUser, AppError> {

    let user_entity = user_repository::find_by_username(db, token)
        .await?
        // Errore se il token/username non esiste
        .ok_or_else(|| AppError::ValidationError("Token non valido.".to_string()))?;

    // Converte l'Entità del DB in un DTO sicuro per la sessione
    Ok(WsUser {
        id: user_entity.id,
        username: user_entity.username,
    })
}