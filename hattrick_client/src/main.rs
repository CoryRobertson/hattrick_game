mod packets;

use crate::packets::packets::{ClientState, GameState};
use macroquad::prelude::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let game_state = Arc::new(Mutex::new(GameState::default()));
    let running_thread_state = Arc::new(Mutex::new(true));
    let connect_thread = spawn_connect_thread(game_state.clone(), running_thread_state.clone());

    loop {
        clear_background(WHITE);

        // get the new game state that was most recently received from the connection thread
        let local_gs = { game_state.lock().unwrap().clone() };

        let ping = SystemTime::now()
            .duration_since(local_gs.clone().time)
            .unwrap();
        let ping_color = {
            if ping.as_millis() > 16 {
                RED
            } else {
                GREEN
            }
        };
        draw_text(
            &*format!("Ping: {:.2}ms", ping.as_secs_f64() * 1000.0),
            10.,
            10.,
            18.,
            ping_color,
        );

        // render each client from their client state
        for client in local_gs.clone().client_list {
            let client_state = client.1;
            draw_circle(
                client_state.mouse_pos.0,
                client_state.mouse_pos.1,
                15.0,
                RED,
            );
        }

        draw_circle(
            local_gs.x as f32 + 200.0,
            local_gs.y as f32 + 200.0,
            15.0,
            RED,
        );

        if is_key_pressed(KeyCode::Escape) {
            let mut end = running_thread_state.lock().unwrap();
            *end = false;
            println!("disconnected from connection thread");
            break;
        }

        frame_delay().await;
        next_frame().await;
    }

    let _ = connect_thread.join();
}

fn spawn_connect_thread(
    game_state: Arc<Mutex<GameState>>,
    running: Arc<Mutex<bool>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:8111").unwrap();
        println!("connected");
        loop {
            let mut buf: [u8; 4096] = [0; 4096];

            let client_packet = ClientState {
                time: SystemTime::now(),
                mouse_pos: mouse_position(),
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
                if !String::from_utf8_lossy(&[value]).contains("\0") {
                    cleaned_buf.push(value);
                }
            }
            let output_from_buf = String::from_utf8(Vec::from(cleaned_buf)).unwrap();

            match serde_json::from_str::<GameState>(&*output_from_buf) {
                Ok(gs) => match game_state.lock() {
                    Ok(mut lock) => {
                        *lock = gs.clone();
                    }
                    Err(e) => {
                        println!("mutex guard error: {}", e);
                    }
                },
                Err(e) => {
                    println!("failed to parse: {}", e);
                }
            };

            if *running.lock().unwrap() == false {
                // if the thread running state has been instructed to stop, then we break out of the loop gracefully
                break;
            }
        }
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
