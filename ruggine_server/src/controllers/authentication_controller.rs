// src/controllers/auth_controller.rs
use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    error::AppError,
    // Importa i DTO per l'autenticazione
    DTOs::authentication::{AuthPayload, AuthResponse},
    // Importa il servizio di autenticazione
    services::auth_service,
    state::AppState,
};

/// Controller per la registrazione di un nuovo utente.
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {

    // 1. Chiama il Service Layer per eseguire tutta la logica
    auth_service::register_user(&state.db, payload).await?;

    // 2. Se tutto va bene, restituisci la risposta
    Ok(Json(AuthResponse {
        message: "Utente registrato con successo".to_string(),
    }))
}

/// Controller per il login di un utente.
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {

    // 1. Chiama il Service Layer
    // Il servizio ora restituisce l'username, che usiamo come token
    let username_token = auth_service::login_user(&state.db, payload).await?;

    // 2. Restituisci il "token" (l'username) nel campo messaggio.
    // Il frontend (LoginPage.jsx) dovrà essere aggiornato
    // per leggere 'data.message' e salvarlo come token.
    Ok(Json(AuthResponse {
        message: username_token,
    }))
}