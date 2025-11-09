use sqlx::SqlitePool;
use tokio::sync::broadcast;

// Rendi 'pub' la struct E I SUOI CAMPI
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub db: SqlitePool,
}