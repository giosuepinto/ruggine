-- migrations/xxxxxxxx_create_chat_tables.sql

-- Tabella per i gruppi e i DM
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 'name' è opzionale. Per i DM sarà NULL.
    name TEXT,
    -- Flag per distinguere un DM (true) da un gruppo (false)
    is_dm BOOLEAN NOT NULL DEFAULT FALSE
);

-- Tabella "ponte" per collegare utenti e gruppi (relazione N-a-N)
CREATE TABLE IF NOT EXISTS group_members (
    group_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    -- Chiave primaria composita per assicurare che un utente
    -- possa essere in un gruppo solo una volta.
    PRIMARY KEY (group_id, user_id)
    );

-- Tabella per salvare tutti i messaggi
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    sender_user_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    -- Salva il timestamp di quando il server ha ricevuto il messaggio
    created_at DATETIME DEFAULT (datetime('now','localtime')),

    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_user_id) REFERENCES users(id) ON DELETE CASCADE
    );-- Add migration script here
