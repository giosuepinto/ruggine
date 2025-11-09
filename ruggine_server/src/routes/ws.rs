// src/routes/ws.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use sqlx::{Pool, Sqlite}; // Aggiunto
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

// Importa i nostri modelli!
use crate::{
    models::{
        ClientMessage, GroupInfo, GroupMember, ServerMessage,
    },
    state::{AppState, ConnectionMap}, // Importa ConnectionMap
};

// Struct per tenere traccia dei dati di questo utente
#[derive(Debug, Clone)] // Aggiunto Clone
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

// =================================================================
// FUNZIONE PRINCIPALE (PESANTEMENTE MODIFICATA)
// =================================================================
pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    // --- Task per INVIARE messaggi a questo client (INVARIATO) ---
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
                tracing::debug!("[{}] Errore invio al client", socket_id);
                break;
            }
        }
    });

    // --- Task per RICEVERE messaggi da questo client (MODIFICATO) ---
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

            // Controlla se l'utente è autenticato prima di processare
            // messaggi che non siano 'Auth'
            if user_info.is_none() && !matches!(msg, ClientMessage::Auth { .. }) {
                tracing::warn!("[{}] Client non autenticato ha provato ad inviare {:?}", socket_id, msg);
                let _ = tx.send(ServerMessage::Error { content: "Devi prima autenticarti.".to_string() }).await;
                continue;
            }

            // Clona i dati necessari per questo loop
            let db = &state_clone.db;
            let connections = &state_clone.connections;

            // Il 'match' ora gestisce tutti i nuovi tipi di messaggi
            match msg {
                // --- Caso 1: Autenticazione (MODIFICATO) ---
                ClientMessage::Auth { token } => {
                    let user_result = sqlx::query_as::<_, (i64, String)>(
                        "SELECT id, username FROM users WHERE username = ?",
                    )
                        .bind(&token)
                        .fetch_optional(db)
                        .await;

                    if let Ok(Some((id, username))) = user_result {
                        let current_user = User { id, username: username.clone() };
                        tracing::info!("[{}] Utente {} (ID: {}) autenticato.", socket_id, username, id);
                        user_info = Some(current_user.clone());

                        // Aggiungi questo socket alla mappa
                        connections.write().await.entry(id).or_default().insert(socket_id, tx.clone());

                        // *** NUOVA LOGICA: INVIA LA LISTA GRUPPI ***
                        let groups = get_user_groups_info(id, db).await;
                        let _ = tx.send(ServerMessage::UpdateGroupList { groups }).await;

                    } else {
                        tracing::warn!("[{}] Autenticazione fallita per token: {}", socket_id, token);
                        let _ = tx.send(ServerMessage::Error { content: "Token non valido.".to_string() }).await;
                        break;
                    }
                }

                // --- Caso 2: Invia Messaggio (EX 'Chat') ---
                ClientMessage::SendGroupMessage { group_id, content } => {
                    if let Some(user) = &user_info {
                        tracing::debug!("[{}] Messaggio da {} al gruppo {}: {}", socket_id, user.username, group_id, content);

                        // TODO: Verificare se l'utente è membro del gruppo

                        // 1. Salva il messaggio nel DB
                        let save_result = sqlx::query(
                            "INSERT INTO messages (group_id, sender_user_id, content) VALUES (?, ?, ?)"
                        )
                            .bind(group_id)
                            .bind(user.id)
                            .bind(&content)
                            .execute(db)
                            .await;

                        if save_result.is_err() {
                            tracing::error!("Errore salvataggio messaggio nel DB");
                            continue;
                        }

                        // 2. Costruisci il messaggio da inoltrare
                        let server_msg = ServerMessage::GroupChat {
                            group_id,
                            from_username: user.username.clone(),
                            content,
                        };

                        // 3. Inoltra solo ai membri del gruppo
                        broadcast_to_group_members(group_id, server_msg, db, connections).await;
                    }
                }

                // --- Caso 3: Crea Gruppo ---
                ClientMessage::CreateGroup { name } => {
                    if let Some(user) = &user_info {
                        tracing::debug!("[{}] {} sta creando il gruppo: {}", socket_id, user.username, name);

                        // Usa una transazione per assicurare che entrambe le query
                        // (crea gruppo, aggiungi membro) funzionino
                        let mut tx = db.begin().await.unwrap(); // Inizia transazione

                        // 1. Crea il gruppo
                        let group_id = sqlx::query(
                            "INSERT INTO groups (name, is_dm) VALUES (?, FALSE)"
                        )
                            .bind(&name)
                            .execute(&mut *tx)
                            .await.unwrap().last_insert_rowid();

                        // 2. Aggiungi il creatore come membro
                        sqlx::query(
                            "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)"
                        )
                            .bind(group_id)
                            .bind(user.id)
                            .execute(&mut *tx)
                            .await.unwrap();

                        tx.commit().await.unwrap(); // Finalizza transazione

                        // 3. Invia la lista gruppi aggiornata a questo utente
                        let groups = get_user_groups_info(user.id, db).await;
                        let _ = connections
                            .read().await
                            .get(&user.id).unwrap()
                            .get(&socket_id).unwrap() // Solo a questo socket
                            .send(ServerMessage::UpdateGroupList { groups }).await;
                    }
                }

                // --- Caso 4: Crea DM (Messaggio Diretto) ---
                ClientMessage::CreateDM { username } => {
                    if let Some(user) = &user_info {
                        // Trova l'utente target
                        let target_user = sqlx::query_as::<_, (i64, String)>(
                            "SELECT id, username FROM users WHERE username = ?"
                        )
                            .bind(&username)
                            .fetch_optional(db)
                            .await.unwrap();

                        if target_user.is_none() {
                            let _ = tx.send(ServerMessage::Error { content: format!("Utente '{}' non trovato.", username) }).await;
                            continue;
                        }
                        let target_user = target_user.unwrap();

                        // TODO: Controllare se un DM esiste già

                        // 1. Crea il "gruppo" DM
                        let mut db_tx = db.begin().await.unwrap();
                        let group_id = sqlx::query(
                            "INSERT INTO groups (is_dm) VALUES (TRUE)"
                        )
                            .execute(&mut *db_tx)
                            .await.unwrap().last_insert_rowid();

                        // 2. Aggiungi entrambi i membri
                        sqlx::query(
                            "INSERT INTO group_members (group_id, user_id) VALUES (?, ?), (?, ?)"
                        )
                            .bind(group_id).bind(user.id) // Membro 1 (creatore)
                            .bind(group_id).bind(target_user.0) // Membro 2 (target)
                            .execute(&mut *db_tx)
                            .await.unwrap();

                        db_tx.commit().await.unwrap();

                        // 3. Invia l'aggiornamento lista a ENTRAMBI gli utenti (se online)
                        let groups_user = get_user_groups_info(user.id, db).await;
                        let groups_target = get_user_groups_info(target_user.0, db).await;

                        // Invia al creatore (tutti i suoi socket)
                        if let Some(sockets) = connections.read().await.get(&user.id) {
                            for socket_tx in sockets.values() {
                                let _ = socket_tx.send(ServerMessage::UpdateGroupList { groups: groups_user.clone() }).await;
                            }
                        }
                        // Invia al target (tutti i suoi socket, se online)
                        if let Some(sockets) = connections.read().await.get(&target_user.0) {
                            for socket_tx in sockets.values() {
                                let _ = socket_tx.send(ServerMessage::UpdateGroupList { groups: groups_target.clone() }).await;
                            }
                        }
                    }
                }

                // --- CASO 5: Invita Utente a Gruppo ---
                ClientMessage::InviteToGroup { group_id, username } => {
                    if let Some(inviter) = &user_info {
                        tracing::debug!("[{}] {} sta invitando {} al gruppo {}", socket_id, inviter.username, username, group_id);

                        // --- Iniziamo una serie di validazioni ---

                        // 1. Cerca l'utente da invitare
                        let target_user = sqlx::query_as::<_, (i64, String)>(
                            "SELECT id, username FROM users WHERE username = ?"
                        )
                            .bind(&username)
                            .fetch_optional(db)
                            .await.unwrap();

                        if target_user.is_none() {
                            let _ = tx.send(ServerMessage::Error { content: format!("Utente '{}' non trovato.", username) }).await;
                            continue;
                        }
                        let (target_user_id, target_username) = target_user.unwrap();

                        // 2. Controlla che il gruppo esista e non sia un DM
                        let group_info = sqlx::query_as::<_, (bool,)>(
                            "SELECT is_dm FROM groups WHERE id = ?"
                        )
                            .bind(group_id)
                            .fetch_optional(db)
                            .await.unwrap();

                        if group_info.is_none() {
                            let _ = tx.send(ServerMessage::Error { content: "Gruppo non trovato.".to_string() }).await;
                            continue;
                        }
                        if group_info.unwrap().0 == true { // .0 accede al primo campo della tupla (is_dm)
                            let _ = tx.send(ServerMessage::Error { content: "Non puoi invitare persone in un DM.".to_string() }).await;
                            continue;
                        }

                        // 3. (Opzionale) Controlla se l'invitante è membro... lo saltiamo per ora

                        // 4. Aggiungi l'utente al gruppo
                        // usiamo "INSERT OR IGNORE" per ignorare l'errore se l'utente è già membro
                        let insert_result = sqlx::query(
                            "INSERT OR IGNORE INTO group_members (group_id, user_id) VALUES (?, ?)"
                        )
                            .bind(group_id)
                            .bind(target_user_id)
                            .execute(db)
                            .await;

                        if insert_result.is_err() {
                            let _ = tx.send(ServerMessage::Error { content: "Errore nell'aggiungere l'utente al gruppo.".to_string() }).await;
                            continue;
                        }

                        // --- Successo! Ora notifica tutti ---

                        // 1. Trova tutti i membri attuali del gruppo
                        let member_ids = sqlx::query_as::<_, (i64,)>("SELECT user_id FROM group_members WHERE group_id = ?")
                            .bind(group_id)
                            .fetch_all(db)
                            .await.unwrap_or_default();

                        // 2. Invia l'aggiornamento a tutti i membri (vecchi E nuovi)
                        let conns = connections.read().await;
                        for (user_id,) in member_ids {
                            if let Some(user_sockets) = conns.get(&user_id) {
                                for socket_tx in user_sockets.values() {
                                    // Invia l'intera lista gruppi aggiornata
                                    // (è un po' inefficiente, ma robusto)
                                    let groups_list = get_user_groups_info(user_id, db).await;
                                    let _ = socket_tx.send(ServerMessage::UpdateGroupList { groups: groups_list }).await;
                                }
                            }
                        }

                        tracing::info!("Utente {} aggiunto al gruppo {} da {}", target_username, group_id, inviter.username);
                    }
                }
            }
        }

        // --- Cleanup (MODIFICATO) ---
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
        _ = (&mut recv_task) => recv_task.abort(),
    };
}

// =================================================================
// NUOVE FUNZIONI HELPER
// =================================================================

/// Recupera tutti i gruppi per un utente e formatta i dati per il client
async fn get_user_groups_info(user_id: i64, db: &Pool<Sqlite>) -> Vec<GroupInfo> {
    let mut groups_info = Vec::new();

    // 1. Trova tutti i group_id per questo utente
    let group_ids = sqlx::query_as::<_, (i64,)>("SELECT group_id FROM group_members WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    // 2. Per ogni gruppo, recupera i dettagli e i membri
    // (Questo è un loop N+1, ma va bene per SQLite e per questo progetto)
    for (group_id,) in group_ids {
        // Recupera i dettagli del gruppo
        let group_details = sqlx::query_as::<_, (Option<String>, bool)>(
            "SELECT name, is_dm FROM groups WHERE id = ?"
        )
            .bind(group_id)
            .fetch_one(db)
            .await;

        if group_details.is_err() { continue; }
        let (group_name, is_dm) = group_details.unwrap();

        // Recupera i membri del gruppo
        let members = sqlx::query_as::<_, (i64, String)>(
            "SELECT u.id, u.username FROM users u
             JOIN group_members gm ON u.id = gm.user_id
             WHERE gm.group_id = ?"
        )
            .bind(group_id)
            .fetch_all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(id, username)| GroupMember { id, username })
            .collect();

        groups_info.push(GroupInfo {
            group_id,
            group_name,
            is_dm,
            members,
        });
    }

    groups_info
}

/// Invia un messaggio a tutti i membri online di un gruppo
async fn broadcast_to_group_members(
    group_id: i64,
    message: ServerMessage,
    db: &Pool<Sqlite>,
    connections: &ConnectionMap,
) {
    // 1. Trova tutti gli ID utente in questo gruppo
    let member_ids = sqlx::query_as::<_, (i64,)>(
        "SELECT user_id FROM group_members WHERE group_id = ?"
    )
        .bind(group_id)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    // 2. Ottieni un lock in lettura sulla mappa delle connessioni
    let connections_map = connections.read().await;

    // 3. Itera solo sugli ID dei membri
    for (user_id,) in member_ids {
        // Controlla se questo membro è online (presente nella mappa)
        if let Some(user_sockets) = connections_map.get(&user_id) {
            // Invia a tutti i dispositivi di questo membro
            for (socket_id, tx) in user_sockets.iter() {
                if tx.send(message.clone()).await.is_err() {
                    tracing::debug!("Invio fallito a user {} (socket {})", user_id, socket_id);
                }
            }
        }
    }
}
