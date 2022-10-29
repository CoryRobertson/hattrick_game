mod packets;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use macroquad::prelude::*;
use once_cell::unsync::Lazy;

use crate::packets::packets::{ClientState, GameState};



// static mut STATIC_GAME_STATE: Lazy<GameState> = Lazy::new(|| {
//     GameState{ time: SystemTime::UNIX_EPOCH, x: 0.0, y: 0.0, client_list: Default::default() }
// });


#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let mut game_state = Arc::new(Mutex::new(GameState::default()));


    let connect_game_state = Arc::clone(&game_state);
    let connect_thread = thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:8111").unwrap();
        println!("connected");
        loop {
            let mut buf: [u8 ; 4096] = [0 ; 4096];

            let client_packet = ClientState{ time: SystemTime::now(), mouse_pos: mouse_position() };
            let ser = serde_json::to_string(&client_packet).unwrap();

            let _ = stream.read(&mut buf);
            let _ = stream.write(ser.as_bytes());
            let _ = stream.flush();
            let mut cleaned_buf = vec![];
            for value in buf { // make small buffer of the data into a vector sent by the server
                if !String::from_utf8_lossy(&[value]).contains("\0") {
                    cleaned_buf.push(value);
                }
            }
            let o = String::from_utf8(Vec::from(cleaned_buf)).unwrap();
            match serde_json::from_str::<GameState>(&*o) {
                Ok(gs) => {
                    //println!("{:?}", gs.client_list);

                    //unsafe { *STATIC_GAME_STATE = gs.clone(); }
                    match connect_game_state.lock().borrow_mut() {
                        Ok(lock) => {
                            lock.x = gs.x.clone();
                            lock.y = gs.y.clone();
                            lock.time = gs.time.clone();
                            lock.client_list = gs.client_list.clone();
                        }
                        Err(e) => {
                            println!("mutex guard error: {}",e);
                        }
                    }
                }
                Err(e) => {println!("failed to parse: {}", e);}
            };
            //println!("aquiring lock");
            // let mut lock = connect_game_state.lock().unwrap();
            // lock.time = deser.time;


            // for client in deser.clone().client_list {
            //     println!("{},{}",client.1.mouse_pos.0,client.1.mouse_pos.1);
            // }
            let _ = stream.flush();
        }

    });

    let game_game_state = Arc::clone(&game_state);
    loop {
        {
            let local_gs = {
              game_game_state.lock().unwrap().clone()
            };
            // let local_gs = unsafe { STATIC_GAME_STATE.clone() };
            clear_background(WHITE);

            for client in local_gs.clone().client_list {
                let client_state = client.1;
                //println!("{} {}", client_state.mouse_pos.0, client_state.mouse_pos.1);
                draw_circle(client_state.mouse_pos.0, client_state.mouse_pos.1, 15.0, RED);

            }

            //println!("{:?}", mouse_position());

            draw_circle(local_gs.x as f32 + 200.0, local_gs.y as f32 + 200.0, 15.0, RED);

            draw_text(format!("Server Time: {}", local_gs).as_str(), 50., 50., 12., BLACK);
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        // sleep(Duration::from_millis(500));
        frame_delay().await;
        next_frame().await;

    }
    let _ = connect_thread.join();
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