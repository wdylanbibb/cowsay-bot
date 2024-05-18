use bonsaidb::{
    core::schema::{Collection, SerializedCollection},
    local::{
        config::{Builder, StorageConfiguration},
        Database, Error,
    },
};
use serde::{Deserialize, Serialize};
use serenity::{
    all::Channel,
    model::prelude::{ChannelId, GuildId},
};

use crate::commands::Context;

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "fortune-channel", primary_key = u64, natural_id = Some(self.guild.into()))]
pub struct FortuneChannel {
    guild: GuildId,
    pub channel: ChannelId,
}

impl FortuneChannel {
    pub fn new(guild: GuildId, channel: ChannelId) -> Self {
        FortuneChannel { guild, channel }
    }
}

pub fn open() -> Result<Database, Error> {
    Database::open::<FortuneChannel>(StorageConfiguration::new("fortune-channels.bonsaidb"))
}

#[derive(Debug)]
enum GuildAccessError {
    SetChannel,
    RemoveChannel,
}

impl std::fmt::Display for GuildAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetChannel => write!(f, "No server associated with channel"),
            Self::RemoveChannel => write!(f, "Server ID not accessable from message"),
        }
    }
}

impl std::error::Error for GuildAccessError {}

pub async fn set_channel(ctx: Context<'_>, channel: Channel) -> Result<(), crate::commands::Error> {
    let channel_id = channel.id();
    let guild_id = channel
        .guild()
        .ok_or(GuildAccessError::SetChannel)?
        .guild_id;

    let db = open()?;

    let fortune_channel = match FortuneChannel::get::<_, u64>(&guild_id.into(), &db)? {
        Some(mut fortune_channel) => {
            fortune_channel.contents.channel = channel_id;
            fortune_channel.update(&db)?;
            fortune_channel
        }
        None => FortuneChannel::new(guild_id, channel_id).push_into(&db)?,
    };

    if fortune_channel.eq(&FortuneChannel::get::<_, u64>(&guild_id.into(), &db)?.unwrap()) {
        ctx.reply("Channel successfully establisted as the Hourly Fortune channel!")
            .await?;
    }

    Ok(())
}

pub async fn remove_channel(ctx: Context<'_>) -> Result<(), crate::commands::Error> {
    let guild_id = ctx.guild_id().ok_or(GuildAccessError::RemoveChannel)?;

    let db = open()?;

    match FortuneChannel::get(&guild_id.get(), &db)? {
        Some(channel_doc) => {
            channel_doc.delete(&db)?;
            ctx.reply("Channel successfully removed!").await?;
        }
        None => {
            ctx.reply("No channel set! Set a channel for hourly fortunes using the `set_channel` command!").await?;
        }
    }

    Ok(())
}
