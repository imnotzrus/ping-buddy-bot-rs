//! Callback query handler for inline button interactions.
//!
//! This module processes callback queries from inline keyboard buttons,
//! handling topic subscriptions and topic creation requests.

use teloxide::payloads::{
  AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters,
};
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;

use crate::constants::{NEW_TOPIC_CALLBACK, TOPIC_WRAPPER};
use crate::handler::ui::{build_topic_buttons, Messages};
use crate::storage::Storage;
use crate::traits::{ChatIdExt, UserRefExt};
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(
  bot: Bot,
  query: CallbackQuery,
  storage: Storage,
) -> Result {
  let msg = some_rtn_ok!(query.regular_message());
  let user = query.from.user_ref();

  // Verify user authorization if this is a reply
  if let Some(replied) = msg.reply_to_message() {
    let user_ref = some_rtn_ok!(replied.from.as_ref().map(|u| u.user_ref()));
    if user != user_ref {
      log::warn!(
        "Unauthorized callback attempt by {} for message from {}",
        user,
        user_ref
      );
      bot
        .answer_callback_query(&query.id)
        .text("This is not your subscription list")
        .show_alert(true)
        .await?;
      return Ok(());
    }
  }

  let topic = some_rtn_ok!(query.data.as_ref());
  let mut data = storage.write().await;

  match topic.as_str() {
    NEW_TOPIC_CALLBACK => {
      // Handle "Create New Topic" button
      bot.answer_callback_query(&query.id).await?;
      let response = bot
        .send_message(msg.chat.id, Messages::ask_topic(&user))
        .await?;
      data
        .push_creating_message_id(response.cid(), response.id.0)
        .await?;
    }
    topic
      if topic.starts_with(TOPIC_WRAPPER) && topic.ends_with(TOPIC_WRAPPER) =>
    {
      // Handle personalized topic toggle (subscribe/unsubscribe)
      let topic = topic
        .trim_start_matches(TOPIC_WRAPPER)
        .trim_end_matches(TOPIC_WRAPPER);

      let was_subscribed =
        data.is_subscriber_in_topic(msg.cid(), &user, topic).await?;

      if was_subscribed {
        if let Err(e) = data
          .unset_subscriber_from_topic(msg.cid(), topic, &user)
          .await
        {
          log::error!(
            "Failed to unsubscribe {} from topic '{}': {}",
            user,
            topic,
            e
          );
          bot.answer_callback_query(&query.id).await?;
          return Ok(());
        }
        bot
          .answer_callback_query(&query.id)
          .text(format!("Unsubscribed from {}", topic))
          .await?;
      } else {
        let user_slice = &[user.as_str()];
        if let Err(e) = data
          .set_topic_and_subscribers(msg.cid(), topic, user_slice)
          .await
        {
          log::error!(
            "Failed to subscribe {} to topic '{}': {}",
            user,
            topic,
            e
          );
          bot.answer_callback_query(&query.id).await?;
          bot
            .send_message(msg.chat.id, format!("Failed to subscribe: {}", e))
            .await?;
          return Ok(());
        }
        bot
          .answer_callback_query(&query.id)
          .text(format!("Subscribed to {}", topic))
          .await?;
      }

      let all_topics = data.get_topics(msg.cid()).await?.unwrap_or_default();
      let subscriptions = data
        .get_topics_from_subscriber(msg.cid(), &user)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
      let modified_topics = all_topics
        .iter()
        .map(|t| (t, subscriptions.contains(&t)))
        .collect::<Vec<_>>();
      _ = bot
        .edit_message_reply_markup(msg.chat.id, msg.id)
        .reply_markup(build_topic_buttons(modified_topics, true))
        .await;
    }
    topic => {
      // Handle direct topic subscription (from welcome message)
      let user_slice = &[user.as_str()];
      if let Err(e) = data
        .set_topic_and_subscribers(msg.cid(), topic, user_slice)
        .await
      {
        log::error!("Failed to subscribe {} to topic '{}': {}", user, topic, e);
        bot.answer_callback_query(&query.id).await?;
        bot
          .send_message(msg.chat.id, format!("Failed to subscribe: {}", e))
          .await?;
        return Ok(());
      }

      bot
        .answer_callback_query(&query.id)
        .text(format!("Subscribed to {}", topic))
        .await?;
      bot
        .send_message(
          msg.chat.id,
          Messages::user_subscribed_topic(&user, topic),
        )
        .await?;
    }
  }

  Ok(())
}
