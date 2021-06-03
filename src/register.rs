// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use crate::boterror::BotError;
use crate::models::User;
use crate::{actions, Bot, DataHolder, DbPool, DbPoolType};
use diesel::prelude::*;
use diesel::MysqlConnection;
use hyper::client::{Client};
use hyper::header::USER_AGENT;
use hyper::http::request::Builder;

use hyper::{Body, Method};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serenity::http::CacheHttp;
use serenity::model::guild::Member;
use serenity::model::id::RoleId;
use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions, Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
};
use crate::site::site_client::SiteClient;


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
    let conn = pool.get();
    if conn.is_err() {
        //TODO handle
    }
    let conn = conn.unwrap();
    let result = is_registered(msg.author.id, &conn);
    if result.is_err() {
        result
            .err()
            .unwrap()
            .discord_message(
                msg,
                "Unable to make database query. Please try again.",
                &ctx,
            )
            .await;
        return Ok(());
    }
    if result.unwrap() {
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
            .await;
        return Ok(());
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
            .await;
        return Ok(());
    }
    let username = option.unwrap().replace("u/", "");
    let user = validate_user(&*username, &_x.site_client).await;
    let result1 = is_registered_reddit(username.clone(), &conn);
    if result1.is_err() {
        //    pub async fn discord_message(&self, message: &Message, error: &str, context: &Context) {
        result1
            .err()
            .unwrap()
            .discord_message(msg, "Unable to verify Reddit Name.", &ctx)
            .await;
        return Ok(());
    }
    if result1.unwrap() {
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
            .await;
        return Ok(());
    }
    if user.is_err() {
        user.err()
            .unwrap()
            .discord_message(msg, "Unable to verify Reddit Name.", &ctx)
            .await;
        return Ok(());
    } else if !user.unwrap() {
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
            .await;
    } else {
        let id = msg
            .channel_id
            .to_channel(&ctx.http)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .guild_id
            .member(&ctx.http, &msg.author.id)
            .await
            .unwrap();
        let result2 = register_user(&*username, &id, &conn);
        if result2.is_err() {
            result2
                .err()
                .unwrap()
                .discord_message(&msg, "Unable to approve you", &ctx)
                .await;
            return Ok(());
        }
        let result2 = register_user_discord(&ctx, &*username, id).await;
        if result2.is_err() {
            result2.err().unwrap().discord_message(&msg, "You were approved however,we were unable to add a Discord role. Please have a mod add it for you.", &ctx).await;
            return Ok(());
        }
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
            .await;
    }
    Ok(())
}

async fn register_user_discord(
    context: &Context,
    reddit_username: &str,
    mut member: Member,
) -> Result<(), BotError> {
    let x = member
        .add_role(&context.http, RoleId(830277916944236584))
        .await;
    member
        .edit(&context.http, |e| {
            e.nickname(reddit_username.clone().to_string())
        })
        .await;
    if x.is_err() {
        return Err(BotError::SerenityError(x.err().unwrap()));
    }
    return Ok(());
}

fn register_user(
    reddit_username: &str,
    member: &Member,
    conn: &MysqlConnection,
) -> Result<(), BotError> {
    let user = User {
        uid: 0,
        discord_id: member.user.id.to_string(),
        reddit_username: reddit_username.to_string().clone(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    };
    let result = actions::add_user(&user, &conn);
    if result.is_err() {
        return Err(BotError::DBError(result.err().unwrap()));
    }
    return Ok(());
}

async fn validate_user(p0: &str, site_client: &SiteClient) -> Result<bool, BotError> {
    let x = site_client.get_user(p0.parse().unwrap()).await;
    if let Err(error) = x {
        return Err(error);
    } else if let Ok(ok) = x {
        return Ok(ok.is_some());
    }
    return Ok(false);
}

fn is_registered(p0: UserId, connect: &MysqlConnection) -> Result<bool, BotError> {
    let x = actions::get_user_by_discord(p0.0.to_string(), connect);
    if x.is_err() {
        return Err(BotError::DBError(x.err().unwrap()));
    }
    let result = x.unwrap();
    return Ok(result.is_some());
}

fn is_registered_reddit(p0: String, connect: &MysqlConnection) -> Result<bool, BotError> {
    let x = actions::get_user_by_reddit(p0, connect);
    if x.is_err() {
        return Err(BotError::DBError(x.err().unwrap()));
    }
    let result = x.unwrap();
    return Ok(result.is_some());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteUser {
    pub id: i64,
    pub username: String,
    pub status: String,
    pub moderator: String,
    pub created: i64,
}
