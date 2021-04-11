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
use serenity::model::id::ChannelId;

pub async fn refresh_server_count(status: &Context) {
    let channel = ChannelId(830636660197687316);
    let i = channel.to_channel(&status.http).await.unwrap().guild().unwrap().guild_id.members(&status.http, None, None).await.unwrap().len();
    channel.to_channel(&status.http).await.unwrap().guild().unwrap().edit(&status.http, |c| {
        c.name(format!("Server Size: {}", i))
    }).await;
}
