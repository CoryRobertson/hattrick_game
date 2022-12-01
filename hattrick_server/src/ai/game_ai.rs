use crate::ai::pong_ai::get_pong_state_for_ai;
use crate::GameStateRW;
use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::clientstate::ClientState;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::keystate::KeyState;
use hattrick_packets_lib::pong::PongClientState;
use hattrick_packets_lib::tank::TankClientState;
use hattrick_packets_lib::team::Team;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

/// Delay in milliseconds for how long to wait between ai ticks.
static AI_TICK_DELAY_MS: u64 = 8;

/// This function takes in the game state arc mutex, the running state arc mutex, an ip address, and the team to connect to and joins the given ip game server.
/// It will mutate the game state each frame by locking the mutex. To stop the connection thread, set the running state to false. This thread also concludes when connection is lost.
pub fn spawn_ai_thread(
    game_state: GameStateRW,
    running: Arc<Mutex<bool>>,
    team_id: Team,
    name: String,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut local_gs: GameState = { game_state.read().unwrap().clone() };
        println!("connected");
        let name = name;
        let mut previous_pcs = PongClientState::default();
        let mut _previous_tcs = TankClientState::default();
        // TODO: eventually generate a random number to be used for each ai.
        //  Each ai would have a "seed" that tells them specific things about their gameplay. For example a seed could determine the offset they play with their paddle in pong,
        //  or how close they let the ball get before they stop moving.
        loop {
            // Do ai logic on one of these three lines
            let mut client_packet = ClientInfo {
                time: SystemTime::now(),
                mouse_pos: {
                    // TODO: this is gonna have to be refactored into each ai specific source file. But I will do that once it is needed. Alternatively the ai function could modify it so we dont have to use this at all.
                    let x = match &local_gs.game_type {
                        GameType::PONG(pgs) => pgs.ball_x,
                        GameType::TANK(_) => 0.0,
                    };
                    (x, 0.0)
                },
                team_id: team_id.clone(),
                key_state: KeyState {
                    w_key: false,
                    a_key: false,
                    s_key: false,
                    d_key: false,
                    space_bar: false,
                },
            };
            let pcs: PongClientState =
                get_pong_state_for_ai(&team_id, &local_gs, &mut client_packet, &previous_pcs); // use an ai function to make this pong client state

            let tcs: TankClientState = TankClientState::default();

            // clone previous state just incase we want to act upon our previous actions.
            previous_pcs = pcs.clone();
            _previous_tcs = tcs.clone();

            let client_state: ClientState = ClientState {
                time: client_packet.time,
                mouse_pos: client_packet.mouse_pos,
                key_state: client_packet.key_state,
                pong_client_state: pcs, // use modified pong client state
                tank_client_state: tcs, // use modified tank client state
                team_id: client_packet.team_id,
            };

            {
                let mut lock = game_state.write().unwrap();
                local_gs = lock.clone();
                lock.client_list.insert(name.clone(), client_state);
            } // update the servers game state from this ai

            sleep(Duration::from_millis(AI_TICK_DELAY_MS)); // ai tick rate, probably can be pretty slow

            if !(*running.lock().unwrap()) {
                // stop ai if our running lock is off.
                break;
            }
        }
        *running.lock().unwrap() = false;
        println!("ai thread finished");
    })
}
