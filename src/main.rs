mod commands;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
};

use serenity::{
    async_trait,
    client::bridge::gateway::{ShardManager, GatewayIntents},
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    model::{
        event::VoiceServerUpdateEvent, 
        gateway::Ready,
    },
    prelude::*,
};

use lavalink_rs::{
    LavalinkClient,
};

use tracing::{error, info};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};

use commands::{
    math::*,
    meta::*,
    urls::*,
    player::*,
};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn voice_server_update(&self, ctx: Context, voice: VoiceServerUpdateEvent) {
        if let Some(guild_id) = voice.guild_id {
            let data = ctx.data.read().await;
            let voice_server_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut voice_server = voice_server_lock.write().await;
            voice_server.insert(guild_id);
        }
    }

}

#[group]
#[commands(ping, quit, hello)]
struct General;

#[group]
#[commands(multiply, add, compute)]
struct Math;

#[group]
#[commands(set, get, whereis, cd, mkdir, rmdir, rm, ls, save)]
struct UrlSet;

#[group]
#[commands(join, leave, play, stop)]
struct Player;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");


    let token = env::var("IOLAA_DISCORD_TOKEN")
                    .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            println!("Info: {:?}", info);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };


    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                   .owners(owners)
                   .prefix("&"))
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP)
        .group(&PLAYER_GROUP)
        .group(&URLSET_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        // .intents(GatewayIntents::all())
        .intents(   GatewayIntents::DIRECT_MESSAGES
                  | GatewayIntents::GUILDS 
                  | GatewayIntents::GUILD_MESSAGES
                  | GatewayIntents::GUILD_EMOJIS
                  | GatewayIntents::GUILD_PRESENCES
                  | GatewayIntents::GUILD_VOICE_STATES
                  | GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .await
        .expect("Err creating client");
    
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());

        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<VoiceGuildUpdate>(Arc::new(RwLock::new(HashSet::new())));

        let mut lava_client = LavalinkClient::new(bot_id);

        lava_client.set_host("127.0.0.1");

        let lava = lava_client.initialize(LavalinkHandler).await?;
        data.insert::<Lavalink>(lava);

    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
