use teloxide::prelude::Message;

use crate::command::Command;
use crate::storage::Storage;
use crate::Bot;
use crate::Result;

pub mod all;
pub mod list;

pub async fn handle(
  bot: Bot,
  msg: Message,
  cmd: Command,
  storage: Storage,
) -> Result {
  match cmd {
    Command::All => all::handle(bot, msg, storage).await,
    Command::List => list::handle(bot, msg, storage).await,
  }
}
