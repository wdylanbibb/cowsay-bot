use bonsaidb::{
    core::schema::Collection,
    local::{
        config::{Builder, StorageConfiguration},
        Database, Error,
    },
};
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId};

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "fortune-channel", primary_key = u64, natural_id = |channel: &FortuneChannel| Some(channel.guild.into()))]
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
