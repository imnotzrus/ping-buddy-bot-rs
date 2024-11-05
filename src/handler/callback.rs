use teloxide::payloads::EditMessageReplyMarkupSetters;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;

use crate::Bot;
use crate::handler::{NEW_TOPIC, topic_buttons, TOPIC_WRAPPER};
use crate::handler::helper_messages::{AskTopic, UserSubscribedTopic};
use crate::handler::message::{ChatIdExtractImpl, UserRefExtractImpl};
use crate::storage::Storage;
use crate::utils::some_rtn_ok;

pub async fn handle(
  bot: Bot,
  query: CallbackQuery,
  storage: Storage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  bot.answer_callback_query(&query.id).await?;
  let msg = some_rtn_ok!(query.regular_message());
  let user = query.from.user_ref();
  if let Some(replied) = msg.reply_to_message() {
    let user_ref = some_rtn_ok!(replied.from.as_ref().map(|u| u.user_ref()));
    if user != user_ref {
      return Ok(());
    }
  }

  let topic = some_rtn_ok!(query.data.as_ref());
  let mut data = storage.write().await;
  match topic.as_str() {
    NEW_TOPIC => {
      let msg = bot.send_message(msg.chat.id, AskTopic::msg(user)).await?;
      data.push_creating_message_id(msg.cid(), msg.id.0);
    }
    topic
      if topic.starts_with(TOPIC_WRAPPER) && topic.ends_with(TOPIC_WRAPPER) =>
    {
      let topic = topic
        .trim_start_matches(TOPIC_WRAPPER)
        .trim_end_matches(TOPIC_WRAPPER);
      if data.is_subscriber_in_topic(msg.cid(), &user, topic) {
        data.unset_subscriber_from_topic(msg.cid(), topic, &user);
      } else {
        data.set_topic_and_subscribers(msg.cid(), topic, [&user]);
      };
      let all_topics = some_rtn_ok!(data.get_topics(msg.cid()));
      let subscriptions = data
        .get_topics_from_subscriber(msg.cid(), &user)
        .map(|ts| ts.collect::<Vec<_>>())
        .unwrap_or_default();
      let modified_topics = all_topics
        .map(|t| (t, subscriptions.contains(&t)))
        .collect::<Vec<_>>();
      _ = bot
        .edit_message_reply_markup(msg.chat.id, msg.id)
        .reply_markup(topic_buttons(modified_topics, true))
        .await;
    }
    topic => {
      data.set_topic_and_subscribers(msg.cid(), topic, [&user]);
      bot
        .send_message(msg.chat.id, UserSubscribedTopic::msg(user, topic))
        .await?;
    }
  };
  Ok(())
}
