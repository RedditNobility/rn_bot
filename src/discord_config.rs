use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Channels {
    pub bot_log: u64,
    pub error_log: u64,
    pub welcome_log: u64,
    pub general: u64,

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roles {
    pub registered: u64,
}