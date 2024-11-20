use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::constants::BOT_MSG_TTL;
use crate::Bot;

const NEW_TOPIC: &str = "new_topic";
const TOPIC_WRAPPER: char = '#';

pub mod callback;
pub mod helper_messages;
pub mod message;

fn topic_buttons<L, T>(topics: L, is_personalized: bool) -> InlineKeyboardMarkup
where
  T: AsRef<str>,
  L: AsRef<[(T, bool)]>,
{
  let mut buttons: Vec<Vec<_>> = Vec::new();
  for pair in topics.as_ref().chunks(2) {
    let row = pair
      .as_ref()
      .iter()
      .map(|(topic, is_subscribed)| {
        let topic_button = if *is_subscribed {
          format!("{} ✓", topic.as_ref())
        } else {
          format!("+ {}", topic.as_ref())
        };
        let topic = if is_personalized {
          format!("{TOPIC_WRAPPER}{}{TOPIC_WRAPPER}", topic.as_ref())
        } else {
          topic.as_ref().to_string()
        };
        InlineKeyboardButton::callback(topic_button, topic)
      })
      .collect();
    buttons.push(row);
  }
  buttons.push(build_footer());
  InlineKeyboardMarkup::new(buttons)
}

fn build_footer() -> Vec<InlineKeyboardButton> {
  vec![InlineKeyboardButton::callback(
    String::from("Create"),
    String::from(NEW_TOPIC),
  )]
}

fn remove_message(bot: Bot, msg: Message) {
  tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(BOT_MSG_TTL)).await;
    _ = bot.delete_message(msg.chat.id, msg.id).await;
  });
}
