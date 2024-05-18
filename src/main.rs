use std::{
    collections::HashSet,
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use bonsaidb::core::schema::SerializedCollection;
use serenity::{
    async_trait,
    gateway::{ActivityData, ShardManager},
    http::Http,
    model::prelude::{ChannelId, GuildId, Ready},
    prelude::{Context, EventHandler, GatewayIntents, TypeMapKey},
    Client,
};

use chrono::{offset::Utc, Timelike};
use tracing::{error, info};

mod commands;
mod utils;

mod db;

use commands::cowsay;

use crate::commands::{set_channel, Error};

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache build successfully!");

        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    if Utc::now().time().minute() == 0 {
                        let db = db::open().expect("error accessing database");

                        for guild_doc in db::FortuneChannel::all(&db)
                            .query()
                            .expect("error accessing channels")
                        {
                            info!("Found guild document {guild_doc:?} in database");
                            let channel = guild_doc.contents.channel;

                            message_cowsay(Arc::clone(&ctx1), channel).await;
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    set_status_to_fortune(Arc::clone(&ctx2)).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

async fn message_cowsay(ctx: Arc<Context>, channel: ChannelId) {
    match utils::cowsay::random_cowsay_fortune() {
        Ok(cowsay) => {
            let message = channel.say(&ctx, format!("```{}```", cowsay)).await;
            if let Err(why) = message {
                error!("Error sending message: {:?}", why);
            }
        }
        Err(e) => error!("Error executing commands: {:?}", e),
    }
}

async fn set_status_to_fortune(ctx: Arc<Context>) {
    match utils::fortune::fortune() {
        // Ok(v) => ctx.set_activity(Activity::playing(v)).await,
        Ok(v) => ctx.set_activity(Some(ActivityData::playing(v))),
        Err(e) => error!("Error executing commands: {:?}", e),
    }
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[poise::command(prefix_command)]
pub async fn register(ctx: crate::commands::Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), bonsaidb::core::Error> {
    dotenv::dotenv().expect("Failed to find .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    let (_owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    info!("{:?}", cowsay::cowsay().parameters[0].type_setter);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                cowsay::cowsay(),
                set_channel::set_channel(),
                set_channel::remove_channel(),
                register(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(crate::commands::Data {})
            })
        })
        .build();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
