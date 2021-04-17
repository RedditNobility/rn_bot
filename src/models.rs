use serde::{Serialize, Deserialize};
use diesel::{Queryable};
use crate::schema::*;
use std::str::FromStr;
use crate::event::NewEvent;
use std::time::{UNIX_EPOCH, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct User {
    pub uid: i64,
    pub discord_id: String,
    pub reddit_username: String,
    pub created: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct Event {
    pub eid: i64,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub active: bool,
    pub end: Option<i64>,
    pub created: i64,
}

impl Event {
    pub fn create(new_event: NewEvent) -> Event {
        Event {
            eid: 0,
            name: new_event.name.clone(),
            description: new_event.description.clone(),
            creator: new_event.creator.unwrap().clone(),
            active: true,
            end: None,
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
        }
    }
}