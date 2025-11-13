// src/repositories/user_repository.rs
use sqlx::{Sqlite, Pool, Transaction, FromRow, SqlitePool};
use crate::error::AppError;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

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