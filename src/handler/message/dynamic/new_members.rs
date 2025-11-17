//! Handler for new chat members (bot or users joining).

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Message;
use teloxide::requests::Requester;

use crate::constants::{BOT_USERNAME, GENERAL_TOPIC};
use crate::handler::ui::{build_topic_buttons, Messages};
use crate::storage::Storage;
use crate::traits::{ChatIdExt, UserRefExt};
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
  let new_members = some_rtn_ok!(msg.new_chat_members());

  for member in new_members {
    let mut data = storage.write().await;

    let (text, topics): (String, Vec<(String, bool)>) = if member.is_bot {
      // Handle bot joining
      if matches!(member.username.as_deref(), Some(BOT_USERNAME)) {
        log::info!("Bot joined chat {}", msg.cid());

        // Initialize general topic
        let empty_slice: &[&str] = &[];
        if let Err(e) = data
          .set_topic_and_subscribers(msg.cid(), GENERAL_TOPIC, empty_slice)
          .await
        {
          log::error!("Failed to initialize general topic: {}", e);
        }

        (
          Messages::bot_join(),
          vec![(GENERAL_TOPIC.to_string(), false)],
        )
      } else {
        // Ignore other bots
        continue;
      }
    } else {
      // Handle user joining
      let user_ref = member.user_ref();
      log::info!("New member {} joined chat {}", user_ref, msg.cid());

      let topics = data
        .get_topics(msg.cid())
        .await?
        .map(|ts| ts.into_iter().map(|t| (t, false)).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![(GENERAL_TOPIC.to_string(), false)]);

      (Messages::welcome_new_member(&user_ref), topics)
    };

    let topics_ref: Vec<(&str, bool)> =
      topics.iter().map(|(t, b)| (t.as_str(), *b)).collect();
    bot
      .send_message(msg.chat.id, text)
      .reply_markup(build_topic_buttons(topics_ref, false))
      .await?;
  }

  Ok(())
}
