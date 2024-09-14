use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
  #[command(description = "Avvia il bot")]
  Start,
  #[command(description = "Mostra questo testo di aiuto")]
  Help,
  #[command(description = "Crea una stanza")]
  Create,
  #[command(description = "Unisciti a una stanza. Uso: /join <codice>")]
  Join(String),
  #[command(description = "Scegli il tuo personaggio. Uso: /character <nome>")]
  Character(String),
  #[command(description = "Inizia la partita (solo host)")]
  Play,
  #[command(description = "Assegna i personaggi casualmente (solo host)")]
  Startgame,
  #[command(description = "Termina la partita (solo host)")]
  End,
  #[command(description = "Mostra lo stato delle stanze")]
  Info,
}