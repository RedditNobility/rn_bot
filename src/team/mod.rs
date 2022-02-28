use std::collections::HashMap;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
use serenity::model::guild::Role;
use serenity::model::id::RoleId;
use crate::{Bot, DataHolder};
use crate::bot_error::BotError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamObject {
    pub role: u64,
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamCreation {
    pub role: Option<RoleId>,
    pub name: Option<String>,
    pub id: Option<String>,
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
#[sub_commands(mteam_add, mteam_remove)]
async fn mod_team(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(referenced) = &msg.referenced_message {
        let mut data = ctx.data.write().await;
        let bot: &mut Bot = data.get_mut::<DataHolder>().unwrap();
        let map = &mut bot.active_data;

        let option = map.remove(&referenced.id.0);
        if option.is_none() {
            msg.reply(&ctx.http, "Please Reference the Creation Message").await?;
        }
        let message_data = option.unwrap();
        //delete Old Value
        referenced.delete(&ctx.http).await?;
        let mut message_data: TeamCreation = serde_json::from_str(&message_data)?;
        if message_data.role.is_none() {
            let role = msg.mention_roles.first();
            if let Some(value) = role {
                message_data.role = Some(*value);
            }
        } else if message_data.name.is_none() {
            if let Some(name) = args.current() {
                message_data.name = Some(name.to_string());
            }
        } else if message_data.id.is_none() {
            if let Some(id) = args.current() {
                message_data.id = Some(id.to_string());
            }
        }
        teams_message(ctx, msg, message_data, map).await?;
    } else {
        msg.reply(&ctx.http, "Please Reference the Creation Message").await?;
    }

    Ok(())
}

#[command("add")]
async fn mteam_add(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let bot: &mut Bot = data.get_mut::<DataHolder>().unwrap();
    let map = &mut bot.active_data;


    if let Some(referenced) = &msg.referenced_message {
        let option = map.remove(&referenced.id.0);

        let message_data = option.unwrap();
        let message_data: TeamCreation = serde_json::from_str(&message_data)?;
        if message_data.id.is_none() || message_data.role.is_none() || message_data.name.is_none() {
            //TODO resend creation message
        }
        let team = TeamObject {
            role: message_data.role.unwrap().0,
            name: message_data.name.unwrap(),
            id: message_data.id.unwrap(),
        };
        let teams_path = Path::new("teams.json");
        let mut teams = get_teams()?;
        teams.push(team);

        let result = serde_json::to_string_pretty(&teams).unwrap();
        let mut file = OpenOptions::new().create(true).write(true).open(&teams_path).unwrap();
        file.write_all(result.as_bytes()).unwrap();
    } else {
        teams_message(ctx, msg, TeamCreation {
            role: None,
            name: None,
            id: None,
        }, map).await?;
    }
    Ok(())
}

pub async fn teams_message(ctx: &Context, msg: &Message, creation: TeamCreation, map: &mut HashMap<u64, String>) -> Result<(), BotError> {
    let header = if creation.role.is_none() {
        "Please Respond !mteam @Role to set role"
    } else if creation.name.is_none() {
        "Please Respond !mteam name to set name"
    } else if creation.id.is_none() {
        "Please Respond !mteam id to set name"
    } else {
        "Please Respond !mteam add to confirm"
    };
    let result = serde_json::to_string(&creation)?;
    let message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg);
            m.embed(|e| {
                e.title(header);
                e.field("Role", creation.role.unwrap_or( RoleId(0)), false);
                e.field("Name", creation.name.unwrap_or_else(|| "null".to_string()), false);
                e.field("id", creation.id.unwrap_or_else(|| "null".to_string()), false);
                e.footer(|f| {
                    f.text("Robotic Monarch");
                    f
                });

                e
            });
            m
        })
        .await?;
    map.insert(message.id.0, result);
    Ok(())
}

pub fn get_teams() -> Result<Vec<TeamObject>, BotError> {
    let teams_path = Path::new("teams.json");

    let teams: Vec<TeamObject> = if !teams_path.exists() {
        let value = vec![];
        let result = serde_json::to_string_pretty(&value).unwrap();
        let mut file = OpenOptions::new().create(true).write(true).open(&teams_path).unwrap();
        file.write_all(result.as_bytes()).unwrap();
        value
    } else {
        let value = read_to_string(&teams_path).unwrap();
        toml::from_str(&value).unwrap()
    };
    Ok(teams)
}

#[command("remove")]
async fn mteam_remove(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main sub function!").await?;

    Ok(())
}
