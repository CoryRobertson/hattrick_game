use crate::clientstate::ClientState;
use crate::gametypes::GameType;
use crate::gametypes::GameType::{PONG, TANK};
use crate::pong::{PongClientState, PongGameState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use crate::get_vote_count_for_number;
use crate::tank::{TankClientState, TankGameState};

pub static VOTE_TIME: f32 = 10.0;

/// GameState holds the game type, system time, and list of players. This is the single struct that is sent to each client every frame of gameplay.
/// Examples of things that go in GameState are things that need to be known by literally all clients, and the server, at the same time for gameplay to work properly.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub time: SystemTime,
    pub game_type: GameType,
    pub client_list: HashMap<String, ClientState>,
    pub vote_running: bool,
    pub vote_start_time: Option<SystemTime>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SystemTime: {:?}, player count: {}",
            self.time,
            self.client_list.len()
        )
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            time: SystemTime::now(),
            game_type: PONG(PongGameState::default()),
            client_list: Default::default(),
            vote_running: false,
            vote_start_time: None,
        }
    }
}

impl GameState {
    pub fn try_conclude_vote(&mut self) {
        if self.vote_running {
            println!("vote running");
            if let Some(vote_start_time) = self.vote_start_time {
                println!("vote time {}", SystemTime::now().duration_since(vote_start_time).unwrap().as_secs_f32());
                if SystemTime::now().duration_since(vote_start_time).unwrap().as_secs_f32() >= VOTE_TIME {
                    let possible_outcomes = vec![PONG(PongGameState::default()), TANK(TankGameState::default())]; // pong = 1, tank = 2, etc = 3..

                    // let pong_votes = get_vote_count_for_number(1,&self);
                    // let tank_votes = get_vote_count_for_number(2,&self);
                    let (_vote_count,voted_game_type) = possible_outcomes.into_iter()
                        .enumerate()
                        .map(|(index,game_type)| { // map each game type with its respective vote count as well.
                            (get_vote_count_for_number((index + 1) as u8, &self), game_type)
                        })
                        .reduce(|acc, new_game| {
                            if new_game.0 > acc.0 {
                                new_game // if new game vote count is higher than acc vote count, return the new game vote count
                            } else {
                                acc // else return the acc game
                            }
                        }).unwrap();

                    println!("won game type: {:?}, {}", voted_game_type,_vote_count);

                    self.game_type = voted_game_type;
                    self.vote_start_time = None;
                    for (_,client) in &mut self.client_list {
                        // probably not the best way to reset all the game modes, but I am happy with it for now.
                        client.vote_number = 0;
                        client.tank_client_state = TankClientState::default();
                        client.pong_client_state = PongClientState::default();
                    }

                }
            } else {
                self.vote_start_time = Some(SystemTime::now());
            }
        }
    }
}
