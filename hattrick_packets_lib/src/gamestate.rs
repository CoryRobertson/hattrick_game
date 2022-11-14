use crate::gametypes::GameType;
use crate::packets::ClientState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// GameState holds the game type, system time, and list of players. This is the single struct that is sent to each client every frame of gameplay.
/// Examples of things that go in GameState are things that need to be known by literally all clients, and the server, at the same time for gameplay to work properly.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub time: SystemTime,
    pub game_type: GameType,
    pub client_list: HashMap<String, ClientState>,
}
