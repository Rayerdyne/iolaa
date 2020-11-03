use serenity::{
    client::Context,
    model::{channel::Message},
    framework::standard::{
        Args, CommandResult, macros::command
    },
};

// use songbird::{
//     id::GuildId,
// };

#[command]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let guild = match msg.guild(&ctx.cache).await {
        Some(g) => g,
        None => {
            msg.channel_id.say(&ctx.http, "😑 You must be in a voice channel!").await?;
            return Ok(());
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
                println!("Err starting source: {:?}", why);

                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            },
        };

        handler.play_source(source);

        msg.channel_id.say(&ctx.http, "Playing song").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await?;
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