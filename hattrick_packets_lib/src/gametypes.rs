use crate::pong::{PongClientState, PongGameState};
use crate::tank::{TankClientState, TankGameState};
use serde::{Deserialize, Serialize};

/// GameType is the game mode that is being played, for example pong, each game mode contains a struct within the enumeration that stores the games data like any objects the game should render
/// On top of that, the GameType is to be pattern matched for each frame, allowing the workflow of adding new game types to be really easy :)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameType {
    PONG(PongGameState),
    TANK(TankGameState),
}

/// GameTypeClient is a enum for client states to hold onto that contain the given game type as well as the variables that are specific to that game type.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameTypeClient {
    PONG(PongClientState),
    TANK(TankClientState),
}
