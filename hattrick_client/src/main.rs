use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::gamestate::{GameState, MAX_VOTE_NUM};
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::keystate::KeyState;
use hattrick_packets_lib::pong::{get_pong_paddle_width, PONG_BALL_RADIUS, PONG_PADDLE_HEIGHT};
use hattrick_packets_lib::tank::{TANK_BULLET_RADIUS, TANK_HEIGHT, TANK_WIDTH};
use hattrick_packets_lib::team::Team;
use hattrick_packets_lib::team::Team::{BlueTeam, RedTeam};
use hattrick_packets_lib::{
    get_angle_of_travel_degrees, get_vote_count_for_number, round_number, two_point_angle,
    GAME_HEIGHT, GAME_WIDTH,
};
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

                draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, GRAY);

                // get the new game state that was most recently received from the connection thread
                let local_gs = { game_state.lock().unwrap().clone() };
                // game type independent code
                {
                    let ping = SystemTime::now().duration_since(local_gs.time).unwrap(); // time from last game state to now, including game framerate added, making this number rather high on average.
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

                    if local_gs.vote_running {
                        draw_text(
                            &format!("Vote Running {}", get_vote_count_for_number(2, &local_gs)),
                            50.0,
                            50.0,
                            16.0,
                            BLACK,
                        );

                        local_gs
                            .client_list
                            .iter()
                            .filter(|client| client.1.vote_number != 0)
                            .enumerate()
                            .for_each(|(index, (_, client))| {
                                let y = 60 + (index * 10);
                                draw_text(
                                    &format!("client vote:{}", client.vote_number),
                                    50.0,
                                    y as f32,
                                    16.0,
                                    BLACK,
                                );
                            });
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
                            let width =
                                get_pong_paddle_width(&local_gs.client_list, &client_state.team_id);

                            let time_since_last_power_hit = SystemTime::now()
                                .duration_since(client_state.pong_client_state.time_of_power_hit)
                                .unwrap()
                                .as_secs_f32();

                            // draw power hit circles on paddle
                            if time_since_last_power_hit <= 1.0 {
                                // y coordinate amount to add depending on the team of the client.
                                let circle_y_modifier = {
                                    match client_state.team_id {
                                        RedTeam => PONG_PADDLE_HEIGHT,
                                        BlueTeam => 0.0,
                                    }
                                };
                                draw_circle(
                                    client_pos.0,
                                    client_pos.1 + circle_y_modifier,
                                    5.0,
                                    ORANGE,
                                ); // draw circle on left side of paddle
                                draw_circle(
                                    client_pos.0 + width,
                                    client_pos.1 + circle_y_modifier,
                                    5.0,
                                    ORANGE,
                                ); // draw circle on right side of paddle
                            }

                            draw_rectangle(
                                client_pos.0,
                                client_pos.1,
                                width,
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

                        #[cfg(debug_assertions)]
                        draw_text(
                            &format!("DEBUG: {}, {}", pgs.ball_xvel, pgs.ball_yvel),
                            pgs.ball_x + 20.0,
                            pgs.ball_y,
                            18.0,
                            BLACK,
                        );
                        draw_circle(pgs.ball_x, pgs.ball_y, PONG_BALL_RADIUS, BLACK);

                        #[cfg(debug_assertions)]
                        draw_circle(pgs.ball_x + (pgs.ball_xvel * 5.0), pgs.ball_y, 5.0, BLACK);
                        draw_poly(
                            pgs.ball_x,
                            pgs.ball_y,
                            3,
                            PONG_BALL_RADIUS,
                            get_angle_of_travel_degrees(
                                pgs.ball_x,
                                pgs.ball_y,
                                pgs.ball_xvel,
                                pgs.ball_yvel,
                            ),
                            GRAY,
                        );

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
                    GameType::TANK(tgs) => {
                        for client in &local_gs.client_list {
                            // alias variables for code clarity
                            let cx = client.1.tank_client_state.tank_x;
                            let cy = client.1.tank_client_state.tank_y;
                            let rot = client.1.tank_client_state.rotation;

                            // angle from the tank to the mouse
                            let mouse_angle =
                                round_number(&two_point_angle((cx, cy), client.1.mouse_pos), 2);

                            // team color for the tank
                            let team_color = {
                                match client.1.team_id {
                                    RedTeam => RED,
                                    BlueTeam => BLUE,
                                }
                            };

                            draw_text(
                                &format!(
                                    "Red Score: {} Blue Score: {}",
                                    tgs.red_score, tgs.blue_score
                                ),
                                30.0,
                                40.0,
                                18.0,
                                BLACK,
                            );

                            // debug info for each tank
                            #[cfg(debug_assertions)]
                            draw_text(
                                format!(
                                    "DEBUG Tank speed: {},{}, ANGLE: {}",
                                    client.1.tank_client_state.tank_x_vel,
                                    client.1.tank_client_state.tank_y_vel,
                                    mouse_angle
                                )
                                .as_str(),
                                cx,
                                cy + 5.0,
                                18.0,
                                BLACK,
                            );

                            // tank polygon for body of tank
                            draw_poly(
                                cx + (TANK_WIDTH / 2.0),
                                cy + (TANK_HEIGHT / 2.0),
                                5,
                                (TANK_WIDTH + TANK_HEIGHT) / 2.0,
                                rot,
                                team_color,
                            );

                            // tank polygon for barrel of the tank
                            draw_poly(
                                cx + (TANK_WIDTH / 2.0),
                                cy + (TANK_HEIGHT / 2.0),
                                3,
                                (TANK_WIDTH + TANK_HEIGHT) / 4.0,
                                mouse_angle,
                                GREEN,
                            );
                            let dir_of_travel = {
                                let angle = client.1.tank_client_state.rotation.to_radians();
                                // println!("{}", angle);
                                (angle.cos() * 15.0, angle.sin() * 15.0)
                            }; // get the direction of travel

                            draw_circle(
                                dir_of_travel.0 + cx + (TANK_WIDTH / 2.0),
                                dir_of_travel.1 + cy + (TANK_HEIGHT / 2.0),
                                4.0,
                                BLACK,
                            ); // draw the direction of travel bubble on the tanks
                        } // render all clients

                        for bullet in &tgs.bullets {
                            draw_circle(bullet.x, bullet.y, TANK_BULLET_RADIUS, GREEN);
                        } // render all bullets
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
        let mut vote_num: u8 = 0;
        loop {
            let mut buf: [u8; 8192] = [0; 8192];

            let client_packet = ClientInfo {
                time: SystemTime::now(),
                mouse_pos: mouse_position(),
                team_id: team_id.clone(),
                key_state: KeyState::new(),
                vote_number: {
                    match &_local_gs {
                        None => vote_num,
                        Some(gs) => {
                            if gs.vote_running {
                                if is_key_pressed(KeyCode::Left) {
                                    vote_num = (vote_num as i32 - 1).clamp(0, MAX_VOTE_NUM) as u8;
                                }
                                if is_key_pressed(KeyCode::Right) {
                                    vote_num = (vote_num as i32 + 1).clamp(0, MAX_VOTE_NUM) as u8;
                                }
                            } else {
                                vote_num = 0;
                            }
                            vote_num
                        }
                    }
                },
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
            //let output_from_buf = String::from_utf8(cleaned_buf).unwrap();

            // only read the output if its ok, if not skip a frame. This can happen because of parsing errors.
            match String::from_utf8(cleaned_buf) {
                Ok(output_from_buf) => {
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
                Err(err) => {
                    println!("Skipped packet due to parse error: \n{}", err);
                    //println!("cleaned buffer length: {}, contents: {:?}",cleaned_buf.len(), cleaned_buf);
                }
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
