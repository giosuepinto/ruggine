use serde::{Deserialize, Serialize};

// Messaggi inviati DAL CLIENT al SERVER
#[derive(Deserialize, Debug)]
#[serde(tag = "type")] // Usa il campo "type" per distinguere l'enum
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },

    #[serde(rename = "chat")]
    Chat { content: String },
}

// Messaggi inviati DAL SERVER al CLIENT
#[derive(Serialize, Debug, Clone)] // Clone è importante per il broadcast
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "chat")]
    Chat { from: String, content: String },

    #[serde(rename = "info")]
    Info { content: String },
}