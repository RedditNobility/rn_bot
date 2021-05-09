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
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
};

use crate::{main, utils, Bot, DataHolder};
use serenity::prelude::*;
use tokio::sync::Mutex;

#[group]
#[commands(rcount)]
struct Admin;

#[command("refresh_count")]
#[required_permissions("ADMINISTRATOR")]
async fn rcount(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "Refreshed Server Count").await?;
    utils::refresh_server_count(ctx).await;
    Ok(())
}
