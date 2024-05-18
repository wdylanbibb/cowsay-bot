use serenity::all::Channel;

use crate::{
    commands::{Context, Error},
    db,
};

#[poise::command(prefix_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn set_channel(ctx: Context<'_>, channel: Channel) -> Result<(), Error> {
    let channel_id = channel.id();
    let message = channel_id
        .say(&ctx.http(), "Testing message permissions in channel...")
        .await?;
    message.delete(&ctx.http()).await?;

    db::set_channel(ctx, channel).await?;

    Ok(())
    // if let Some(arg) = args.current() {
    //     match ChannelId::from_str(arg) {
    //         Ok(channel) => match channel.say(&ctx.http, "test").await {
    //             Ok(message) => {
    //                 message.delete(&ctx.http).await?;
    //
    //                 // TODO: clean this up, move it out to `db` crate
    //
    //                 if let Some(guild) = msg.guild_id {
    //                     let db = db::open().expect("error accessing database");
    //
    //                     let fortune_channel =
    //                         match db::FortuneChannel::get::<_, u64>(guild.into(), &db)
    //                             .expect("error accessing database")
    //                         {
    //                             Some(mut fortune_channel) => {
    //                                 fortune_channel.contents.channel = channel;
    //                                 fortune_channel.update(&db).expect("error updating channel");
    //                                 fortune_channel
    //                             }
    //                             None => db::FortuneChannel::new(guild, channel)
    //                                 .push_into(&db)
    //                                 .expect("error pushing channel into database"),
    //                         };
    //
    //                     if fortune_channel.eq(&db::FortuneChannel::get::<_, u64>(guild.into(), &db)
    //                         .expect("unable to retrieve from database")
    //                         .expect("document not found"))
    //                     {
    //                         msg.reply(
    //                             &ctx.http,
    //                             format!(
    //                                 "{arg} successfully established as the Hourly Fortune channel!"
    //                             ),
    //                         )
    //                         .await?;
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 error!("Error sending message in channel: {:?}", e);
    //                 msg.reply(
    //                     &ctx.http,
    //                     format!("Error sending message in {}! Error message: {}", arg, e),
    //                 )
    //                 .await?;
    //             }
    //         },
    //         Err(e) => {
    //             error!("Error sending message in channel: {:?}", e);
    //             msg.reply(
    //                 &ctx.http,
    //                 format!("Error sending message in {}! Error message: {}", arg, e),
    //             )
    //             .await?;
    //         }
    //     }
    // }
    // Ok(())
}

#[poise::command(prefix_command)]
pub async fn remove_channel(ctx: Context<'_>) -> Result<(), Error> {
    db::remove_channel(ctx).await
    // let guild_id = match msg.guild_id {
    //     Some(guild_id) => guild_id,
    //     None => {
    //         msg.reply(&ctx.http, "Server ID not accessable from message!")
    //             .await?;
    //         return Ok(());
    //     }
    // };
    //
    // let db = db::open().expect("error accessing database");
    //
    // match db::FortuneChannel::get(guild_id.0, &db).expect("error accessing database") {
    //     Some(channel_doc) => match channel_doc.delete(&db) {
    //         Ok(()) => {
    //             msg.reply(&ctx.http, "Channel successfully removed!")
    //                 .await?;
    //         }
    //         Err(e) => {
    //             error!("Error deleting channel from database: {e:?}");
    //             msg.reply(
    //                 &ctx.http,
    //                 format!("Error removing channel! Error message: {e}"),
    //             )
    //             .await?;
    //         }
    //     },
    //     None => {
    //         msg.reply(
    //             &ctx.http,
    //             "No channel set! Set a channel for hourly fortunes using the `set_channel` command!",
    //         ).await?;
    //     }
    // }
}
