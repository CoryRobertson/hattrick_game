use hattrick_packets_lib::packets::*;
use once_cell::unsync::Lazy;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use hattrick_packets_lib::packets::GameType::PONG;

//TODO: write a game_sub_state module that contains which game is being played, then have it transition from two different states that the client will render. Use an enum

// super bad practice to do this, probably move away from this eventually.
// if this ends up backfiring, use a RWLock instead, probably rather fruitful as multiple clients reading at same time is permitted. once a client needs to make a change to their ClientState, they can upgrade to a write lock on the rwlock
// following this, we could use a function that detects how different the client state is from the current ClientState and only update it if the difference is high enough.
static mut STATIC_GAME_STATE: Lazy<GameState> = Lazy::new(|| GameState {
    time: SystemTime::UNIX_EPOCH,
    // x: 0.0,
    // y: 0.0,
    game_type: PONG(PongGameState::default()),
    client_list: Default::default(),
});

fn main() {
    println!("I am the server!");
    let server = TcpListener::bind("0.0.0.0:8111").unwrap();
    let mut client_threads: Vec<JoinHandle<()>> = vec![];

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
                client_threads.push(handle_client(r));
            }

            println!("Client count: {}", client_threads.len());
        }
    });

    let game_thread = spawn_game_thread();

    let _ = connect_thread.join();
    let _ = game_thread.join();
}

fn spawn_game_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        loop {
            let copy_gs = unsafe { STATIC_GAME_STATE.clone() };
            //println!("client list len: {}", copy_gs.client_list.len());
            if copy_gs.client_list.len() >= 1 {
                unsafe {
                    // basic game logic goes here
                    STATIC_GAME_STATE.time = SystemTime::now();
                    match copy_gs.game_type {
                        PONG(mut gs) => {
                            let ball_radius = hattrick_packets_lib::PONG_BALL_RADIUS;

                            // blue team top of screen, red team bottom
                            // ball physics
                            gs.ball_x += gs.ball_xvel;
                            gs.ball_y += gs.ball_yvel;

                            {
                                if gs.ball_x < 0.0 + ball_radius { // left screen wall
                                    gs.ball_xvel *= -1.0;
                                }

                                if gs.ball_x > 800.0 - ball_radius { // right screen wall
                                    gs.ball_xvel *= -1.0;
                                }

                                if gs.ball_y > 600.0 - ball_radius { // ball hits bottom screen wall

                                    gs.ball_xvel = {
                                        if gs.ball_xvel < 0.0 { -5.0 } else { 5.0 }
                                    };
                                    gs.ball_yvel = -5.0;
                                    gs.blue_points += 1;
                                }

                                if gs.ball_y < 0.0 + ball_radius { // ball hits top screen wall

                                    gs.ball_xvel = {
                                        if gs.ball_xvel < 0.0 { -5.0 } else { 5.0 }
                                    };
                                    gs.ball_yvel = 5.0;
                                    gs.red_points += 1;
                                }
                            } // bounce checks for ball on walls


                            for client in &copy_gs.client_list {
                                let cs = client.1;

                                let cx = cs.mouse_pos.0; // client x
                                let cy = {
                                    if cs.team_id == 0 {
                                        cs.mouse_pos.1 + ball_radius
                                    } else {
                                        cs.mouse_pos.1 - ball_radius
                                    }
                                }; // client y after taking into account the ball radius, cheap way to do it i know :)
                                let cw = hattrick_packets_lib::PONG_PADDLE_WIDTH; // client width
                                let ch = hattrick_packets_lib::PONG_PADDLE_HEIGHT; // client height

                                if gs.ball_y > cy && gs.ball_y < cy + ch {
                                    if gs.ball_x > cx && gs.ball_x < cx + cw {
                                        gs.ball_yvel *= -1.0;
                                        if gs.ball_xvel > 0.0 { gs.ball_xvel += 1.0; } else { gs.ball_xvel -= 1.0; }
                                        if gs.ball_yvel > 0.0 { gs.ball_yvel += 1.0; } else { gs.ball_yvel -= 1.0; }
                                        println!("bounced with new vel: {} {}", gs.ball_xvel, gs.ball_yvel);
                                    }
                                } // bounce checks for ball on paddles of clients
                            } // client loop for game state
                            STATIC_GAME_STATE.game_type = PONG(gs.clone());
                        }
                    }
                }
            }
            sleep(Duration::from_millis(16)); // maybe remove this? at the moment unsure
        }
    })
}

fn handle_client(stream: TcpStream) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut client_stream = stream;
        client_stream
            .set_write_timeout(Option::from(Duration::from_secs(5)))
            .unwrap();
        client_stream
            .set_read_timeout(Option::from(Duration::from_secs(5)))
            .unwrap();
        let uuid = Uuid::new_v4().to_string();
        unsafe {
            STATIC_GAME_STATE.client_list.insert(
                uuid.to_string(),
                ClientState {
                    time: SystemTime::now(),
                    mouse_pos: (0.0, 0.0),
                    team_id: 0
                },
            );
        }

        loop {
            let ser = unsafe { serde_json::to_string(&*STATIC_GAME_STATE).unwrap() };
            let write = client_stream.write(ser.as_bytes());
            let flush = client_stream.flush();
            let mut buf: [u8; 4096] = [0; 4096];
            let read = client_stream.read(&mut buf);
            let mut cleaned_buf = vec![];

            for value in buf {
                // make small buffer of the data into a vector sent by the server
                if !String::from_utf8_lossy(&[value]).contains('\0') {
                    cleaned_buf.push(value);
                }
            }

            let clean = String::from_utf8(cleaned_buf).unwrap();
            match serde_json::from_str::<ClientState>(&clean) {
                Ok(mut c) => {
                    // here we can decide if we want to do anything with the client state given if it is different enough,
                    // this would allow us to only take changes if they are large enough, compressing how often we have to lock the game state, if we decide to be threadsafe.
                    let client_x = {
                        if c.team_id == 0 {
                            10.0
                        } else {
                            550.0
                        }
                    };

                    c.mouse_pos.1 = client_x;

                    unsafe {
                        STATIC_GAME_STATE
                            .client_list
                            .insert(uuid.to_string(), c.clone())
                    };
                }
                Err(e) => {
                    println!("client disconnected: {e}");
                    unsafe { STATIC_GAME_STATE.client_list.remove(&*uuid) };
                    break;
                }
            };

            if write.is_err() || flush.is_err() || read.is_err() {
                println!("client disconnected: Socket closed");
                unsafe { STATIC_GAME_STATE.client_list.remove(&*uuid) };
                break;
            }
        }
    })
}

fn _distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let g1 = (x2-x1).powi(2);
    let g2 = (y2-y1).powi(2);
    return (g1 + g2).sqrt();
}