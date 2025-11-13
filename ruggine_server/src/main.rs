// --- Dichiarazione Moduli ---
// Questi dicono a Rust di cercare i file/cartelle
mod error;
mod services;
mod state;
mod DTOs; // Ora punta alla cartella 'models'

// === MODULI AGGIUNTI ===
mod repositories;
mod controllers;
mod router;
// === FINE AGGIUNTE ===

// --- Importazioni Principali ---
use crate::{
    // La nostra funzione che crea il router, ora dal modulo 'router'
    router::create_router,
    state::AppState,       // Il nostro stato
};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup Logging (Invariato) ---
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruggine_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Setup Database (Invariato) ---
    let db_options = SqliteConnectOptions::from_str("sqlite:ruggine.db")?
        .create_if_missing(true);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    tracing::debug!("Database connesso e migrazioni applicate.");

    // --- Setup Stato Applicazione (Invariato) ---
    // (Presuppone che state.rs sia stato aggiornato come da nostra discussione)
    let connections = Arc::new(RwLock::new(HashMap::new()));

    let app_state = Arc::new(AppState {
        db: db_pool,
        connections,
    });

    // --- Setup CORS (Invariato) ---
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // --- Creazione Router (Invariato nella logica, usa il nuovo import) ---
    let app = create_router() // Chiama la funzione dal modulo 'router'
        .with_state(app_state)
        .layer(cors);

    // --- Avvio Server (Invariato) ---
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!(">> Server in ascolto su {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}