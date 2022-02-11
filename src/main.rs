#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Sub;
use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Duration, Local};
use craftping::sync::ping;
use craftping::{Error, Response};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::{self};
use diesel::MysqlConnection;

use rand::seq::SliceRandom;
use regex::Regex;
use rraw::auth::PasswordAuthenticator;
use rraw::me::Me;

use serenity::client::bridge::gateway::GatewayIntents;
use serenity::model::gateway::Activity;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::prelude::User;
use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{command, group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
};
use std::net::TcpStream;

use serenity::http::CacheHttp;
use tokio::time::sleep;

use crate::site::site_client::SiteClient;
use crate::site::Authenticator;
// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use crate::utils::{refresh_reddit_count, refresh_server_count, subreddit_info, user_info};

mod actions;
mod admin;
mod boterror;
mod dnd;
mod event;
mod models;
mod moderator;
mod register;
mod schema;
pub mod site;
mod utils;

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
    pub site_client: SiteClient,
}

impl Bot {
    fn new(site_client: SiteClient) -> Bot {
        Bot {
            start_time: Local::now(),
            site_client,
        }
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
        status.set_activity(Activity::playing("A coup!")).await;

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
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    let error_log = ChannelId(834210453265317900);

    let error_message = match error {
        DispatchError::LackingPermissions(permission) => {
            if msg.author.id.eq(&UserId(487471903779586070)) {
                let x = ctx
                    .http()
                    .get_emoji(msg.guild_id.unwrap().0, 941471475804807240)
                    .await
                    .unwrap();

                let string = format!("Listen. Tux Might like you. However, this does not give you special treatment with his little project here: {}", x);
                msg.reply(&ctx.http, string).await.unwrap();
            }
            format!("User {} is missing permission: {}", msg.author.name, permission)
        }
        DispatchError::OnlyForOwners => {
            format!("User {} has executed a command that is for Owners", msg.author.name)
        }
        DispatchError::LackingRole => {
            format!("User {} has executed a command that is for a specific role.", msg.author.name)
        }
        value => {
            format!("User {} has executed incorrectly. In Some way: {:?}", msg.author.name, value)
        }
    };
    error_log
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Dispatch Error"));
                e.description(error_message);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await.unwrap();
}

#[hook]
async fn after(
    context: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    if let Err(error) = command_result {
        msg.reply(&context.http, "Unable to Execute that command at this time")
            .await;

        let error_log = ChannelId(834210453265317900);

        error_log
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title(format!("An Error has occurred on Command {}", command_name));
                    e.description(error.to_string());
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

embed_migrations!();
#[tokio::main]
async fn main() {
    if let Err(error) = dotenv::dotenv() {
        println!("Unable to load dotenv {}", error);
        return;
    }

    let file_appender = tracing_appender::rolling::hourly("log/discord", "discord.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

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
    let application_id = std::env::var("APPLICATION_ID")
        .expect("Expected a Application ID in the environment")
        .parse::<u64>()
        .expect("Application ID must be a u64");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .prefix(&*std::env::var("COMMAND").unwrap_or("!".to_string()))
                .delimiters(vec![", ", ","])
        })
        .unrecognised_command(unknown_command)
        .on_dispatch_error(dispatch_error)
        .after(after)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&moderator::MOD_GROUP)
        .group(&dnd::DND_GROUP)
        .group(&admin::ADMIN_GROUP)
        .group(&register::REGISTER_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Err creating client");

    {
        let authenticator = Authenticator {
            token: None,
            username: std::env::var("SITE_USERNAME").unwrap(),
            password: std::env::var("SITE_PASSWORD").unwrap(),
        };

        let mut data = client.data.write().await;
        data.insert::<DataHolder>(Bot::new(SiteClient::new(authenticator).await));
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
    let _msg = msg
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
    let _msg = msg
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
    let _msg = msg
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
    let hostname = "play.redditnobility.org";
    let port = 25565;
    let mut stream = TcpStream::connect((hostname, port)).unwrap();
    let pong: Result<Response, Error> = ping(&mut stream, hostname, port);
    let _msg = msg.channel_id.send_message(&ctx.http, |m| {
        m.reference_message(msg);
        m.embed(|e| {
            e.title("RedditNobility Minecraft Vanilla Server Info");
            e.field("IP", hostname, true);
            e.field("Online", pong.is_ok(), true);
            if pong.is_ok() {
                let response = pong.unwrap();
                e.field("Minecraft Version", response.version.replace("TuxServer ", ""), true);
                e.field("Online Players", response.online_players, true);
            }
            e.field("Description", "An open vanilla survival game. Open to all nobility and even friend(Just message KingTuxWH or Darth_Dan).", false);
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
    let hostname = "46.105.77.36";
    let port = 25579;
    let mut stream = TcpStream::connect((hostname, port)).unwrap();
    let pong: Result<Response, Error> = ping(&mut stream, hostname, port);
    let _msg = msg
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
