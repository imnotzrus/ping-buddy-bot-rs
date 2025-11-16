//! Callback query handler for inline button interactions.
//!
//! This module processes callback queries from inline keyboard buttons,
//! handling topic subscriptions and topic creation requests.

use teloxide::payloads::{
  AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters,
  EditMessageTextSetters,
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
      log::debug!(
        "User {} initiated topic creation in chat {}",
        user,
        msg.cid()
      );
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
        log::info!(
          "User {} unsubscribed from topic '{}' in chat {}",
          user,
          topic,
          msg.cid()
        );
        // Acknowledge with feedback
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
        log::info!(
          "User {} subscribed to topic '{}' in chat {}",
          user,
          topic,
          msg.cid()
        );
        // Acknowledge with feedback
        bot
          .answer_callback_query(&query.id)
          .text(format!("Subscribed to {}", topic))
          .await?;
      }

      // Release the write lock before reading updated data
      drop(data);

      // Re-acquire read lock to get fresh data
      let data = storage.read().await;

      // Update the inline keyboard to reflect new subscription state
      let all_topics = some_rtn_ok!(data.get_topics(msg.cid()).await?);
      let subscriptions = data
        .get_topics_from_subscriber(msg.cid(), &user)
        .await?
        .unwrap_or_default();

      log::debug!(
        "After toggle - All topics: {:?}, User subscriptions: {:?}",
        all_topics,
        subscriptions
      );

      let modified_topics: Vec<_> = all_topics
        .iter()
        .map(|t| {
          let is_subscribed = subscriptions.contains(t);
          log::debug!("Topic '{}' subscription status: {}", t, is_subscribed);
          (t.as_str(), is_subscribed)
        })
        .collect();

      // Try to edit message text with keyboard to force UI update
      let message_text = msg.text().unwrap_or("Your subscriptions");
      let keyboard = build_topic_buttons(&modified_topics, true);
      let edit_result = bot
        .edit_message_text(msg.chat.id, msg.id, message_text)
        .reply_markup(keyboard.clone())
        .await;

      // If text edit fails (e.g., text unchanged), try just editing markup
      if let Err(e) = edit_result {
        log::debug!("Text edit failed, trying markup only: {}", e);
        if let Err(e2) = bot
          .edit_message_reply_markup(msg.chat.id, msg.id)
          .reply_markup(keyboard)
          .await
        {
          log::warn!("Failed to update keyboard: {}", e2);
        }
      }
    }
    topic => {
      // Handle direct topic subscription (from welcome message)
      log::debug!(
        "User {} subscribing to topic '{}' in chat {}",
        user,
        topic,
        msg.cid()
      );

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
