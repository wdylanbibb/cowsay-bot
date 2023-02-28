use bonsaidb::core::schema::Collection;
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
