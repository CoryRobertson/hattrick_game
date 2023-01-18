use crate::keystate::KeyState;
use crate::pong::PongClientState;
use crate::tank::TankClientState;
use crate::team::Team;
use crate::team::Team::BlueTeam;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// ClientState is a struct that the server generates using the data given from each client in the form of a ClientInfo struct. This separation allows for good programming ergonomics.
/// It also allows the server to be able to make decisions to ignore specific client info, if it is not possible, for example, if a client info packet says blue team, and then red, then blue again, it is
/// unlikely that the server should allow it, and because of this interpretation, could be stopped. This is not a feature at the moment but is an idea of why this distinction was made.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub time: SystemTime,
    pub team_id: Team,
    pub mouse_pos: (f32, f32),
    pub key_state: KeyState,
    pub pong_client_state: PongClientState,
    pub tank_client_state: TankClientState,
    pub vote_number: u8,
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

impl Default for ClientState {
    fn default() -> Self {
        ClientState {
            time: SystemTime::now(),
            // pos: (0.0, 0.0),
            team_id: BlueTeam,
            mouse_pos: (0.0, 0.0),
            key_state: KeyState::default(),
            pong_client_state: Default::default(),
            tank_client_state: TankClientState::default(),
            vote_number: 0,
        }
    }
}
