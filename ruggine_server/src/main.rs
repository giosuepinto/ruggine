// --- Dichiarazione Moduli ---
// Questi dicono a Rust di cercare i file/cartelle
mod error;
mod routes;
mod services;
mod state;

// --- Importazioni Principali ---
use crate::{
    routes::create_router, // La nostra funzione che crea il router
    state::AppState, // Il nostro stato
};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup Logging ---
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruggine_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Setup Database ---
    let db_options = SqliteConnectOptions::from_str("sqlite:ruggine.db")?
        .create_if_missing(true);

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    tracing::debug!("Database connesso e migrazioni applicate.");

    // --- Setup Stato Applicazione ---
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx, db: db_pool });

    // --- Setup CORS ---
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permetti qualsiasi origine
        .allow_methods(Any) // Permetti qualsiasi metodo (GET, POST, etc.)
        .allow_headers(Any); // Permetti qualsiasi header

    // --- Creazione Router ---
    let app = create_router() // Chiama la funzione dal modulo 'routes'
        .with_state(app_state) // Applica lo stato
        .layer(cors); // Applica il layer CORS

    // --- Avvio Server ---
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!(">> Server in ascolto su {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}