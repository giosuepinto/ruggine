-- migrations/xxxx_create_invitations_table.sql

CREATE TABLE IF NOT EXISTS invitations (
                                           id INTEGER PRIMARY KEY AUTOINCREMENT,
                                           from_user_id INTEGER NOT NULL,
                                           to_user_id INTEGER NOT NULL,
                                           group_id INTEGER NOT NULL,
                                           status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'accepted', 'rejected'
                                           created_at DATETIME DEFAULT (datetime('now','localtime')),
    updated_at DATETIME DEFAULT (datetime('now','localtime')),

    FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
    );

-- Trigger per aggiornare automaticamente updated_at quando cambia lo stato
CREATE TRIGGER IF NOT EXISTS update_invitations_timestamp
AFTER UPDATE ON invitations
BEGIN
UPDATE invitations SET updated_at = datetime('now','localtime') WHERE id = NEW.id;
END;

-- Assicuriamoci che esista il Gruppo Globale (ID 1)
INSERT OR IGNORE INTO groups (id, name, is_dm) VALUES (1, 'Global Chat', FALSE);-- Add migration script here
