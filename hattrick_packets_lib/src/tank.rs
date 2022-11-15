use serde::{Deserialize, Serialize};

pub static TANK_MAX_SPEED: f32 = 6.0;
pub static TANK_ACCEL: f32 = 0.2;
pub static TANK_TURN_SPEED: f32 = 2.0;

pub static TANK_WIDTH: f32 = 20.0;
pub static TANK_HEIGHT: f32 = 20.0;

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
    pub tank_x_vel: f32,
    pub tank_y_vel: f32,
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
            tank_x_vel: 0.0,
            tank_y_vel: 0.0,
        }
    }
}
