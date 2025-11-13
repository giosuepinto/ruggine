// src/controllers/ws_controller.rs

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
use tracing;
use uuid::Uuid;

use crate::{
    error::AppError,
    // Importa i DTO per il WebSocket
    DTOs::web_socket::{ClientMessage, ServerMessage},
    // Importa i servizi necessari
    services::{chat_service, user_service},
    state::AppState,
};

/// Gestore principale per l'upgrade della connessione a WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Gestisce il ciclo di vita di una singola connessione WebSocket.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4();

    // Canale per inviare messaggi *a* questo client
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    // --- Task per INVIARE messaggi (Invariato) ---
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Errore serializzazione messaggio: {}", e);
                    continue;
                }
            };
            if sender.send(Message::Text(json_msg.into())).await.is_err() {
                break;
            }
        }
    });

    // --- Task per RICEVERE messaggi (Ora è un "Centralino") ---
    // 'user_info' ora contiene i dati della sessione
    let mut user_info: Option<user_service::WsUser> = None;
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

            // Controlla se l'utente è autenticato
            if user_info.is_none() && !matches!(msg, ClientMessage::Auth { .. }) {
                let _ = tx.send(ServerMessage::Error {
                    content: "Devi prima autenticarti.".to_string()
                }).await;
                continue;
            }

            // Clona lo stato per usarlo nel task
            let state_c = Arc::clone(&state_clone);

            // Gestisce gli errori in modo centralizzato
            let handle_result = |result: Result<(), AppError>| async {
                if let Err(e) = result {
                    tracing::warn!("Errore nell'eseguire l'azione: {:?}", e);
                    let _ = tx.send(ServerMessage::Error { content: e.to_string() }).await;
                }
            };

            // === IL NUOVO CENTRALINO (MATCH) ===
            match msg {

                // --- Caso 1: Autenticazione ---
                ClientMessage::Auth { token } => {
                    match user_service::authenticate_ws_user(&state_c.db, &token).await {
                        Ok(user) => {
                            tracing::info!("[{}] Utente {} (ID: {}) autenticato.", socket_id, user.username, user.id);

                            // Aggiungi questo socket alla mappa
                            state_c.connections.write().await
                                .entry(user.id)
                                .or_default()
                                .insert(socket_id, tx.clone());

                            // *** CHIAMA IL CHAT_SERVICE ***
                            // Carica TUTTI i dati (gruppi, membri, cronologia)
                            match chat_service::get_user_data_for_client(&state_c.db, user.id).await {
                                Ok(groups_data) => {
                                    // Invia la lista completa al client
                                    let _ = tx.send(ServerMessage::UpdateGroupList { groups: groups_data }).await;
                                    // Salva la sessione utente
                                    user_info = Some(user);
                                },
                                Err(e) => handle_result(Err(e)).await,
                            }
                        },
                        Err(e) => {
                            handle_result(Err(e)).await;
                            break; // Autenticazione fallita, chiudi loop
                        }
                    }
                }

                // --- Caso 2: Invia Messaggio ---
                ClientMessage::SendGroupMessage { group_id, content } => {
                    if let Some(user) = &user_info {
                        let result = chat_service::handle_send_message(
                            state_c, user.id, &user.username, group_id, &content
                        ).await;
                        handle_result(result).await;
                    }
                }

                // --- Caso 3: Crea Gruppo ---
                ClientMessage::CreateGroup { name } => {
                    if let Some(user) = &user_info {
                        match chat_service::handle_create_group(&state_c.db, user.id, &name).await {
                            Ok(new_groups_list) => {
                                // Invia l'aggiornamento SOLO a questo socket
                                let _ = tx.send(ServerMessage::UpdateGroupList { groups: new_groups_list }).await;
                            },
                            Err(e) => handle_result(Err(e)).await,
                        }
                    }
                }

                // --- Caso 4: Crea DM ---
                ClientMessage::CreateDM { username } => {
                    if let Some(user) = &user_info {
                        match chat_service::handle_create_dm(state_c, user.id, &username).await {
                            Ok(new_groups_list) => {
                                // Invia l'aggiornamento (che notifica anche l'altro utente)
                                let _ = tx.send(ServerMessage::UpdateGroupList { groups: new_groups_list }).await;
                            },
                            Err(e) => handle_result(Err(e)).await,
                        }
                    }
                }

                // --- Caso 5: Invita a Gruppo ---
                ClientMessage::InviteToGroup { group_id, username } => {
                    if let Some(user) = &user_info {
                        // Questo servizio notificherà tutti i membri
                        let result = chat_service::handle_invite_to_group(
                            state_c, &user.username, group_id, &username
                        ).await;
                        handle_result(result).await;
                    }
                }
            }
        }

        // --- Cleanup (Modificato) ---
        if let Some(user) = user_info {
            tracing::info!("[{}] Client {} disconnesso.", socket_id, user.username);
            let mut connections = state_clone.connections.write().await;
            if let Some(user_sockets) = connections.get_mut(&user.id) {
                user_sockets.remove(&socket_id);
                if user_sockets.is_empty() {
                    connections.remove(&user.id);
                }
            }
        } else {
            tracing::debug!("[{}] Client anonimo disconnesso.", socket_id);
        }
    });

    // Esegui i task (Invariato)
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}