//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```

use diesel::prelude::*;
use diesel::r2d2::{self};
use rand::seq::SliceRandom;
use serenity::prelude::*;
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
use std::sync::{Arc, Mutex};
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    thread,
};

mod actions;
mod admin;
mod boterror;
mod event;
mod models;
mod moderator;
mod register;
mod schema;
mod utils;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

type DbPoolType = Arc<r2d2::Pool<ConnectionManager<MysqlConnection>>>;

pub struct DbPool(DbPoolType);

impl TypeMapKey for DbPool {
    type Value = DbPoolType;
}

// 0.7.2
pub struct DataHolder {}

impl DataHolder {
    fn new() -> DataHolder {
        DataHolder {}
    }
}

pub struct Bot {
    pub start_time: DateTime<Local>,
    pub reddit: Option<RedditClient>,
}

impl Bot {
    fn new(client: Option<RedditClient>) -> Bot {
        Bot {
            start_time: Local::now(),
            reddit: client,
        }
    }
    fn set_client(&mut self, client: RedditClient) {
        self.reddit = Some(client)
    }
    fn test(&mut self) {
        println!("test");
    }
}

impl TypeMapKey for DataHolder {
    type Value = Bot;
}

impl Bot {}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, status: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        status.online().await;
        status.set_activity(Activity::playing("A coup!")).await;
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id.to_string().eq("829825560930156615") {
            return;
        }

        if msg.author.id.to_string().eq("411465364103495680") {
            if msg.content.contains("*") {
                msg.react(&ctx.http, '🙄').await;
            }
        }
        if msg.is_own(ctx.cache).await {
            return;
        }
        let x = ctx.data.read().await;

        if msg.content.clone().to_lowercase().contains("fuck zorthan") {
            let msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("No Fucking Zorthan");
                        e.description("Zorthan does not deserve anything. Let him go!");
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
    async fn guild_member_addition(&self, status: Context, guild: GuildId, member: Member) {
        println!("Test");
        let channel = ChannelId(830415533673414696);
        let file = lines_from_file(Path::new("resources").join("welcome-jokes"));

        let option: &String = file.choose(&mut rand::thread_rng()).unwrap();
        let msg = channel
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
        refresh_server_count(&status).await;
    }
    async fn guild_member_removal(
        &self,
        status: Context,
        guild: GuildId,
        _new: User,
        _old_if_available: Option<Member>,
    ) {
        let channel = ChannelId(840919470695645184);
        let file = lines_from_file(Path::new("resources").join("exit-messages"));

        let option: &String = file.choose(&mut rand::thread_rng()).unwrap();
        let msg = channel
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
        refresh_server_count(&status).await;
    }
}

#[group]
#[commands(about, serverinfo, minecraft, botinfo)]
struct General;

#[help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use crate::utils::{refresh_reddit_count, refresh_server_count};
use chrono::{DateTime, Duration, Local};
use craftping::sync::ping;
use craftping::{Error, Response};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::MysqlConnection;
use new_rawr::auth::PasswordAuthenticator;
use new_rawr::client::RedditClient;
use serenity::builder::CreateEmbed;
use serenity::cache::FromStrAndCache;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::http::routing::RouteInfo::CreateMessage;
use serenity::http::AttachmentType;
use serenity::model::channel::{Reaction, ReactionType};
use serenity::model::gateway::Activity;
use serenity::model::guild::Member;
use serenity::model::guild::Target::Emoji;
use serenity::model::id::{ChannelId, EmojiId, GuildId};
use serenity::model::prelude::User;
use serenity::model::webhook::Webhook;
use serenity::utils::MessageBuilder;
use serenity::{futures::future::BoxFuture, FutureExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Sub;
use std::path::Path;
use std::thread::sleep;
use std::time::Instant;

fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        &format!("Try this again in {} seconds.", info.as_secs()),
                    )
                    .await;
            }
        };
    }
    .boxed()
}
embed_migrations!();
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<MysqlConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let connection = pool.get().unwrap();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout());
    let final_pool = Arc::new(pool);
    // Configure the client with your Discord bot token in the environment.
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .prefix(&*std::env::var("COMMAND").unwrap_or("!".to_string()))
                .delimiters(vec![", ", ","])
        })
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&moderator::MOD_GROUP)
        .group(&admin::ADMIN_GROUP)
        .group(&register::REGISTER_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DataHolder>(Bot::new(None));
        data.insert::<DbPool>(final_pool.clone());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "This is a small test-bot! : )")
        .await?;

    Ok(())
}

#[command]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let string = msg
        .guild_id
        .unwrap()
        .members(&ctx.http, None, None)
        .await
        .unwrap()
        .len()
        .to_string();
    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title("RedditNobility Server Info");
                e.field("Members", string, true);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await;

    Ok(())
}

#[command]
async fn botinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let x: &mut Bot = data.get_mut::<DataHolder>().unwrap();
    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title("Robotic Monarch Bot Info");
                e.field("Uptime", Local::now().sub(x.start_time).format(), true);
                e.field("Host", hostname::get().unwrap().to_str().unwrap(), false);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await;

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

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[command]
#[sub_commands(vanilla, modded)]
async fn minecraft(ctx: &Context, msg: &Message) -> CommandResult {
    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title("RedditNobility Minecraft Server Info");
                e.field("Available Options", "vanilla or modded", true);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await;

    Ok(())
}

#[command]
#[aliases("van")]
#[description("Gets information about the Vanilla MC Server")]
async fn vanilla(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pong: Result<Response, Error> = ping("play.redditnobility.org", 25565);
    let msg = msg.channel_id.send_message(&ctx.http, |m| {
        m.reference_message(msg);
        m.embed(|e| {
            e.title("RedditNobility Minecraft Vanilla Server Info");
            e.field("IP", "play.redditnobility.org", true);
            e.field("Online", pong.is_ok(), true);
            if pong.is_ok() {
                let response = pong.unwrap();
                e.field("Minecraft Version", response.version.replace("TuxServer ", ""), true);
                e.field("Online Players", response.online_players, true);
            }
            e.field("Description", "An open vanilla survival game. Open to all nobility and even friend(Just message KingTuxWH or Darth_Dan). Become whitelisted at https://forms.gle/1sxSqtGzpVnKt4jHA", false);
            e.footer(|f| {
                f.text("Robotic Monarch");
                f
            });

            e
        });
        m
    }).await;
    Ok(())
}

#[command]
#[aliases("mod")]
#[description("Gets information about the modded MC Server")]
async fn modded(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pong: Result<Response, Error> = ping("46.105.77.36", 25579);

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title("RedditNobility Minecraft Modded Server Info");
                e.field("IP", "play.mod.redditnobility.org", true);
                e.field("Online", pong.is_ok(), true);
                if pong.is_ok() {
                    let response = pong.unwrap();
                    e.field("Minecraft Version", response.version, true);
                    e.field("Online Players", response.online_players, true);
                }
                e.field("Description", "A modded Minecraft server.", false);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await;
    Ok(())
}
