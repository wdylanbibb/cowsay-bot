use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
};
use tracing::error;

use crate::utils;

#[command]
pub async fn fortune(ctx: &Context, msg: &Message) -> CommandResult {
    match utils::cowsay::cowsay() {
        Ok(v) => {
            msg.reply(&ctx.http, format!("```{}```", v)).await?;
        }
        Err(e) => {
            error!("Error executing commands: {:?}", e);
            msg.reply(&ctx.http, "Something went wrong executing cowsay!")
                .await?;
        }
    }
    Ok(())
}
