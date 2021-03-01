use log::error;
use serenity::{framework::StandardFramework, http::Http, prelude::*};
use std::{collections::HashSet, env, sync::Arc};
#[macro_use]
extern crate quick_error;
mod modules;
use modules::database;
use modules::events;
use modules::streamer;
use modules::types::*;

mod commands;
use commands::{general::*, owner::*};
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::main]
async fn main() {
    kankyo::load(false).expect("Failed to load .env file");
    env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("No token found!");
    let database_credentials = env::var("DATABASE_URL").expect("No database credentials found!");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let reqwest_client = Arc::new(reqwest::Client::new());
    let twitter_streamer = Arc::new(streamer::Streamer::new(vec![]));
    let pool = database::get_pool(&database_credentials).await.unwrap();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .prefix("=")
                .on_mention(Some(bot_id))
                .case_insensitivity(true)
        })
        .after(events::after_hook)
        .before(events::before_hook)
        .on_dispatch_error(events::dispatch_error)
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP)
        .help(&HELP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(events::Handler {
            streaming: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client!");
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ReqwestClient>(reqwest_client);
        data.insert::<ConnectionPool>(pool.clone());
        data.insert::<TwitterStreamer>(twitter_streamer);
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
