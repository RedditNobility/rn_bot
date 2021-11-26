use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};

use crate::utils;

#[group]
#[commands(rcount)]
struct Admin;

#[command("refresh_count")]
#[required_permissions("ADMINISTRATOR")]
async fn rcount(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "Refreshed Server Count").await?;
    utils::refresh_server_count(ctx).await;
    Ok(())
}
