use crate::clientstate::ClientState;
use crate::team::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

pub static TANK_MAX_SPEED: f32 = 60.0;
pub static TANK_ACCEL: f32 = 100.0;
pub static TANK_TURN_SPEED: f32 = 45.0;
pub static TANK_FRICTION: f32 = 0.96;

pub static TANK_WIDTH: f32 = 20.0;
pub static TANK_HEIGHT: f32 = 20.0;

/// Cooldown in seconds for how long a tank must wait between shots.
pub static TANK_SHOT_COOLDOWN: f64 = 1.0;

/// Velocity in pixels per second for a tank bullet to travel
pub static TANK_BULLET_VELOCITY: f32 = 300.0;

/// The maximum allowed bounces for each bullet, inclusive.
pub static TANK_BULLET_BOUNCE_COUNT_MAX: i32 = 3;

/// The radius of the tank bullet.
pub static TANK_BULLET_RADIUS: f32 = 5.0;

//TODO: create a tank bullet struct, has an x, y, xvel, yvel, and maybe a reference to its owner? or just the owners uuid.
//  The bullet hits a tank, and the tank will be reset to a random position? unsure. The bullet has a function called check collide, which takes in the client list,
//  and checks its point distance from each tank on the map on each frame to find if it is to hit a tank.
//  It also has a function called step, which moves its location based on its xvel and yvel.

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TankGameState {
    pub red_score: i32,
    pub blue_score: i32,
    pub bullets: Vec<TankBullet>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TankClientState {
    pub rotation: f32,
    pub tank_x: f32,
    pub tank_y: f32,
    pub tank_x_vel: f32,
    pub tank_y_vel: f32,
    pub last_shot_time: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TankBullet {
    pub x: f32,
    pub y: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub bounce_count: i32,
    pub team: Team,
}

impl Default for TankClientState {
    fn default() -> Self {
        TankClientState {
            rotation: 0.0,
            tank_x: 0.0,
            tank_y: 0.0,
            tank_x_vel: 0.0,
            tank_y_vel: 0.0,
            last_shot_time: SystemTime::now(),
        }
    }
}

/// respawn_tank takes in a mutable TankClientState, and
pub fn respawn_tank(
    tank_client_state: &mut TankClientState,
    _bullets: &Vec<TankBullet>,
    _clients: &HashMap<String, ClientState>,
) {
    // TODO: write function that takes in list of all bullets, list of all clients, and finds a location furthest from all of them.
    tank_client_state.tank_x = 0.0;
    tank_client_state.tank_y = 0.0;
}
