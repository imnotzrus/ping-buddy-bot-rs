//! Extension traits for User types.

use teloxide::types::User;

/// Create a Telegram user reference (clickable mention)
pub trait UserRefExt {
  /// Generate a markdown-formatted user reference link
  fn user_ref(&self) -> String;
}

impl UserRefExt for User {
  fn user_ref(&self) -> String {
    let user_name =
      self.username.as_deref().unwrap_or(self.first_name.as_str());
    format!("[{user_name}](tg://user?id={})", self.id.0)
  }
}
