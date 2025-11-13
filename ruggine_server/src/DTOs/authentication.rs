// src/models/authentication_controller
use serde::{Deserialize, Serialize};

/// DTO per le richieste HTTP a /login e /register.
#[derive(Deserialize, Debug)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

/// DTO per le risposte HTTP da /login e /register.
#[derive(Serialize, Debug, Clone)]
pub struct AuthResponse {
    pub message: String,
}