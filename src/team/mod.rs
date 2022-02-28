use serde::{Serialize, Deserialize};
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamObject {
    pub role: u64,
    pub name: String,
    pub id: String,
}


#[group]
#[commands(team, mod_team)]
struct Team;

#[command("team")]
async fn team(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

#[command("mteam")]
#[required_permissions("ADMINISTRATOR")]
async fn mod_team(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}
