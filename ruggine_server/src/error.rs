
// ============================================================================
// error.rs - Gestione centralizzata degli errori custom per il backend Ruggine
// Definisce l'enum AppError e le conversioni/risposte per errori comuni (DB, hashing, validazione)
// ============================================================================

// Import standard e di terze parti
use std::fmt;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use password_hash;
use serde_json;
use sqlx;

// ============================================================================
// Enum e trait per la gestione degli errori applicativi
// ============================================================================
#[derive(Debug)]
pub enum AppError {
    SqlxError(sqlx::Error),
    HashError(password_hash::Error),
    ValidationError(String),
}

// Implementazione del trait Display per AppError
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_message = match self {
            AppError::SqlxError(e) => format!("Errore DB: {}", e),
            AppError::HashError(e) => format!("Errore Hashing: {}", e),
            AppError::ValidationError(msg) => msg.to_string(), // Usa direttamente il messaggio
        };
        write!(f, "{}", error_message)
    }
}

// Conversione AppError in risposta HTTP per Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SqlxError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Errore DB: {}", e),
            ),
            AppError::HashError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Errore Hashing: {}", e),
            ),
            // --- AGGIUNGI QUESTO CASO ---
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST, // Codice 400
                msg,
            ),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

// Conversioni automatiche da errori esterni a AppError
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::SqlxError(e)
    }
}
impl From<password_hash::Error> for AppError {
    fn from(e: password_hash::Error) -> Self {
        AppError::HashError(e)
    }
}