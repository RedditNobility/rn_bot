use diesel::MysqlConnection;
use diesel::prelude::*;

use crate::models;
use crate::models::{User};

pub fn get_user_by_discord(discord: String, conn: &MysqlConnection) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found = users.filter(discord_id.eq(discord)).first::<models::User>(conn).optional()?;

    Ok(found)
}

pub fn get_user_by_reddit(reddit: String, conn: &MysqlConnection) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found = users.filter(reddit_username.eq(reddit)).first::<models::User>(conn).optional()?;

    Ok(found)
}

pub fn add_user(user: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::insert_into(users).values(user).execute(conn).unwrap();

    Ok(())
}