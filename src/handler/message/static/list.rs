use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::handler::message::{ChatIdExtractImpl, UserRefExtractImpl};
use crate::handler::topic_buttons;
use crate::storage::Storage;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
  let data = storage.read().await;
  let all_topics = some_rtn_ok!(data.get_topics(msg.cid()));
  let subscriptions = data
    .get_topics_from_subscriber(msg.cid(), &user)
    .map(|ts| ts.collect::<Vec<_>>())
    .unwrap_or_default();
  let modified_topics = all_topics
    .map(|t| (t, subscriptions.contains(&t)))
    .collect::<Vec<_>>();
  bot
    .send_message(msg.chat.id, "Your subscriptions")
    .reply_markup(topic_buttons(modified_topics, true))
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;
  Ok(())
}
