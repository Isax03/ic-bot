use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Room {
  pub host: UserId,
  pub players: HashMap<UserId, Player>,
  pub status: RoomStatus,
}

#[derive(Clone, Debug)]
pub struct Player {
  pub username: String,
  pub character: Option<String>,
  pub assigned_character: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoomStatus {
  Waiting,
  Started,
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
pub type UserId = i64;

impl Room {
  pub fn new(host: UserId) -> Self {
    Room {
      host,
      players: HashMap::new(),
      status: RoomStatus::Waiting,
    }
  }
}