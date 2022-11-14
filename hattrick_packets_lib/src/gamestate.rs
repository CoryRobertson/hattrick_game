use crate::clientstate::ClientState;
use crate::gametypes::GameType;
use crate::gametypes::GameType::PONG;
use crate::pong::PongGameState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

/// GameState holds the game type, system time, and list of players. This is the single struct that is sent to each client every frame of gameplay.
/// Examples of things that go in GameState are things that need to be known by literally all clients, and the server, at the same time for gameplay to work properly.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub time: SystemTime,
    pub game_type: GameType,
    pub client_list: HashMap<String, ClientState>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SystemTime: {:?}, player count: {}",
            self.time,
            self.client_list.len()
        )
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            time: SystemTime::now(),
            game_type: PONG(PongGameState::default()),
            client_list: Default::default(),
        }
    }
}
