use crate::modules::streamer::Streamer;
use reqwest::Client as Reqwest;
use serenity::{client::bridge::gateway::ShardManager, prelude::*};
use sqlx::PgPool;
use std::sync::Arc;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<Reqwest>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct TwitterStreamer;

impl TypeMapKey for TwitterStreamer {
    type Value = Streamer;
}
