
// ============================================================================
// user_service.rs - Gestione utenti connessi via WebSocket e autenticazione JWT
// Strutture e funzioni per sessione utente e validazione token
// ============================================================================

// Import di terze parti
use sqlx::SqlitePool;
use jsonwebtoken::{decode, DecodingKey, Validation};

// Import interni
use crate::error::AppError;
use crate::dtos::authentication::Claims;

// Struct "Sessione" per un utente connesso via WebSocket (senza password_hash)
#[derive(Debug, Clone)]
pub struct WsUser {
    pub id: i64,
    pub username: String,
}

// Autentica un utente tramite il suo token (username) e restituisce i suoi dati di sessione
pub async fn authenticate_ws_user(
    _db: &SqlitePool, // Non ci serve più il DB per validare il token!
    token: &str
) -> Result<WsUser, AppError> {

    let secret = "segreto_super_segreto_da_non_mettere_nel_codice"; // Stessa chiave di sopra

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| AppError::ValidationError("Token JWT non valido o scaduto.".to_string()))?;

    // Il token è valido! Estraiamo i dati.
    Ok(WsUser {
        id: token_data.claims.user_id,
        username: token_data.claims.sub,
    })
}