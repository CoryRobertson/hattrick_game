use serde::{Deserialize, Serialize};

/// Team is the team selection enumeration that the player can choose, at the moment, it is stored on the client and sent to server. This might be changed later :)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Team {
    RedTeam,
    BlueTeam,
}
