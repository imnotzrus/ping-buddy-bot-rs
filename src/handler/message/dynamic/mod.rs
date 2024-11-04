use teloxide::prelude::Message;
use teloxide::types::MessageKind;

use crate::Bot;
use crate::storage::Storage;

pub mod common;
pub mod new_members;

pub async fn handle(
  bot: Bot,
  msg: Message,
  storage: Storage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  match msg.kind {
    MessageKind::NewChatMembers(_) => {
      new_members::handle(bot, msg, storage).await
    }
    MessageKind::Common(_) => common::handle(bot, msg, storage).await,
    _ => Ok(()),
  }
}
