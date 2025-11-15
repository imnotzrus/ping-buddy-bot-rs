#![deny(warnings)]

use std::sync::Arc;

use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::requests::RequesterExt;
use teloxide::types::{ParseMode, Update};
use teloxide::update_listeners::webhooks;
use teloxide::Bot as TBot;

use crate::command::Command;
use crate::env::Env;
use crate::storage::Storage;

// Re-export error types for convenience
pub use crate::error::{BotError, Result};

mod command;
mod handler;
mod storage;

mod constants;
mod env;
mod error;
mod utils;
mod validation;

type Bot = DefaultParseMode<TBot>;

pub fn setup_logger() {
  pretty_env_logger::init();
}

pub async fn spin_up() -> Result {
    log::info!("Starting Ping Buddy Bot...");
    
    let env = Env::init()?;

    let bot = TBot::from_env().parse_mode(ParseMode::MarkdownV2);

    log::info!("Setting up webhook listener...");
    let options = webhooks::Options::new(
        env.inbound
            .as_str()
            .parse()
            .map_err(|e| BotError::AddrParse(format!("{}", e)))?,
        env.outbound
            .as_str()
            .parse()
            .map_err(|e| BotError::AddrParse(format!("{}", e)))?,
    );
    
    let listener = webhooks::axum(Clone::clone(&bot), options)
        .await
        .map_err(|e| BotError::Config(format!("Failed to build webhook listener: {}", e)))?;

    let storage = Storage::default();
    
    log::info!("Building message handler...");
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(handler::message::r#static::handle),
                )
                .branch(dptree::entry().endpoint(handler::message::dynamic::handle)),
        )
        .branch(
            Update::filter_callback_query().endpoint(handler::callback::handle),
        );

    log::info!("Bot is ready and listening for updates");
    
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            Arc::new(|err| async move {
                log::error!("Dispatcher error: {:?}", err);
            }),
        )
        .await;
    
    log::info!("Bot shutting down");
    Ok(())
}
