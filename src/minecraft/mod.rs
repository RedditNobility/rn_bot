use craftping::sync::ping;
use craftping::{Error, Response};
use std::net::TcpStream;
// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};



#[group]
#[commands(minecraft)]
struct Minecraft;


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
