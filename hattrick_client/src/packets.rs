use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use crate::packets::packets::GameState;

pub mod packets {
    use std::time::SystemTime;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameState {

        pub time: SystemTime,
        pub x: f64,
        pub y: f64,

    }


    pub struct ClientState {

        pub time: SystemTime,

    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self.time)
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState{ time: SystemTime::now(), x: 0., y: 0. }
    }
}