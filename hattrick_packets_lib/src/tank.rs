use crate::clientstate::ClientState;
use crate::team::Team;
use crate::{GAME_HEIGHT, GAME_WIDTH};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// keep toying with different values, have not found something i like quite yet.
pub static TANK_MAX_SPEED: f32 = 60.0;
pub static TANK_ACCEL: f32 = 500.0;
pub static TANK_TURN_SPEED: f32 = 45.0;
pub static TANK_FRICTION: f32 = 0.98;

pub static TANK_WIDTH: f32 = 20.0;
pub static TANK_HEIGHT: f32 = 20.0;

/// Cool down in seconds for how long a tank must wait between shots.
pub static TANK_SHOT_COOL_DOWN: f64 = 1.0;

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

impl TankBullet {
    /// Mutates bullet to add to its position and bounce off of walls,
    /// difference: &f32 is the difference in time between the last call of this function to allow for inconsistent processing speed.
    pub fn step(&mut self, difference: &f32) {
        if self.x >= GAME_WIDTH - TANK_BULLET_RADIUS || self.x <= 0.0 + TANK_BULLET_RADIUS {
            // if bullet x is out of the game screen
            self.x_vel *= -1.0;
            self.bounce_count += 1;
        }
        if self.y >= GAME_HEIGHT - TANK_BULLET_RADIUS || self.y <= 0.0 + TANK_BULLET_RADIUS {
            // if bullet y is out of the game screen
            self.y_vel *= -1.0;
            self.bounce_count += 1;
        }

        self.x += self.x_vel * difference;
        self.y += self.y_vel * difference;
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
