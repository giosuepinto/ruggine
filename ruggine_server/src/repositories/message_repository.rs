
// ============================================================================
// message_repository.rs - Gestione dei messaggi di chat nel database
// Funzioni per salvataggio, recupero e formattazione messaggi
// ============================================================================


// Import di terze parti
use sqlx::{FromRow, Pool, Sqlite};

// Import interni
use crate::error::AppError;

// Rappresenta i dati di un messaggio di chat recuperati dal DB, già formattati per il client
#[derive(Debug, FromRow)]
pub struct ChatMessage {
    pub from_username: String,
    pub content: String,
}


// ============================================================================
// Funzioni di scrittura (creazione messaggi)
// ============================================================================

// Salva un singolo messaggio di chat nel database (operazione atomica)
pub async fn create_message(
    db: &Pool<Sqlite>,
    group_id: i64,
    sender_user_id: i64,
    content: &str,
) -> Result<(), AppError> {

    // Query estratta direttamente da webSocket_controller
    sqlx::query(
        "INSERT INTO messages (group_id, sender_user_id, content) VALUES (?, ?, ?)",
    )
        .bind(group_id)
        .bind(sender_user_id)
        .bind(content)
        .execute(db)
        .await?;

    Ok(())
}


// ============================================================================
// Funzioni di lettura (recupero messaggi)
// ============================================================================

// Recupera gli ultimi 50 messaggi per un dato gruppo (cronologia)
pub async fn get_history_for_group(
    db: &Pool<Sqlite>,
    group_id: i64,
) -> Result<Vec<ChatMessage>, AppError> {

    // Facciamo un JOIN con la tabella 'users' per ottenere l'username
    // e ordiniamo per 'created_at' per avere i messaggi nell'ordine corretto.
    let messages = sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT
            u.username AS from_username,
            m.content
        FROM messages m
        JOIN users u ON m.sender_user_id = u.id
        WHERE m.group_id = ?
        ORDER BY m.created_at ASC
        LIMIT 50
        "#
    )
        .bind(group_id)
        .fetch_all(db)
        .await?;

    Ok(messages)
}