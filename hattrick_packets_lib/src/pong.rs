use crate::clientstate::ClientState;
use crate::team::Team;
use crate::team::Team::{BlueTeam, RedTeam};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static PONG_PADDLE_WIDTH: f32 = 100.0;
pub static PONG_PADDLE_HEIGHT: f32 = 10.0;
pub static PONG_BALL_RADIUS: f32 = 15.0;
pub static PONG_BALL_VEL_ADD_MAX: f32 = 5.0;
pub static PONG_BALL_VEL_ADD_MIN: f32 = 0.1;
pub static BLUE_TEAM_PADDLE_Y: f32 = 10.0;
pub static RED_TEAM_PADDLE_Y: f32 = 550.0;

pub static PADDLE_MOVE_SPEED: f32 = 0.5; // 0.5 seems to feel pretty good a the moment

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

//TODO: probably replace PONG_BALL_VEL_ADD_MAX and PONG_BALL_VEL_ADD_MIN with functions that take in current velocity, so we can cap the velocity?

pub fn get_pong_paddle_width(client_list: &HashMap<String, ClientState>, team: &Team) -> f32 {
    match team {
        RedTeam => {
            let mut red_count = 0;
            for client in client_list {
                let c = client.1;
                if c.team_id == RedTeam {
                    red_count += 1;
                }
            }
            PONG_PADDLE_WIDTH / red_count as f32
        }
        BlueTeam => {
            let mut blue_count = 0;
            for client in client_list {
                let c = client.1;
                if c.team_id == BlueTeam {
                    blue_count += 1;
                }
            }
            PONG_PADDLE_WIDTH / blue_count as f32
        }
    }
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

impl Default for PongClientState {
    fn default() -> Self {
        PongClientState {
            paddle_x: 0.0,
            paddle_y: 0.0,
        }
    }
}
