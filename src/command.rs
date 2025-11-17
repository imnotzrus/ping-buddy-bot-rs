//! Bot command definitions.
//!
//! This module defines all static commands that the bot responds to.

use teloxide::macros::BotCommands;

/// Available bot commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
  /// List topics
  List,
  /// Tag everyone
  All,
}
