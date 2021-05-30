use std::string::ToString;

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        Args,
        buckets::{LimitedFor, RevertBucket},
        CommandGroup,
        CommandOptions, CommandResult, DispatchError, help_commands, HelpOptions, macros::{check, command, group, help, hook}, Reason,
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

use crate::{actions, Bot, DataHolder, DbPool, DbPoolType, site};
use crate::boterror::BotError;
use crate::models::User;
use crate::schema::events::columns::active;

#[group]
#[commands(event, mod_info)]
#[allowed_roles("Moderator")]
struct Mod;

#[command("mod-info")]
#[sub_commands(create)]
async fn mod_info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let bot: &Bot = data.get::<DataHolder>().unwrap();
    let pool: &DbPoolType = data.get::<DbPool>().unwrap();
    let conn = pool.get();
    if conn.is_err() {
        //TODO handle
    }
    let conn = conn.unwrap();
    let mut user: Option<site::model::User> = None;
    if !msg.mentions.is_empty() {
        let x = actions::get_user_by_discord(msg.mentions.get(0).unwrap().id.to_string(), &conn);
        if let Err(error) = x {
            BotError::DBError(error).discord_message(msg, "Unable to retrieve User", ctx).await;
            return Ok(());
        }
        let option = x.unwrap();
        if let None = option {
            msg.reply_ping(&ctx.http, "user is not registered").await;
            return Ok(());
        }
        let reddit_user = bot.site_client.get_user(option.unwrap().reddit_username).await;
        if let Err(error) = reddit_user {
            error.discord_message(msg, "Unable to make site request", ctx).await;
            return Ok(());
        }
        let reddit_user = reddit_user.unwrap();
        if let None = reddit_user {
            msg.reply_ping(&ctx.http, "I am going to need a beer for this one").await;
            return Ok(());
        }
        user = reddit_user;
    }

    if let Some(u) = user {
        let _msg = msg.channel_id.send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title(u.username);
                e.field("Level", u.level.to_string(), true);
                e.field("Status", u.status.to_string(), true);
                e.field("Moderator", u.moderator, true);
                e.field("Discoverer", u.discoverer.to_string(), true);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        }).await;
    } else {
        msg.reply_ping(&ctx.http, "What!").await;
    }

    Ok(())
}

#[command("event")]
#[sub_commands(create)]
async fn event(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command]
#[aliases("start")]
#[description("Start a new event!")]
async fn create(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;
    let mut data = ctx.data.write().await;
    let x: &mut Bot = data.get_mut::<DataHolder>().unwrap();
    x.test();
    Ok(())
}
