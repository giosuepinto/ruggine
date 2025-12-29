
// ============================================================================
// invitation_repository.rs - Gestione degli inviti tra utenti e gruppi
// Funzioni CRUD per inviti, transazioni e query correlate
// ============================================================================


// Import di terze parti
use sqlx::{FromRow, Pool, Sqlite, Transaction};

// Import interni
use crate::error::AppError;
use crate::dtos::web_socket::InvitationInfo;

#[derive(Debug, FromRow)]
pub struct Invitation {
    #[allow(dead_code)]
    pub id: i64,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub group_id: i64,
    pub status: String,
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String
}


// ============================================================================
// Funzioni di scrittura (creazione/modifica inviti)
// ============================================================================

// Crea un nuovo invito (versione standard).
pub async fn create_invitation(
    db: &Pool<Sqlite>,
    from_user_id: i64,
    to_user_id: i64,
    group_id: i64,
) -> Result<i64, AppError> {
    let id = sqlx::query(
        "INSERT INTO invitations (from_user_id, to_user_id, group_id, status) VALUES (?, ?, ?, 'pending')"
    )
        .bind(from_user_id)
        .bind(to_user_id)
        .bind(group_id)
        .execute(db)
        .await?
        .last_insert_rowid();

    Ok(id)
}

// Versione transazionale di create_invitation. Fondamentale per la creazione atomica dei DM (Gruppo + Membro + Invito).
pub async fn create_invitation_tx(
    tx: &mut Transaction<'_, Sqlite>,
    from_user_id: i64,
    to_user_id: i64,
    group_id: i64,
) -> Result<i64, AppError> {
    let id = sqlx::query(
        "INSERT INTO invitations (from_user_id, to_user_id, group_id, status) VALUES (?, ?, ?, 'pending')"
    )
        .bind(from_user_id)
        .bind(to_user_id)
        .bind(group_id)
        .execute(&mut **tx)
        .await?
        .last_insert_rowid();

    Ok(id)
}

// Aggiorna lo stato di un invito (accetta/rifiuta).
pub async fn update_status(
    db: &Pool<Sqlite>,
    invitation_id: i64,
    new_status: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE invitations SET status = ? WHERE id = ?")
        .bind(new_status)
        .bind(invitation_id)
        .execute(db)
        .await?;
    Ok(())
}

// =================================================================
// LETTURA
// =================================================================

// Inviti RICEVUTI (partner = chi mi ha invitato)
pub async fn find_received_invites(
    db: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Vec<InvitationInfo>, AppError> {
    let invites = sqlx::query_as::<_, InvitationInfo>(
        r#"
        SELECT
            i.id,
            u.username as partner_username,
            COALESCE(g.name, 'DM Request') as group_name,
            i.group_id,
            i.status,
            i.created_at as sent_at
        FROM invitations i
        JOIN users u ON i.from_user_id = u.id
        JOIN groups g ON i.group_id = g.id
        WHERE i.to_user_id = ? AND i.status = 'pending'
        ORDER BY i.created_at DESC
        "#
    )
        .bind(user_id)
        .fetch_all(db)
        .await?;

    Ok(invites)
}

// Inviti INVIATI (partner = chi ho invitato)
pub async fn find_sent_invites(
    db: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Vec<InvitationInfo>, AppError> {
    let invites = sqlx::query_as::<_, InvitationInfo>(
        r#"
        SELECT
            i.id,
            u.username as partner_username,
            COALESCE(g.name, 'DM Request') as group_name,
            i.group_id,
            i.status,
            i.created_at as sent_at
        FROM invitations i
        JOIN users u ON i.to_user_id = u.id
        JOIN groups g ON i.group_id = g.id
        WHERE i.from_user_id = ?
          AND (i.status = 'pending' OR i.status = 'rejected')
        ORDER BY i.created_at DESC
        "#
    )
        .bind(user_id)
        .fetch_all(db)
        .await?;

    Ok(invites)
}

// Recupera un invito per ID.
pub async fn find_by_id(
    db: &Pool<Sqlite>,
    invitation_id: i64,
) -> Result<Option<Invitation>, AppError> {
    let inv = sqlx::query_as::<_, Invitation>("SELECT * FROM invitations WHERE id = ?")
        .bind(invitation_id)
        .fetch_optional(db)
        .await?;
    Ok(inv)
}

// Cerca se esiste già un invito DM pendente tra due utenti.
pub async fn find_pending_dm_invitation(
    db: &Pool<Sqlite>,
    user_a: i64,
    user_b: i64,
) -> Result<Option<Invitation>, AppError> {
    let inv = sqlx::query_as::<_, Invitation>(
        r#"
        SELECT i.* FROM invitations i
        JOIN groups g ON i.group_id = g.id
        WHERE g.is_dm = TRUE
          AND i.status = 'pending'
          AND (
            (i.from_user_id = ? AND i.to_user_id = ?) OR
            (i.from_user_id = ? AND i.to_user_id = ?)
          )
        "#
    )
        .bind(user_a).bind(user_b)
        .bind(user_b).bind(user_a)
        .fetch_optional(db)
        .await?;

    Ok(inv)
}

// Verifica se esiste un invito DM rifiutato tra due utenti negli ultimi 2 minuti.
pub async fn is_in_cooldown_dm(
    db: &Pool<Sqlite>,
    user_a: i64,
    user_b: i64,
) -> Result<bool, AppError> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM invitations i
        JOIN groups g ON i.group_id = g.id
        WHERE g.is_dm = TRUE
          AND i.status = 'rejected'
          AND (
            (i.from_user_id = ? AND i.to_user_id = ?) OR
            (i.from_user_id = ? AND i.to_user_id = ?)
          )
          AND i.updated_at > datetime('now', '-2 minutes')
        "#
    )
        .bind(user_a).bind(user_b)
        .bind(user_b).bind(user_a)
        .fetch_one(db)
        .await?;

    Ok(count.0 > 0)
}

// Controlla lo stato di un invito di gruppo tra due utenti.
pub async fn check_group_invitation_status(
    db: &Pool<Sqlite>,
    from_user: i64,
    to_user: i64,
    group_id: i64,
) -> Result<String, AppError> {
    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invitations WHERE from_user_id = ? AND to_user_id = ? AND group_id = ? AND status = 'pending'"
    )
        .bind(from_user).bind(to_user).bind(group_id)
        .fetch_one(db).await?;

    if pending.0 > 0 { return Ok("pending".to_string()); }

    let cooldown: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invitations WHERE from_user_id = ? AND to_user_id = ? AND group_id = ? AND status = 'rejected' AND updated_at > datetime('now', '-2 minutes')"
    )
        .bind(from_user).bind(to_user).bind(group_id)
        .fetch_one(db).await?;

    if cooldown.0 > 0 { return Ok("cooldown".to_string()); }

    Ok("ok".to_string())
}