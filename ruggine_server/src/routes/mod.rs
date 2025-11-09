use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

// Importa i nostri moduli
use crate::state::AppState;

// Dichiara i file 'fratelli'
pub mod auth;
pub mod ws;

// Funzione helper che 'main.rs' userà per costruire il router
pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(auth::register_handler))
        .route("/login", post(auth::login_handler))
        .route("/ws", get(ws::ws_handler))
}