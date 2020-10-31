use crate::ShardManagerContainer;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::utils::MessageBuilder;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};

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