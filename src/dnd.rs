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

use crate::boterror::BotError;
use crate::{actions, site, Bot, DataHolder, DbPool, DbPoolType};

use serenity::model::gateway::Activity;
use serenity::model::id::RoleId;

#[group]
#[commands(dnd)]
#[allowed_roles("High Council Of DND")]
struct DND;

#[command("dnd")]
#[sub_commands(add, remove)]
async fn dnd(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command]
#[description("Adds User to the Role")]
async fn add(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if msg.mentions.is_empty() {
        msg.reply(&ctx.http, "Please Mention a user to add").await?;
    }
    for x in msg.mentions.iter() {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let mut member = guild.member(&ctx.http, x.id).await.unwrap();
        member.add_role(&ctx.http, 939176496709382205).await.unwrap();
        msg.reply(&ctx.http, format!("Adding DND Role to {}", x.name)).await;
    }
    Ok(())
}

#[command]
#[description("Removes a User from the Role")]
async fn remove(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if msg.mentions.is_empty() {
        msg.reply(&ctx.http, "Please Mention a user to add").await?;
    }
    for x in msg.mentions.iter() {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let mut member = guild.member(&ctx.http, x.id).await.unwrap();
        if member.roles.contains(&RoleId(939176496709382205)){
            member.remove_role(&ctx.http, 939176496709382205).await.unwrap();
        }
        msg.reply(&ctx.http, format!("Removing DND Role to {}", x.name)).await;
    }
    Ok(())
}
