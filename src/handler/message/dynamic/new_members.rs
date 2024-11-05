use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Message;
use teloxide::requests::Requester;

use crate::Bot;
use crate::constants::{BOT_USERNAME, GENERAL_TOPIC};
use crate::handler::helper_messages::{
  BotJoinMessage, WelcomeNewMemberMessage,
};
use crate::handler::message::{ChatIdExtractImpl, UserRefExtractImpl};
use crate::handler::topic_buttons;
use crate::storage::Storage;
use crate::utils::some_rtn_ok;

pub async fn handle(
  bot: Bot,
  msg: Message,
  storage: Storage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let new_members = some_rtn_ok!(msg.new_chat_members());
  for member in new_members {
    let mut data = storage.write().await;
    let (text, topics) = if member.is_bot {
      if matches!(member.username.as_deref(), Some(BOT_USERNAME)) {
        data.set_topic_and_subscribers(
          msg.cid(),
          GENERAL_TOPIC,
          Vec::<&str>::new(),
        );
        (BotJoinMessage::msg(), vec![(GENERAL_TOPIC, false)])
      } else {
        continue;
      }
    } else {
      let user_ref = member.user_ref();
      let topics = data
        .get_topics(msg.cid())
        .map(|ts| ts.map(|t| (t, false)).collect())
        .unwrap_or(vec![(GENERAL_TOPIC, false)]);
      (WelcomeNewMemberMessage::msg(user_ref), topics)
    };
    bot
      .send_message(msg.chat.id, text)
      .reply_markup(topic_buttons(topics, false))
      .await?;
  }
  Ok(())
}
