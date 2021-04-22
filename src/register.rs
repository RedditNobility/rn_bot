// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use std::{collections::{HashMap, HashSet}, env, fmt::Write, sync::Arc};
use serenity::{
    prelude::*,
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        Args, CommandOptions, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, Reason, StandardFramework,
        buckets::{RevertBucket, LimitedFor},
        macros::{command, group, help, check, hook},
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions},
};
use serde::{Serialize, Deserialize};
use serenity::prelude::*;
use tokio::sync::Mutex;
use crate::{Bot, DataHolder, actions};
use hyper::{Body, Method, Request, StatusCode};
use hyper::client::{Client, HttpConnector};
use hyper::header::USER_AGENT;
use hyper::http::request::Builder;
use hyper::Uri;
use hyper_tls::HttpsConnector;
use serenity::model::guild::Member;
use serenity::http::CacheHttp;
use serenity::model::id::RoleId;
use diesel::MysqlConnection;
use diesel::prelude::*;
use crate::models::User;
use std::time::{UNIX_EPOCH, SystemTime};
use std::sync::MutexGuard;
use crate::boterror::BotError;

#[group]
#[commands(register)]
struct Register;

#[command]
#[aliases("login")]
#[description("Gets you registered to the server")]
async fn register(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let x: &mut Bot = data.get_mut::<DataHolder>().unwrap();
    let option = _args.current();
    if x.connection.is_poisoned() {
        x.reset_connection();
    }
    let result = is_registered(msg.author.id, &x.connection.clone().lock().unwrap());
    if result.is_err() {
        x.reset_connection();
        result.err().unwrap().discord_message(msg, "Unable to make database query. Please try again.", &ctx).await;
        return Ok(());
    }
    if result.unwrap() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("You are already registered");
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });
                e
            });
            m
        }).await;
        return Ok(());
    }

    if option.is_none() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Please provide your Reddit username");
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });
                e
            });
            m
        }).await;
    }
    let username = option.unwrap().replace("u/", "");
    let user = validate_user(&*username).await;
    let result1 = is_registered_reddit(username.clone(), &x.connection.clone().lock().unwrap());
    if result1.is_err() {
        //    pub async fn discord_message(&self, message: &Message, error: &str, context: &Context) {
        result1.err().unwrap().discord_message(msg, "Unable to verify Reddit Name.", &ctx).await;
        return Ok(());
    }
    if result1.unwrap() {
        msg.channel_id.send_message(&ctx.http, |m| {
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
        }).await;
        return Ok(());
    }
    if user.is_err() {
        user.err().unwrap().discord_message(msg, "Unable to verify Reddit Name.", &ctx).await;
        return Ok(());
    } else if !user.unwrap() {
        msg.channel_id.send_message(&ctx.http, |m| {
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
        }).await;
    } else {
        let mut id = msg.channel_id.to_channel(&ctx.http).await.unwrap().guild().unwrap().guild_id.member(&ctx.http, &msg.author.id).await.unwrap();
        register_user(&ctx, &*username, id, &x.connection.clone()).await;
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("You have been registered");
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });
                e
            });
            m
        }).await;
    }
    Ok(())
}

async fn register_user(context: &Context, reddit_username: &str, mut member: Member, connect: &Arc<std::sync::Mutex<MysqlConnection>>) -> Result<(), BotError> {
    let x = member.add_role(&context.http, RoleId(830277916944236584)).await;
    member.edit(&context.http, |e| {
        e.nickname(reddit_username.clone().to_string())
    }).await;
    if x.is_err() {
        return Err(BotError::SerenityError(x.err().unwrap()));
    }
    let user = User {
        uid: 0,
        discord_id: member.user.id.to_string(),
        reddit_username: reddit_username.to_string().clone(),
        created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64,
    };
    let result = actions::add_user(&user, &connect.lock().unwrap());
    if result.is_err() {
        return Err(BotError::DBError(result.err().unwrap()));
    }
    return Ok(());
}


async fn validate_user(p0: &str) -> Result<bool, BotError> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let mut builder = (Builder::new()).header(USER_AGENT, "RedditNobilityBot").method(Method::GET).uri(format!("https://redditnobility.org/api/user/{}", p0));
    let result1 = builder.body(Body::empty());
    if result1.is_err() {
        return Err(BotError::HyperHTTPError(result1.err().unwrap()));
    }
    let request = result1.unwrap();
    let result = client.request(request).await;
    if result.is_err() {
        return Err(BotError::HyperError(result.err().unwrap()));
    }
    let response = result.unwrap();
    if response.status().is_success() {
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let string = String::from_utf8(bytes.to_vec()).unwrap();
        let user: WebsiteUser = serde_json::from_str(&*string).unwrap();
        return Ok(user.status == "Approved");
    } else if response.status().as_u16() == 404 {
        return Ok(false);
    } else {
        return Err(BotError::HTTPError(response.status().clone()));
    }
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