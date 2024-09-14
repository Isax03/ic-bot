use rand::Rng;
use crate::rooms::Room;

pub fn generate_code() -> String {
  rand::thread_rng()
    .sample_iter(&rand::distributions::Alphanumeric)
    .take(6)
    .map(char::from)
    .collect()
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