//! Handler for common messages (topic pings and topic creation).

use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::handler::message::list_users;
use crate::handler::ui::{remove_message, Messages};
use crate::storage::Storage;
use crate::traits::{ChatIdExt, TopicExt, UserRefExt};
use crate::utils::some_rtn_ok;
use crate::validation::validate_topic;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  match msg.reply_to_message() {
    None => {
      // Handle topic ping (e.g., /topic_name)
      let topic = some_rtn_ok!(msg.topic());
      let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));

      let response = list_users(storage, &msg, topic, user).await?;
      let response_msg = bot
        .send_message(msg.chat.id, response.text())
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

      if response.should_remove() {
        remove_message(bot, response_msg);
      }
    }
    Some(replied_msg) => {
      // Handle topic creation response
      let mut data = storage.write().await;

      if data
        .has_create_message_id(replied_msg.cid(), replied_msg.id.0)
        .await?
      {
        let Some(topic) = msg.text() else {
          bot
            .send_message(msg.chat.id, Messages::invalid_topic_format())
            .await?;
          return Ok(());
        };

        // Validate topic name
        if let Err(validation_err) = validate_topic(topic) {
          log::warn!("Invalid topic name '{}': {}", topic, validation_err);
          bot
            .send_message(msg.chat.id, Messages::invalid_topic(topic))
            .await?;
          return Ok(());
        }

        let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));

        // Create topic and subscribe user
        let user_slice = &[user.as_str()];
        if let Err(e) = data
          .set_topic_and_subscribers(msg.cid(), topic, user_slice)
          .await
        {
          log::error!("Failed to create topic '{}': {}", topic, e);
          bot
            .send_message(msg.chat.id, format!("Failed to create topic: {}", e))
            .await?;
          return Ok(());
        }

        data
          .pop_creating_message_id(replied_msg.cid(), replied_msg.id.0)
          .await?;

        log::info!(
          "User {} created topic '{}' in chat {}",
          user,
          topic,
          msg.cid()
        );

        bot
          .send_message(
            msg.chat.id,
            Messages::user_subscribed_topic(user, topic),
          )
          .await?;
        remove_message(bot, replied_msg.clone());
      } else {
        bot
          .send_message(msg.chat.id, Messages::invalid_request())
          .await?;
      }
    }
  }

  Ok(())
}
