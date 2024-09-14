mod handlers;
mod rooms;
mod commands;
mod utils;

use std::sync::Arc;
use teloxide::prelude::*;
use crate::rooms::Rooms;
use crate::commands::Command;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
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