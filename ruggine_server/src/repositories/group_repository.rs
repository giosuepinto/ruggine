
// ============================================================================
// group_repository.rs - Gestione dei gruppi e membri nel database
// Funzioni CRUD per gruppi, membri, DM e query correlate
// ============================================================================

// Import di terze parti
use sqlx::{FromRow, Pool, Sqlite, Transaction};

// Import interni
use crate::error::AppError;
use crate::dtos::web_socket::GroupMember;

// Rappresenta una riga della tabella 'groups' nel database.
// Nota: 'id' è incluso perché ci serve quando leggiamo dal DB.
#[derive(Debug, FromRow)]
pub struct Group {
    #[allow(dead_code)]
    pub id: i64,
    pub name: Option<String>,
    pub is_dm: bool,
}


// ============================================================================
// Funzioni di creazione (scrittura): richiedono una transazione per atomicità
// ============================================================================

// Inserisce un nuovo gruppo (DM o standard) nel database. Restituisce l'ID creato.
pub async fn create_group(
    tx: &mut Transaction<'_, Sqlite>, // Richiede una transazione attiva
    name: Option<&str>,
    is_dm: bool,
) -> Result<i64, AppError> {

    let query_str = if is_dm {
        "INSERT INTO groups (is_dm) VALUES (TRUE)" //
    } else {
        "INSERT INTO groups (name, is_dm) VALUES (?, FALSE)" //
    };

    let query = if is_dm {
        sqlx::query(query_str)
    } else {
        sqlx::query(query_str).bind(name)
    };

    let group_id = query
        .execute(&mut **tx) // Esegue all'interno della transazione
        .await?
        .last_insert_rowid();

    Ok(group_id)
}

// Aggiunge un singolo utente a un gruppo (usato per il creatore).
pub async fn add_member(
    tx: &mut Transaction<'_, Sqlite>, // Richiede una transazione attiva
    group_id: i64,
    user_id: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)", //
    )
        .bind(group_id)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

// // Aggiunge i due membri di un DM in una singola query.
// // Specifica per la logica di 'CreateDM'.
// pub async fn add_dm_members(
//     tx: &mut Transaction<'_, Sqlite>, // Richiede una transazione attiva
//     group_id: i64,
//     user_id_1: i64,
//     user_id_2: i64,
// ) -> Result<(), AppError> {
//     sqlx::query(
//         "INSERT INTO group_members (group_id, user_id) VALUES (?, ?), (?, ?)", //
//     )
//         .bind(group_id)
//         .bind(user_id_1)
//         .bind(group_id)
//         .bind(user_id_2)
//         .execute(&mut **tx)
//         .await?;

//     Ok(())
// }

// =================================================================
// FUNZIONI DI MODIFICA
// Questa usa il Pool perché è un'operazione atomica
// =================================================================

// Aggiunge un utente a un gruppo, ignorando l'errore se è già membro.
// Usato per la logica di 'InviteToGroup'.
pub async fn add_member_or_ignore(
    db: &Pool<Sqlite>, // Usa il pool principale
    group_id: i64,
    user_id: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO group_members (group_id, user_id) VALUES (?, ?)", //
    )
        .bind(group_id)
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

// =================================================================
// FUNZIONI DI LETTURA
// Queste usano il Pool (db) perché sono solo operazioni di lettura.
// =================================================================

// Trova i dettagli di un singolo gruppo (come 'is_dm').
pub async fn find_group_by_id(
    db: &Pool<Sqlite>,
    group_id: i64,
) -> Result<Option<Group>, AppError> {
    let group = sqlx::query_as::<_, Group>(
        "SELECT id, name, is_dm FROM groups WHERE id = ?", 
    )
        .bind(group_id)
        .fetch_optional(db)
        .await?;

    Ok(group)
}

// Recupera tutti gli ID dei gruppi a cui un utente appartiene.
pub async fn find_group_ids_by_user(
    db: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Vec<i64>, AppError> {
    let ids = sqlx::query_as::<_, (i64,)>(
        "SELECT group_id FROM group_members WHERE user_id = ?", //
    )
        .bind(user_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(id,)| id) // Estrae l'i64 dalla tupla
        .collect();

    Ok(ids)
}

// Recupera tutti i membri (con ID e Username) di un dato gruppo.
pub async fn find_members_by_group(
    db: &Pool<Sqlite>,
    group_id: i64,
) -> Result<Vec<GroupMember>, AppError> {
    let members = sqlx::query_as::<_, GroupMember>(
        "SELECT u.id, u.username FROM users u
         JOIN group_members gm ON u.id = gm.user_id
         WHERE gm.group_id = ?", //
    )
        .bind(group_id)
        .fetch_all(db)
        .await?;

    Ok(members)
}

// Recupera solo gli ID di tutti i membri di un dato gruppo.
// Molto efficiente per la logica di broadcast.
pub async fn find_member_ids_by_group(
    db: &Pool<Sqlite>,
    group_id: i64,
) -> Result<Vec<i64>, AppError> {
    let ids = sqlx::query_as::<_, (i64,)>(
        "SELECT user_id FROM group_members WHERE group_id = ?", //
    )
        .bind(group_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|(id,)| id) // Estrae l'i64 dalla tupla
        .collect();

    Ok(ids)
}

// Cerca se esiste già un gruppo DM attivo tra due utenti.
pub async fn find_existing_dm_between_users(
    db: &Pool<Sqlite>,
    user_id_1: i64,
    user_id_2: i64,
) -> Result<Option<i64>, AppError> {
    // Query che cerca un gruppo DM esistente tra due utenti
    let result: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT g.id
        FROM groups g
        JOIN group_members gm1 ON g.id = gm1.group_id
        JOIN group_members gm2 ON g.id = gm2.group_id
        WHERE g.is_dm = TRUE
          AND gm1.user_id = ?
          AND gm2.user_id = ?
        LIMIT 1
        "#
    )
        .bind(user_id_1)
        .bind(user_id_2)
        .fetch_optional(db)
        .await?;

    Ok(result.map(|r| r.0))
}