// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use crate::bot_error::BotError;
use crate::models::User;
use crate::{actions, Bot, DataHolder, DbPool, DbPoolType};
use diesel::MysqlConnection;

use serenity::model::guild::Member;
use serenity::model::id::RoleId;
use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{channel::Message, id::UserId},
};

use crate::site::site_client::SiteClient;
use std::time::{SystemTime, UNIX_EPOCH};

#[group]
#[commands(register)]
struct Register;

#[command]
#[aliases("login")]
#[description("Gets you registered to the server")]
async fn register(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let _x: &Bot = data.get::<DataHolder>().unwrap();
    let option = _args.current();
    let pool: &DbPoolType = data.get::<DbPool>().unwrap();
    let conn = pool.get()?;
    let value = is_registered(msg.author.id, &conn)?;
    if let Some(user) = value {
        return if msg
            .author
            .has_role(&ctx.http, msg.guild_id.unwrap(), RoleId(830277916944236584))
            .await?
        {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("You are already registered");
                        e.footer(|f| {
                            f.text("Robotic Monarch");
                            f
                        });
                        e
                    });
                    m
                })
                .await?;
            Ok(())
        } else {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("You are already registered! Adding role!");
                        e.footer(|f| {
                            f.text("Robotic Monarch");
                            f
                        });
                        e
                    });
                    m
                })
                .await?;

            register_user_discord(
                ctx,
                user.reddit_username.as_str(),
                msg.member(&ctx.http).await?,
            )
                .await?;

            Ok(())
        };
    }

    if option.is_none() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Please provide your Reddit username");
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });
                    e
                });
                m
            })
            .await?;
        return Ok(());
    }

    let username = option.unwrap().replace("u/", "");
    let user = validate_user(&username, &_x.site_client).await?;
    let result1 = is_registered_reddit(username.clone(), &conn)?;

    if result1 {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("That name has already been claimed.");
                    e.description("If this is you please contact a mod for help.");
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });
                    e
                });
                m
            })
            .await?;
        return Ok(());
    }

    if !user {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Your username is not found in the database.");
                    e.description("If this username is correct please get a mod");
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });
                    e
                });
                m
            })
            .await?;
    } else {
        let id = msg
            .channel_id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .unwrap()
            .guild_id
            .member(&ctx.http, &msg.author.id)
            .await?;
        register_user(&*username, &id, &conn)?;
        register_user_discord(ctx, &*username, id).await?;

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("You have been registered");
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });
                    e
                });
                m
            })
            .await?;
    }
    Ok(())
}

async fn register_user_discord(
    context: &Context,
    reddit_username: &str,
    mut member: Member,
) -> Result<(), BotError> {
    member
        .add_role(&context.http, RoleId(830277916944236584))
        .await?;
    member
        .edit(&context.http, |e| e.nickname(reddit_username.to_string()))
        .await?;

    Ok(())
}

fn register_user(
    reddit_username: &str,
    member: &Member,
    conn: &MysqlConnection,
) -> Result<(), BotError> {
    let user = User {
        uid: 0,
        discord_id: member.user.id.to_string(),
        reddit_username: reddit_username.to_string(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    };
    let result = actions::add_user(&user, conn);
    if result.is_err() {
        return Err(BotError::DBError(result.err().unwrap()));
    }
    Ok(())
}

async fn validate_user(username: &str, site_client: &SiteClient) -> Result<bool, BotError> {
    let x = site_client.get_user(username).await?;

    Ok(x.is_some())
}

fn is_registered(p0: UserId, connect: &MysqlConnection) -> Result<Option<User>, BotError> {
    let x = actions::get_user_by_discord(p0.0.to_string(), connect);
    if x.is_err() {
        return Err(BotError::DBError(x.err().unwrap()));
    }
    let result = x.unwrap();
    Ok(result)
}

fn is_registered_reddit(
    reddit_username: String,
    connect: &MysqlConnection,
) -> Result<bool, BotError> {
    let x = actions::get_user_by_reddit(reddit_username, connect);
    if x.is_err() {
        return Err(BotError::DBError(x.err().unwrap()));
    }
    let result = x.unwrap();

    Ok(result.is_some())
}

