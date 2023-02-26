use std::str::FromStr;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{ChannelId, Message},
    prelude::Context,
};
use tracing::error;

#[command]
#[num_args(1)]
#[required_permissions(MANAGE_CHANNELS)]
pub async fn set_channel(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.current() {
        Some(arg) => match ChannelId::from_str(arg) {
            Ok(channel) => match channel.say(&ctx.http, "test").await {
                Ok(message) => {
                    if let Err(why) = message.delete(ctx).await {
                        error!("Error deleting message from channel: {:?}", why);
                    }
                }
                Err(e) => {
                    error!("Error sending message in channel: {:?}", e);
                    msg.reply(
                        &ctx.http,
                        format!("Error sending message in {}! Error message: {}", arg, e),
                    )
                    .await?;
                }
            },
            Err(e) => {
                error!("Error sending message in channel: {:?}", e);
                msg.reply(
                    &ctx.http,
                    format!("Error sending message in {}! Error message: {}", arg, e),
                )
                .await?;
            }
        },
        None => (),
    }
    Ok(())
}
