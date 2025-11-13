// src/router.rs
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::{
    // Importa i NUOVI controller
    controllers::{authentication_controller, web_socket_controller},
    state::AppState,
};

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        // Rotte di Autenticazione
        .route("/register", post(authentication_controller::register_handler))
        .route("/login", post(authentication_controller::login_handler))
        // Rotta WebSocket
        .route("/ws", get(web_socket_controller::ws_handler))
}