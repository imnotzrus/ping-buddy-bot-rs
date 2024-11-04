#![cfg_attr(debug_assertions, allow(warnings))]

use teloxide::adaptors::DefaultParseMode;
use teloxide::Bot as TBot;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::requests::RequesterExt;
use teloxide::types::{ParseMode, Update};

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
  let bot = TBot::from_env().parse_mode(ParseMode::MarkdownV2);
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
    .dispatch()
    .await;
}
