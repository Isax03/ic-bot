use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use teloxide::utils::markdown::escape;
use crate::rooms::{Rooms, Room, Player, RoomStatus};
use crate::commands::Command;
use crate::utils::{generate_code, assign_characters};

pub async fn handle_command(
  bot: Bot,
  msg: Message,
  cmd: Command,
  rooms: Rooms,
) -> ResponseResult<()> {
  match cmd {
    Command::Start => start(bot, msg).await,
    Command::Help => help(bot, msg).await,
    Command::Create => create(bot, msg, rooms).await,
    Command::Join(code) => join(bot, msg, rooms, code).await,
    Command::Character(character) => set_character(bot, msg, rooms, character).await,
    Command::Play => play(bot, msg, rooms).await,
    Command::Startgame => start_game(bot, msg, rooms).await,
    Command::End => end_game(bot, msg, rooms).await,
    Command::Info => info(bot, msg, rooms).await,
  }
}

async fn start(bot: Bot, msg: Message) -> ResponseResult<()> {
  bot
    .send_message(
      msg.chat.id,
      "Ma salve\\! Vuoi giocare a *Indovina Chi*\\?\nBeh questo è il posto giusto😉\n\nPuoi esplorare i comandi usando /help\nBuon divertimento\\!\\!")
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
  Ok(())
}

async fn help(bot: Bot, msg: Message) -> ResponseResult<()> {
  bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
  Ok(())
}

async fn create(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let username = from.username.clone().unwrap_or_default();

  let mut rooms = rooms.lock().await;
  if rooms.values().any(|room| room.players.contains_key(&user_id)) {
    bot.send_message(msg.chat.id, "You are already in a room.").await?;
    return Ok(());
  }

  let code = generate_code();
  let mut room = Room::new(user_id);
  room.players.insert(user_id, Player {
    username: username.clone(),
    character: None,
    assigned_character: None,
  });
  rooms.insert(code.clone(), room);

  let response = format!(
    "Stanza creata\\!\nIl codice è: `{}`",
    escape(&code)
  );

  bot.send_message(msg.chat.id, response)
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
  Ok(())
}

async fn join(bot: Bot, msg: Message, rooms: Rooms, code: String) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let username = from.username.clone().unwrap_or_default();

  let mut rooms = rooms.lock().await;
  if let Some(room) = rooms.get_mut(&code) {
    if room.players.contains_key(&user_id) {
      bot.send_message(msg.chat.id, "You are already in this room.").await?;
    } else {
      room.players.insert(user_id, Player {
        username,
        character: None,
        assigned_character: None,
      });
      bot.send_message(msg.chat.id, format!("You have joined room {}.", code)).await?;
    }
  } else {
    bot.send_message(msg.chat.id, "Invalid room code.").await?;
  }
  Ok(())
}

async fn set_character(bot: Bot, msg: Message, rooms: Rooms, character: String) -> ResponseResult<()> {
  let user_id = msg.from.unwrap().id.0;

  let mut rooms = rooms.lock().await;
  for room in rooms.values_mut() {
    if let Some(player) = room.players.get_mut(&user_id) {
      if room.status != RoomStatus::Started {
        bot.send_message(msg.chat.id, "You can't set a character before the game has started.").await?;
      } else if player.character.is_some() {
        bot.send_message(msg.chat.id, "You have already set a character.").await?;
      } else {
        player.character = Some(character);
        bot.send_message(msg.chat.id, "Character set successfully.").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "You are not in any room.").await?;
  Ok(())
}

async fn play(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let user_id = msg.from.unwrap().id.0;

  let mut rooms = rooms.lock().await;
  for (_, room) in rooms.iter_mut() {
    if room.host == user_id {
      if room.players.len() < 2 {
        bot.send_message(msg.chat.id, "There must be at least 2 players to start the game.").await?;
      } else if room.status == RoomStatus::Waiting {
        room.status = RoomStatus::Started;
        bot.send_message(msg.chat.id, "The game has started! All players must choose a character.").await?;
      } else {
        bot.send_message(msg.chat.id, "The game has already started.").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "You are not the host of any room.").await?;
  Ok(())
}

async fn start_game(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let user_id = msg.from.unwrap().id.0;

  let mut rooms = rooms.lock().await;
  for (_, room) in rooms.iter_mut() {
    if room.host == user_id {
      if room.players.values().any(|p| p.character.is_none()) {
        bot.send_message(msg.chat.id, "All players must choose a character before starting the game.").await?;
      } else {
        assign_characters(room);
        for (player_id, player) in &room.players {
          let assigned_character = player.assigned_character.as_ref().unwrap();
          let message = format!("Your assigned character: {}\n\nOther players:\n{}",
                                assigned_character,
                                room.players.iter()
                                  .filter(|&(id, _)| id != player_id)
                                  .map(|(_, p)| format!("{} must guess {}", p.username, p.assigned_character.as_ref().unwrap()))
                                  .collect::<Vec<_>>()
                                  .join("\n")
          );
          bot.send_message(ChatId(*player_id as i64), message).await?;
        }
        bot.send_message(msg.chat.id, "The game has started! Assignments have been sent.").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "You are not the host of any room.").await?;
  Ok(())
}

async fn end_game(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let user_id = msg.from.unwrap().id.0;

  let mut rooms = rooms.lock().await;
  if let Some(code) = rooms.iter().find_map(|(code, room)| if room.host == user_id { Some(code.clone()) } else { None }) {
    rooms.remove(&code);
    bot.send_message(msg.chat.id, "The game has ended and the room has been closed.").await?;
  } else {
    bot.send_message(msg.chat.id, "You are not the host of any room.").await?;
  }
  Ok(())
}

async fn info(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let rooms = rooms.lock().await;

  if rooms.is_empty() {
    bot.send_message(msg.chat.id, "There are no active rooms at the moment.")
      .await?;
    return Ok(());
  }

  let info = rooms.iter().map(|(code, room)| {
    let host_username = room.players.get(&room.host).map(|p| p.username.clone()).unwrap_or_default();
    let players_info = room.players.iter()
      .map(|(_, p)| format!("{}", escape(&p.username)))
      .collect::<Vec<_>>()
      .join(", ");
    format!("Room: `{}`\nHost: {}\nPlayers: {}\nStatus: {:?}",
            escape(code), escape(&host_username), players_info, room.status)
  }).collect::<Vec<_>>().join("\n\n");

  bot
    .send_message(msg.chat.id, info)
    .parse_mode(ParseMode::MarkdownV2)
    .await?;

  Ok(())
}