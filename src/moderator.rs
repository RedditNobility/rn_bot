use std::string::ToString;

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};

use crate::{actions, site, Bot, DataHolder, DbPool, DbPoolType};

use serenity::model::gateway::Activity;

#[group]
#[commands(event, mod_info, set_status)]
#[allowed_roles("Moderator")]
struct Mod;
#[command("setStatus")]
async fn set_status(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let x = args.message();
    ctx.online().await;
    ctx.set_activity(Activity::playing(x)).await;
    return Ok(());
}
#[command("mod-info")]
async fn mod_info(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let bot: &Bot = data.get::<DataHolder>().unwrap();
    let pool: &DbPoolType = data.get::<DbPool>().unwrap();
    let conn = pool.get()?;
    let mut user: Option<site::model::User> = None;
    if !msg.mentions.is_empty() {
        let x = actions::get_user_by_discord(msg.mentions.get(0).unwrap().id.to_string(), &conn)?;
        if x.is_none() {
            msg.reply_ping(&ctx.http, "user is not registered").await?;
            return Ok(());
        }
        let reddit_user = bot
            .site_client
            .get_user(x.unwrap().reddit_username)
            .await?;
        if reddit_user.is_none() {
            msg.reply_ping(&ctx.http, "I am going to need a beer for this one")
                .await?;
            return Ok(());
        }
        user = reddit_user;
    } else {
        msg.reply_ping(&ctx.http, "Please mention a user").await?;
    }

    if let Some(u) = user {
        let _msg = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.reference_message(msg);
                m.embed(|e| {
                    e.title(u.username);
                    e.field("Status", u.status.to_string(), true);
                    if !u.reviewer.is_empty() {
                        e.field("Reviewer", u.reviewer, true);
                    }
                    e.field("Discoverer", u.discoverer.to_string(), true);
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
        msg.reply_ping(&ctx.http, "What!").await.unwrap();
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
