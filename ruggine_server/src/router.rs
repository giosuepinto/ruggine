
// ============================================================================
// router.rs - Definizione delle rotte HTTP e WebSocket dell'applicazione
// Crea il router Axum e collega i controller alle rotte
// ============================================================================

// Import di terze parti
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

// Import interni
use crate::{
    controllers::{authentication_controller, web_socket_controller},
    state::AppState,
};


// Crea e restituisce il router Axum con tutte le rotte dell'applicazione
pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        // Rotte di autenticazione
        .route("/register", post(authentication_controller::register_handler))
        .route("/login", post(authentication_controller::login_handler))
        // Rotta WebSocket
        .route("/ws", get(web_socket_controller::ws_handler))
}