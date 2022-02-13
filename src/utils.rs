use rraw::me::Me;
use std::fs::{ read};
use std::path::Path;
use chrono::Duration;

use rraw::utils::error::APIError;
use rust_embed::RustEmbed;
use serenity::model::id::ChannelId;
use serenity::{model::channel::Message, prelude::*};

use crate::bot_error::BotError;
use hyper::StatusCode;
use num_format::{Locale, ToFormattedString};
use regex::Matches;
use rraw::auth::AnonymousAuthenticator;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    #[allow(dead_code)]
    pub fn file_get(file: &str) -> Vec<u8> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            read(buf).unwrap()
        } else {
            Resources::get(file).unwrap().data.to_vec()
        }
    }
    #[allow(dead_code)]
    pub fn file_get_string(file: &str) -> String {
        let vec = Resources::file_get(file);
        String::from_utf8(vec).unwrap()
    }
    pub fn lines_from_resource(filename: &str) -> Vec<String> {
        let string = Resources::file_get_string(filename);
        string.lines().map(|l|l.to_string()).collect()
    }
}

pub async fn refresh_server_count(status: &Context) -> Result<(), BotError> {
    let channel = ChannelId(830636660197687316);
    let server_size = channel
        .to_channel(&status.http)
        .await?
        .guild()
        .unwrap()
        .guild_id
        .members(&status.http, None, None)
        .await?
        .into_iter()
        .filter(|x| !x.user.bot)
        .count();
    channel
        .to_channel(&status.http)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .edit(&status.http, |c| {
            c.name(format!("Server Size: {}", server_size))
        })
        .await?;
    Ok(())
}

pub async fn subreddit_info(ctx: &Context, matches: Matches<'_, '_>, msg: &Message) {
    for x in matches {
        let text = x.as_str().replace("r/", "");
        let me = Me::login(
            AnonymousAuthenticator::new(),
            "Reddit Nobility Bot u/KingTuxWH".to_string(),
        )
        .await
        .unwrap();
        let subreddit = me.subreddit(text.clone());
        match subreddit.about().await {
            Ok(sub) => {
                let _msg = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.reference_message(msg);
                        m.embed(|e| {
                            let subreddit1 = sub.data;
                            e.url(format!("https://reddit.com{}", subreddit1.url.unwrap()));
                            e.title(subreddit1.display_name.unwrap());
                            e.field(
                                "Members",
                                subreddit1
                                    .subscribers
                                    .unwrap()
                                    .to_formatted_string(&Locale::en),
                                true,
                            );
                            e.field(
                                "Description",
                                subreddit1
                                    .public_description
                                    .unwrap_or_else(|| "Missing Description".to_string()),
                                false,
                            );
                            e.footer(|f| {
                                f.text("Robotic Monarch");
                                f
                            });

                            e
                        });
                        m
                    })
                    .await;
            }
            Err(err) => match err {
                APIError::ExhaustedListing => {}
                APIError::HTTPError(http) => {
                    if http == StatusCode::FORBIDDEN {
                        let _msg = msg
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.reference_message(msg);
                                m.embed(|e| {
                                    e.url(format!("https://reddit.com/r/{}", text.clone()));
                                    e.title(text.clone());
                                    e.field("Description", "Hidden Sub", false);
                                    e.footer(|f| {
                                        f.text("Robotic Monarch");
                                        f
                                    });

                                    e
                                });
                                m
                            })
                            .await;
                    }
                }
                APIError::ReqwestError(_) => {}
                APIError::JSONError(_) => {}
                APIError::ExpiredToken => {}
                APIError::Custom(_) => {}
                APIError::NotFound => {}
            },
        };
    }
}

pub async fn user_info(ctx: &Context, matches: Matches<'_, '_>, msg: &Message) {
    for x in matches {
        let text = x.as_str().replace("u/", "");
        let me = Me::login(
            AnonymousAuthenticator::new(),
            "Reddit Nobility Bot u/KingTuxWH".to_string(),
        )
        .await
        .unwrap();
        let user = me.user(text.clone());
        match user.about().await {
            Ok(user) => {
                let _msg = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.reference_message(msg);
                        m.embed(|e| {
                            let user = user.data;
                            e.url(format!("https://reddit.com/u/{}", user.name));
                            e.field(
                                "Total Karma",
                                user.total_karma
                                    .unwrap_or(0)
                                    .to_formatted_string(&Locale::en),
                                true,
                            );
                            e.field(
                                "Comment Karma",
                                user.comment_karma
                                    .unwrap_or(0)
                                    .to_formatted_string(&Locale::en),
                                true,
                            );
                            e.field(
                                "Link Karma",
                                user.link_karma
                                    .unwrap_or(0)
                                    .to_formatted_string(&Locale::en),
                                true,
                            );
                            e.title(user.name);
                            if let Some(img) = user.snoovatar_img {
                                if !img.is_empty() {
                                    e.image(img);
                                } else if let Some(img) = user.icon_img {
                                    e.image(img);
                                }
                            } else if let Some(img) = user.icon_img {
                                e.image(img);
                            }
                            e.footer(|f| {
                                f.text("Robotic Monarch");
                                f
                            });

                            e
                        });
                        m
                    })
                    .await;
            }
            Err(err) => match err {
                APIError::ExhaustedListing => {}
                APIError::HTTPError(_) => {}
                APIError::ReqwestError(_) => {}
                APIError::JSONError(_) => {}
                APIError::ExpiredToken => {}
                APIError::Custom(_) => {}
                APIError::NotFound => {}
            },
        };
    }
}

pub async fn refresh_reddit_count(status: Context, me: &Me) -> Result<(), BotError> {
    let channel = ChannelId(833707456990281818);

    let subreddit = me.subreddit("RedditNobility".to_string());
    let result = subreddit.about().await;
    let count = match result {
        Ok(ok) => ok.data.subscribers.unwrap().to_string(),
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
                APIError::NotFound => {}
            }
            "Error".to_string()
        }
    };

    channel
        .to_channel(&status.http)
        .await?
        .guild()
        .unwrap()
        .edit(&status.http, |c| {
            c.name(format!("Reddit Subscribers: {}", count))
        })
        .await?;
    Ok(())
}


pub trait DurationFormat {
    fn format(&self) -> String;
}

impl DurationFormat for Duration {
    fn format(&self) -> String {
        let days = self.num_days();
        let hours = self.num_hours() - (days * 24);
        let minutes = self.num_minutes() - (self.num_hours() * 60);
        let seconds = self.num_seconds() - (self.num_minutes() * 60);
        if days > 0 {
            return format!(
                "{} days {} hours {} minutes {} seconds",
                days, hours, minutes, seconds
            );
        } else if hours > 0 {
            return format!("{} hours {} minutes {} seconds", hours, minutes, seconds);
        } else if minutes > 0 {
            return format!(" {} minutes {} seconds", minutes, seconds);
        }
        return format!("{} seconds", seconds);
    }
}
