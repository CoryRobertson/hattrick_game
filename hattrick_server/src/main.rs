use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::clientstate::ClientState;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType::{PONG, TANK};
use hattrick_packets_lib::keystate::KeyState;
use hattrick_packets_lib::pong::{
    get_pong_paddle_width, PongClientState, PongGameState, PONG_BALL_RADIUS, PONG_BALL_VEL_ADD_MAX,
    PONG_BALL_VEL_ADD_MIN, PONG_PADDLE_HEIGHT, PONG_PADDLE_WIDTH,
};
use hattrick_packets_lib::tank::{
    respawn_tank, TankBullet, TankGameState, TANK_ACCEL, TANK_BULLET_BOUNCE_COUNT_MAX,
    TANK_BULLET_RADIUS, TANK_BULLET_VELOCITY, TANK_FRICTION, TANK_HEIGHT, TANK_MAX_SPEED,
    TANK_SHOT_COOLDOWN, TANK_TURN_SPEED, TANK_WIDTH,
};
use hattrick_packets_lib::team::Team::BlueTeam;
use hattrick_packets_lib::team::Team::RedTeam;
use hattrick_packets_lib::{distance, round_digits, two_point_angle, GAME_HEIGHT, GAME_WIDTH};
use rand::Rng;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

static GAME_LOOP_THREAD_DELAY_MS: u64 = 1;

type GameStateRW = Arc<RwLock<GameState>>;

fn main() {
    println!("I am the server!");
    let server = TcpListener::bind("0.0.0.0:8111").unwrap();
    let game_state_rwl: GameStateRW = Arc::new(RwLock::new(GameState::default()));
    let mut client_threads: Vec<JoinHandle<()>> = vec![];
    game_state_rwl.write().unwrap().game_type = TANK(TankGameState::default());

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

    let _ = connect_thread.join();
    let _ = game_thread.join();
}

/// This function spawns the game thread, that handles running the entire game server while reading the games state.
fn spawn_game_thread(game_state_rw: GameStateRW) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut previous_time = SystemTime::now();
        loop {
            // let copy_gs = unsafe { STATIC_GAME_STATE.clone() };
            let copy_gs = {
                let lock = game_state_rw.read().unwrap();
                lock.clone()
            };
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
                    PONG(mut gs) => {
                        let ball_radius = PONG_BALL_RADIUS;

                        // blue team top of screen, red team bottom
                        // ball physics multiplied by delta time since last "frame" allows us to run game speed  independent of application run speed.
                        gs.ball_x += (gs.ball_xvel * difference) * 16.0; // magic number multiplier
                        gs.ball_y += (gs.ball_yvel * difference) * 16.0;

                        {
                            if gs.ball_x < 0.0 + ball_radius {
                                // left screen wall
                                gs.ball_xvel *= -1.0;
                            }

                            if gs.ball_x > GAME_WIDTH - ball_radius {
                                // right screen wall
                                gs.ball_xvel *= -1.0;
                            }

                            if gs.ball_y > GAME_HEIGHT - ball_radius {
                                // FIXME: currently a bug where sometimes two points are scored for red team, unsure as to why yet. Hard to reproduce at the moment, also might happen with the other point score check, not enough testing.
                                // ball hits bottom screen wall
                                let default_xvel = PongGameState::default().ball_xvel;
                                let default_yvel = PongGameState::default().ball_yvel;
                                gs.ball_xvel = {
                                    if gs.ball_xvel < 0.0 {
                                        -default_xvel
                                    } else {
                                        default_xvel
                                    }
                                };
                                gs.ball_yvel = -default_yvel;
                                gs.blue_points += 1;
                                #[cfg(debug_assertions)]
                                println!(
                                    "blue points scored with ball xvel: {} and ball yvel: {}",
                                    gs.ball_xvel, gs.ball_yvel
                                );
                            }

                            if gs.ball_y < 0.0 + ball_radius {
                                // ball hits top screen wall
                                let default_xvel = PongGameState::default().ball_xvel;
                                let default_yvel = PongGameState::default().ball_yvel;
                                gs.ball_xvel = {
                                    if gs.ball_xvel < 0.0 {
                                        -default_xvel
                                    } else {
                                        default_xvel
                                    }
                                };
                                gs.ball_yvel = default_yvel;
                                gs.red_points += 1;
                                #[cfg(debug_assertions)]
                                println!(
                                    "red points scored with ball xvel: {} and ball yvel: {}",
                                    gs.ball_xvel, gs.ball_yvel
                                );
                            }
                        } // bounce checks for ball on walls

                        for client in &copy_gs.client_list {
                            let cs = client.1;

                            let cx = cs.pong_client_state.paddle_x; // client x
                            let cy = {
                                if cs.team_id == BlueTeam {
                                    // cs.pos.1 + ball_radius
                                    cs.pong_client_state.paddle_y + ball_radius
                                } else {
                                    // cs.pos.1 - ball_radius
                                    cs.pong_client_state.paddle_y - ball_radius
                                }
                            }; // client y after taking into account the ball radius, cheap way to do it i know :)
                            let cw = get_pong_paddle_width(&copy_gs.client_list, &cs.team_id); // client width
                            let ch = PONG_PADDLE_HEIGHT; // client height

                            if (gs.ball_y > cy && gs.ball_y < cy + ch)
                                && (gs.ball_x > cx && gs.ball_x < cx + cw)
                            {
                                // first expression is height check for bouncing, second expression is lefty and righty check for bouncing
                                //FIXME: bug, when ball bounces off paddle, instead of inverting its velocity, it should set it so that it goes away from the paddle.
                                //  This is because if the ball is inside the paddle it will internally bounce a lot, which is bad.
                                gs.ball_yvel *= -1.0; // change the uppy downy velocity of the ball to its opposite
                                let rand_xvel_change: f32 = rand::thread_rng()
                                    .gen_range(PONG_BALL_VEL_ADD_MIN..PONG_BALL_VEL_ADD_MAX); // generate a random new x velocity change for when a bounce needs to occur
                                let rand_yvel_change: f32 = rand::thread_rng()
                                    .gen_range(PONG_BALL_VEL_ADD_MIN..PONG_BALL_VEL_ADD_MAX); // generate a random new y velocity change for when a bounce needs to occur

                                if gs.ball_xvel > 0.0 {
                                    // if ball hits paddle, add a random amount of x velocity to the ball, in the direction it is currently traveling
                                    gs.ball_xvel += rand_xvel_change;
                                } else {
                                    gs.ball_xvel -= rand_xvel_change;
                                }

                                if gs.ball_yvel > 0.0 {
                                    // ditto from comment above
                                    gs.ball_yvel += rand_yvel_change;
                                } else {
                                    gs.ball_yvel -= rand_yvel_change;
                                }

                                #[cfg(debug_assertions)]
                                println!(
                                    "bounced with: new xvel ({}), new yvel ({}): {} {}",
                                    rand_xvel_change, rand_yvel_change, gs.ball_xvel, gs.ball_yvel
                                );
                            } // bounce checks for ball on paddles of clients
                        } // client loop for game state

                        {
                            // STATIC_GAME_STATE.game_type = PONG(gs);
                            // STATIC_GAME_STATE.time = SystemTime::now();
                            let mut lock = game_state_rw.write().unwrap();
                            lock.game_type = PONG(gs);
                            lock.time = SystemTime::now();
                        }
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

                            if client_key_state.space_bar && last_shot_diff > TANK_SHOT_COOLDOWN {
                                client.1.tank_client_state.last_shot_time = SystemTime::now();
                                println!("shot time: {:?}", last_shot_diff);

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

                        for c in &client_list {
                            println!("{:?}", c.1.tank_client_state);
                        }

                        for mut bullet in &mut tgs.bullets {
                            if bullet.x >= GAME_WIDTH - TANK_BULLET_RADIUS
                                || bullet.x <= 0.0 + TANK_BULLET_RADIUS
                            {
                                bullet.x_vel *= -1.0;
                                bullet.bounce_count += 1;
                            }
                            if bullet.y >= GAME_HEIGHT - TANK_BULLET_RADIUS
                                || bullet.y <= 0.0 + TANK_BULLET_RADIUS
                            {
                                bullet.y_vel *= -1.0;
                                bullet.bounce_count += 1;
                            }
                            bullet.x += bullet.x_vel * difference;
                            bullet.y += bullet.y_vel * difference;
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
                            // STATIC_GAME_STATE.game_type = TANK(tgs);
                            // STATIC_GAME_STATE.client_list = client_list;
                            // STATIC_GAME_STATE.time = SystemTime::now();
                            let mut lock = game_state_rw.write().unwrap();
                            lock.game_type = TANK(tgs);
                            lock.client_list = client_list;
                            lock.time = SystemTime::now();
                        }
                    }
                }
            }
            previous_time = SystemTime::now();
            sleep(Duration::from_millis(GAME_LOOP_THREAD_DELAY_MS)); // maybe remove this? at the moment unsure
        }
    })
}

/// This function handles a given client, it spawns a thread that will relay the game state to them, as well as take in their client info and insert that info into the game states client list under their uuid.
/// Each client is denoted by a random gen uuid.
/// The thread is closed and their client state is removed when they either disconnect, or the thread closes.
fn handle_client(stream: TcpStream, game_state_rw: GameStateRW) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut client_stream = stream;
        client_stream
            .set_write_timeout(Option::from(Duration::from_secs(5)))
            .unwrap();
        client_stream
            .set_read_timeout(Option::from(Duration::from_secs(5)))
            .unwrap();
        let uuid = Uuid::new_v4().to_string();
        {
            // STATIC_GAME_STATE.client_list.insert(
            //     uuid.to_string(),
            //     ClientState {
            //         time: SystemTime::now(),
            //         // pos: (0.0, 0.0),
            //         team_id: BlueTeam,
            //         mouse_pos: (0.0, 0.0),
            //         key_state: KeyState::default(),
            //
            //         pong_client_state: Default::default(),
            //         tank_client_state: Default::default(),
            //     },
            // );
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
        }

        loop {
            // TODO: write logic that takes a timestamp when ever a write is successfully sent to a client, and if the last successful write happened more than 5 seconds ago, we can drop the client, otherwise keep waiting on them.
            //  alternatively, let a specific number of packets be dropped before dropping a client.
            // let local_gs = unsafe { &*STATIC_GAME_STATE };
            let local_gs = {
                let lock = game_state_rw.read().unwrap();
                lock.clone()
            };
            // FIXME: there is a bug here relating to the lock.
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
                                        BlueTeam => 10.0,
                                        RedTeam => 550.0,
                                    }
                                };

                                let client_x = c.mouse_pos.0 - (PONG_PADDLE_WIDTH / 2.0); // the client renders the rectangle from its top left corner, so to center it, we subtract half the paddle width

                                let client_state: ClientState = ClientState {
                                    // create the new client state from the information we have from the client info.
                                    time: c.time,
                                    team_id: c.team_id,
                                    mouse_pos: c.mouse_pos,
                                    key_state: c.key_state,
                                    pong_client_state: PongClientState {
                                        paddle_x: client_x,
                                        paddle_y: client_y,
                                    },
                                    tank_client_state: prev_client.tank_client_state,
                                };

                                {
                                    // STATIC_GAME_STATE
                                    //     .client_list
                                    //     .insert(uuid.to_string(), client_state)
                                    // let mut lock = game_state_rw.write().unwrap();
                                    local_gs.client_list.insert(uuid.to_string(), client_state);
                                };
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
                                    // STATIC_GAME_STATE
                                    //     .client_list
                                    //     .insert(uuid.to_string(), client_state);
                                    local_gs.client_list.insert(uuid.to_string(), client_state);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("client disconnected: {}", e);
                        {
                            // STATIC_GAME_STATE.client_list.remove(&*uuid)
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
        }
    })
}
