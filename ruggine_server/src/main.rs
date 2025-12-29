
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
use tokio::time::{interval, Duration};
// HTTP, CORS e tracing
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// Logging CPU e sistema
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use chrono::Local;

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

    // ================== Task logging CPU (requisito di progetto) ==================
    // Avvia un task asincrono che logga l'utilizzo della CPU ogni 2 minuti
    tokio::spawn(async move {
        // Configura sysinfo per leggere solo la CPU (ottimizzazione risorse)
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        // Timer periodico ogni 2 minuti (120 secondi)
        let mut ticker = interval(Duration::from_secs(120));

        loop {
            ticker.tick().await; // Attende 2 minuti

            // Aggiorna informazioni sulla CPU
            sys.refresh_cpu_all();

            // Calcola utilizzo globale della CPU
            let global_cpu_usage = sys.global_cpu_usage();

            // Crea la stringa di log con timestamp e utilizzo CPU
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] CPU Usage: {:.2}%\n", timestamp, global_cpu_usage);

            // Scrive nel file (modalità append)
            // Nota: std::fs è bloccante ma accettabile ogni 2 minuti.
            // In produzione si preferirebbe tokio::fs.
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