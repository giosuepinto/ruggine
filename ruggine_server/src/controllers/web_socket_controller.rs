// src/controllers/web_socket_controller.rs

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
    dtos::web_socket::{ClientMessage, ServerMessage},
    services::{chat_service, user_service},
    state::AppState,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4();

    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

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

            if user_info.is_none() && !matches!(msg, ClientMessage::Auth { .. }) {
                let _ = tx.send(ServerMessage::Error {
                    content: "Devi prima autenticarti.".to_string()
                }).await;
                continue;
            }

            let state_c = Arc::clone(&state_clone);

            let handle_result = |result: Result<(), AppError>| async {
                if let Err(e) = result {
                    tracing::warn!("Errore nell'eseguire l'azione: {:?}", e);
                    let _ = tx.send(ServerMessage::Error { content: e.to_string() }).await;
                }
            };

            match msg {
                ClientMessage::Auth { token } => {
                    match user_service::authenticate_ws_user(&state_c.db, &token).await {
                        Ok(user) => {
                            tracing::info!("[{}] Utente {} (ID: {}) autenticato.", socket_id, user.username, user.id);

                            state_c.connections.write().await
                                .entry(user.id)
                                .or_default()
                                .insert(socket_id, tx.clone());

                            match chat_service::get_user_data_for_client(&state_c.db, user.id).await {
                                Ok(groups_data) => {
                                    let _ = tx.send(ServerMessage::UpdateGroupList { groups: groups_data }).await;
                                },
                                Err(e) => handle_result(Err(e)).await,
                            }

                            chat_service::refresh_invitations_for_user(Arc::clone(&state_c), user.id).await;

                            user_info = Some(user);
                        },
                        Err(e) => {
                            handle_result(Err(e)).await;
                            break;
                        }
                    }
                }

                ClientMessage::SendGroupMessage { group_id, content } => {
                    if let Some(user) = &user_info {
                        let result = chat_service::handle_send_message(
                            Arc::clone(&state_c),
                            user.id,
                            &user.username,
                            group_id,
                            &content
                        ).await;
                        handle_result(result).await;
                    }
                }

                ClientMessage::CreateGroup { name } => {
                    if let Some(user) = &user_info {
                        match chat_service::handle_create_group(&state_c.db, user.id, &name).await {
                            Ok(groups) => {
                                let _ = tx.send(ServerMessage::UpdateGroupList { groups }).await;
                            },
                            Err(e) => handle_result(Err(e)).await,
                        }
                    }
                }

                ClientMessage::CreateDM { username } => {
                    if let Some(user) = &user_info {
                        // MODIFICATO: Ora gestisce solo l'invio, non la lista gruppi
                        let result = chat_service::handle_create_dm(Arc::clone(&state_c), user.id, &username).await;
                        handle_result(result).await;
                        // Tracing log per debug
                        tracing::info!("CreateDM request processed for target: {}", username);
                    }
                }

                ClientMessage::InviteToGroup { group_id, username } => {
                    if let Some(user) = &user_info {
                        let result = chat_service::handle_invite_to_group(
                            Arc::clone(&state_c),
                            &user.username,
                            group_id,
                            &username
                        ).await;
                        handle_result(result).await;
                    }
                }

                ClientMessage::HandleInvitation { invitation_id, accept } => {
                    if let Some(user) = &user_info {
                        let result = chat_service::handle_invitation_response(
                            Arc::clone(&state_c),
                            user.id,
                            invitation_id,
                            accept
                        ).await;
                        handle_result(result).await;
                    }
                }
            }
        }

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

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}