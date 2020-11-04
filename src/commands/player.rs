use std::sync::Arc;
use serenity::{
    prelude::{TypeMapKey, RwLock, Context},
    model::{channel::Message, guild::Guild},
    framework::standard::{
        Args, CommandResult, macros::command
    },
    utils::MessageBuilder,
};

use songbird::tracks::TrackHandle;

struct Player {
    handle: Option<TrackHandle>,
    has_joined: bool,
    is_paused: bool,
    queue: Vec<String>
}

impl TypeMapKey for Player {
    type Value = Arc<RwLock<Player>>;
}

impl Player {
    fn new() -> Self {
        Self {
            handle: None,
            has_joined: false,
            is_paused: false,
            queue: vec![],
        }
    }
}

#[command]
pub async fn cache(_ctx: &Context, _msg: &Message) -> CommandResult {
    // msg.channel_id.say(&ctx.http, format!("{:?}", &ctx.cache).as_str()).await?;
    // msg.channel_id.say(&ctx.http, format!("nb guilds: {:?}", &ctx.cache.guild_count()).as_str()).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let guild: Guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.channel_id.say(&ctx.http, "😑 You must be in a voice channel!").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    
    let _handler = manager.join(guild_id, connect_to).await;

    let player_lock = {
        let read_data = ctx.data.read().await;
        read_data.get::<Player>().expect("Expected Player in TypeMap D-:").clone()
    };
    let mut player = player_lock.write().await;
    player.has_joined = true;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild: Guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    manager.leave(guild_id).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let player_lock = {
        let read_data = ctx.data.read().await;
        read_data.get::<Player>().expect("Expected Player in TypeMap D-:").clone()
    };
    {
        let player = player_lock.read().await;

        if !player.has_joined {
            join(ctx, msg, Args::new("", &[])).await?;
        }
    }

    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "😅 Must provide a URL to a video or audio!").await?;
            return Ok(());
        },
    };

    if !url.starts_with("http") {
        msg.channel_id.say(&ctx.http, "🤦 You must provide a valid URL...").await?;
        return Ok(());
    }

    let guild = match msg.guild(&ctx.cache).await  {
        Some(g) => g,
        None => {
            msg.channel_id.say(&ctx.http, "Could not get guild...").await?;
            return Ok(())
        }
    };
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("-----------Err starting source: {:?}------------------", why);

                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            },
        };

        let audio_name = match &source.metadata.title {
            Some(title) => title.clone(),
            None        => String::from("Unknown"),
        };
        // println!("Metadata: ({:?})", &source.metadata);
        // println!("reader: ({:?})", &source.reader);

        let player_lock = {
            let read_data = ctx.data.read().await;
            read_data.get::<Player>().expect("Expected Player in TypeMap D-:").clone()
        };
        {
            let mut player = player_lock.write().await;
            player.handle = Some(handler.play_source(source));
        }

        let ans = MessageBuilder::new()
            .push("Playing: ")      .push_underline(&audio_name)
            .push("🎵")             .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    } else {
        msg.channel_id.say(&ctx.http, "😑 I'm not in a voice channel!").await?;
    }
    Ok(())
}

#[command]
pub async fn pause(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}

#[command]
pub async fn stop(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}