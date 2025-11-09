use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use password_hash; // Importa la cassa
use serde_json; // Per il JSON nell'errore
use sqlx; // Per sqlx::Error

// Rendi 'pub' l'enum
pub enum AppError {
    SqlxError(sqlx::Error),
    HashError(password_hash::Error),
    ValidationError(String),
}

// Rendi 'pub' l'implementazione
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

// Rendi 'pub' le conversioni
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