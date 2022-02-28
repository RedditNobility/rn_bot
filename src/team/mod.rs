use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub role: u64,
    pub name: String,
    pub id: String,
}