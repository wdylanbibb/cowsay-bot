use std::str::FromStr;

use bonsaidb::{
    core::schema::SerializedCollection,
    local::{
        config::{Builder, StorageConfiguration},
        Database,
    },
};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{ChannelId, Message},
    prelude::Context,
};
use tracing::error;

use crate::db;

#[command]
#[num_args(1)]
#[required_permissions(MANAGE_CHANNELS)]
pub async fn set_channel(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.current() {
        Some(arg) => match ChannelId::from_str(arg) {
            Ok(channel) => match channel.say(&ctx.http, "test").await {
                Ok(message) => {
                    message.delete(&ctx.http).await?;

                    // TODO: clean this up, move it out to `db` crate
                    if let Some(guild) = msg.guild_id {
                        let db = Database::open::<db::FortuneChannel>(StorageConfiguration::new(
                            "fortune-channels.bonsaidb",
                        ))
                        .expect("error opening database");

                        let fortune_channel =
                            match db::FortuneChannel::get::<_, u64>(guild.into(), &db)
                                .expect("error accessing database")
                            {
                                Some(mut fortune_channel) => {
                                    fortune_channel.contents.channel = channel;
                                    fortune_channel.update(&db).expect("error updating channel");
                                    fortune_channel
                                }
                                None => db::FortuneChannel::new(guild, channel)
                                    .push_into(&db)
                                    .expect("error pushing channel into database"),
                            };

                        if fortune_channel.eq(&db::FortuneChannel::get::<_, u64>(guild.into(), &db)
                            .expect("unable to retrieve from database")
                            .expect("document not found"))
                        {
                            msg.reply(
                                &ctx.http,
                                format!(
                                    "{arg} successfully established as the Hourly Fortune channel!"
                                ),
                            )
                            .await?;
                        }
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
