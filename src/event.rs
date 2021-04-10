use serde::{Serialize, Deserialize};
use diesel::{Queryable};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub name: String,
    pub description: String

}
