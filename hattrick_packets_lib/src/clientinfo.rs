use crate::keystate::KeyState;
use crate::team::Team;
use crate::team::Team::BlueTeam;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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
