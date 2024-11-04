use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
  /// List topics
  List,
  /// Tag everyone
  All,
}
