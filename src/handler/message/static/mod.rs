use teloxide::prelude::Message;

use crate::Bot;
use crate::command::Command;
use crate::storage::Storage;

pub mod all;
pub mod list;

pub async fn handle(
  bot: Bot,
  msg: Message,
  cmd: Command,
  storage: Storage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  match cmd {
    Command::All => all::handle(bot, msg, storage).await,
    Command::List => list::handle(bot, msg, storage).await,
  }
}
