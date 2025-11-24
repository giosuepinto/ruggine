// src/DTOs/web_socket.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// === DAL CLIENT AL SERVER ===

/// Enum per tutti i messaggi WebSocket inviati DAL CLIENT.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },

    #[serde(rename = "create_dm")]
    CreateDM { username: String },

    #[serde(rename = "create_group")]
    CreateGroup { name: String },

    #[serde(rename = "send_group_message")]
    SendGroupMessage { group_id: i64, content: String },

    #[serde(rename = "invite_to_group")]
    InviteToGroup {
        group_id: i64,
        username: String,
    },

    #[serde(rename = "handle_invitation")]
    HandleInvitation {
        invitation_id: i64,
        accept: bool, // true = accetta, false = rifiuta
    },
}

// === DAL SERVER AL CLIENT ===

// Enum per tutti i messaggi WebSocket inviati DAL SERVER.
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "group_chat")]
    GroupChat {
        group_id: i64,
        from_username: String,
        content: String,
    },

    #[serde(rename = "error")]
    Error { content: String },

    #[serde(rename = "update_group_list")]
    UpdateGroupList { groups: Vec<GroupInfo> },

    // --- MODIFICATO: Ora contiene le due liste separate ---
    #[serde(rename = "update_invitations_list")]
    UpdateInvitationsList {
        received: Vec<InvitationInfo>,
        sent: Vec<InvitationInfo>
    },
}

// === DTOs (DATA TRANSFER OBJECTS) ===

/// DTO che rappresenta una singola chat (gruppo o DM) per il client.
#[derive(Serialize, Debug, Clone)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub is_dm: bool,
    pub members: Vec<GroupMember>,
    pub messages: Vec<ServerMessage>, // Cronologia
}

/// DTO che rappresenta un utente all'interno di un `GroupInfo`.
#[derive(Serialize, Debug, Clone, FromRow)]
pub struct GroupMember {
    pub id: i64,
    pub username: String,
}

/// DTO per le informazioni di un invito.
#[derive(Serialize, Debug, Clone, FromRow)]
pub struct InvitationInfo {
    pub id: i64,
    // Rappresenta "l'altro utente" (chi invita O chi riceve)
    pub partner_username: String,
    pub group_name: String,
    pub group_id: i64,
    pub status: String, // 'pending', 'rejected'
    pub sent_at: String,
}