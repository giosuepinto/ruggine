// src/services/chat_service.rs

use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use tracing;

use crate::{
    error::AppError,
    DTOs::web_socket::{GroupInfo, GroupMember, ServerMessage},
    repositories::{group_repository, message_repository, user_repository, invitation_repository},
    state::{AppState, ConnectionMap},
};

// =================================================================
// FUNZIONI HELPER (Broadcast)
// =================================================================

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

/// Helper per inviare la lista aggiornata degli inviti a un utente
pub async fn refresh_invitations_for_user(
    state: Arc<AppState>,
    user_id: i64,
) {
    let received = invitation_repository::find_received_invites(&state.db, user_id).await.unwrap_or_default();
    let sent = invitation_repository::find_sent_invites(&state.db, user_id).await.unwrap_or_default();

    broadcast_message_to_user(
        &state.connections,
        user_id,
        ServerMessage::UpdateInvitationsList { received, sent }
    ).await;
}

// =================================================================
// FUNZIONI DI LETTURA (Pubbliche)
// =================================================================

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

        // --- FILTRO DM PENDENTI ---
        // Se è un DM e ci sono meno di 2 membri, significa che l'altro non ha ancora accettato.
        // Lo nascondiamo dalla lista principale delle chat.
        if group_details.is_dm && members.len() < 2 {
            continue;
        }

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
// FUNZIONI DI AZIONE (Logica di Business)
// =================================================================

pub async fn handle_send_message(
    state: Arc<AppState>,
    sender_user_id: i64,
    sender_username: &str,
    group_id: i64,
    content: &str,
) -> Result<(), AppError> {

    let members = group_repository::find_member_ids_by_group(&state.db, group_id).await?;
    if !members.contains(&sender_user_id) {
        return Err(AppError::ValidationError("Non sei membro di questo gruppo.".to_string()));
    }

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

    broadcast_to_group_members(
        &state.db,
        &state.connections,
        group_id,
        server_msg
    ).await;

    Ok(())
}

pub async fn handle_create_group(
    db: &Pool<Sqlite>,
    creator_user_id: i64,
    group_name: &str,
) -> Result<Vec<GroupInfo>, AppError> {

    let mut tx = db.begin().await?;
    let group_id = group_repository::create_group(&mut tx, Some(group_name), false).await?;
    group_repository::add_member(&mut tx, group_id, creator_user_id).await?;
    tx.commit().await?;

    get_user_data_for_client(db, creator_user_id).await
}

/// Gestisce la richiesta di creazione DM (invia invito)
pub async fn handle_create_dm(
    state: Arc<AppState>,
    creator_user_id: i64,
    target_username: &str,
) -> Result<(), AppError> {

    // 1. Trova Utente Target
    let target_user = user_repository::find_by_username(&state.db, target_username)
        .await?
        .ok_or_else(|| {
            AppError::ValidationError(format!("Utente '{}' non trovato.", target_username))
        })?;

    if target_user.id == creator_user_id {
        return Err(AppError::ValidationError("Non puoi creare un DM con te stesso.".to_string()));
    }

    // 2. Controlli (Pending & Cooldown)
    if let Some(_) = invitation_repository::find_pending_dm_invitation(&state.db, creator_user_id, target_user.id).await? {
        return Err(AppError::ValidationError("Esiste già una richiesta di DM in attesa.".to_string()));
    }

    if invitation_repository::is_in_cooldown_dm(&state.db, creator_user_id, target_user.id).await? {
        return Err(AppError::ValidationError("Richiesta rifiutata di recente. Attendi 2 minuti.".to_string()));
    }

    // 3. Crea Gruppo DM + Membro (Solo Creatore) + Invito
    let mut tx = state.db.begin().await?;

    let group_id = group_repository::create_group(&mut tx, None, true).await?;
    group_repository::add_member(&mut tx, group_id, creator_user_id).await?;
    invitation_repository::create_invitation_tx(&mut tx, creator_user_id, target_user.id, group_id).await?;

    tx.commit().await?;

    // 4. Notifiche
    // Aggiorna la lista inviti del target (ricevuti)
    refresh_invitations_for_user(Arc::clone(&state), target_user.id).await;
    // Aggiorna la lista inviti del creatore (inviati)
    refresh_invitations_for_user(state, creator_user_id).await;

    tracing::info!("Richiesta DM inviata: {} -> {}", creator_user_id, target_username);
    Ok(())
}

pub async fn handle_invite_to_group(
    state: Arc<AppState>,
    inviter_username: &str,
    group_id: i64,
    target_username: &str,
) -> Result<(), AppError> {

    let target_user = user_repository::find_by_username(&state.db, target_username)
        .await?
        .ok_or_else(|| AppError::ValidationError(format!("Utente '{}' non trovato.", target_username)))?;

    let inviter_user = user_repository::find_by_username(&state.db, inviter_username)
        .await?
        .ok_or_else(|| AppError::ValidationError("Invitante non trovato.".to_string()))?;

    let group_info = group_repository::find_group_by_id(&state.db, group_id)
        .await?
        .ok_or_else(|| AppError::ValidationError("Gruppo non trovato.".to_string()))?;

    if group_info.is_dm {
        return Err(AppError::ValidationError("Non puoi invitare persone in un DM.".to_string()));
    }

    let current_members = group_repository::find_member_ids_by_group(&state.db, group_id).await?;
    if current_members.contains(&target_user.id) {
        return Err(AppError::ValidationError("L'utente è già membro del gruppo.".to_string()));
    }

    let status_check = invitation_repository::check_group_invitation_status(
        &state.db,
        inviter_user.id,
        target_user.id,
        group_id
    ).await?;

    if status_check == "pending" {
        return Err(AppError::ValidationError("Esiste già un invito in attesa per questo utente.".to_string()));
    }
    if status_check == "cooldown" {
        return Err(AppError::ValidationError("Invito rifiutato di recente. Attendi 2 minuti.".to_string()));
    }

    invitation_repository::create_invitation(&state.db, inviter_user.id, target_user.id, group_id).await?;

    refresh_invitations_for_user(Arc::clone(&state), target_user.id).await;
    // Aggiorna anche chi ha inviato (per vedere la richiesta pending nella UI)
    refresh_invitations_for_user(state, inviter_user.id).await;

    tracing::info!("Invito creato: {} -> {} per gruppo {}", inviter_username, target_username, group_id);
    Ok(())
}

pub async fn handle_invitation_response(
    state: Arc<AppState>,
    user_id: i64,
    invitation_id: i64,
    accept: bool,
) -> Result<(), AppError> {

    let invitation = invitation_repository::find_by_id(&state.db, invitation_id)
        .await?
        .ok_or_else(|| AppError::ValidationError("Invito non trovato.".to_string()))?;

    if invitation.to_user_id != user_id {
        return Err(AppError::ValidationError("Non puoi gestire un invito non tuo.".to_string()));
    }

    if invitation.status != "pending" {
        return Err(AppError::ValidationError("Questo invito non è più valido.".to_string()));
    }

    if accept {
        invitation_repository::update_status(&state.db, invitation_id, "accepted").await?;
        group_repository::add_member_or_ignore(&state.db, invitation.group_id, user_id).await?;

        let updated_groups = get_user_data_for_client(&state.db, user_id).await?;
        broadcast_message_to_user(
            &state.connections, user_id,
            ServerMessage::UpdateGroupList { groups: updated_groups }
        ).await;

        let member_ids = group_repository::find_member_ids_by_group(&state.db, invitation.group_id).await?;
        for member_id in member_ids {
            if member_id == user_id { continue; }
            let groups = get_user_data_for_client(&state.db, member_id).await?;
            broadcast_message_to_user(
                &state.connections, member_id,
                ServerMessage::UpdateGroupList { groups }
            ).await;
        }
    } else {
        invitation_repository::update_status(&state.db, invitation_id, "rejected").await?;
    }

    refresh_invitations_for_user(Arc::clone(&state), user_id).await;
    // Aggiorna anche chi aveva mandato l'invito, così vede che lo stato è cambiato
    refresh_invitations_for_user(state, invitation.from_user_id).await;

    Ok(())
}