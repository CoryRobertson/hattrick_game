use serde::{Deserialize, Serialize};

pub static TANK_MOVE_SPEED: f32 = 1.0;
pub static TANK_TURN_SPEED: f32 = 2.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TankGameState {
    pub red_score: i32,
    pub blue_score: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TankClientState {
    pub rotation: f32,
    pub tank_x: f32,
    pub tank_y: f32,
}

impl Default for TankGameState {
    fn default() -> Self {
        TankGameState {
            red_score: 0,
            blue_score: 0,
        }
    }
}

impl Default for TankClientState {
    fn default() -> Self {
        TankClientState {
            rotation: 0.0,
            tank_x: 0.0,
            tank_y: 0.0,
        }
    }
}
