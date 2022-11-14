use serde::{Deserialize, Serialize};

pub static PONG_PADDLE_WIDTH: f32 = 100.0;
pub static PONG_PADDLE_HEIGHT: f32 = 10.0;
pub static PONG_BALL_RADIUS: f32 = 15.0;

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
