use egg_mode::stream::StreamMessage;
use futures::prelude::*;
use log::{error, info};

use crate::modules::events;

use egg_mode;
use std::{
    self,
    io::{Read, Write},
};

pub use yansi::Paint;

pub struct Config {
    pub token: egg_mode::Token,
}

pub struct Streamer {
    pub follows: Vec<u64>,
    pub current_stream: Option<&StreamInstance>,
}

pub struct StreamInstance {
    pub follows: Vec<u64>,
    pub restart: bool,
}

impl Config {
    pub fn new() -> Config {
        let con_token = egg_mode::KeyPair::new("", "");
        let access_token = egg_mode::KeyPair::new("", "");
        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: access_token,
        };
        Self { token }
    }
}

impl Streamer {
    pub fn new(follows: Vec<u64>) -> Streamer {
        Self {
            follows,
            current_stream: None,
        }
    }

    pub async fn start_stream(&mut self) {
        let stream_instance = StreamInstance {
            follows: self.follows.clone(),
            restart: false,
        };
        // self.current_stream = Some(&stream_instance);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(futures::future::select(
            stream_instance.stream(),
            receiver.map_err(drop),
        ));
        sender.send(()); // this will cancel the task
    }
}

impl StreamInstance {
    pub async fn stream(&self) {
        let config = Config::new();
        println!("Streaming tweets containing popular programming languages (and also Rust)");
        println!("Ctrl-C to quit\n");

        let stream = egg_mode::stream::filter()
            .follow(&self.follows)
            .start(&config.token)
            .try_for_each(|m| {
                if let StreamMessage::Tweet(tweet) = m {
                    self.print_tweet(&tweet);
                    println!("──────────────────────────────────────");
                } else {
                    println!("{:?}", m);
                }
                futures::future::ok(())
            });
        if let Err(e) = stream.await {
            println!("Stream error: {}", e);
            println!("Disconnected")
        }
    }
    pub fn print_tweet(&self, tweet: &egg_mode::tweet::Tweet) {
        if let Some(ref user) = tweet.user {
            println!(
                "{} (@{}) posted at {}",
                Paint::blue(&user.name),
                Paint::bold(Paint::blue(&user.screen_name)),
                tweet.created_at.with_timezone(&chrono::Local)
            );
        }

        if let Some(ref screen_name) = tweet.in_reply_to_screen_name {
            println!("➜ in reply to @{}", Paint::blue(screen_name));
        }

        if let Some(ref status) = tweet.retweeted_status {
            println!("{}", Paint::red("Retweet ➜"));
            self.print_tweet(status);
            return;
        } else {
            println!("{}", Paint::green(&tweet.text));
        }

        if let Some(source) = &tweet.source {
            println!("➜ via {} ({})", source.name, source.url);
        }

        if let Some(ref place) = tweet.place {
            println!("➜ from: {}", place.full_name);
        }

        if let Some(ref status) = tweet.quoted_status {
            println!("{}", Paint::red("➜ Quoting the following status:"));
            self.print_tweet(status);
        }

        if !tweet.entities.hashtags.is_empty() {
            println!("➜ Hashtags contained in the tweet:");
            for tag in &tweet.entities.hashtags {
                println!("  {}", tag.text);
            }
        }

        if !tweet.entities.symbols.is_empty() {
            println!("➜ Symbols contained in the tweet:");
            for tag in &tweet.entities.symbols {
                println!("  {}", tag.text);
            }
        }

        if !tweet.entities.urls.is_empty() {
            println!("➜ URLs contained in the tweet:");
            for url in &tweet.entities.urls {
                if let Some(expanded_url) = &url.expanded_url {
                    println!("  {}", expanded_url);
                }
            }
        }

        if !tweet.entities.user_mentions.is_empty() {
            println!("➜ Users mentioned in the tweet:");
            for user in &tweet.entities.user_mentions {
                println!("  {}", Paint::bold(Paint::blue(&user.screen_name)));
            }
        }

        if let Some(ref media) = tweet.extended_entities {
            println!("➜ Media attached to the tweet:");
            for info in &media.media {
                println!("  A {:?}", info.media_type);
            }
        }
    }
}
