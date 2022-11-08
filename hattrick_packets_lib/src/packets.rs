use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use crate::packets::GameType::PONG;
use crate::packets::Team::BlueTeam;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub time: SystemTime,
    // pub x: f64,
    // pub y: f64,
    pub game_type: GameType,
    pub client_list: HashMap<String, ClientState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameType {
    PONG(PongGameState),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Team {
    RedTeam,
    BlueTeam,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PongGameState {
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_xvel: f32,
    pub ball_yvel: f32,
    pub red_points: i32,
    pub blue_points: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub time: SystemTime,
    pub mouse_pos: (f32, f32),
    pub team_id: Team,
}

impl Default for PongGameState {
    fn default() -> Self {
        PongGameState{
            ball_x: 50.0,
            ball_y: 50.0,
            ball_xvel: 5.0,
            ball_yvel: 5.0,
            red_points: 0,
            blue_points: 0
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}, cords: {}",
            self.time,
            // self.x,
            // self.y,
            self.client_list.len()
        )
    }
}

impl ClientState {
    /// Takes self, and another ClientState is input, and counts how many differing struct variables there are, at the moment, this function is extremely subjective, and probably shouldn't be used for much.
    pub fn differ_count(&self, cs: &ClientState) -> i32 {
        let mut count = 0;

        if self.mouse_pos.0 != cs.mouse_pos.0 {
            count += 1;
        }

        if self.mouse_pos.1 != cs.mouse_pos.1 {
            count += 1;
        }

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
            // x: 0.,
            // y: 0.,
            game_type: PONG(PongGameState::default()),
            client_list: Default::default(),
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        ClientState {
            time: SystemTime::now(),
            mouse_pos: (0.0, 0.0),
            team_id: BlueTeam,
        }
    }
}
