use crate::clientstate::ClientState;
use crate::team::Team;
use crate::team::Team::{BlueTeam, RedTeam};
use crate::{GAME_HEIGHT, GAME_WIDTH};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub static PONG_PADDLE_WIDTH: f32 = 100.0;
pub static PONG_PADDLE_HEIGHT: f32 = 10.0;
pub static PONG_BALL_RADIUS: f32 = 15.0;
pub static PONG_BALL_VEL_ADD_MAX: f32 = 5.0;
pub static PONG_BALL_VEL_ADD_MIN: f32 = 0.1;
pub static BLUE_TEAM_PADDLE_Y: f32 = 10.0;
pub static RED_TEAM_PADDLE_Y: f32 = 550.0;
pub static PONG_POINTS_TO_WIN: i32 = 10;

pub static PADDLE_MOVE_SPEED: f32 = 0.5; // 0.5 seems to feel pretty good a the moment

pub static POWER_HIT_MODIFIER: f32 = 1.5; // velocity multiplier for added velocity on each paddle bounce for ball
pub static POWER_HIT_LOCK_TIME: f32 = 1.0; // how long to lock a paddle in place when a power hit is initiated
pub static POWER_HIT_COOLDOWN: f32 = 2.0;

/// PongGameState is an example game type struct that holds all the data for the game mode, it should contain anything related to the game-type of its parent, in this case Pong.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PongGameState {
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_xvel: f32,
    pub ball_yvel: f32,
    pub red_points: i32,
    pub blue_points: i32,
    pub ball_last_team_hit: Team,
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
    pub time_of_power_hit: SystemTime,
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
            ball_last_team_hit: BlueTeam,
        }
    }
}

impl PongGameState {
    /// Steps the physics of the ball in pong using the delta time from the previous step, difference should be measured in seconds since last step.
    pub fn step_ball(&mut self, difference: &f32) {
        let ball_radius = PONG_BALL_RADIUS;

        // blue team top of screen, red team bottom
        // ball physics multiplied by delta time since last "frame" allows us to run game speed  independent of application run speed.
        self.ball_x += (self.ball_xvel * difference) * 16.0; // magic number multiplier
        self.ball_y += (self.ball_yvel * difference) * 16.0;

        {
            if self.ball_x < 0.0 + ball_radius {
                // left screen wall
                self.ball_xvel *= -1.0;
            }

            if self.ball_x > GAME_WIDTH - ball_radius {
                // right screen wall
                self.ball_xvel *= -1.0;
            }

            if self.ball_y > GAME_HEIGHT - ball_radius {
                // ball hits bottom screen wall
                let default_xvel = PongGameState::default().ball_xvel;
                let default_yvel = PongGameState::default().ball_yvel;
                self.ball_xvel = {
                    if self.ball_xvel < 0.0 {
                        -default_xvel
                    } else {
                        default_xvel
                    }
                };
                self.ball_yvel = -default_yvel;
                self.blue_points += 1;
                self.ball_y = GAME_HEIGHT - ball_radius;
                self.ball_last_team_hit = RedTeam;
                #[cfg(debug_assertions)]
                println!(
                    "blue points scored with ball xvel: {} and ball yvel: {}",
                    self.ball_xvel, self.ball_yvel
                );
            }

            if self.ball_y < 0.0 + ball_radius {
                // ball hits top screen wall
                let default_xvel = PongGameState::default().ball_xvel;
                let default_yvel = PongGameState::default().ball_yvel;
                self.ball_xvel = {
                    if self.ball_xvel < 0.0 {
                        -default_xvel
                    } else {
                        default_xvel
                    }
                };
                self.ball_yvel = default_yvel;
                self.red_points += 1;
                self.ball_last_team_hit = BlueTeam;
                self.ball_y = 0.0 + ball_radius;
                #[cfg(debug_assertions)]
                println!(
                    "red points scored with ball xvel: {} and ball yvel: {}",
                    self.ball_xvel, self.ball_yvel
                );
            }
        } // bounce checks for ball on walls
    }

    /// Steps the pong game state using the client list, checkins the ball collision on each players paddle. Function does not move ball, nor paddles.
    /// Instead the function changes the velocities of the ball in accordance.
    pub fn step_game_state(&mut self, client_list: &HashMap<String, ClientState>) {
        let ball_radius = PONG_BALL_RADIUS;

        for client in client_list {
            let cs = client.1;

            let cx = cs.pong_client_state.paddle_x; // client x
            let cy = {
                if cs.team_id == BlueTeam {
                    cs.pong_client_state.paddle_y + ball_radius
                } else {
                    cs.pong_client_state.paddle_y - ball_radius
                }
            }; // client y after taking into account the ball radius, cheap way to do it i know :)
            let cw = get_pong_paddle_width(client_list, &cs.team_id); // client width
            let ch = PONG_PADDLE_HEIGHT; // client height

            if (self.ball_last_team_hit != cs.team_id)
                && (self.ball_y > cy && self.ball_y < cy + ch)
                && (self.ball_x > cx && self.ball_x < cx + cw)
            {
                // first expression is height check for bouncing, second expression is lefty and righty check for bouncing
                match &cs.team_id {
                    RedTeam => {
                        // if client is red we make sure the yvel is set to a negative number.
                        self.ball_yvel = -(self.ball_yvel.abs());
                    }
                    BlueTeam => {
                        // if the client is blue we set the yvel to a positive number.
                        self.ball_yvel = self.ball_yvel.abs();
                    }
                } // match block to determine which direction to send the ball in on a collision

                // variables used to randomly add some amount of velocity when a bounce happens on a paddle.
                let rand_xvel_change: f32 =
                    rand::thread_rng().gen_range(PONG_BALL_VEL_ADD_MIN..PONG_BALL_VEL_ADD_MAX); // generate a random new x velocity change for when a bounce needs to occur
                let rand_yvel_change: f32 =
                    rand::thread_rng().gen_range(PONG_BALL_VEL_ADD_MIN..PONG_BALL_VEL_ADD_MAX); // generate a random new y velocity change for when a bounce needs to occur

                // this statement adds the correct direction of velocity, it adds new velocity in the direction of travel already.
                if self.ball_xvel > 0.0 {
                    // if ball hits paddle, add a random amount of x velocity to the ball, in the direction it is currently traveling
                    if cs.key_state.space_bar {
                        self.ball_xvel += rand_xvel_change * POWER_HIT_MODIFIER;
                    } else {
                        self.ball_xvel += rand_xvel_change;
                    }
                } else if cs.key_state.space_bar {
                    self.ball_xvel -= rand_xvel_change * POWER_HIT_MODIFIER;
                } else {
                    self.ball_xvel -= rand_xvel_change;
                }

                if self.ball_yvel > 0.0 {
                    // ditto from comment above
                    if cs.key_state.space_bar {
                        self.ball_yvel += rand_yvel_change * 2.0;
                    } else {
                        self.ball_yvel += rand_yvel_change;
                    }
                } else if cs.key_state.space_bar {
                    self.ball_yvel -= rand_yvel_change * 2.0;
                } else {
                    self.ball_yvel -= rand_yvel_change;
                }

                self.ball_last_team_hit = cs.team_id.clone(); // set the last ball team hit to this clients team id, making it so multi hits on the same paddle can't occur.

                #[cfg(debug_assertions)]
                println!(
                    "bounced with: new xvel ({}), new yvel ({}): {} {}",
                    rand_xvel_change, rand_yvel_change, self.ball_xvel, self.ball_yvel
                );
            } // bounce checks for ball on paddles of clients
        } // client loop for game state
    }
}

impl Default for PongClientState {
    fn default() -> Self {
        PongClientState {
            paddle_x: 0.0,
            paddle_y: 0.0,
            time_of_power_hit: UNIX_EPOCH,
        }
    }
}
