use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        Args, CommandResult, macros::command
    }
};

#[command]
pub async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.single::<f64>()?;
    let two = args.single::<f64>()?;

    let product = one * two;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}

#[command]
pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.single::<f64>()?;
    let two = args.single::<f64>()?;

    let sum = one + two;

    msg.channel_id.say(&ctx.http, sum).await?;

    Ok(())
}

#[command]
pub async fn compute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    let all = args.raw().collect::<Vec<&str>>().join(" ");
    let terms: Vec<&str> = all.split("+").collect();
    let mut sum : f64 = 0.0;
    
    for term in terms {
        let factors: Vec<&str> = term.split("*").collect();
        let mut product : f64 = 1.0;
        for factor in factors {
            product *= factor.trim().parse::<f64>()?;
        }
        sum += product;
    }

    msg.channel_id.say(&ctx.http, format!("It makes: µ`{}`", sum)).await?;

    Ok(())
}