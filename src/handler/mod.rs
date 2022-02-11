use std::path::Path;

use chrono::Duration;

use rand::seq::SliceRandom;
use regex::Regex;
use rraw::auth::PasswordAuthenticator;
use rraw::me::Me;

use serenity::model::gateway::Activity;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::prelude::User;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use tokio::time::sleep;

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use crate::utils::{lines_from_file, refresh_reddit_count, refresh_server_count, subreddit_info, user_info};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, status: Context, _guild: GuildId, member: Member) {
        println!("Test");
        let channel = ChannelId(830415533673414696);
        let file = lines_from_file(Path::new("resources").join("welcome-jokes"));

        let option: &String = file.choose(&mut rand::thread_rng()).unwrap();
        let _msg = channel
            .send_message(&status.http, |m| {
                m.embed(|e| {
                    e.title(format!("Welcome {}", member.user.name.clone()));
                    e.description(option.replace("{name}", &*member.user.name.clone()));
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });

                    e
                });
                m
            })
            .await;
        refresh_server_count(&status).await.unwrap();
    }
    async fn guild_member_removal(
        &self,
        status: Context,
        _guild: GuildId,
        _new: User,
        _old_if_available: Option<Member>,
    ) {
        let channel = ChannelId(840919470695645184);
        let file = lines_from_file(Path::new("resources").join("exit-messages"));

        let option: &String = file.choose(&mut rand::thread_rng()).unwrap();
        let _msg = channel
            .send_message(&status.http, |m| {
                m.embed(|e| {
                    e.title(format!("Goodbye {}", _new.name.clone()));
                    e.description(option.replace("{name}", &*_new.name.clone()));
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });

                    e
                });
                m
            })
            .await;
        refresh_server_count(&status).await.unwrap();
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id.to_string().eq("829825560930156615") {
            return;
        }
        if msg.is_own(&ctx.cache).await {
            return;
        }

        let re = Regex::new("r/[A-Za-z0-9_-]+").unwrap();
        let option = re.find_iter(msg.content.as_str());
        subreddit_info(&ctx, option, &msg).await;

        let user_re = Regex::new("u/[A-Za-z0-9_-]+").unwrap();
        let option = user_re.find_iter(msg.content.as_str());
        user_info(&ctx, option, &msg).await;
    }
    async fn ready(&self, status: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        status.online().await;
        status.set_activity(Activity::listening("For Tasks from the Nobles")).await;

        tokio::spawn(async move {
            let arc = PasswordAuthenticator::new(
                std::env::var("CLIENT_KEY").unwrap().as_str(),
                std::env::var("CLIENT_SECRET").unwrap().as_str(),
                std::env::var("REDDIT_USER").unwrap().as_str(),
                std::env::var("PASSWORD").unwrap().as_str(),
            );
            let reddit = Me::login(
                arc,
                "RedditNobility Discord bot(by u/KingTuxWH)".to_string(),
            )
            .await
            .unwrap();
            loop {
                refresh_reddit_count(status.clone(), &reddit).await.unwrap();
                sleep(Duration::minutes(15).to_std().unwrap()).await;
            }
        })
        .await
        .unwrap();
    }
}
