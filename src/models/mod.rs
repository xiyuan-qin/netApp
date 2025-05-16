use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub msg_type: String,
    pub username: String,
    pub room: String,
    pub text: String,
    pub timestamp: u64,
    pub id: String,
    pub target: Option<String>,
}

pub struct UserSession {
    pub id: String,
    pub username: String,
    pub room: String,
    pub addr: String,
    pub session: actix_ws::Session,
    pub last_heartbeat: Instant,
    pub join_time: Instant,
}

pub struct AppState {
    pub sessions: std::sync::Mutex<std::collections::HashMap<String, UserSession>>,
    pub rooms: std::sync::Mutex<std::collections::HashMap<String, std::collections::HashSet<String>>>,
} 