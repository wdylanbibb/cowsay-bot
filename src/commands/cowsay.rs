use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serenity::{
    builder::CreateApplicationCommand,
    framework::standard::{macros::command, CommandResult},
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        Message,
    },
    prelude::Context,
};
use tracing::error;

use crate::utils;

#[command]
pub async fn cowsay(ctx: &Context, msg: &Message) -> CommandResult {
    match utils::fortune::fortune() {
        Ok(v) => match utils::cowsay::cowsay(
            &v,
            Some(
                utils::cowsay::get_cows()
                    .choose::<StdRng>(&mut SeedableRng::from_entropy())
                    .expect("Cows list empty"),
            ),
        ) {
            Ok(v) => {
                msg.reply(&ctx.http, format!("```{}```", v)).await?;
            }
            Err(e) => {
                error!("Error executing cowsay: {:?}", e);
                msg.reply(
                    &ctx.http,
                    format!(
                        "Something went wrong executing cowsay! Error message: {}",
                        e
                    ),
                )
                .await?;
            }
        },
        Err(e) => {
            error!("Error executing fortune: {:?}", e);
            msg.reply(
                &ctx.http,
                format!(
                    "Something went wrong executing fortune! Error message: {}",
                    e
                ),
            )
            .await?;
        }
    }
    Ok(())
}

pub fn run(options: &[CommandDataOption]) -> String {
    let mut message_data = None;
    let mut cow_data = None;

    for data in options {
        match data.name.as_str() {
            "message" => message_data = Some(data),
            "cow" => cow_data = Some(data),
            _ => (),
        }
    }

    let message = match message_data {
        Some(data) => {
            let resolved = data.resolved.as_ref();
            match resolved {
                Some(value) => {
                    if let CommandDataOptionValue::String(s) = value {
                        s.to_owned()
                    } else {
                        return "Please enter a valid message string!".to_string();
                    }
                }
                None => return "Please enter a valid message string!".to_string(),
            }
        }
        None => match utils::fortune::fortune() {
            Ok(s) => s,
            Err(e) => {
                error!("Error executing fortune: {:?}", e);
                return format!(
                    "Something went wrong executing fortune! Error message: {}",
                    e
                )
                .to_string();
            }
        },
    };

    let cow = match cow_data {
        Some(data) => {
            let resolved = data.resolved.as_ref();
            match resolved {
                Some(value) => {
                    if let CommandDataOptionValue::String(s) = value {
                        s.to_owned()
                    } else {
                        return "Please enter a valid cow string!".to_string();
                    }
                }
                None => return "Please enter a valid cow string!".to_string(),
            }
        }
        None => "default".to_string(),
    };

    let cowsay = utils::cowsay::cowsay(&message, Some(&cow));

    match cowsay {
        Ok(s) => format!("```{}```", s),
        Err(e) => {
            error!("Error executing cowsay: {:?}", e);
            return format!(
                "Something went wrong executing cowsay! Error message: {}",
                e
            )
            .to_string();
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("cowsay")
        .description("Make a cow say something")
        .create_option(|option| {
            option
                .name("message")
                .description("What the cow should say")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("cow")
                .description("what type of cow")
                .kind(CommandOptionType::String);
            let mut cows = utils::cowsay::get_cows();
            cows.truncate(25);
            for cow in cows {
                let name = &cow.split('/').last().unwrap().split('.').next().unwrap();
                option.add_string_choice(name, name);
            }
            option
        })
}
