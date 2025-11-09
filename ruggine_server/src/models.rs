use serde::{Deserialize, Serialize};

// --- NUOVE STRUCT per la UI ---

#[derive(Serialize, Debug, Clone)]
pub struct GroupMember {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub is_dm: bool,
    pub members: Vec<GroupMember>,
}


// Messaggi inviati DAL CLIENT al SERVER
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },

    #[serde(rename = "create_dm")]
    CreateDM { username: String }, // Per iniziare una chat con un utente

    #[serde(rename = "create_group")]
    CreateGroup { name: String }, // Per creare un gruppo

    #[serde(rename = "invite_to_group")]
    InviteToGroup {
        group_id: i64,
        username: String, // L'username della persona da invitare
    },

    #[serde(rename = "send_group_message")]
    SendGroupMessage { group_id: i64, content: String },
}

// Messaggi inviati DAL SERVER al CLIENT
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