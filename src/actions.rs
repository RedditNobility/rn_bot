use diesel::prelude::*;
use diesel::MysqlConnection;

use crate::models;
use crate::models::User;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_user_by_discord(
    discord: String,
    conn: &MysqlConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found = users
        .filter(discord_id.eq(discord))
        .first::<models::User>(conn)
        .optional()?;

    Ok(found)
}

pub fn get_user_by_reddit(
    reddit: String,
    conn: &MysqlConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found = users
        .filter(reddit_username.eq(reddit))
        .first::<models::User>(conn)
        .optional()?;

    Ok(found)
}

pub fn add_user(user: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::insert_into(users)
        .values(user)
        .execute(conn)
        .unwrap();

    Ok(())
}

pub fn get_active_event(
    conn: &MysqlConnection,
) -> Result<Option<models::Event>, diesel::result::Error> {
    use crate::schema::events::dsl::*;

    let found = events
        .filter(active.eq(true))
        .first::<models::Event>(conn)
        .optional()?;

    Ok(found)
}

pub fn end_event(event: i64, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::events::dsl::*;

    diesel::update(events.filter(eid.eq(event)))
        .set((
            active.eq(false),
            end.eq(Option::Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            )),
        ))
        .execute(conn);
    Ok(())
}
