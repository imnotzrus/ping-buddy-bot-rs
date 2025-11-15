/// Handler for /list command - shows all topics with subscription status
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ReplyParameters};

use crate::handler::message::{ChatIdExtractImpl, UserRefExtractImpl};
use crate::handler::topic_buttons;
use crate::storage::Storage;
use crate::utils::some_rtn_ok;
use crate::Bot;
use crate::Result;

pub async fn handle(bot: Bot, msg: Message, storage: Storage) -> Result {
    let user = some_rtn_ok!(msg.from.as_ref().map(|u| u.user_ref()));
    let data = storage.read().await;
    
    let all_topics = some_rtn_ok!(data.get_topics(msg.cid()));
    
    if all_topics.is_empty() {
        bot.send_message(msg.chat.id, "No topics available yet. Create one!")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }
    
    let subscriptions = data
        .get_topics_from_subscriber(msg.cid(), &user)
        .unwrap_or_default();
    
    let modified_topics: Vec<_> = all_topics
        .iter()
        .map(|t| (t.as_str(), subscriptions.contains(t)))
        .collect();
    
    log::debug!(
        "User {} requested topic list in chat {} ({} topics)",
        user,
        msg.cid(),
        modified_topics.len()
    );
    
    bot.send_message(msg.chat.id, "Your subscriptions")
        .reply_markup(topic_buttons(modified_topics, true))
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    
    Ok(())
}
