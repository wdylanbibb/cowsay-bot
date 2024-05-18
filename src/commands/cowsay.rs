use crate::{
    commands::{Context, Error},
    utils,
};

macros::cmd_enum! {
    #[derive(poise::ChoiceParameter)]
    enum Cows("cowsay -l | tail -n +2 | sed 's/ /\\n/g' | head -25") {
        "echo \"$1\" | sed 's/-/ /g' | awk '{for(i=1;i<=NF;i++){ $i=toupper(substr($i,1,1)) substr($i,2) }}1' | sed 's/ //g'"
    }
}

#[poise::command(slash_command)]
pub async fn cowsay(
    ctx: Context<'_>,
    #[description = "What you want the cow to say"] msg: String,
    #[description = "What cow you want to use"] cow: Option<Cows>,
) -> Result<(), Error> {
    let s = utils::cowsay::cowsay(&msg, Some(&cow.unwrap_or(Cows::Default).name()))?;
    ctx.say(format!("```{s}```")).await?;
    Ok(())
}
