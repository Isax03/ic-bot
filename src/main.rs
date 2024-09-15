mod handlers;
mod rooms;
mod commands;
mod utils;

use std::sync::Arc;
use colored::Colorize;
use teloxide::prelude::*;
use crate::rooms::Rooms;
use crate::commands::Command;

#[tokio::main]
async fn main() {
  pretty_env_logger::init();
  colored::control::set_override(true);
  log::info!("Starting bot...");

  let bot = Bot::from_env();

  let user_id = std::env::var("MAIN_USER_ID").unwrap().parse::<i64>().unwrap();

  println!("{}", "!!! Il bot è stato avviato !!!".bright_green().bold());
  bot.send_message(ChatId(user_id), "Il bot è stato avviato✅").await.unwrap();

  let rooms: Rooms = Arc::new(tokio::sync::Mutex::new(Default::default()));

  let handler = Update::filter_message()
    .branch(
      dptree::entry()
        .filter_command::<Command>()
        .endpoint(handlers::handle_command),
    );

  Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![rooms])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}