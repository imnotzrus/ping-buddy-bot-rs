/// Handler for common messages (topic pings and topic creation)
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::handler::helper_messages::{
    InvalidRequest, InvalidTopic, InvalidTopicFormat, UserSubscribedTopic,
};
use crate::handler::message::{
    list_users, ChatIdExtractImpl, TopicExtractImpl, UserRefExtractImpl,
};
use crate::handler::remove_message;
use crate::storage::Storage;
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
            
            log::debug!("User {} pinging topic '{}' in chat {}", user, topic, msg.cid());
            
            let ret = list_users(storage, &msg, topic, user).await;
            let response_msg = bot
                .send_message(msg.chat.id, ret.value())
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            
            if ret.must_remove() {
                remove_message(bot, response_msg);
            }
        }
        Some(replied_msg) => {
            // Handle topic creation response
            let mut data = storage.write().await;
            
            if data.has_create_message_id(replied_msg.cid(), replied_msg.id.0) {
                let Some(topic) = msg.text() else {
                    bot.send_message(msg.chat.id, InvalidTopicFormat::msg())
                        .await?;
                    return Ok(());
                };

                // Validate topic name
                if let Err(validation_err) = validate_topic(topic) {
                    log::warn!("Invalid topic name '{}': {}", topic, validation_err);
                    bot.send_message(msg.chat.id, InvalidTopic::msg(validation_err.to_string()))
                        .await?;
                    return Ok(());
                }

                let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
                
                // Create topic and subscribe user
                if let Err(e) = data.set_topic_and_subscribers(msg.cid(), topic, [&user]) {
                    log::error!("Failed to create topic '{}': {}", topic, e);
                    bot.send_message(msg.chat.id, format!("Failed to create topic: {}", e))
                        .await?;
                    return Ok(());
                }
                
                data.pop_creating_message_id(replied_msg.cid(), replied_msg.id.0);
                
                log::info!("User {} created topic '{}' in chat {}", user, topic, msg.cid());
                
                bot.send_message(msg.chat.id, UserSubscribedTopic::msg(user, topic))
                    .await?;
                remove_message(bot, replied_msg.clone());
            } else {
                bot.send_message(msg.chat.id, InvalidRequest::msg())
                    .await?;
            }
        }
    }

    Ok(())
}
