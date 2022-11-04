use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub time: SystemTime,
    pub x: f64,
    pub y: f64,
    pub client_list: HashMap<String, ClientState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub time: SystemTime,
    pub mouse_pos: (f32, f32),
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}, cords: {}, {}, client list size: {}",
            self.time,
            self.x,
            self.y,
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

        count
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            time: SystemTime::now(),
            x: 0.,
            y: 0.,
            client_list: Default::default(),
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        ClientState {
            time: SystemTime::now(),
            mouse_pos: (0.0, 0.0),
        }
    }
}
