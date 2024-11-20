use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::constants::GENERAL_TOPIC;
use crate::handler::message::{list_users, UserRefExtractImpl};
use crate::handler::remove_message;
use crate::storage::Storage;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
  let ret = list_users(storage, &msg, GENERAL_TOPIC, user).await;
  let msg = bot
    .send_message(msg.chat.id, ret.value())
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;
  if ret.must_remove() {
    remove_message(bot, msg);
  }
  Ok(())
}
