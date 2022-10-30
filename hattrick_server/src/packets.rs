use crate::packets::packets::GameState;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

pub mod packets {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
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
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.time)
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            time: SystemTime::now(),
            x: 0.0,
            y: 0.0,
            client_list: Default::default(),
        }
    }
}
