/**
 *  Copied from https://gitlab.com/nitsuga5124/lavalink-rs/-/blob/master/examples/basic_queue.rs
 *  (changed some things but it isn't significant)
 */

use std::{
    sync::Arc,
    collections::HashSet,
    time::Duration,
};

use serenity::client::bridge::voice::ClientVoiceManager;

use serenity::{
    async_trait,
    prelude::{TypeMapKey, RwLock, Context, Mutex},
    model::{channel::Message, id::GuildId},
    framework::standard::{
        Args, CommandResult, macros::command
    },
};

use lavalink_rs::{
    LavalinkClient,
    model::*,
    gateway::*,
};

pub struct VoiceManager;
pub struct Lavalink;
pub struct VoiceGuildUpdate;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Lavalink {
    type Value = Arc<Mutex<LavalinkClient>>;
}

impl TypeMapKey for VoiceGuildUpdate {
    type Value = Arc<RwLock<HashSet<GuildId>>>;
}

pub struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackStart) {
        println!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackFinish) {
        println!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    // Get guild id
    let guild = msg.guild(&ctx.cache).await.expect("whoops, could not get guild");
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    // Get channel to connect id
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.channel_id.say(&ctx.http, "Not in a voice channel to play in!").await?;

            return Ok(());
        }
    };

    // Get VoiceManager in the TypeMap in the context
    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned()
        .expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_joined = manager.join(guild_id, connect_to).is_some();

    if has_joined {
        drop(manager);

        // remove guid_id from the voice guild update (vgu)
        loop {
            let data = ctx.data.read().await;
            let vgu_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut vgu = vgu_lock.write().await;
            if !vgu.contains(&guild_id) {
                tokio::time::delay_for(Duration::from_millis(500)).await;
            } else {
                vgu.remove(&guild_id);
                break;
            }
        }

        // Get manager
        let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned()
            .expect("Expected VoiceManager in TypeMap.");

        // Get handler
        let manager = manager_lock.lock().await;
        let handler = manager.get(guild_id).unwrap();

        let mut wdata = ctx.data.write().await;
        let lava_client_lock = wdata.get_mut::<Lavalink>()
            .expect("Expected a lavalink client in TypeMap");

        // create lavalink session
        lava_client_lock.lock().await.create_session(guild_id, &handler).await?;

        msg.channel_id.say(&ctx.http, &format!("Joined !")).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Error joining the channel").await?;
    }

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    
    // Get guild id
    let guild = msg.guild(&ctx.cache).await.expect("whoops, could not get guild");
    let guild_id = guild.id;

    // get manager
    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned()
        .expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(guild_id).is_some();

    // quitting and remove lavalink session
    if has_handler {
        manager.remove(guild_id);

        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>()
            .expect("Expected a lavalink client in TypeMap");
        lava_client_lock.lock().await.destroy(guild_id).await?;

        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel !").await?;
    }

    Ok(())
}

#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "Error finding channel info").await?;
            return Ok(());
        },
    };

    // Get manager
    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned()
        .expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(_handler) = manager.get_mut(guild_id) {
        // get lava client
        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let lava_client = lava_client_lock.lock().await;

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            msg.channel_id.say(&ctx, "Could not find any video of the search query.").await?;
            return Ok(());
        }

        drop(lava_client);

        if let Err(why) = LavalinkClient::play(guild_id, query_information.tracks[0].clone())
                                         .queue(Arc::clone(lava_client_lock)).await {
            eprintln!("Error playing on the lavalink client: {}", why);
            return Ok(());
        };
        msg.channel_id.say(&ctx.http, format!("Added to queue: {}", query_information.tracks[0].info.as_ref().unwrap().title)).await?;

    } else {
        msg.channel_id.say(&ctx.http, "Use `join` first, to connect the bot to your current voice channel.").await?;
    }

    Ok(())
}

#[command]
#[aliases(np)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    // Get lava client
    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let lava_client = lava_client_lock.lock().await;

    if let Some(node) = lava_client.nodes.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            msg.channel_id.say(&ctx.http, format!("Now playing: {}", track.track.info.as_ref().unwrap().title)).await?;
        } else {
            msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await?;
        }
    } else {
        msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await?;
    }

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    // Get guild id
    let guild = msg.guild(&ctx.cache).await.expect("whoops, could not get guild");
    let guild_id = guild.id;

    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>()
        .expect("Expected a lavalink client in TypeMap");

    if let Some(track) = lava_client_lock.lock().await.skip(*guild_id.as_u64()).await {
        msg.channel_id.say(&ctx.http, format!("Skipped: {}", track.track.info.as_ref().unwrap().title)).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Nothing to skip.").await?;
    }

    Ok(())
}

#[command]
pub async fn pause(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}

#[command]
pub async fn resume(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}

#[command]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    // Get guild id
    let guild = msg.guild(&ctx.cache).await.expect("whoops, could not get guild");
    let guild_id = guild.id;

    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>()
        .expect("Expected a lavalink client in TypeMap");

    if let Ok(_) = lava_client_lock.lock().await.stop(*guild_id.as_u64()).await {
        msg.channel_id.say(&ctx.http, "Stopped !").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Nothing to skip.").await?;
    }

    Ok(())
}