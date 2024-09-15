use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Room {
  pub host: u64,
  pub players: HashMap<u64, Player>,
  pub status: RoomStatus,
}

#[derive(Clone, Debug, Serialize)]
pub struct Player {
  pub username: String,
  pub character: Option<String>,
  pub assigned_character: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum RoomStatus {
  Waiting,
  Started,
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;

impl Room {
  pub fn new(host: u64) -> Self {
    Room {
      host,
      players: HashMap::new(),
      status: RoomStatus::Waiting,
    }
  }
}