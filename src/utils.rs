use rand::Rng;
use crate::rooms::Room;

pub fn generate_code() -> String {
  let code = rand::thread_rng().gen_range(0..10000);
  format!("{:04}", code)
}

pub fn assign_characters(room: &mut Room) {
  let mut players: Vec<_> = room.players.iter_mut().collect();
  let n_players = players.len();
  let mut rng = rand::thread_rng();
  let shift = rng.gen_range(1..n_players);

  for i in 0..n_players {
    let assigned_index = (i + shift) % n_players;
    let assigned_character = players[assigned_index].1.character.clone();
    players[i].1.assigned_character = assigned_character;
  }
}