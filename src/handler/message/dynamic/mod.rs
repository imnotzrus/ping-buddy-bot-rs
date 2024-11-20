use teloxide::prelude::Message;
use teloxide::types::MessageKind;

use crate::storage::Storage;
use crate::Bot;
use crate::Result;

pub mod common;
pub mod new_members;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  match msg.kind {
    MessageKind::NewChatMembers(_) => {
      new_members::handle(bot, msg, storage).await
    }
    MessageKind::Common(_) => common::handle(bot, msg, storage).await,
    _ => Ok(()),
  }
}
