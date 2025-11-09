// src/routes/ws.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid; // Importa Uuid

use crate::{
    models::{ClientMessage, ServerMessage},
    state::AppState,
};

#[derive(Debug)]
struct User {
    id: i64,
    username: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Assegna un ID unico a QUESTA connessione
    let socket_id = Uuid::new_v4();

    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    // --- Task per INVIARE messaggi a questo client ---
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Errore serializzazione messaggio: {}", e);
                    continue;
                }
            };
            // Usa .into() per convertire String in Utf8Bytes
            if sender.send(Message::Text(json_msg.into())).await.is_err() {
                tracing::debug!("[{}] Errore invio al client", socket_id);
                break;
            }
        }
    });

    // --- Task per RICEVERE messaggi da questo client ---
    let mut user_info: Option<User> = None;
    let state_clone = Arc::clone(&state);

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg_string))) = receiver.next().await {
            let msg: ClientMessage = match serde_json::from_str(&msg_string) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::debug!("[{}] Errore deserializzazione: {}", socket_id, e);
                    continue;
                }
            };

            match msg {
                // --- Caso 1: Autenticazione ---
                ClientMessage::Auth { token } => {
                    tracing::debug!("[{}] Richiesta Auth con token: {}", socket_id, token);

                    let user_result = sqlx::query_as::<_, (i64, String)>(
                        "SELECT id, username FROM users WHERE username = ?",
                    )
                        .bind(&token)
                        .fetch_optional(&state_clone.db)
                        .await;

                    if let Ok(Some((id, username))) = user_result {
                        tracing::info!("[{}] Utente {} (ID: {}) autenticato.", socket_id, username, id);

                        user_info = Some(User { id, username: username.clone() });

                        // 2. Aggiungi questo socket_id alla mappa dell'utente
                        let mut connections = state_clone.connections.write().await;
                        connections
                            .entry(id) // Trova l'utente
                            .or_default() // O crea una nuova mappa per lui
                            .insert(socket_id, tx.clone()); // Inserisci questo socket

                        let _ = tx.send(ServerMessage::Info {
                            content: format!("Autenticato come {}", username)
                        }).await;

                    } else {
                        tracing::warn!("[{}] Autenticazione fallita per token: {}", socket_id, token);
                        let _ = tx.send(ServerMessage::Info {
                            content: "Token non valido.".to_string()
                        }).await;
                        break;
                    }
                }

                // --- Caso 2: Messaggio Chat ---
                ClientMessage::Chat { content } => {
                    if let Some(user) = &user_info {
                        tracing::debug!("[{}] Messaggio da {}: {}", socket_id, user.username, content);

                        let server_msg = ServerMessage::Chat {
                            from: user.username.clone(),
                            content,
                        };

                        // 3. Inoltra a TUTTE le connessioni di TUTTI gli utenti
                        let connections = state_clone.connections.read().await;

                        for (user_id, user_sockets) in connections.iter() {
                            for (sock_id, user_tx) in user_sockets.iter() {
                                if user_tx.send(server_msg.clone()).await.is_err() {
                                    tracing::debug!("Invio fallito a user {} (socket {})", user_id, sock_id);
                                    // (Questo socket verrà pulito al suo disconnect)
                                }
                            }
                        }

                    } else {
                        tracing::warn!("[{}] Messaggio da client non autenticato.", socket_id);
                    }
                }
            }
        }

        // --- Cleanup quando questo client si disconnette ---
        if let Some(user) = user_info {
            tracing::info!("[{}] Client {} disconnesso.", socket_id, user.username);

            // 4. Rimuovi SOLO questo socket_id dalla mappa dell'utente
            let mut connections = state_clone.connections.write().await;
            if let Some(user_sockets) = connections.get_mut(&user.id) {
                user_sockets.remove(&socket_id);
                // Opzionale: se l'utente non ha più socket, rimuovi l'utente
                if user_sockets.is_empty() {
                    connections.remove(&user.id);
                }
            }
        } else {
            tracing::debug!("[{}] Client anonimo disconnesso.", socket_id);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => recv_task.abort(),
    };
}