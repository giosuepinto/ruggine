
// ============================================================================
// state.rs - Stato globale condiviso per il backend Ruggine
// Definisce la struttura AppState e le mappe di connessione WebSocket
// ============================================================================

// Import di terze parti
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

// Import interni
use crate::dtos::web_socket::ServerMessage;

// ============================================================================
// Alias e strutture per la gestione delle connessioni
// ============================================================================
pub type ConnectionMap = Arc<RwLock<HashMap<i64, HashMap<Uuid, mpsc::Sender<ServerMessage>>>>>;

pub struct AppState {
    pub db: SqlitePool,
    pub connections: ConnectionMap,
}