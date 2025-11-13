// src/services/chat_service.rs

use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use tracing;

use crate::{
    error::AppError,
    DTOs::web_socket::{GroupInfo, GroupMember, ServerMessage},
    repositories::{group_repository, message_repository, user_repository},
    state::{AppState, ConnectionMap},
};

// =================================================================
// FUNZIONI HELPER (Broadcast)
// *** SPOSTATE ALL'INIZIO DEL FILE PER RISOLVERE L'ERRORE [E0425] ***
// =================================================================

/// Invia un messaggio a tutti i membri online di un gruppo.
async fn broadcast_to_group_members(
    db: &SqlitePool,
    connections: &ConnectionMap,
    group_id: i64,
    message: ServerMessage,
) {
    let member_ids = match group_repository::find_member_ids_by_group(db, group_id).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Errore nel trovare membri del gruppo {}: {}", group_id, e);
            return;
        }
    };

    let connections_map = connections.read().await;

    for user_id in member_ids {
        if let Some(user_sockets) = connections_map.get(&user_id) {
            for (socket_id, tx) in user_sockets.iter() {
                if tx.send(message.clone()).await.is_err() {
                    tracing::debug!("Invio fallito a user {} (socket {})", user_id, socket_id);
                }
            }
        }
    }
}

/// Invia un messaggio a tutti i socket attivi di un singolo utente.
async fn broadcast_message_to_user(
    connections: &ConnectionMap,
    user_id: i64,
    message: ServerMessage,
) {
    let connections_map = connections.read().await;

    if let Some(user_sockets) = connections_map.get(&user_id) {
        for (socket_id, tx) in user_sockets.iter() {
            if tx.send(message.clone()).await.is_err() {
                tracing::debug!("Invio fallito a user {} (socket {})", user_id, socket_id);
            }
        }
    }
}

// =================================================================
// FUNZIONI DI LETTURA (Pubbliche)
// (Ora possono chiamare gli helper definiti sopra)
// =================================================================

/// Recupera la lista completa dei gruppi per un utente, inclusi i membri
/// e la cronologia dei messaggi di ciascun gruppo.
pub async fn get_user_data_for_client(
    db: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Vec<GroupInfo>, AppError> {

    let group_ids = group_repository::find_group_ids_by_user(db, user_id).await?;
    let mut groups_info = Vec::new();

    for group_id in group_ids {
        let group_details = group_repository::find_group_by_id(db, group_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("Dati corrotti: utente {} è membro di un gruppo inesistente {}", user_id, group_id);
                AppError::ValidationError("Gruppo non trovato".to_string())
            })?;

        let members = group_repository::find_members_by_group(db, group_id).await?;
        let history = message_repository::get_history_for_group(db, group_id).await?;

        let messages = history.into_iter().map(|msg| ServerMessage::GroupChat {
            group_id,
            from_username: msg.from_username,
            content: msg.content,
        }).collect();

        groups_info.push(GroupInfo {
            group_id,
            group_name: group_details.name,
            is_dm: group_details.is_dm,
            members,
            messages,
        });
    }

    Ok(groups_info)
}

// =================================================================
// FUNZIONI DI AZIONE (Logica di Business Pubblica)
// =================================================================

/// Gestisce la logica di business per l'invio di un messaggio.
pub async fn handle_send_message(
    state: Arc<AppState>,
    sender_user_id: i64,
    sender_username: &str,
    group_id: i64,
    content: &str,
) -> Result<(), AppError> {

    message_repository::create_message(
        &state.db,
        group_id,
        sender_user_id,
        content
    ).await?;

    let server_msg = ServerMessage::GroupChat {
        group_id,
        from_username: sender_username.to_string(),
        content: content.to_string(),
    };

    // Questa chiamata ora è valida
    broadcast_to_group_members(
        &state.db,
        &state.connections,
        group_id,
        server_msg
    ).await;

    Ok(())
}

/// Gestisce la creazione di un nuovo gruppo.
pub async fn handle_create_group(
    db: &Pool<Sqlite>,
    creator_user_id: i64,
    group_name: &str,
) -> Result<Vec<GroupInfo>, AppError> {

    let mut tx = db.begin().await?;

    let group_id = group_repository::create_group(
        &mut tx,
        Some(group_name),
        false
    ).await?;

    group_repository::add_member(&mut tx, group_id, creator_user_id).await?;

    tx.commit().await?;

    get_user_data_for_client(db, creator_user_id).await
}

/// Gestisce la creazione di un nuovo DM.
pub async fn handle_create_dm(
    state: Arc<AppState>,
    creator_user_id: i64,
    target_username: &str,
) -> Result<Vec<GroupInfo>, AppError> {

    let target_user = user_repository::find_by_username(&state.db, target_username)
        .await?
        .ok_or_else(|| {
            AppError::ValidationError(format!("Utente '{}' non trovato.", target_username))
        })?;

    if target_user.id == creator_user_id {
        return Err(AppError::ValidationError("Non puoi creare un DM con te stesso.".to_string()));
    }

    let mut db_tx = state.db.begin().await?;
    let group_id = group_repository::create_group(&mut db_tx, None, true).await?;

    group_repository::add_dm_members(
        &mut db_tx,
        group_id,
        creator_user_id,
        target_user.id
    ).await?;

    db_tx.commit().await?;

    let groups_target = get_user_data_for_client(&state.db, target_user.id).await?;

    // Questa chiamata ora è valida
    broadcast_message_to_user(
        &state.connections,
        target_user.id,
        ServerMessage::UpdateGroupList { groups: groups_target }
    ).await;

    get_user_data_for_client(&state.db, creator_user_id).await
}


/// Gestisce l'invito di un utente a un gruppo.
pub async fn handle_invite_to_group(
    state: Arc<AppState>,
    inviter_username: &str,
    group_id: i64,
    target_username: &str,
) -> Result<(), AppError> {

    let target_user = user_repository::find_by_username(&state.db, target_username)
        .await?
        .ok_or_else(|| {
            AppError::ValidationError(format!("Utente '{}' non trovato.", target_username))
        })?;

    let group_info = group_repository::find_group_by_id(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::ValidationError("Gruppo non trovato.".to_string()))?;

    if group_info.is_dm {
        return Err(AppError::ValidationError("Non puoi invitare persone in un DM.".to_string()));
    }

    group_repository::add_member_or_ignore(
        &state.db,
        group_id,
        target_user.id
    ).await?;

    let member_ids = group_repository::find_member_ids_by_group(
        &state.db,
        group_id
    ).await?;

    for user_id in member_ids {
        let groups_list = get_user_data_for_client(&state.db, user_id).await?;

        // Questa chiamata ora è valida
        broadcast_message_to_user(
            &state.connections,
            user_id,
            ServerMessage::UpdateGroupList { groups: groups_list }
        ).await;
    }

    tracing::info!("Utente {} aggiunto al gruppo {} da {}", target_username, group_id, inviter_username);
    Ok(())
}