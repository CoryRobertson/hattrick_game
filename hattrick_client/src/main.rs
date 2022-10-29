mod packets;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use macroquad::prelude::*;
use once_cell::unsync::Lazy;

use crate::packets::packets::{ClientState, GameState};



static mut STATIC_GAME_STATE: Lazy<GameState> = Lazy::new(|| {
    GameState{ time: SystemTime::UNIX_EPOCH, x: 0.0, y: 0.0, client_list: Default::default() }
});


#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let connect_thread = thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:8111").unwrap();
        println!("connected");
        loop {
            let mut buf: [u8 ; 512] = [0 ; 512];

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
            let deser: GameState = serde_json::from_str(&*o).unwrap();
            //println!("aquiring lock");
            // let mut lock = connect_game_state.lock().unwrap();
            // lock.time = deser.time;

            unsafe { *STATIC_GAME_STATE = deser.clone(); }
            for client in deser.clone().client_list {
                println!("{},{}",client.1.mouse_pos.0,client.1.mouse_pos.1);
            }

        }

    });

    let mut count = 0;
    loop {
        {
            // let local_gs = game_state.lock().unwrap();
             let local_gs = unsafe { STATIC_GAME_STATE.clone() };

            clear_background(WHITE);

            draw_circle(local_gs.x as f32 + 200.0, local_gs.y as f32 + 200.0, 15.0, RED);

            draw_text(format!("Server Time: {}, {}", local_gs, count).as_str(), 50., 50., 12., BLACK);
        }

        count += 1;
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