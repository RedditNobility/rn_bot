use rraw::me::Me;
use rraw::responses::GenericResponse;
use rraw::responses::subreddit::AboutSubreddit;
use rraw::utils::error::APIError;
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
use serenity::model::id::ChannelId;

use crate::{Bot, DataHolder};

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

pub async fn refresh_reddit_count(status: Context, me: &Me) {
    let channel = ChannelId(833707456990281818);

    let subreddit = me.subreddit("RedditNobility".to_string());
    let result = subreddit.about().await;
    let count = match result {
        Ok(ok) => {
            ok.data.subscribers.unwrap().to_string()
        }
        Err(er) => {
            match er {
                APIError::ExhaustedListing => {
                    println!("Ex");
                }
                APIError::HTTPError(s) => {
                    println!("Status {}", s);
                }
                APIError::ReqwestError(_) => {
                    println!("Request");

                }
                APIError::JSONError(_) => {
                    println!("JSON");
                }
                APIError::ExpiredToken => {
                    println!("Expired");
                }
                APIError::Custom(s) => {
                    println!("Error: {}", s);

                }
            }
            "Error".to_string()
        }
    };

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
