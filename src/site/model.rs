use serde::{Deserialize, Serialize};

use std::fmt::{Display, Error, Formatter};
use std::io::Write;
use std::str::FromStr;
use strum_macros::Display;
use strum_macros::EnumString;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub id: i64,
    pub user: i64,
    pub token: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKey {
    pub id: i64,
    pub api_key: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProperties {
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

impl UserProperties {
    pub fn set_avatar(&mut self, avatar: String) {
        self.avatar = Some(avatar);
    }
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    //The Reddit Username
    pub username: String,
    //USER, MODERATOR, ADMIN
    pub level: Level,
    //FOUND, DENIED, APPROVED, BANNED
    pub status: Status,
    //When was their status changed from FOUND to DENIED or APPROVED
    pub status_changed: i64,
    //Who found the user BOT if bot
    pub discoverer: String,
    //The Moderator who approved them or denied them. If the user was banned it will still be set to who approved them
    pub moderator: String,
    // Custom Properties done through json.
    pub properties: UserProperties,
    //When the data was created
    pub created: i64,
}



#[derive(Debug, Deserialize, Serialize, Clone, Display, PartialEq, EnumString, )]
pub enum Status {
    Found,
    Denied,
    Approved,
    Banned,
}

//Found, Approved, Denied, Banned
#[derive(Debug, Deserialize, Serialize, Clone, Display, PartialEq, EnumString, )]
pub enum Level {
    Admin,
    Moderator,
    User,
    Client,
}