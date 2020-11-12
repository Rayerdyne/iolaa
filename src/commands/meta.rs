use crate::ShardManagerContainer;

use serenity::{
    prelude::Context,
    model::prelude::Message,
    utils::MessageBuilder,
    framework::standard::{
        CommandResult, // Args,
        macros::command,
    }
};


// #[command]
// #[min_args(1)]
// #[sub_commands(dog)]
// pub async fn test(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
//     msg.channel_id.say(&ctx.http, 
//         format!("Running **test** command with {} args length !", args.len())).await?;
//     Ok(())
// }

// #[command]
// #[aliases(d)]
// pub async fn dog(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
//     msg.channel_id.say(&ctx.http, 
//         format!("Running **dog** command with {} woofs !", args.len())).await?;
//     Ok(())
// }

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong !").await?;

    Ok(())
}

#[command]
async fn hello(ctx: &Context, msg: &Message) -> CommandResult {
    let response = MessageBuilder::new()
        .push("Hello ")
        .user(&msg.author)
        .push(" !")
        .build();
    
    msg.channel_id.say(&ctx.http, &response).await?;

    Ok(())
}

#[command]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.channel_id.say(&ctx.http, "See ya!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.channel_id.say(&ctx.http, "There was a problem getting the shard manager").await?;

        return Ok(());
    }

    Ok(())
}