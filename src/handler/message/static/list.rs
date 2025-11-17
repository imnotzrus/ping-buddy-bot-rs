//! Handler for /list command - shows all topics with subscription status.

use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{
  InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyParameters,
};

use crate::constants::NEW_TOPIC_CALLBACK;
use crate::handler::ui::build_topic_buttons;
use crate::storage::Storage;
use crate::traits::{ChatIdExt, UserRefExt};
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
  let data = storage.read().await;

  let all_topics = match data.get_topics(msg.cid()).await? {
    Some(topics) => topics,
    None => {
      let keyboard =
        InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
          "Create New Topic",
          NEW_TOPIC_CALLBACK,
        )]]);
      bot
        .send_message(msg.chat.id, "No topics available yet\\. Create one\\!")
        .reply_markup(keyboard)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
      return Ok(());
    }
  };

  if all_topics.is_empty() {
    let keyboard =
      InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Create New Topic",
        NEW_TOPIC_CALLBACK,
      )]]);
    bot
      .send_message(msg.chat.id, "No topics available yet\\. Create one\\!")
      .reply_markup(keyboard)
      .reply_parameters(ReplyParameters::new(msg.id))
      .await?;
    return Ok(());
  }

  let subscriptions = data
    .get_topics_from_subscriber(msg.cid(), &user)
    .await?
    .unwrap_or_default();

  let modified_topics: Vec<_> = all_topics
    .iter()
    .map(|t| (t.as_str(), subscriptions.contains(t)))
    .collect();

  bot
    .send_message(msg.chat.id, "Your subscriptions")
    .reply_markup(build_topic_buttons(modified_topics, true))
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;

  Ok(())
}
