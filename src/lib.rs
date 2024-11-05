#![cfg_attr(debug_assertions, allow(warnings))]

use std::env;
use std::sync::Arc;

use teloxide::adaptors::DefaultParseMode;
use teloxide::Bot as TBot;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::requests::RequesterExt;
use teloxide::types::{ParseMode, Update};
use teloxide::update_listeners::webhooks;

use crate::command::Command;
use crate::storage::Storage;

mod command;
mod handler;
mod storage;

mod constants;
mod utils;

type Bot = DefaultParseMode<TBot>;

pub fn setup_logger() {
  pretty_env_logger::init();
}

pub async fn spin_up() {
  let inbound = env::var("INBOUND")
    .expect("Missing `INBOUND` address config")
    .parse()
    .expect("Unable to parse `INBOUND` from string");
  let outbound = env::var("OUTBOUND")
    .expect("Missing `OUTBOUND` address config")
    .parse()
    .expect("Unable to parse `INBOUND` from string");

  let bot = TBot::from_env().parse_mode(ParseMode::MarkdownV2);
  let options = webhooks::Options::new(inbound, outbound);
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
}
