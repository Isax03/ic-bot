use std::collections::HashMap;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, User};
use teloxide::utils::command::BotCommands;
use teloxide::utils::markdown::escape;
use crate::rooms::{Rooms, Room, Player, RoomStatus};
use crate::commands::Command;
use crate::utils::{generate_code, assign_characters};
use colored::*;
use serde::Serialize;

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
    Command::Join(code) => {
      if code.trim().is_empty() {
        bot.send_message(msg.chat.id, "Per favore, specifica un codice stanza. Uso: /join <codice>").await?;
        Ok(())
      } else {
        join(bot, msg, rooms, code).await
      }
    },
    Command::Leave => leave(bot, msg, rooms).await,
    Command::Character(character) => {
      if character.trim().is_empty() {
        bot.send_message(msg.chat.id, "Per favore, specifica un personaggio. Uso: /character <nome>").await?;
        Ok(())
      } else {
        set_character(bot, msg, rooms, character).await
      }
    },
    Command::Play => play(bot, msg, rooms).await,
    Command::Startgame => start_game(bot, msg, rooms).await,
    Command::End => end_game(bot, msg, rooms).await,
    Command::Info => info(bot, msg, rooms).await,
  }
}

fn get_display_name(user: &User) -> String {
  user.username.clone().map(|u| format!("@{}", u)).unwrap_or_else(|| user.first_name.clone())
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
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  if rooms.values().any(|room| room.players.contains_key(&user_id)) {
    bot.send_message(msg.chat.id, "Sei già in una stanza").await?;
    return Ok(());
  }

  let code = generate_code();
  let mut room = Room::new(user_id);
  room.players.insert(user_id, Player {
    username: display_name.clone(),
    character: None,
    assigned_character: None,
  });
  rooms.insert(code.clone(), room);

  let response = format!(
    "Stanza creata\\!\nIl codice è: `{}`",
    escape(&code)
  );

  println!("{} {} ha creato la stanza {}", "CREATED:".green().bold(), display_name.green().italic(), code.green().italic());

  bot.send_message(msg.chat.id, response)
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
  Ok(())
}

async fn join(bot: Bot, msg: Message, rooms: Rooms, code: String) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  if rooms.values().any(|room| room.players.contains_key(&user_id)) {
    bot.send_message(msg.chat.id, "Sei già in un'altra stanza").await?;
  } else if let Some(room) = rooms.get_mut(&code) {
    if room.players.contains_key(&user_id) {
      bot.send_message(msg.chat.id, "Sei già in questa stanza").await?;
    } else {
      room.players.insert(user_id, Player {
        username: display_name.clone(),
        character: None,
        assigned_character: None,
      });
      // Notifica tutti i giocatori eccetto il nuovo arrivato
      for player_id in room.players.keys() {
        if player_id != &user_id {
          bot.send_message(ChatId(*player_id as i64), format!("{} è entrato nella stanza", display_name)).await?;
        }
      }
      bot.send_message(msg.chat.id, format!("Sei entrato nella stanza {}", code)).await?;
      println!("{} {} è entrato nella stanza {}", "JOINED:".blue().bold(), display_name.blue().italic(), code.blue().italic());
    }
  } else {
    bot.send_message(msg.chat.id, format!("Nessuna stanza con codice {} trovata", code)).await?;
  }
  Ok(())
}

async fn leave(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  if let Some(code) = rooms.iter().find_map(|(code, room)| if room.players.contains_key(&user_id) { Some(code.clone()) } else { None }) {
    if let Some(room) = rooms.get_mut(&code) {
      println!("{} {} ha lasciato la stanza {}", "LEFT:".yellow().bold(), display_name.yellow().italic(), code.yellow().italic());
      room.players.remove(&user_id);
      for player_id in room.players.keys() {
        bot.send_message(ChatId(*player_id as i64), format!("{} ha lasciato la stanza", display_name)).await?;
      }
      if room.host == user_id {
        if let Some((&new_host, _)) = room.players.iter().next() {
          room.host = new_host;
          for player_id in room.players.keys() {
            bot.send_message(ChatId(*player_id as i64), format!("{} è il nuovo host della stanza", room.players.get(&new_host).unwrap().username)).await?;
          }
        }
      }
      if room.players.is_empty() {
        println!("{} La stanza {} è vuota ed è stata chiusa", "CLOSED:".red().bold(), code.red().italic());
        rooms.remove(&code);
      }
    }
    bot.send_message(msg.chat.id, "Hai lasciato la stanza").await?;
  } else {
    bot.send_message(msg.chat.id, "Non sei in nessuna stanza").await?;
  }
  Ok(())
}

async fn set_character(bot: Bot, msg: Message, rooms: Rooms, character: String) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  for (code, room) in rooms.iter_mut() {
    if let Some(player) = room.players.get_mut(&user_id) {
      if room.status != RoomStatus::Started {
        bot.send_message(msg.chat.id, "Non puoi scegliere un personaggio prima che la partita sia iniziata").await?;
      } else if player.character.is_some() {
        bot.send_message(msg.chat.id, "Hai già scelto un personaggio").await?;
      } else {
        player.character = Some(character.clone());
        println!("{} {} ha scelto il personaggio {} nella stanza {}", "CHARACTER:".purple().bold(), display_name.purple().italic(), character.purple().italic(), code.purple().italic());
        bot.send_message(msg.chat.id, "Personaggio impostato con successo").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "Non sei in nessuna stanza").await?;
  Ok(())
}

async fn play(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  for (code, room) in rooms.iter_mut() {
    if room.host == user_id {
      if room.players.len() < 2 {
        bot.send_message(msg.chat.id, "Ci devono essere almeno 2 giocatori per preparare la partita").await?;
      } else if room.status == RoomStatus::Waiting {
        room.status = RoomStatus::Started;
        println!("{} {} ha iniziato la preparazione del gioco nella stanza {}", "PLAY:".yellow().bold(), display_name.yellow().italic(), code.yellow().italic());
        for player_id in room.players.keys() {
          bot.send_message(ChatId(*player_id as i64), "Il gioco sta per cominciare! Tutti i giocatori devono scegliere un personaggio").await?;
        }
      } else {
        bot.send_message(msg.chat.id, "Il gioco è già stato inizializzato").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "Non sei l'host di nessuna stanza").await?;
  Ok(())
}

async fn start_game(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  for (code, room) in rooms.iter_mut() {
    if room.host == user_id {
      if room.players.values().any(|p| p.character.is_none()) {
        let missing_players: Vec<_> = room.players.values()
          .filter(|p| p.character.is_none())
          .map(|p| p.username.clone())
          .collect();
        let message = format!(
          "Tutti i giocatori devono scegliere un personaggio prima di iniziare il gioco. Mancano: {}",
          missing_players.join(", ")
        );
        bot.send_message(msg.chat.id, message).await?;
      } else {
        assign_characters(room);
        for (player_id, _) in &room.players {
          let message = format!("Personaggi assegnati:\n\n{}",
                                room.players.iter()
                                  .filter(|&(id, _)| id != player_id)
                                  .map(|(_, p)| format!("{} -> {}", p.username, p.assigned_character.as_ref().unwrap()))
                                  .collect::<Vec<_>>()
                                  .join("\n")
          );
          bot.send_message(ChatId(*player_id as i64), message).await?;
        }
        println!("{} {} ha avviato il gioco nella stanza {}", "STARTGAME:".magenta().bold(), display_name.magenta().italic(), code.magenta().italic());
        bot.send_message(msg.chat.id, "Il gioco è iniziato! I personaggi sono stati assegnati").await?;
      }
      return Ok(());
    }
  }
  bot.send_message(msg.chat.id, "Non sei l'host di nessuna stanza").await?;
  Ok(())
}

async fn end_game(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let from = msg.from.unwrap();
  let user_id = from.id.0;
  let display_name = get_display_name(&from);

  let mut rooms = rooms.lock().await;
  if let Some((code, room)) = rooms.iter().find(|(_, room)| room.host == user_id) {
    let code = code.clone();

    // Notifica tutti i giocatori
    for (&player_id, player) in &room.players {
      if player_id != user_id {  // Esclude l'host
        let chat_id = ChatId(player_id as i64);
        if let Err(e) = bot.send_message(chat_id, format!("La stanza è stata chiusa da {}", display_name).to_string()).await {
          println!("{} Impossibile notificare il giocatore {}: {}", "ERROR:".red().bold(), player.username, e);
        }
      }
    }

    // Rimuovi la stanza
    rooms.remove(&code);

    println!("{} {} ha terminato il gioco nella stanza {}", "END:".red().bold(), display_name.red().italic(), code.red().italic());
    bot.send_message(msg.chat.id, "Il gioco è finito! La stanza è stata chiusa").await?;
  } else {
    bot.send_message(msg.chat.id, "Non sei l'host di nessuna stanza").await?;
  }
  Ok(())
}
async fn info(bot: Bot, msg: Message, rooms: Rooms) -> ResponseResult<()> {
  let rooms = rooms.lock().await;

  if rooms.is_empty() {
    bot.send_message(msg.chat.id, "Non ci sono stanze attive al momento").await?;
    return Ok(());
  }

  let info = rooms.iter().map(|(code, room)| {
    let host_username = room.players.get(&room.host).map(|p| p.username.clone()).unwrap_or_default();
    let players_info = room.players.iter()
      .map(|(_, p)| format!("{}", escape(&p.username)))
      .collect::<Vec<_>>()
      .join(", ");
    format!("N° Stanza: `{}`\nHost: {}\nGiocatori: {}\nStato: {:?}",
            escape(code), escape(&host_username), players_info, room.status)
  }).collect::<Vec<_>>().join("\n\n");

  bot
    .send_message(msg.chat.id, info)
    .parse_mode(ParseMode::MarkdownV2)
    .await?;

  // Log dettagliato per debug
  let debug_info: Vec<DebugRoom> = rooms.iter().map(|(code, room)| {
    DebugRoom {
      code: code.clone(),
      host: room.host,
      players: room.players.iter().map(|(id, player)| {
        (id.to_string(), DebugPlayer {
          username: player.username.clone(),
          character: player.character.clone(),
          assigned_character: player.assigned_character.clone(),
        })
      }).collect(),
      status: room.status,
    }
  }).collect();

  println!("{} \n{}", "INFO:".cyan().bold(), serde_json::to_string_pretty(&debug_info).unwrap().cyan());

  Ok(())
}

#[derive(Serialize)]
struct DebugRoom {
  code: String,
  host: u64,
  players: HashMap<String, DebugPlayer>,
  status: RoomStatus,
}

#[derive(Serialize)]
struct DebugPlayer {
  username: String,
  character: Option<String>,
  assigned_character: Option<String>,
}