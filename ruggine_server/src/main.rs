// src/main.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions,SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::str::FromStr;

// Importiamo l'algoritmo
use argon2::Argon2;

// Importiamo i TRAIT e i TIPI dalla cassa 'password-hash' principale
use password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
// --- Nuovi Struct per Utenti e JSON ---
#[derive(Deserialize)]
struct AuthPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    message: String,
}

// --- Gestore di errori aggiornato ---
enum AppError {
    SqlxError(sqlx::Error),
    HashError(password_hash::Error), // Modificato
}

// --- Modifica a AppState ---
struct AppState {
    tx: broadcast::Sender<String>,
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { // Modifica: ora restituisce un Result
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruggine_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Connessione al Database (NUOVA VERSIONE) ---

    // 1. Definiamo le opzioni di connessione, inclusa la creazione del file
    let db_options = SqliteConnectOptions::from_str("sqlite:ruggine.db")?
        .create_if_missing(true);

    // 2. Creiamo il "pool" di connessioni usando quelle opzioni
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
        .await?; // Modifica: da .expect() a '?'

    tracing::debug!("Database connesso.");

    // --- ESECUZIONE AUTOMATICA MIGRAZIONI ---
    // Questa è la magia! Cerca la cartella 'migrations'
    // e applica tutte le migrazioni necessarie.
    // È sicuro da eseguire ogni volta.
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    tracing::debug!("Migrazioni del database verificate e applicate.");
    // --- Fine Sezione Migrazioni ---


    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState { tx, db: db_pool });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .with_state(app_state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await?; // Modifica: da .unwrap() a '?'

    tracing::debug!(">> Server in ascolto su {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await?; // Modifica: da .unwrap() a '?'

    Ok(()) // Modifica: Aggiunto Ok(()) alla fine
}

// --- Nuovo Handler: Registrazione ---
async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::debug!("Tentativo di registrazione per: {}", payload.username);

    // Hash della password (con la nuova funzione corretta)
    let password_hash = hash_password(&payload.password)?;

    // Salva nel DB
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&payload.username)
        .bind(&password_hash)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok(Json(AuthResponse {
            message: "Utente registrato con successo".to_string(),
        })),
        Err(e) => {
            tracing::warn!("Errore registrazione: {:?}", e);
            Err(AppError::SqlxError(e))
        }
    }
}

// --- Nuovo Handler: Login ---
async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::debug!("Tentativo di login per: {}", payload.username);

    // Cerca utente nel DB
    let user = sqlx::query_as::<_, (String,)>(
        "SELECT password_hash FROM users WHERE username = ?",
    )
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    if let Some((hash,)) = user {
        // Verifica password (con la nuova funzione corretta)
        if verify_password(&payload.password, &hash)? {
            tracing::debug!("Login riuscito per: {}", payload.username);
            Ok(Json(AuthResponse {
                message: format!("Login riuscito per {}", payload.username),
            }))
        } else {
            tracing::debug!("Password errata per: {}", payload.username);
            Ok(Json(AuthResponse {
                message: "Password errata".to_string(),
            }))
        }
    } else {
        Ok(Json(AuthResponse {
            message: "Utente non trovato".to_string(),
        }))
    }
}

// --- Funzioni Helper per Password (Corrette con API moderna) ---
fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng); // Genera un salt sicuro
    let argon2 = Argon2::default(); // Configurazione di default

    // Hashing della password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, password_hash::Error> {
    // Analizza l'hash dal database
    let parsed_hash = PasswordHash::new(hash)?;

    // Verifica che la password corrisponda all'hash
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}


// --- Nuovo Gestore Errori ---
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SqlxError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Errore DB: {}", e)),
            AppError::HashError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Errore Hashing: {}", e)),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}
// Necessario per usare '?'
impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::SqlxError(e) } }
// Aggiornato per il nuovo tipo di errore
impl From<password_hash::Error> for AppError { fn from(e: password_hash::Error) -> Self { AppError::HashError(e) } }


// --- La nostra funzione ws_handler (INVARIATA) ---
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// --- La nostra funzione handle_socket (Leggermente modificata) ---
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    tracing::debug!("Nuovo client connesso! (Ancora anonimo)");

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                tracing::debug!("Errore invio messaggio al client, connessione chiusa.");
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = receiver.next().await {

            // NOTA: il client ora invia "Username: Messaggio"
            // Il nostro server lo prende e lo inoltra a tutti.
            // Non c'è ancora una logica "intelligente" qui.
            let msg_string = msg.to_string();
            tracing::debug!("Messaggio ricevuto: {}", msg_string);

            // Inoltra il messaggio a tutti
            let _ = tx.send(msg_string);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    tracing::debug!("Client disconnesso.");
}