/// Handler module for processing bot updates
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::constants::{BOT_MSG_TTL, NEW_TOPIC_CALLBACK, TOPIC_WRAPPER};
use crate::Bot;

pub mod callback;
pub mod helper_messages;
pub mod message;

/// Constant for new topic callback
pub(crate) const NEW_TOPIC: &str = NEW_TOPIC_CALLBACK;

/// Build inline keyboard buttons for topic subscription
/// 
/// # Arguments
/// * `topics` - List of (topic_name, is_subscribed) tuples
/// * `is_personalized` - Whether to wrap topics with TOPIC_WRAPPER for personalized callbacks
fn topic_buttons<L, T>(topics: L, is_personalized: bool) -> InlineKeyboardMarkup
where
    T: AsRef<str>,
    L: AsRef<[(T, bool)]>,
{
    let mut buttons: Vec<Vec<_>> = Vec::new();
    
    // Create rows of 2 buttons each
    for pair in topics.as_ref().chunks(2) {
        let row = pair
            .iter()
            .map(|(topic, is_subscribed)| {
                let topic_button = if *is_subscribed {
                    format!("{} ✓", topic.as_ref())
                } else {
                    format!("+ {}", topic.as_ref())
                };
                let callback_data = if is_personalized {
                    format!("{TOPIC_WRAPPER}{}{TOPIC_WRAPPER}", topic.as_ref())
                } else {
                    topic.as_ref().to_string()
                };
                InlineKeyboardButton::callback(topic_button, callback_data)
            })
            .collect();
        buttons.push(row);
    }
    
    // Add footer with "Create" button
    buttons.push(build_footer());
    InlineKeyboardMarkup::new(buttons)
}

/// Build footer row with "Create new topic" button
fn build_footer() -> Vec<InlineKeyboardButton> {
    vec![InlineKeyboardButton::callback(
        "Create New Topic",
        NEW_TOPIC_CALLBACK,
    )]
}

/// Schedule a message for deletion after BOT_MSG_TTL seconds
/// 
/// This is used for temporary messages that should auto-delete
fn remove_message(bot: Bot, msg: Message) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(BOT_MSG_TTL)).await;
        if let Err(e) = bot.delete_message(msg.chat.id, msg.id).await {
            log::warn!(
                "Failed to delete message {} in chat {}: {}",
                msg.id,
                msg.chat.id,
                e
            );
        } else {
            log::debug!("Successfully deleted message {} in chat {}", msg.id, msg.chat.id);
        }
    });
}
