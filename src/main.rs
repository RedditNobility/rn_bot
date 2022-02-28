#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::collections::HashSet;
use std::ops::Sub;
use std::sync::Arc;

use chrono::{DateTime, Local};


use diesel::r2d2::ConnectionManager;
use diesel::r2d2::{self};
use diesel::MysqlConnection;

use serenity::client::bridge::gateway::GatewayIntents;

use serenity::model::id::ChannelId;

use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    model::{channel::Message, id::UserId},
    prelude::*,
};

use serenity::http::CacheHttp;

use crate::site::site_client::SiteClient;
use crate::site::Authenticator;
use crate::utils::DurationFormat;
// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.

mod actions;
mod admin;
mod bot_error;
mod dnd;
mod event;
mod handler;
mod models;
mod moderator;
mod register;
mod schema;
pub mod site;
mod utils;
mod minecraft;
pub mod channels;

type DbPoolType = Arc<r2d2::Pool<ConnectionManager<MysqlConnection>>>;

pub struct DbPool(DbPoolType);

impl TypeMapKey for DbPool {
    type Value = DbPoolType;
}

// 0.7.2
pub struct DataHolder {}

impl DataHolder {}

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

#[group]
#[commands(about, serverinfo, botinfo)]
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
            format!(
                "User {} is missing permission: {}",
                msg.author.name, permission
            )
        }
        DispatchError::OnlyForOwners => {
            format!(
                "User {} has executed a command that is for Owners",
                msg.author.name
            )
        }
        DispatchError::LackingRole => {
            format!(
                "User {} has executed a command that is for a specific role.",
                msg.author.name
            )
        }
        value => {
            format!(
                "User {} has executed incorrectly. In Some way: {:?}",
                msg.author.name, value
            )
        }
    };
    error_log
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Dispatch Error in command {}", &msg.content));
                e.description(error_message);
                e.url(msg.link());
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await
        .unwrap();
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
            .await
            .unwrap();

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
            .await
            .unwrap();
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
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
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
                .prefix(&*std::env::var("COMMAND").expect("Missing Command Prefix!"))
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
        .group(&minecraft::MINECRAFT_GROUP)
        .group(&register::REGISTER_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(handler::Handler)
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
    let commit_hash = env!("VERGEN_GIT_SHA");
    let branch = env!("VERGEN_GIT_BRANCH");
    let timestamp = env!("VERGEN_BUILD_TIMESTAMP");
    msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title("Bot About Info");
                e.field("Head Developer", "KingTux#0042", false);
                e.field("Branch", branch, true);
                e.field("timestamp", timestamp, true);
                e.description("The Custom Discord Bot for the Reddit Nobility Community");
                e.url(format!("https://github.com/RedditNobility/rn_bot/commit/{}",commit_hash));
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
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

