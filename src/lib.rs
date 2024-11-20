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

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

mod command;
mod handler;
mod storage;

mod constants;
mod env;
mod utils;

type Bot = DefaultParseMode<TBot>;

pub fn setup_logger() {
  pretty_env_logger::init();
}

pub async fn spin_up() -> Result {
  let env = Env::init()?;

  let bot = TBot::from_env().parse_mode(ParseMode::MarkdownV2);

  let options =
    webhooks::Options::new(env.inbound.parse()?, env.outbound.parse()?);
  let listener = webhooks::axum(Clone::clone(&bot), options)
    .await
    .expect("Unable to build listener");

  let storage = Storage::default();
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

  Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![storage])
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(listener, Arc::new(|_| async {}))
    .await;
  Ok(())
}
