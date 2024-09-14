use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
  #[command(description = "Start the bot")]
  Start,
  #[command(description = "Display this help message")]
  Help,
  #[command(description = "Create a new room")]
  Create,
  #[command(description = "Join a room")]
  Join(String),
  #[command(description = "Choose your character")]
  Character(String),
  #[command(description = "Start the game")]
  Play,
  #[command(description = "Assign characters randomly and start the game")]
  Startgame,
  #[command(description = "End the game")]
  End,
  #[command(description = "Display game information")]
  Info,
}