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


use crate::{Bot, DataHolder};
use serenity::model::id::ChannelId;


pub async fn refresh_server_count(status: &Context) {
    let channel = ChannelId(830636660197687316);
    let i = channel
        .to_channel(&status.http)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .guild_id
        .members(&status.http, None, None)
        .await
        .unwrap()
        .len();
    channel
        .to_channel(&status.http)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .edit(&status.http, |c| c.name(format!("Server Size: {}", i)))
        .await;
}

pub async fn refresh_reddit_count(status: &Context) {
    let channel = ChannelId(833707456990281818);
    let mut data = status.data.write().await;

    let x: &mut Bot = data.get_mut::<DataHolder>().unwrap();
    let mut count: String = "".to_string();
    let subreddit = x.reddit.as_ref().unwrap().subreddit("RedditNobility");
    let result = subreddit.about();
    if result.is_err() {
        println!("{}", result.err().unwrap());
        count = "Error".to_string();
    } else {
        count = result.unwrap().subscribers().to_string();
    }

    channel
        .to_channel(&status.http)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .edit(&status.http, |c| {
            c.name(format!("Reddit Subscribers: {}", count))
        })
        .await;
}
