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
mod moderator;
mod event;

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
use rand::seq::SliceRandom;

// 0.7.2
pub struct DataHolder {}

impl DataHolder {
    fn new() -> DataHolder {
        DataHolder {}
    }
}

pub struct Bot {}

impl Bot {
    fn new() -> Bot {
        Bot {}
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
        status.set_activity(Activity::playing("A coup!")).await
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_own(ctx.cache).await { return; }
        let x = ctx.data.read().await;

        if msg.content.clone().to_lowercase().contains("fuck zorthan") {
            let msg = msg.channel_id.send_message(&ctx.http, |m| {
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
            }).await;
        }
    }
    async fn guild_member_addition(&self, status: Context, guild: GuildId, member: Member) {
        println!("Test");
        let channel = ChannelId(830415533673414696);
        let file = lines_from_file(Path::new("resources").join("welcome-jokes"));

        let option: &String = file.choose(&mut rand::thread_rng()).unwrap();
        let msg = channel.send_message(&status.http, |m| {
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
        }).await;
    }
}

#[group]
#[commands(about)]
struct General;


#[help]
#[individual_command_tip =
"Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
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
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::{futures::future::BoxFuture, FutureExt};
use serenity::model::gateway::Activity;
use serenity::utils::MessageBuilder;
use serenity::builder::CreateEmbed;
use serenity::http::routing::RouteInfo::CreateMessage;
use serenity::http::AttachmentType;
use std::path::Path;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::guild::Member;
use serenity::cache::FromStrAndCache;
use std::io::{BufReader, BufRead};
use std::fs::File;
use serenity::client::bridge::gateway::GatewayIntents;

fn _dispatch_error_no_macro<'fut>(ctx: &'fut mut Context, msg: &'fut Message, error: DispatchError) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        };
    }.boxed()
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = std::env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );


    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .prefix("!")
            .delimiters(vec![", ", ","]))
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP).group(&moderator::MOD_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework).intents(GatewayIntents::all())
        .await
        .expect("Err creating client");


    {
        let mut data = client.data.write().await;
        data.insert::<DataHolder>(Bot::new());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await?;

    Ok(())
}


fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

