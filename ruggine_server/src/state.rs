// src/state.rs

use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid; // Importa Uuid

// Importa i nostri modelli
use crate::DTOs::web_socket::ServerMessage;

// NUOVO ALIAS:
// Mappa: user_id -> (Mappa: socket_id -> Canale di invio)
pub type ConnectionMap = Arc<RwLock<HashMap<i64, HashMap<Uuid, mpsc::Sender<ServerMessage>>>>>;

pub struct AppState {
    pub db: SqlitePool,
    pub connections: ConnectionMap,
}