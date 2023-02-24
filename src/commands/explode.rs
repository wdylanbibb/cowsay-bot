use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

#[command]
async fn explode(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        &ctx.http,
        "https://tenor.com/view/explode-boom-explosion-gif-17473499",
    )
    .await?;
    Ok(())
}
