// src/main.rs

mod error;
mod services;
mod state;
mod dtos;
mod repositories;
mod controllers;
mod router;

use crate::{
    router::create_router,
    state::AppState,
};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// --- NUOVI IMPORT PER CPU LOGGING ---
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::time::{interval, Duration};
use chrono::Local;

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

    // --- Setup Stato (Invariato) ---
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let app_state = Arc::new(AppState {
        db: db_pool,
        connections,
    });

    // =================================================================
    // NUOVO: Task di Logging CPU (Requisito di progetto)
    // =================================================================
    tokio::spawn(async move {
        // Configura sysinfo per leggere solo la CPU (per risparmiare risorse)
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        // Timer ogni 2 minuti (120 secondi)
        let mut ticker = interval(Duration::from_secs(120));

        loop {
            ticker.tick().await; // Aspetta 2 minuti

            // Aggiorna info CPU
            sys.refresh_cpu_all();

            // Calcola uso globale CPU
            let global_cpu_usage = sys.global_cpu_usage();

            // Crea il messaggio di log
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] CPU Usage: {:.2}%\n", timestamp, global_cpu_usage);

            // Scrivi nel file (modalità append)
            // Usa una chiamata bloccante std::fs, ma ogni 2 min è accettabile.
            // In produzione useremmo tokio::fs, ma qui va bene così.
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open("server_cpu_stats.log")
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(log_line.as_bytes()) {
                        tracing::error!("Impossibile scrivere log CPU: {}", e);
                    }
                },
                Err(e) => tracing::error!("Impossibile aprire file log CPU: {}", e),
            }
        }
    });
    // =================================================================

    // --- Setup CORS & Server (Invariato) ---
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router()
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!(">> Server in ascolto su {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}