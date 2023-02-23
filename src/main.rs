use std::{
    collections::HashSet,
    env, io,
    process::{Command, Stdio},
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            macros::{command, group},
            CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::prelude::{Activity, ChannelId, GuildId, Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents, Mutex, TypeMapKey},
    Client,
};

use chrono::{offset::Utc, Timelike};
use tracing::{error, info};

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
                        message_cowsay(Arc::clone(&ctx1)).await;
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

fn get_fortune() -> io::Result<String> {
    let result = Command::new("fortune").output()?.stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                get_fortune()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn get_cowsay() -> io::Result<String> {
    let cowdir = Command::new("ls")
        .arg("/usr/share/cows")
        .stdout(Stdio::piped())
        .spawn()?;
    let random_cow = Command::new("shuf")
        .arg("-n1")
        .stdin(cowdir.stdout.unwrap())
        .output()?
        .stdout;
    let random_cow = match str::from_utf8(&random_cow) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let fortune = Command::new("fortune").stdout(Stdio::piped()).spawn()?;
    let result = Command::new("cowsay")
        .args(["-f", random_cow.trim()])
        .stdin(fortune.stdout.unwrap())
        .output()?
        .stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                get_cowsay()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

async fn message_cowsay(ctx: Arc<Context>) {
    match get_cowsay() {
        Ok(cowsay) => {
            let message = ChannelId(1078089397972500481)
                .say(&ctx, format!("```{}```", cowsay))
                .await;
            if let Err(why) = message {
                error!("Error sending message: {:?}", why);
            }
        }
        Err(e) => error!("Error executing commands: {:?}", e),
    }
}

async fn set_status_to_fortune(ctx: Arc<Context>) {
    match get_fortune() {
        Ok(v) => ctx.set_activity(Activity::playing(v)).await,
        Err(e) => error!("Error executing commands: {:?}", e),
    }
}

#[command]
async fn fortune(ctx: &Context, msg: &Message) -> CommandResult {
    match get_cowsay() {
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

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(fortune)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to find .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP);

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
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
