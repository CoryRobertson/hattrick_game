use crate::clientstate::ClientState;
use crate::team::Team;
use crate::{distance, GAME_HEIGHT, GAME_WIDTH};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// keep toying with different values, have not found something i like quite yet.
pub static TANK_MAX_SPEED: f32 = 60.0;
pub static TANK_ACCEL: f32 = 500.0;
pub static TANK_TURN_SPEED: f32 = 45.0;
pub static TANK_FRICTION: f32 = 0.96;

pub static TANK_WIN_SCORE: i32 = 10;

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
            rotation: rand::thread_rng().gen_range(0.0..360.0),
            tank_x: rand::thread_rng().gen_range(0.0..GAME_WIDTH),
            tank_y: rand::thread_rng().gen_range(0.0..GAME_HEIGHT),
            tank_x_vel: 0.0,
            tank_y_vel: 0.0,
            last_shot_time: UNIX_EPOCH,
        }
    }
}

impl TankGameState {
    /// Removes all bullets in the game state that have >= the bounce limit each
    pub fn remove_dead_bullets(&mut self) {
        for index in 0..self.bullets.len() {
            if let Some(bullet) = self.bullets.get(index) {
                if bullet.bounce_count >= TANK_BULLET_BOUNCE_COUNT_MAX {
                    self.bullets.remove(index);
                }
            }
        }
    }
}

/// respawn_tank takes in a mutable TankClientState, and
pub fn respawn_tank(
    tank_client_state: &mut TankClientState,
    _bullets: &[TankBullet],
    _clients: &HashMap<String, ClientState>,
) {
    let position: (f32, f32, f32) = (0..10) // generate 10 random positions to potentially respawn the player
        .into_iter()
        .map(|_| {
            // map with _ because we dont care about the individual numbers.
            // rand x and y values
            let rx = rand::thread_rng().gen_range(0.0..GAME_WIDTH);
            let ry = rand::thread_rng().gen_range(0.0..GAME_HEIGHT);

            // from those random x and y values, generate the distance to the closest tank in the game
            let closest_tank_dist = _clients
                .iter()
                .map(|(_, client_state)| {
                    // map only each client state, as we dont care  about their uuid s
                    distance(
                        client_state.tank_client_state.tank_x,
                        client_state.tank_client_state.tank_y,
                        rx,
                        ry,
                    ) // map the distance from the random x and y to the every given tank x and y
                })
                .reduce(|acc, dist| if dist < acc { dist } else { acc }) // keep only the lowest value of distance, meaning drop all values that are greater than the smallest value
                .unwrap();
            (rx, ry, closest_tank_dist) // map the x and y position and the closest tank distance to each x and y position
        })
        .reduce(|(acc_x, acc_y, acc_dist), (pos_x, pos_y, pos_dist)| {
            if acc_dist > pos_dist {
                (acc_x, acc_y, acc_dist)
            } else {
                (pos_x, pos_y, pos_dist)
            } // now instead of keeping the lowest, keep the highest distance value, meaning the furthest possible point from all tanks, from a specific number of randomly generated points.
        })
        .unwrap();

    // move tanks new position
    tank_client_state.tank_x = position.0;
    tank_client_state.tank_y = position.1;

    // zero out the tanks velocity
    tank_client_state.tank_x_vel = 0.0;
    tank_client_state.tank_y_vel = 0.0;

    // randomize the tanks rotation so its a little different every time
    tank_client_state.rotation = rand::thread_rng().gen_range(0.0..360.0);

    // set their last shot time to unix epoch so they can shoot immediately no matter what
    tank_client_state.last_shot_time = UNIX_EPOCH;
}
