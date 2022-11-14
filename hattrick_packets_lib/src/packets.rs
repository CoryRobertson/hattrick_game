use crate::gamestate::GameState;
use crate::gametypes::GameType::PONG;
use crate::gametypes::GameTypeClient;
use crate::packets::Team::BlueTeam;
use crate::pong::{PongClientState, PongGameState};
use macroquad::prelude::KeyCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

/// Team is the team selection enumeration that the player can choose, at the moment, it is stored on the client and sent to server. This might be changed later :)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Team {
    RedTeam,
    BlueTeam,
}

/// Client info is the struct that each client creates, serializes, and sends to the server, it is not meant to be used directly to store client data, but to be interpreted.
/// Example, client sends a ClientInfo that has mouse position of x: 150.0, y: 300.0, system time is irrelevant here, and team id is blue.
/// For pong, blue means the client is bound to the top of the screen, meaning that his y value of his mouse is ignored and instead set to where ever his team is meant to be, and the x is only used for his position.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub time: SystemTime,
    pub mouse_pos: (f32, f32),
    pub team_id: Team,
    pub key_state: KeyState,
}

/// KeyState is a struct that contains all keys that the game listens to.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyState {
    pub w_key: bool,
    pub a_key: bool,
    pub s_key: bool,
    pub d_key: bool,
    pub space_bar: bool,
}

impl KeyState {
    /// new() by default returns a key state with all inputs checked from the keyboard.
    pub fn new() -> KeyState {
        KeyState {
            w_key: macroquad::prelude::is_key_down(KeyCode::W),
            a_key: macroquad::prelude::is_key_down(KeyCode::A),
            s_key: macroquad::prelude::is_key_down(KeyCode::S),
            d_key: macroquad::prelude::is_key_down(KeyCode::D),
            space_bar: macroquad::prelude::is_key_down(KeyCode::Space),
        }
    }
    /// default() initialized all states to false.
    pub fn default() -> KeyState {
        KeyState {
            w_key: false,
            a_key: false,
            s_key: false,
            d_key: false,
            space_bar: false,
        }
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "W:{}, A:{}, S:{}, D:{}, SPACE:{}",
            self.w_key, self.a_key, self.s_key, self.d_key, self.space_bar
        )
    }
}

/// ClientState is a struct that the server generates using the data given from each client in the form of a ClientInfo struct. This separation allows for good programming ergonomics.
/// It also allows the server to be able to make decisions to ignore specific client info, if it is not possible, for example, if a client info packet says blue team, and then red, then blue again, it is
/// unlikely that the server should allow it, and because of this interpretation, could be stopped. This is not a feature at the moment but is an idea of why this distinction was made.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub time: SystemTime,
    pub team_id: Team,
    pub mouse_pos: (f32, f32),
    pub key_state: KeyState,
    pub game_type_info: GameTypeClient,
}

/// Default for a GameType's state struct is gonna be the starting point for that given game, in this case, the pong game starts with these values.
impl Default for PongGameState {
    fn default() -> Self {
        PongGameState {
            ball_x: 50.0,
            ball_y: 50.0,
            ball_xvel: 5.0,
            ball_yvel: 5.0,
            red_points: 0,
            blue_points: 0,
        }
    }
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

impl ClientState {
    /// Takes self, and another ClientState is input, and counts how many differing struct variables there are,
    /// at the moment, this function is extremely subjective, and probably shouldn't be used for much.
    pub fn differ_count(&self, cs: &ClientState) -> i32 {
        let mut count = 0;

        // if self.pos.0 != cs.pos.0 {
        //     count += 1;
        // }
        //
        // if self.pos.1 != cs.pos.1 {
        //     count += 1;
        // }

        if self.time != cs.time {
            count += 1;
        }

        if self.team_id != cs.team_id {
            count += 1;
        }

        count
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

/// Probably shouldn't ever use a default client info, unless the deserialization fails?
impl Default for ClientInfo {
    fn default() -> Self {
        ClientInfo {
            time: SystemTime::now(),
            mouse_pos: (0.0, 0.0),
            team_id: BlueTeam,
            key_state: KeyState::default(),
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        ClientState {
            time: SystemTime::now(),
            // pos: (0.0, 0.0),
            team_id: BlueTeam,
            mouse_pos: (0.0, 0.0),
            key_state: KeyState::default(),
            game_type_info: GameTypeClient::PONG(PongClientState {
                paddle_x: 0.0,
                paddle_y: 0.0,
            }),
        }
    }
}
