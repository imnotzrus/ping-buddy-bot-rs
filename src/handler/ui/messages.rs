//! User-facing message templates.
//!
//! This module contains all message templates sent to users,
//! ensuring consistent formatting and tone across the bot.

use std::fmt::Display;

/// Collection of message templates used throughout the bot
pub struct Messages;

impl Messages {
  /// Message sent when the bot joins a group
  pub fn bot_join() -> String {
    String::from(
      "Hello, I am your *Ping Buddy*\nI'm here to help you ping people\n\n*Please subscribe to default topic*",
    )
  }

  /// Message sent to welcome new members to the group
  pub fn welcome_new_member<M>(member: M) -> String
  where
    M: Display,
  {
    format!(
      "Hi {member}, welcome to the group!\n*Please subscribe to topic you want*",
    )
  }

  /// Message prompting user to provide a topic name
  pub fn ask_topic<U>(user: U) -> String
  where
    U: Display,
  {
    format!("What *topic* do you want to create, {user}?\n\n\\(Please *reply* this message\\)")
  }

  /// Message when no one is subscribed to a topic
  pub fn no_one_in_topic<T>(topic: T) -> String
  where
    T: Display,
  {
    format!("There is no one in topic `{topic}` yet :\\(")
  }

  /// Message when only the sender is subscribed to a topic
  pub fn no_one_except_sender<T>(topic: T) -> String
  where
    T: Display,
  {
    format!("No one but you subscribed topic `{topic}`")
  }

  /// Message listing users subscribed to a topic
  pub fn list_users<L, U>(users: L) -> String
  where
    L: AsRef<[U]>,
    U: AsRef<str>,
  {
    users
      .as_ref()
      .iter()
      .map(AsRef::as_ref)
      .collect::<Vec<_>>()
      .join(" ")
  }

  /// Message for invalid topic format errors
  pub fn invalid_topic_format() -> String {
    String::from("Invalid topic format, try another one :\\(")
  }

  /// Message for invalid topic name errors
  pub fn invalid_topic<T>(reason: T) -> String
  where
    T: Display,
  {
    format!("`{reason}` is not a valid topic, try another one :\\(")
  }

  /// Message for invalid or expired requests
  pub fn invalid_request() -> String {
    String::from("This request is no longer valid.")
  }

  /// Message confirming user subscription to a topic
  pub fn user_subscribed_topic<T, U>(user: U, topic: T) -> String
  where
    T: Display,
    U: Display,
  {
    format!("{user} subscribed topic `{topic}`")
  }
}

/// Return type for topic ping responses
pub enum MessageResponse {
  /// No one subscribed to the topic
  NoOne(String),
  /// Only the sender is subscribed
  NoOneExceptSender(String),
  /// List of subscribed users
  Users(String),
}

impl MessageResponse {
  /// Get the message text
  pub fn text(&self) -> &str {
    match self {
      MessageResponse::NoOne(text)
      | MessageResponse::NoOneExceptSender(text)
      | MessageResponse::Users(text) => text.as_str(),
    }
  }

  /// Check if this message should be auto-deleted
  pub fn should_remove(&self) -> bool {
    matches!(
      self,
      MessageResponse::NoOne(_) | MessageResponse::NoOneExceptSender(_)
    )
  }

  /// Create a "no one subscribed" response
  pub fn no_one<T>(topic: T) -> Self
  where
    T: Display,
  {
    Self::NoOne(Messages::no_one_in_topic(topic))
  }

  /// Create a "only sender subscribed" response
  pub fn no_one_except_sender<T>(topic: T) -> Self
  where
    T: Display,
  {
    Self::NoOneExceptSender(Messages::no_one_except_sender(topic))
  }

  /// Create a user list response
  pub fn users<L, U>(users: L) -> Self
  where
    L: AsRef<[U]>,
    U: AsRef<str>,
  {
    Self::Users(Messages::list_users(users))
  }
}
