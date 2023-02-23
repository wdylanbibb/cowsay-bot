use std::{
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
    model::prelude::{Activity, ChannelId, GuildId, Message, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

use chrono::{offset::Utc, Timelike};

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("$fortune") {
            match cowsay() {
                Ok(cowsay) => {
                    if let Err(why) = msg.reply(&ctx.http, format!("```{}```", cowsay)).await {
                        eprintln!("Error sending message: {:?}", why);
                    }
                }
                Err(e) => eprintln!("Error executing commands: {:?}", e),
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // We use the cache_ready event just in case some cache operation is required in whatever use
    // case you have for this.
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache build successfully!");

        // It's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);

        // We need to check that the loop is not already running when this event triggers,
        // as this event triggers every time the bot enters or leaves a guild, along every time the
        // ready shard triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
        if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                loop {
                    // We clone Context again here, because Arc is owned, so it moves to the new
                    // function.
                    // We check if it is the top of the hour
                    if Utc::now().time().minute() == 0 {
                        message_cowsay(Arc::clone(&ctx1)).await;
                    }
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            // And of course, we can run more than one thread at different timings.
            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    set_status_to_fortune(Arc::clone(&ctx2)).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

fn fortune() -> io::Result<String> {
    let result = Command::new("fortune").output()?.stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                fortune()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn cowsay() -> io::Result<String> {
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
                cowsay()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

async fn message_cowsay(ctx: Arc<Context>) {
    match cowsay() {
        Ok(cowsay) => {
            let message = ChannelId(1078089397972500481)
                .say(&ctx, format!("```{}```", cowsay))
                .await;
            if let Err(why) = message {
                eprintln!("Error sending message: {:?}", why);
            }
        }
        Err(e) => eprintln!("Error executing commands: {:?}", e),
    }
}

async fn set_status_to_fortune(ctx: Arc<Context>) {
    match fortune() {
        Ok(v) => ctx.set_activity(Activity::playing(v)).await,
        Err(e) => eprintln!("Error executing commands: {:?}", e),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to find .env file");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
