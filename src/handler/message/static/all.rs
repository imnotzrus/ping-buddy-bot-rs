//! Handler for /all command - pings everyone in the general topic.

use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::constants::GENERAL_TOPIC;
use crate::handler::message::list_users;
use crate::handler::ui::remove_message;
use crate::storage::Storage;
use crate::traits::UserRefExt;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
  let response = list_users(storage, &msg, GENERAL_TOPIC, user).await?;
  let sent_msg = bot
    .send_message(msg.chat.id, response.text())
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;
  if response.should_remove() {
    remove_message(bot, sent_msg);
  }
  Ok(())
}
