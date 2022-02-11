use serde::{Deserialize, Serialize};

use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub moderator: bool,
    #[serde(default)]
    pub submit: bool,
    #[serde(default)]
    pub review_user: bool,
    #[serde(default)]
    pub login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub id: i64,
    pub user: i64,
    pub token: String,
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
    pub discord_id: i64,
    //The Reddit Username
    pub username: String,
    //USER, MODERATOR, ADMIN
    pub permissions: UserPermissions,
    //FOUND, DENIED, APPROVED, BANNED
    pub status: Status,
    //When was their status changed from FOUND to DENIED or APPROVED
    pub status_changed: i64,
    //Who found the user BOT if bot
    pub discoverer: String,
    //The Moderator who approved them or denied them. If the user was banned it will still be set to who approved them
    pub reviewer: String,
    // Custom Properties done through json.
    pub properties: UserProperties,
    //When the data was created
    pub created: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Display, PartialEq, EnumString)]
pub enum Status {
    Found,
    Denied,
    Approved,
    Banned,
}

//Found, Approved, Denied, Banned
#[derive(Debug, Deserialize, Serialize, Clone, Display, PartialEq, EnumString)]
pub enum Level {
    Admin,
    Moderator,
    User,
    Client,
}
