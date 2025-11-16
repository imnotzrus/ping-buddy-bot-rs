//! Message handler utilities and routing.

use crate::handler::ui::messages::MessageResponse;
use crate::storage::Storage;
use crate::traits::ChatIdExt;
use teloxide::types::Message;

pub mod dynamic;
pub mod r#static;

/// List all users subscribed to a topic, excluding the requesting user
///
/// Returns appropriate message based on subscription status
pub(crate) async fn list_users<T, U>(
  storage: Storage,
  msg: &Message,
  topic: T,
  user: U,
) -> crate::Result<MessageResponse>
where
  T: AsRef<str>,
  U: AsRef<str>,
{
  let data = storage.read().await;
  let topic_ref = topic.as_ref();
  let user_ref = user.as_ref();

  match data
    .get_subscribers_from_topic(msg.cid(), topic_ref)
    .await?
  {
    Some(users) => {
      if users.is_empty() {
        Ok(MessageResponse::no_one(topic_ref))
      } else {
        // Filter out the requesting user
        let other_users: Vec<_> =
          users.into_iter().filter(|u| u != user_ref).collect();

        if other_users.is_empty() {
          Ok(MessageResponse::no_one_except_sender(topic_ref))
        } else {
          Ok(MessageResponse::users(other_users))
        }
      }
    }
    None => Ok(MessageResponse::no_one(topic_ref)),
  }
}
