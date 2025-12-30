
// ============================================================================
// main.rs - Entry point del server Rust "Ruggine"
// Avvia il server, gestisce setup logging, database, stato globale e task periodici.
// Questo file contiene la funzione main e la configurazione iniziale del backend.
// ============================================================================

// Moduli del progetto
mod error;
mod services;
mod state;
mod dtos;
mod repositories;
mod controllers;
mod router;
mod cpu_logger;

// Import interni
use crate::{
    router::create_router,
    state::AppState,
};

// Import di terze parti
// SQLX e gestione DB
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
// Standard library
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
// Tokio e async
use tokio::sync::RwLock;
// HTTP, CORS e tracing
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


// ===================== FUNZIONE PRINCIPALE =====================
/// Funzione principale: avvia il server, setup logging, DB, stato e task CPU.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ================== Setup logging ==================
    // Inizializza il sistema di logging e tracing per debug e monitoraggio
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruggine_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ================== Setup database ==================
    // Configura e connette il database SQLite, applica le migrazioni
    let db_options = SqliteConnectOptions::from_str("sqlite:ruggine.db")?
        .create_if_missing(true);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::debug!("Database connesso e migrazioni applicate.");

    // ================== Setup stato condiviso ==================
    // Crea lo stato condiviso dell'applicazione (DB pool e connessioni WebSocket)
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let app_state = Arc::new(AppState {
        db: db_pool,
        connections,
    });

    // ================== Task logging CPU  ==================
    // Avvia il task di logging CPU (ogni 2 minuti)
    cpu_logger::spawn_cpu_logger(120);

    // ================== Setup CORS e server HTTP ==================
    // Configura CORS e avvia il server HTTP Axum sulla porta 3000
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Avvia il server HTTP Axum sulla porta 3000
    // Crea l'applicazione Axum con stato e CORS
    let app = create_router()
        .with_state(app_state)
        .layer(cors);

    // Definisce l'indirizzo di ascolto e avvia il listener TCP
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Logga l'avvio del server e serve le richieste
    tracing::debug!(">> Server in ascolto su {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}