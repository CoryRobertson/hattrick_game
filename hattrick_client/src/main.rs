use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::keystate::KeyState;
use hattrick_packets_lib::pong::{PONG_BALL_RADIUS, PONG_PADDLE_HEIGHT, PONG_PADDLE_WIDTH};
use hattrick_packets_lib::team::Team;
use hattrick_packets_lib::team::Team::{BlueTeam, RedTeam};
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

enum LocalState {
    AwaitingIp,
    Playing,
    LostConnection,
}

// IDEA: potentially move the server code into a library, and let the client spawn a server hosting thread to host from their client??

#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let game_state = Arc::new(Mutex::new(GameState::default()));
    let running_thread_state = Arc::new(Mutex::new(true));
    let mut connect_thread = None;
    let mut local_state = LocalState::AwaitingIp;
    let mut _ip = String::new();

    #[cfg(debug_assertions)]
    {
        _ip = "localhost:8111".to_string();
    }
    let mut team_id = BlueTeam; // BLUE = 0, RED = 1

    loop {
        // check game state to decide what we are doing
        match local_state {
            // state for when the player is at the main menu and we are waiting for them to type in an ip address.
            LocalState::AwaitingIp => {
                clear_background(Color {
                    r: 80.0 / 255.0,
                    g: 80.0 / 255.0,
                    b: 80.0 / 255.0,
                    a: 1.0,
                });

                #[cfg(debug_assertions)]
                draw_text(
                    format!(
                        "DEBUG MOUSEPOS: {},{}",
                        mouse_position().0,
                        mouse_position().1
                    )
                    .as_str(),
                    10.0,
                    300.0,
                    18.0,
                    BLACK,
                );

                root_ui().label(None, "IP Address");
                root_ui().input_text(0, "", &mut _ip);

                if root_ui().button(None, "Blue Team") {
                    team_id = BlueTeam;
                }
                if root_ui().button(None, "Red Team") {
                    team_id = RedTeam;
                }

                let team_color = {
                    match &team_id {
                        BlueTeam => BLUE,
                        RedTeam => RED, //_ => {GRAY}
                    }
                };

                // draw_text("Team Color")
                root_ui().label(None, "Team: ");
                draw_rectangle(40.0, 85.0, 10.0, 10.0, team_color);

                if root_ui().button(None, "Connect") {
                    connect_thread = Some(spawn_connect_thread(
                        game_state.clone(),
                        running_thread_state.clone(),
                        _ip.clone(),
                        team_id.clone(),
                    ));
                    local_state = LocalState::Playing;
                }

                frame_delay().await;
                next_frame().await;
            }

            // state for when a game is being played.
            LocalState::Playing => {
                clear_background(WHITE);

                #[cfg(debug_assertions)]
                draw_text(
                    format!(
                        "DEBUG MOUSEPOS: {},{}",
                        mouse_position().0,
                        mouse_position().1
                    )
                    .as_str(),
                    10.0,
                    30.0,
                    18.0,
                    BLACK,
                );

                // get the new game state that was most recently received from the connection thread
                let local_gs = { game_state.lock().unwrap().clone() };

                // game type independent code
                {
                    let ping = SystemTime::now()
                        .duration_since(local_gs.clone().time)
                        .unwrap(); // time from last game state to now, including game framerate added, making this number rather high on average.
                    let ping_color = {
                        if ping.as_millis() > 16 {
                            RED
                        } else {
                            GREEN
                        }
                    };

                    if *running_thread_state.lock().unwrap() {
                        draw_text(
                            &format!("Ping: {:.2}ms", ping.as_secs_f64() * 1000.0),
                            10.,
                            10.,
                            18.,
                            ping_color,
                        );
                    } else {
                        local_state = LocalState::LostConnection;
                    }
                }

                // game type dependent code
                match &local_gs.game_type {
                    GameType::PONG(pgs) => {
                        // render each client from their client state as a pong paddle
                        for client in &local_gs.client_list {
                            let client_state = client.1;
                            let client_pos = (
                                client_state.pong_client_state.paddle_x,
                                client_state.pong_client_state.paddle_y,
                            );
                            let team_color = {
                                match client_state.team_id {
                                    BlueTeam => BLUE,
                                    RedTeam => RED, //_ => {GRAY}
                                }
                            };

                            draw_rectangle(
                                client_pos.0,
                                client_pos.1,
                                PONG_PADDLE_WIDTH,
                                PONG_PADDLE_HEIGHT,
                                team_color,
                            );

                            #[cfg(debug_assertions)]
                            draw_text(
                                format!("DEBUG: {}", client_state.key_state).as_str(),
                                client_pos.0,
                                client_pos.1,
                                18.0,
                                BLACK,
                            );
                        }
                        // draw the ball from the servers data
                        draw_circle(pgs.ball_x, pgs.ball_y, PONG_BALL_RADIUS, BLACK);
                        // println!("BALL CORDS: {},{}", pgs.ball_x,pgs.ball_y);
                        draw_text(
                            format!(
                                "Blue points: {}, Red points: {}",
                                pgs.blue_points, pgs.red_points
                            )
                            .as_str(),
                            10.0,
                            20.0,
                            18.0,
                            BLACK,
                        )
                    }
                    GameType::TANK(_tgs) => {
                        for client in &local_gs.client_list {
                            let cx = client.1.tank_client_state.tank_x;
                            let cy = client.1.tank_client_state.tank_y;
                            let team_color = {
                                match client.1.team_id {
                                    RedTeam => RED,
                                    BlueTeam => BLUE,
                                }
                            };
                            draw_text(
                                format!("DEBUG: {:?}", client.1.tank_client_state).as_str(),
                                cx,
                                cy + 5.0,
                                18.0,
                                BLACK,
                            );
                            draw_rectangle(cx, cy, 10.0, 10.0, team_color);
                        }
                    }
                }

                frame_delay().await;
                next_frame().await;
            }

            // state for when a game was being played, but the connection was lost.
            LocalState::LostConnection => {
                clear_background(WHITE);
                draw_text(
                    "Error, lost connection to host",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    20.0,
                    RED,
                );
                if root_ui().button(None, "Reconnect?") {
                    let mut lock = running_thread_state.lock().unwrap();
                    *lock = true;
                    connect_thread = Some(spawn_connect_thread(
                        game_state.clone(),
                        running_thread_state.clone(),
                        _ip.clone(),
                        team_id.clone(),
                    ));
                    local_state = LocalState::Playing;
                }
                if root_ui().button(None, "Back to main menu") {
                    local_state = LocalState::AwaitingIp;
                }
                frame_delay().await;
                next_frame().await;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            let mut end = running_thread_state.lock().unwrap();
            *end = false;
            println!("disconnected from connection thread");
            break;
        } // program exit key

        #[cfg(debug_assertions)] // debug keys
        {
            if is_key_pressed(KeyCode::F1) {
                local_state = LocalState::AwaitingIp;
            }

            if is_key_pressed(KeyCode::F2) {
                local_state = LocalState::Playing;
            }

            if is_key_pressed(KeyCode::F3) {
                local_state = LocalState::LostConnection;
            }
        }
    }

    // make sure connection thread is joined before exiting main
    if let Some(t) = connect_thread {
        let _ = t.join();
    }
}

/// This function takes in the game state arc mutex, the running state arc mutex, an ip address, and the team to connect to and joins the given ip game server.
/// It will mutate the game state each frame by locking the mutex. To stop the connection thread, set the running state to false. This thread also concludes when connection is lost.
fn spawn_connect_thread(
    game_state: Arc<Mutex<GameState>>,
    running: Arc<Mutex<bool>>,
    ip_address: String,
    team_id: Team,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut stream = TcpStream::connect(ip_address).unwrap();
        let _ = stream.set_read_timeout(Option::from(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Option::from(Duration::from_secs(5)));
        let mut _local_gs: Option<GameState> = None;
        println!("connected");
        loop {
            let mut buf: [u8; 8192] = [0; 8192];

            let client_packet = ClientInfo {
                time: SystemTime::now(),
                mouse_pos: mouse_position(),
                team_id: team_id.clone(),
                key_state: KeyState::new(),
            };

            let ser = serde_json::to_string(&client_packet).unwrap();

            let read = stream.read(&mut buf);

            let write = stream.write(ser.as_bytes());
            let flush = stream.flush();

            if read.is_err() || write.is_err() || flush.is_err() {
                // gracefully close thread if internet connection
                break;
            }

            let mut cleaned_buf = vec![];
            for value in buf {
                // make small buffer of the data into a vector sent by the server
                if !String::from_utf8_lossy(&[value]).contains('\0') {
                    cleaned_buf.push(value);
                }
            }
            let output_from_buf = String::from_utf8(cleaned_buf).unwrap();

            match serde_json::from_str::<GameState>(&output_from_buf) {
                Ok(gs) => match game_state.lock() {
                    Ok(mut lock) => {
                        *lock = gs.clone();
                        _local_gs = Some(gs.clone());
                    }
                    Err(e) => {
                        println!("mutex guard error: {e}");
                    }
                },
                Err(e) => {
                    println!("failed to parse: {e}");
                }
            };

            if !(*running.lock().unwrap()) {
                // if the thread running state has been instructed to stop, then we break out of the loop gracefully
                break;
            }
        }
        *running.lock().unwrap() = false;
        println!("connection thread finished");
    })
}

/// This function simply sleeps the given thread for the duration of time necessary to keep the game running at a maximum of 60 fps.
/// To calculate number of milliseconds to wait each frame, divide 1000 by the desired framerate.
/// E.g. 1000.0 ms/60.0 = 16.66_ ms meaning each frame needs to be 16.66_ ms delayed to make a good 60 fps.
async fn frame_delay() {
    let minimum_frame_time = 1. / 60.;
    let frame_time = get_frame_time();
    //println!("Frame time: {}ms", frame_time * 1000.);
    if frame_time < minimum_frame_time {
        let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
        //println!("Sleep for {}ms", time_to_sleep);
        sleep(Duration::from_millis(time_to_sleep as u64));
    }
}
