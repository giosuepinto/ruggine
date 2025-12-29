
// ============================================================================
// authentication.rs - Data Transfer Objects (DTO) per autenticazione
// Strutture per payload e risposta di login/registrazione, claims JWT
// ============================================================================

// Import di terze parti
use serde::{Deserialize, Serialize};

// DTO per le richieste HTTP a /login e /register
#[derive(Deserialize, Debug)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

// DTO per le risposte HTTP da /login e /register
#[derive(Serialize, Debug, Clone)]
pub struct AuthResponse {
    pub message: String,
}

// Claims JWT per autenticazione
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (Username)
    pub user_id: i64, // ID Utente
    pub exp: usize,  // Scadenza (Expiration)
}