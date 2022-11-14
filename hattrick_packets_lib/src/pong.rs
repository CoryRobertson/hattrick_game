use serde::{Deserialize, Serialize};

/// PongGameState is an example game type struct that holds all the data for the game mode, it should contain anything related to the game-type of its parent, in this case Pong.
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
pub struct PongClientState {
    pub paddle_x: f32,
    pub paddle_y: f32,
}
