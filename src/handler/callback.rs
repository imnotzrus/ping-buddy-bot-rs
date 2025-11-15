/// Handler for inline keyboard button callbacks
use teloxide::payloads::EditMessageReplyMarkupSetters;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;

use crate::constants::TOPIC_WRAPPER;
use crate::handler::helper_messages::{AskTopic, UserSubscribedTopic};
use crate::handler::message::{ChatIdExtractImpl, UserRefExtractImpl};
use crate::handler::{topic_buttons, NEW_TOPIC};
use crate::storage::Storage;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, query: CallbackQuery, storage: Storage) -> Result {
    // Acknowledge the callback to remove loading state
    bot.answer_callback_query(&query.id).await?;
    
    let msg = some_rtn_ok!(query.regular_message());
    let user = query.from.user_ref();
    
    // Verify user authorization if this is a reply
    if let Some(replied) = msg.reply_to_message() {
        let user_ref = some_rtn_ok!(replied.from.as_ref().map(|u| u.user_ref()));
        if user != user_ref {
            log::warn!("Unauthorized callback attempt by {} for message from {}", user, user_ref);
            return Ok(());
        }
    }

    let topic = some_rtn_ok!(query.data.as_ref());
    let mut data = storage.write().await;
    
    match topic.as_str() {
        NEW_TOPIC => {
            // Handle "Create New Topic" button
            log::debug!("User {} initiated topic creation in chat {}", user, msg.cid());
            let response = bot.send_message(msg.chat.id, AskTopic::msg(&user)).await?;
            data.push_creating_message_id(response.cid(), response.id.0);
        }
        topic if topic.starts_with(TOPIC_WRAPPER) && topic.ends_with(TOPIC_WRAPPER) => {
            // Handle personalized topic toggle (subscribe/unsubscribe)
            let topic = topic
                .trim_start_matches(TOPIC_WRAPPER)
                .trim_end_matches(TOPIC_WRAPPER);
            
            let was_subscribed = data.is_subscriber_in_topic(msg.cid(), &user, topic);
            
            if was_subscribed {
                if let Err(e) = data.unset_subscriber_from_topic(msg.cid(), topic, &user) {
                    log::error!("Failed to unsubscribe {} from topic '{}': {}", user, topic, e);
                }
                log::info!("User {} unsubscribed from topic '{}' in chat {}", user, topic, msg.cid());
            } else {
                if let Err(e) = data.set_topic_and_subscribers(msg.cid(), topic, [&user]) {
                    log::error!("Failed to subscribe {} to topic '{}': {}", user, topic, e);
                    bot.send_message(msg.chat.id, format!("Failed to subscribe: {}", e))
                        .await?;
                    return Ok(());
                }
                log::info!("User {} subscribed to topic '{}' in chat {}", user, topic, msg.cid());
            }
            
            // Update the inline keyboard to reflect new subscription state
            let all_topics = some_rtn_ok!(data.get_topics(msg.cid()));
            let subscriptions = data
                .get_topics_from_subscriber(msg.cid(), &user)
                .unwrap_or_default();
            
            let modified_topics: Vec<_> = all_topics
                .iter()
                .map(|t| (t.as_str(), subscriptions.iter().any(|s| s == t)))
                .collect();
            
            if let Err(e) = bot
                .edit_message_reply_markup(msg.chat.id, msg.id)
                .reply_markup(topic_buttons(modified_topics, true))
                .await
            {
                log::warn!("Failed to update keyboard: {}", e);
            }
        }
        topic => {
            // Handle direct topic subscription (from welcome message)
            log::debug!("User {} subscribing to topic '{}' in chat {}", user, topic, msg.cid());
            
            if let Err(e) = data.set_topic_and_subscribers(msg.cid(), topic, [&user]) {
                log::error!("Failed to subscribe {} to topic '{}': {}", user, topic, e);
                bot.send_message(msg.chat.id, format!("Failed to subscribe: {}", e))
                    .await?;
                return Ok(());
            }
            
            bot.send_message(msg.chat.id, UserSubscribedTopic::msg(&user, topic))
                .await?;
        }
    }
    
    Ok(())
}
