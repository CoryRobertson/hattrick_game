use hattrick_packets_lib::packets::*;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};
use hattrick_packets_lib::packets::Team::{BlueTeam, RedTeam};


enum LocalState {
    AwaitingIp,
    Playing,
    LostConnection,
}

#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let game_state = Arc::new(Mutex::new(GameState::default()));
    let running_thread_state = Arc::new(Mutex::new(true));
    let mut connect_thread = None;
    let mut local_state = LocalState::AwaitingIp;
    let mut ip = String::new();
    #[cfg(debug_assertions)]
    {
        ip = "localhost:8111".to_string();
    }
    let mut team_id = BlueTeam; // BLUE = 0, RED = 1

    loop {
        match local_state { // check game state to decide what we are doing

            LocalState::AwaitingIp => {
                clear_background(Color {
                    r: 80.0 / 255.0,
                    g: 80.0 / 255.0,
                    b: 80.0 / 255.0,
                    a: 1.0,
                });

                #[cfg(debug_assertions)]
                draw_text(format!("DEBUG MOUSEPOS: {},{}", mouse_position().0, mouse_position().1).as_str(),10.0,300.0,18.0,BLACK);

                root_ui().label(None, "IP Address");
                root_ui().input_text(0, "", &mut ip);

                if root_ui().button(None,"Blue Team") {team_id = BlueTeam;}
                if root_ui().button(None,"Red Team") {team_id = RedTeam;}

                let team_color = {
                    match &team_id {
                        BlueTeam => {BLUE}
                        RedTeam => {RED}
                        //_ => {GRAY}
                    }
                };

                // draw_text("Team Color")
                root_ui().label(None,"Team: ");
                draw_rectangle(40.0,85.0,10.0,10.0,team_color);

                if root_ui().button(None, "Connect") {
                    connect_thread = Some(spawn_connect_thread(
                        game_state.clone(),
                        running_thread_state.clone(),
                        ip.clone(),
                        team_id.clone(),
                    ));
                    local_state = LocalState::Playing;
                }

                frame_delay().await;
                next_frame().await;
            }

            LocalState::Playing => {
                clear_background(WHITE);

                #[cfg(debug_assertions)]
                draw_text(format!("DEBUG MOUSEPOS: {},{}", mouse_position().0, mouse_position().1).as_str(),10.0,30.0,18.0,BLACK);

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
                            let team_color = {
                                match client_state.team_id {
                                    BlueTeam => {BLUE}
                                    RedTeam => {RED}
                                    //_ => {GRAY}
                                }
                            };
                            draw_rectangle(client_state.mouse_pos.0,client_state.mouse_pos.1,hattrick_packets_lib::PONG_PADDLE_WIDTH,hattrick_packets_lib::PONG_PADDLE_HEIGHT,team_color);
                        }
                        // draw the ball from the servers data
                        draw_circle(pgs.ball_x,pgs.ball_y,hattrick_packets_lib::PONG_BALL_RADIUS,BLACK);
                        // println!("BALL CORDS: {},{}", pgs.ball_x,pgs.ball_y);
                        draw_text(
                            format!("Blue points: {}, Red points: {}", pgs.blue_points,pgs.red_points).as_str(),
                            10.0,20.0,18.0,BLACK
                        )
                    }
                }

                frame_delay().await;
                next_frame().await;
            }

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
                        ip.clone(),
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

fn spawn_connect_thread(
    game_state: Arc<Mutex<GameState>>,
    running: Arc<Mutex<bool>>,
    ip_address: String,
    team_id: Team
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut stream = TcpStream::connect(ip_address).unwrap();
        let _ = stream.set_read_timeout(Option::from(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Option::from(Duration::from_secs(5)));
        let mut local_gs: Option<GameState> = None;
        println!("connected");
        loop {
            let mut buf: [u8; 4096] = [0; 4096];

            let client_packet = ClientState {
                time: SystemTime::now(),
                mouse_pos: {
                    let pos: (f32,f32) = match &local_gs {
                        None => { mouse_position() }
                        Some(gs) => {
                            match &gs.game_type {
                                GameType::PONG(_) => { // offset the mouse position when we are playing pong so we can have the cursor be at the center of the rectangle
                                    (mouse_position().0 - 50.0, mouse_position().1)
                                }
                            }
                        }
                    };
                    //(mouse_position().0 - 50.0, mouse_position().1)
                    pos
                },
                team_id: team_id.clone(),
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
                        local_gs = Some(gs.clone());
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
