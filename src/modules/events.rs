use crate::modules::types::*;
use log::{error, info};
use serenity::{
    async_trait,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
    model::gateway::Ready,
    model::id::GuildId,
    model::prelude::Permissions,
    prelude::*,
};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Handler {
    pub streaming: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if !self.streaming.load(Ordering::Relaxed) {
            let data = ctx.data.read().await;
            let twitter_streamer = data.get::<TwitterStreamer>().unwrap();
            self.streaming.store(true, Ordering::Relaxed);
            twitter_streamer.start_stream().await;
        }
    }
}

#[hook]
pub async fn after_hook(ctx: &Context, msg: &Message, _cmd_name: &str, error: Result<(), CommandError>) {
    //  Print out an error if it happened
    if let Err(why) = error {
        error!(
            "Command error [ {}#{} {} ]: {:?}",
            msg.author.name, msg.author.discriminator, msg.content, why
        );
        let _ = msg.channel_id.say(&ctx, format!("```rs\n{}\n```", why)).await;
    // no error, just log the command usage
    } else {
        info!(
            "{}#{} used {} (took TODO)",
            msg.author.name, msg.author.discriminator, msg.content
        )
    }
}

#[hook]
pub async fn before_hook(_ctx: &Context, _msg: &Message, cmd_name: &str) -> bool {
    info!("Running command {}", cmd_name);
    true
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(s, reason) => {
            let _ = msg
                .channel_id
                .say(ctx, format!("The command checks have failed {}\n{:?}", s, reason))
                .await;
        }
        DispatchError::CommandDisabled(s) => {
            let _ = msg
                .channel_id
                .say(ctx, format!("This command is disabled! {}", s))
                .await;
        }
        DispatchError::OnlyForDM => {
            let _ = msg
                .channel_id
                .say(ctx, "This command can only be used in DMs.")
                .await;
        }
        DispatchError::OnlyForGuilds => {
            let _ = msg
                .channel_id
                .say(ctx, "This command can only be used in servers.")
                .await;
        }
        DispatchError::LackingPermissions(Permissions::ADMINISTRATOR) => {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    "You need to be an **Administrator** to execute this command!",
                )
                .await;
        }
        DispatchError::LackingPermissions(perms) => {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    format!("You require **{:?}** permission to execute this command!", perms),
                )
                .await;
        }
        DispatchError::NotEnoughArguments { min, given } => {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    format!(
                        "Not enough arguments! ({}) This command requires at least {}",
                        given, min
                    ),
                )
                .await;
        }
        DispatchError::OnlyForOwners => {
            let _ = msg.channel_id.say(ctx, "Only the bot owner can use this!").await;
        }
        _ => error!("Unhandled dispatch error: {:?}", error),
    }
}
