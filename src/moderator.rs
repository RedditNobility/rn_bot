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

use serenity::prelude::*;
use tokio::sync::Mutex;
use crate::{Bot, DataHolder};


#[group]
#[commands(event)]
struct Mod;

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