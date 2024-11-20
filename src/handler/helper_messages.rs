use std::fmt::Display;

pub struct BotJoinMessage;
impl BotJoinMessage {
  pub fn msg() -> String {
    String::from(
      "Hello, I am your *Ping Buddy*\nI'm here to help you ping people\n\n*Please subscribe to default topic*",
    )
  }
}

pub struct WelcomeNewMemberMessage;
impl WelcomeNewMemberMessage {
  pub fn msg<M>(mem: M) -> String
  where
    M: Display,
  {
    format!(
      "Hi {mem}, welcome to the group!\n*Please subscribe to topic you want*",
    )
  }
}

pub struct AskTopic;
impl AskTopic {
  pub fn msg<U>(user: U) -> String
  where
    U: Display,
  {
    format!("What *topic* do you want to create, {user}?\n\n\\(Please *reply* this message\\)")
  }
}

pub enum Return {
  NoOne(String),
  NoOneExcept(String),
  Users(String),
}
impl Return {
  pub fn value(&self) -> &str {
    match self {
      Return::NoOne(text) | Return::NoOneExcept(text) | Return::Users(text) => {
        text.as_str()
      }
    }
  }

  pub fn must_remove(&self) -> bool {
    match self {
      Return::NoOne(_) | Return::NoOneExcept(_) => true,
      Return::Users(_) => false,
    }
  }

  pub fn no_one<T>(topic: T) -> Self
  where
    T: Display,
  {
    Self::NoOne(format!("There is no one in topic `{topic}` yet :\\("))
  }

  pub fn no_one_except_sender<T>(topic: T) -> Self
  where
    T: Display,
  {
    Self::NoOneExcept(format!("No one but you subscribed topic `{topic}`"))
  }

  pub fn users<L, U>(users: L) -> Self
  where
    L: AsRef<[U]>,
    U: AsRef<str>,
  {
    let users = users
      .as_ref()
      .iter()
      .map(AsRef::as_ref)
      .collect::<Vec<_>>()
      .join(" ");
    Self::Users(users)
  }
}

pub struct InvalidTopicFormat;
impl InvalidTopicFormat {
  pub fn msg() -> String {
    String::from("Invalid topic format, try another one :\\(")
  }
}

pub struct InvalidTopic;
impl InvalidTopic {
  pub fn msg<T>(topic: T) -> String
  where
    T: Display,
  {
    format!("`{topic}` is not a valid topic, try another one :\\(")
  }
}

pub struct InvalidRequest;
impl InvalidRequest {
  pub fn msg() -> String {
    String::from("This request is no longer valid.")
  }
}

pub struct UserSubscribedTopic;
impl UserSubscribedTopic {
  pub fn msg<T, U>(user: U, topic: T) -> String
  where
    T: Display,
    U: Display,
  {
    format!("{user} subscribed topic `{topic}`",)
  }
}
