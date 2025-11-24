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

use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;
use crate::DTOs::authentication::Claims;
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
) -> Result<String, AppError> {

    // 1. Validazione Input
    if payload.username.trim().is_empty() || payload.password.is_empty() {
        return Err(AppError::ValidationError("Username e password non possono essere vuoti.".to_string()));
    }

    // 2. Cerca Utente nel DB
    let user = user_repository::find_by_username(db, &payload.username)
        .await?
        .ok_or_else(|| AppError::ValidationError("Invalid username or password.".to_string()))?;

    // 3. Verifica Password
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::ValidationError("Invalid username or password.".to_string()));
    }

    // 4. GENERA JWT TOKEN (Nuova Logica)
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // Scade in 24 ore
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.username.clone(),
        user_id: user.id,
        exp: expiration as usize,
    };

    // NOTA: In produzione, questa chiave segreta deve stare in una variabile d'ambiente (.env)!
    // Per il progetto universitario, una costante qui è accettabile ma va commentata.
    let secret = "segreto_super_segreto_da_non_mettere_nel_codice";

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|e| AppError::ValidationError(format!("Errore creazione token: {}", e)))?;

    Ok(token)
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