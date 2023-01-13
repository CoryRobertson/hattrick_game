use crate::ai::game_ai::spawn_ai_thread;
use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::clientstate::ClientState;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType::{PONG, TANK};
use hattrick_packets_lib::keystate::KeyState;
use hattrick_packets_lib::pong::{get_pong_paddle_width, PongClientState, BLUE_TEAM_PADDLE_Y, PADDLE_MOVE_SPEED, PONG_PADDLE_WIDTH, POWER_HIT_LOCK_TIME, RED_TEAM_PADDLE_Y, POWER_HIT_COOLDOWN};
use hattrick_packets_lib::tank::{
    respawn_tank, TankBullet, TankGameState, TANK_ACCEL, TANK_BULLET_BOUNCE_COUNT_MAX,
    TANK_BULLET_RADIUS, TANK_BULLET_VELOCITY, TANK_FRICTION, TANK_HEIGHT, TANK_MAX_SPEED,
    TANK_SHOT_COOL_DOWN, TANK_TURN_SPEED, TANK_WIDTH,
};
use hattrick_packets_lib::team::Team::BlueTeam;
use hattrick_packets_lib::team::Team::RedTeam;
use hattrick_packets_lib::{distance, round_digits, two_point_angle, GAME_WIDTH};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

mod ai;

/// Delay in milliseconds to wait between game loop, at the moment 1 seems to work just as good as anything lower than 16, but unsure for the most part.
static GAME_LOOP_THREAD_DELAY_MS: u64 = 1;

type GameStateRW = Arc<RwLock<GameState>>;

fn main() {
    println!("I am the server!");
    let server = TcpListener::bind("0.0.0.0:8111").unwrap();
    let game_state_rwl: GameStateRW = Arc::new(RwLock::new(GameState::default()));
    let ai_running = Arc::new(Mutex::new(true));
    let mut client_threads: Vec<JoinHandle<()>> = vec![];
    // game_state_rwl.write().unwrap().game_type = TANK(TankGameState::default());

    // A connection handling thread for receiving new clients.
    let connect_game_state = game_state_rwl.clone();
    let connect_thread = thread::spawn(move || {
        for response in server.incoming() {
            for i in 0..client_threads.len() {
                match client_threads.get(i) {
                    None => {}
                    Some(t) => {
                        if t.is_finished() {
                            client_threads.remove(i);
                        }
                    }
                }
            }
            if let Ok(r) = response {
                client_threads.push(handle_client(r, Arc::clone(&connect_game_state)));
            }
            println!("Client count: {}", client_threads.len());
        }
    });

    let game_thread = spawn_game_thread(Arc::clone(&game_state_rwl));

    sleep(Duration::from_secs(2));

    let mut ai_list: Vec<JoinHandle<()>> = vec![];

    for a in 0..1 {
        // number of ai to spawn
        let ai_name = format!("ai{}", a);
        let team = {
            if a % 2 == 0 {
                RedTeam
            } else {
                BlueTeam
            }
        };
        ai_list.push(spawn_ai_thread(
            Arc::clone(&game_state_rwl),
            Arc::clone(&ai_running),
            team,
            ai_name,
        ));
    }

    let _ = connect_thread.join();
    let _ = game_thread.join();

    *ai_running.lock().unwrap() = false; // stop ai after game thread has concluded
    for ai in ai_list {
        let _ = ai.join();
    }
}

/// This function spawns the game thread, that handles running the entire game server while reading the games state.
fn spawn_game_thread(game_state_rw: GameStateRW) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut previous_time = SystemTime::now(); // initialize the previous time with now.

        loop {
            let copy_gs = {
                let lock = game_state_rw.read().unwrap();
                lock.clone()
            }; // make a copy of the game state, so we can read it to make decisions for the game thread.
            let difference = {
                let d = SystemTime::now()
                    .duration_since(previous_time)
                    .unwrap()
                    .as_secs_f64() as f32;
                if d > 1.0 {
                    16.0 / 1000.0
                } else {
                    d
                }
            }; // difference in time between last game thread loop, useful to making things non-frame rate dependent on the server side.

            if !copy_gs.client_list.is_empty() {
                // basic game logic goes here
                match copy_gs.game_type {
                    PONG(mut pgs) => {
                        // step the physics of the ball
                        pgs.step_ball(&difference);
                        // step the game state using the clients
                        pgs.step_game_state(&copy_gs.client_list);

                        // update the game state that is on the server with the new game state that has been stepped.
                        {
                            let mut lock = game_state_rw.write().unwrap();
                            lock.game_type = PONG(pgs);
                            lock.time = SystemTime::now();
                        } // at the end of the game loop for pong, we add all the new data into the game state.
                    }

                    TANK(mut tgs) => {
                        let mut client_list = copy_gs.client_list.clone();

                        for mut client in &mut client_list {
                            let client_key_state = &client.1.key_state;
                            let x_ratio = {
                                let rad = client.1.tank_client_state.rotation.to_radians();
                                if rad.cos().is_nan() {
                                    0.0
                                } else {
                                    rad.cos()
                                }
                            };

                            let y_ratio = {
                                let rad = client.1.tank_client_state.rotation.to_radians();
                                if rad.sin().is_nan() {
                                    0.0
                                } else {
                                    rad.sin()
                                }
                            };

                            let current_speed = (client.1.tank_client_state.tank_x_vel.powi(2)
                                + client.1.tank_client_state.tank_y_vel.powi(2))
                            .sqrt();

                            if client_key_state.d_key {
                                client.1.tank_client_state.rotation += TANK_TURN_SPEED * difference;
                            }
                            if client_key_state.a_key {
                                client.1.tank_client_state.rotation -= TANK_TURN_SPEED * difference;
                            }
                            if client_key_state.w_key && current_speed < TANK_MAX_SPEED {
                                client.1.tank_client_state.tank_x_vel += TANK_ACCEL * difference;
                                client.1.tank_client_state.tank_y_vel += TANK_ACCEL * difference;
                            }
                            if client_key_state.s_key && -current_speed > -TANK_MAX_SPEED {
                                client.1.tank_client_state.tank_x_vel -= TANK_ACCEL * difference;
                                client.1.tank_client_state.tank_y_vel -= TANK_ACCEL * difference;
                            }

                            let last_shot_diff = SystemTime::now()
                                .duration_since(client.1.tank_client_state.last_shot_time)
                                .unwrap()
                                .as_secs_f64();

                            if client_key_state.space_bar && last_shot_diff > TANK_SHOT_COOL_DOWN {
                                client.1.tank_client_state.last_shot_time = SystemTime::now();
                                // println!("shot time: {:?}", last_shot_diff);

                                let tx = client.1.tank_client_state.tank_x;
                                let ty = client.1.tank_client_state.tank_y;

                                // the bullet xvel and yvel are added from TANK_WIDTH or TANK_HEIGHT /2 because we want to spawn the bullet from the middle of the tank, not the top left corner
                                // which is where its x and y coordinates lie.
                                let bullet_xvel = {
                                    let deg = two_point_angle(
                                        (tx + (TANK_WIDTH / 2.0), ty + (TANK_HEIGHT / 2.0)),
                                        client.1.mouse_pos,
                                    )
                                    .to_radians();
                                    if deg.cos().is_nan() {
                                        0.0
                                    } else {
                                        deg.cos() * TANK_BULLET_VELOCITY
                                    }
                                };

                                // see previous comments
                                let bullet_yvel = {
                                    let deg = two_point_angle(
                                        (tx + (TANK_WIDTH / 2.0), ty + (TANK_HEIGHT / 2.0)),
                                        client.1.mouse_pos,
                                    )
                                    .to_radians();
                                    if deg.sin().is_nan() {
                                        0.0
                                    } else {
                                        deg.sin() * TANK_BULLET_VELOCITY
                                    }
                                };

                                // see previous comments
                                tgs.bullets.push(TankBullet {
                                    x: tx + (TANK_WIDTH / 2.0),
                                    y: ty + (TANK_HEIGHT / 2.0),
                                    x_vel: bullet_xvel,
                                    y_vel: bullet_yvel,
                                    bounce_count: 0,
                                    team: client.1.team_id.clone(),
                                })
                            } // shoot bullet from a tank

                            client.1.tank_client_state.tank_x_vel *= TANK_FRICTION;
                            client.1.tank_client_state.tank_y_vel *= TANK_FRICTION;

                            if client.1.tank_client_state.tank_x_vel.abs() < 0.05
                                && client.1.tank_client_state.tank_y_vel.abs() < 0.05
                            {
                                client.1.tank_client_state.tank_x_vel = 0.0;
                                client.1.tank_client_state.tank_y_vel = 0.0;
                            } // if velocity is very small, make it 0 so there is no slow drifting for tanks.

                            client.1.tank_client_state.tank_x +=
                                (client.1.tank_client_state.tank_x_vel * difference) * x_ratio;
                            client.1.tank_client_state.tank_y +=
                                (client.1.tank_client_state.tank_y_vel * difference) * y_ratio;

                            round_digits(&mut client.1.tank_client_state.tank_x_vel, 4);
                            round_digits(&mut client.1.tank_client_state.tank_y_vel, 4);
                            round_digits(&mut client.1.tank_client_state.tank_x, 4);
                            round_digits(&mut client.1.tank_client_state.tank_y, 4);
                        } // input handling for clients

                        for bullet in &mut tgs.bullets {
                            bullet.step(&difference);
                        } // do physics for bullets

                        for index in 0..tgs.bullets.len() {
                            if let Some(bullet) = tgs.bullets.get(index) {
                                if bullet.bounce_count >= TANK_BULLET_BOUNCE_COUNT_MAX {
                                    tgs.bullets.remove(index);
                                }
                            }
                        } // remove all bullets that have more than the maximum allowed bounces for bullets.

                        // bad practice cloning happening here.
                        let copy_client_list = client_list.clone();
                        let copy_bullets_list = tgs.bullets.clone();
                        for index in 0..copy_bullets_list.len() {
                            if let Some(bullet) = copy_bullets_list.get(index) {
                                for client in &mut client_list {
                                    if distance(
                                        bullet.x,
                                        bullet.y,
                                        client.1.tank_client_state.tank_x,
                                        client.1.tank_client_state.tank_y,
                                    ) < TANK_BULLET_RADIUS + (TANK_WIDTH + TANK_HEIGHT) / 2.0
                                        && bullet.team != client.1.team_id
                                    {
                                        respawn_tank(
                                            &mut client.1.tank_client_state,
                                            &copy_bullets_list,
                                            &copy_client_list,
                                        );
                                        tgs.bullets.remove(index);
                                    }
                                }
                            }
                        } // check for bullet collision on clients, and remove bullet if collision occurs.

                        {
                            let mut lock = game_state_rw.write().unwrap();
                            lock.game_type = TANK(tgs);
                            lock.client_list = client_list;
                            lock.time = SystemTime::now();
                        } // at the end of the game loop where game mechanics run, we now move all the new data into the game state for the server.
                    }
                }
            } // only run the game loop if there are clients connected.

            previous_time = SystemTime::now(); // constantly update game system time for previous time. Useful for calculating the difference in time.

            sleep(Duration::from_millis(GAME_LOOP_THREAD_DELAY_MS)); // maybe remove this? at the moment unsure.
        } // loop to constantly update the game thread, see function comment for more info.
    })
}

/// This function handles a given client, it spawns a thread that will relay the game state to them, as well as take in their client info and insert that info into the game states client list under their uuid.
/// Each client is denoted by a random gen uuid.
/// The thread is closed and their client state is removed when they either disconnect, or the thread closes.
fn handle_client(stream: TcpStream, game_state_rw: GameStateRW) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut client_stream = stream;

        {
            client_stream
                .set_write_timeout(Option::from(Duration::from_secs(5)))
                .unwrap();

            client_stream
                .set_read_timeout(Option::from(Duration::from_secs(5)))
                .unwrap();
        } // set the read and write timeout for the client.

        let uuid = Uuid::new_v4().to_string();

        {
            let mut lock = game_state_rw.write().unwrap();
            lock.client_list.insert(
                uuid.to_string(),
                ClientState {
                    time: SystemTime::now(),
                    // pos: (0.0, 0.0),
                    team_id: BlueTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: KeyState::default(),

                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                },
            );
        } // block to add new client to client list with all default data.

        loop {
            // TODO: write logic that takes a timestamp when ever a write is successfully sent to a client, and if the last successful write happened more than 5 seconds ago, we can drop the client, otherwise keep waiting on them.
            //  alternatively, let a specific number of packets be dropped before dropping a client.
            let local_gs = {
                let lock = game_state_rw.read().unwrap();
                lock.clone()
            }; // lock read of local gs gets locked once we need to loop through the client handler once again.
            if let Ok(ser) = serde_json::to_string(&local_gs) {
                let write = client_stream.write(ser.as_bytes());
                let flush = client_stream.flush();
                let mut buf: [u8; 8192] = [0; 8192];
                let read = client_stream.read(&mut buf);
                let mut cleaned_buf = vec![];

                for value in buf {
                    // make small buffer of the data into a vector sent by the server
                    if !String::from_utf8_lossy(&[value]).contains('\0') {
                        cleaned_buf.push(value);
                    }
                }

                let clean = String::from_utf8(cleaned_buf).unwrap();
                match serde_json::from_str::<ClientInfo>(&clean) {
                    Ok(c) => {
                        // here we can decide if we want to do anything with the client state given if it is different enough,
                        // this would allow us to only take changes if they are large enough, compressing how often we have to lock the game state, if we decide to be threadsafe.

                        let mut local_gs = game_state_rw.write().unwrap();
                        /*
                        Lock the game state once we have received all data from the client,
                        at this point, all we need to do is manage it, so we can lock the entire game state now.
                        */

                        let prev_client = match local_gs.client_list.get(&*uuid) {
                            None => ClientState::default(),
                            Some(client) => client.clone(),
                        };

                        match &local_gs.game_type {
                            // depending on the game type, handle the clients info differently.
                            PONG(_pgs) => {
                                let client_y = {
                                    // set the clients y coordinate based on their team, top for blue, bottom for red
                                    match &c.team_id {
                                        BlueTeam => BLUE_TEAM_PADDLE_Y,
                                        RedTeam => RED_TEAM_PADDLE_Y,
                                    }
                                };

                                let previous_client_x = prev_client.pong_client_state.paddle_x;

                                // client x representing the clients paddle location, variable to be within the game width static variable.
                                let client_x;
                                // subtract half of the paddle width from the mouse position so we can center it on the players mouse,
                                // since drawing for this game lib draws from top left
                                let paddle_half_width =
                                    get_pong_paddle_width(&local_gs.client_list, &c.team_id) / 2.0;
                                let middle_of_paddle = c.mouse_pos.0 - paddle_half_width;

                                let time_since_last_power_hit = SystemTime::now()
                                    .duration_since(prev_client.pong_client_state.time_of_power_hit)
                                    .unwrap()
                                    .as_secs_f32();

                                // only move paddle if the difference in its x position and the mouse x position is larger than a specific amount (probably needs tuning).
                                // also only move the paddle if the time we last power hit is greater or equal to the lock time, so that a power hit locks the paddle in place
                                if (middle_of_paddle - previous_client_x).abs()
                                    > paddle_half_width / 10.0
                                    && time_since_last_power_hit >= POWER_HIT_LOCK_TIME
                                {
                                    if middle_of_paddle < previous_client_x {
                                        // mouse is to the left of the paddle at the moment
                                        // TODO: maybe slow paddle move speed by 20% when the power move time is < the cool down? unsure if good idea or not.
                                        client_x = previous_client_x - PADDLE_MOVE_SPEED;
                                    } else {
                                        // mouse is to the right of the paddle at the moment
                                        client_x = previous_client_x + PADDLE_MOVE_SPEED;
                                    }
                                } else {
                                    // if we dont move the paddle at all, just give it its previous value.
                                    client_x = previous_client_x;
                                }

                                // variable to update power hit time before we move the key state somewhere else,
                                // power hit time is either updated to now or the previous depending on if the client is pressing space
                                let update_power_hit_time = {
                                    if c.key_state.space_bar && time_since_last_power_hit >= POWER_HIT_COOLDOWN {
                                        SystemTime::now()
                                    } else {
                                        prev_client.pong_client_state.time_of_power_hit
                                    }
                                };

                                let client_state: ClientState = ClientState {
                                    // create the new client state from the information we have from the client info.
                                    time: c.time,
                                    team_id: c.team_id,
                                    mouse_pos: c.mouse_pos,
                                    key_state: c.key_state,
                                    pong_client_state: PongClientState {
                                        paddle_x: client_x
                                            .clamp(0.0, GAME_WIDTH - PONG_PADDLE_WIDTH),
                                        paddle_y: client_y,
                                        time_of_power_hit: update_power_hit_time,
                                    },
                                    tank_client_state: prev_client.tank_client_state,
                                };

                                {
                                    local_gs.client_list.insert(uuid.to_string(), client_state);
                                }
                            }

                            TANK(_tgs) => {
                                let client_state: ClientState = ClientState {
                                    // create the new client state from the information we have from the client info.
                                    time: c.time,
                                    team_id: c.team_id,
                                    mouse_pos: c.mouse_pos,
                                    key_state: c.key_state.clone(),
                                    pong_client_state: prev_client.pong_client_state,
                                    tank_client_state: prev_client.tank_client_state,
                                };

                                {
                                    local_gs.client_list.insert(uuid.to_string(), client_state);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("client disconnected: {}", e);
                        {
                            let mut lock = game_state_rw.write().unwrap();
                            lock.client_list.remove(&*uuid);
                        };
                        break;
                    }
                };

                if write.is_err() || flush.is_err() || read.is_err() {
                    println!("client disconnected: Socket closed");
                    {
                        // STATIC_GAME_STATE.client_list.remove(&*uuid)
                        let mut lock = game_state_rw.write().unwrap();
                        lock.client_list.remove(&*uuid);
                    };
                    break;
                }
            } // only even attempt to make a packet to send to the client if we successfully serialize it, this can fail when the unsafe copy of STATIC_GAME_STATE is corrupted.
        } // loop that constantly requests data from the client, also replicates the current game state to the client, and changes the clients state to their current input if they have any.
    })
}
