use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::handler::helper_messages::{
  InvalidRequest, InvalidTopic, InvalidTopicFormat, UserSubscribedTopic,
};
use crate::handler::message::{
  list_users, ChatIdExtractImpl, TopicExtractImpl, UserRefExtractImpl,
};
use crate::handler::remove_message;
use crate::storage::Storage;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

fn is_valid_topic(topic: &str) -> bool {
  topic.chars().all(|c| c.is_alphanumeric())
    && topic.as_bytes()[0].is_ascii_alphabetic()
}

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  match msg.reply_to_message() {
    None => {
      let topic = some_rtn_ok!(msg.topic());
      let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
      let ret = list_users(storage, &msg, topic, user).await;
      let msg = bot
        .send_message(msg.chat.id, ret.value())
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
      if ret.must_remove() {
        remove_message(bot, msg);
      }
    }
    Some(replied_msg) => {
      let mut data = storage.write().await;
      if data.has_create_message_id(replied_msg.cid(), replied_msg.id.0) {
        let Some(topic) = msg.text() else {
          bot
            .send_message(msg.chat.id, InvalidTopicFormat::msg())
            .await?;
          return Ok(());
        };
        if !is_valid_topic(topic) {
          bot
            .send_message(msg.chat.id, InvalidTopic::msg(topic))
            .await?;
          return Ok(());
        };
        let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
        data.set_topic_and_subscribers(msg.cid(), topic, [&user]);
        data.pop_creating_message_id(replied_msg.cid(), replied_msg.id.0);
        bot
          .send_message(msg.chat.id, UserSubscribedTopic::msg(user, topic))
          .await?;
        remove_message(bot, replied_msg.clone());
      } else {
        bot.send_message(msg.chat.id, InvalidRequest::msg()).await?;
      }
    }
  }

  Ok(())
}
