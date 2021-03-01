use super::super::modules::types::*;
use crate::modules::pagination;
use chrono::Utc;
use serenity::{
    builder::CreateMessage,
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    gateway::ConnectionStage,
    model::{
        channel::Message,
        id::UserId,
        prelude::{Guild, GuildChannel, Member, Role},
    },
    prelude::*,
};
use serenity_utils::conversion::Conversion;
use serenity_utils::menu::Menu;
use std::collections::HashSet;

#[group]
#[commands(ping, menu, channel)]
struct General;

#[help]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command]
#[description = "Get the current latency."]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let start = Utc::now();
    let mut message = msg.channel_id.say(&ctx, "🏓").await?;
    let end = Utc::now();

    let round_trip = end - start;

    let data = ctx.data.read().await;
    let mut latencies = Vec::<String>::new();

    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let shard_manager = data.get::<ShardManagerContainer>().unwrap();
    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events over the shard
    for (id, runner) in runners.iter() {
        latencies.push(match runner.stage {
            ConnectionStage::Connected => match runner.latency {
                Some(latency) => format!(
                    ":green_circle: **Shard {}** `{}`ms",
                    id.0 + 1,
                    latency.as_millis()
                ),
                _ => format!(":cyclone: **Shard {}** `waiting for heartbeat`", id.0 + 1),
            },
            ConnectionStage::Connecting
            | ConnectionStage::Handshake
            | ConnectionStage::Resuming
            | ConnectionStage::Identifying => format!(":cyclone: **Shard {}** `connecting`", id),
            ConnectionStage::Disconnected | ConnectionStage::__Nonexhaustive => {
                format!(":x: **Shard {}** `disconnected`", id)
            }
        });
    }
    let _ = message
        .edit(&ctx.http, |m| {
            m.embed(|e| {
                e.description(format!(
                    ":ping_pong: Command roundtrip: `{}`ms\n{}",
                    round_trip.num_milliseconds(),
                    latencies.join("\n")
                ));
                e
            });
            m.content("");
            m
        })
        .await?;

    Ok(())
}

#[command]
async fn menu(ctx: &Context, msg: &Message) -> CommandResult {
    let mut page_one = CreateMessage::default();
    page_one.content("Page number one!").embed(|e| {
        e.description("The first page!");

        e
    });

    let mut page_two = CreateMessage::default();
    page_two.content("Page number two!").embed(|e| {
        e.description("The second page!");

        e
    });

    let pages = [page_one, page_two];

    // Creates a new menu.
    let menu = Menu::new(ctx, msg, &pages, pagination::simple_options());

    // Runs the menu and returns optional `Message` used to display the menu.
    let _ = menu.run().await?;

    Ok(())
}

#[command]
async fn channel(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(guild_id) = msg.guild_id {
        if let Some(channel) = GuildChannel::from_guild_id_and_str(ctx, guild_id, args.rest()).await {
            msg.channel_id
                .say(&ctx.http, format!("u mean {} ?", channel.mention()))
                .await?;
        } else {
            msg.channel_id
                .say(&ctx.http, "No channel found from the given input.")
                .await?;
        }
    }
    Ok(())
}
