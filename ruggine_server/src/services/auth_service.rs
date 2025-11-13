// src/services/auth_service.rs

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use password_hash;
use sqlx::SqlitePool;

// === NUOVI IMPORT ===
use crate::{
    error::AppError,
    // Importa il DTO per ricevere i dati dal controller
    DTOs::authentication::AuthPayload,
    // Importa il repository per accedere al DB
    repositories::user_repository,
};
// === FINE NUOVI IMPORT ===


// =================================================================
// FUNZIONI DI SERVIZIO (Logica di Business)
// =================================================================

/// Logica di business per la registrazione di un nuovo utente.
pub async fn register_user(
    db: &SqlitePool,
    payload: AuthPayload
) -> Result<(), AppError> {

    // 1. Logica di Business (Validazione) [logica spostata da authentication_controller]
    if payload.username.trim().is_empty() {
        return Err(AppError::ValidationError("Il campo Username non può essere vuoto.".to_string()));
    }
    if payload.username.len() < 3 {
        return Err(AppError::ValidationError("Il campo Username deve essere lungo almeno 3 caratteri.".to_string()));
    }
    if payload.password.is_empty() {
        return Err(AppError::ValidationError("Il campo Password non può essere vuoto".to_string()));
    }
    if payload.password.len() < 8 {
        return Err(AppError::ValidationError("Il campo Password deve essere lungo almeno 8 caratteri.".to_string()));
    }

    // 2. Logica di Business (Hashing)
    let password_hash = hash_password(&payload.password)?;

    // 3. Chiamata al Data Access Layer
    //    Controlliamo prima se l'utente esiste
    if user_repository::find_by_username(db, &payload.username).await?.is_some() {
        return Err(AppError::ValidationError("Username già in uso.".to_string()));
    }

    //    Se non esiste, lo creiamo
    user_repository::create_user(db, &payload.username, &password_hash).await?;

    Ok(())
}

/// Logica di business per il login di un utente.
pub async fn login_user(
    db: &SqlitePool,
    payload: AuthPayload
) -> Result<String, AppError> { // Restituisce il token (username)

    // 1. Logica di Business (Validazione) [logica spostata da authentication_controller]
    if payload.username.trim().is_empty() || payload.password.is_empty() {
        return Err(AppError::ValidationError("Username e password non possono essere vuoti.".to_string()));
    }

    // 2. Chiamata al Data Access Layer
    let user = user_repository::find_by_username(db, &payload.username)
        .await?
        // Se 'find_by_username' restituisce None, l'utente non esiste.
        .ok_or_else(|| AppError::ValidationError("Invalid username or password.".to_string()))?; //

    // 3. Logica di Business (Verifica)
    if !verify_password(&payload.password, &user.password_hash)? {
        // Password errata
        return Err(AppError::ValidationError("Invalid username or password.".to_string())); //
    }

    // Successo! Restituiamo l'username che useremo come token
    Ok(user.username)
}


// =================================================================
// FUNZIONI HELPER (Hashing)
// (Queste rimangono invariate dal tuo file originale)
// =================================================================

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}