use crate::modules::types::*;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::Row;

#[group]
#[commands(quit, dbe, dbq)]
struct Owner;

#[command]
#[owners_only]
#[description = "Gently terminates the bot process"]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.channel_id.say(&ctx, "Goodbye!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.channel_id
            .say(&ctx, "There was a problem getting the shard manager")
            .await?;
    }

    Ok(())
}

#[command]
#[owners_only]
#[usage = "<sql command>"]
#[description = "Executes a command against the database"]
async fn dbe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let affected_rows = sqlx::query(args.rest()).execute(pool).await?;
    msg.channel_id.say(&ctx, format!("{:?}", affected_rows)).await?;

    Ok(())
}

#[command]
#[owners_only]
#[usage = "<sql query>"]
#[description = "Queries data from the database"]
async fn dbq(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();
    let mut bot_msg = Vec::new();

    let rows = sqlx::query(args.rest()).fetch_all(pool).await?;
    for row in rows {
        let mut items = vec![];
        for i in 0..row.len() {
            //TODO: doesnt actually work
            items.push(format!("{:?}", row.column(i)))
        }
        bot_msg.push(items.join(", "));
    }
    msg.channel_id.say(&ctx, bot_msg.join("\n")).await?;

    Ok(())
}
