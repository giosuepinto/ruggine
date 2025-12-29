
// ============================================================================
// user_repository.rs - Gestione degli utenti nel database
// Funzioni CRUD per utenti e query correlate
// ============================================================================

// Import di terze parti
use sqlx::{Sqlite, Pool, FromRow, SqlitePool};

// Import interni
use crate::error::AppError;


// Struttura che rappresenta un utente nel database
#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}


// Crea un nuovo utente nel database
pub async fn create_user(
    db: &SqlitePool,
    username: &str,
    hash: &str
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}


// Trova un utente per username
pub async fn find_by_username(
    db: &Pool<Sqlite>,
    username: &str
) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash FROM users WHERE username = ?"
    )
        .bind(username)
        .fetch_optional(db)
        .await?;
    Ok(user)
}