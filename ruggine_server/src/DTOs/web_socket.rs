// src/models/webSocket_controller
use serde::{Deserialize, Serialize};
use sqlx::FromRow; //

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

    #[serde(rename = "invite_to_group")]
    InviteToGroup {
        group_id: i64,
        username: String,
    },

    #[serde(rename = "send_group_message")]
    SendGroupMessage { group_id: i64, content: String },
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
}

/// DTO che rappresenta una singola chat (gruppo o DM) per il client.
#[derive(Serialize, Debug, Clone)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub is_dm: bool,
    pub members: Vec<GroupMember>,
    pub messages: Vec<ServerMessage>, // Cronologia
}

// === DTO CONDIVISO ===

/// DTO che rappresenta un utente all'interno di un `GroupInfo`.
#[derive(Serialize, Debug, Clone, FromRow)]
pub struct GroupMember {
    pub id: i64,
    pub username: String,
}