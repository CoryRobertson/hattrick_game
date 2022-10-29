mod packets;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use macroquad::prelude::*;

use crate::packets::packets::GameState;

type GameStateType = Arc<Mutex<GameState>>;

static mut StaticGameState: GameState = GameState::default();

#[macroquad::main("???")]
async fn main() {
    println!("I am the client");

    let mut game_state = Arc::new(Mutex::new(GameState::default()));


    let connect_game_state = Arc::clone(&game_state);
    let connect_thread = thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:8111").unwrap();
        println!("connected");
        loop {
            let mut buf: [u8 ; 512] = [0 ; 512];

            let _ = stream.read(&mut buf);
            let _ = stream.write(&[0]);
            let mut cleaned_buf = vec![];
            for value in buf { // make small buffer of the data into a vector sent by the server
                if !String::from_utf8_lossy(&[value]).contains("\0") {
                    cleaned_buf.push(value);
                }
            }
            let o = String::from_utf8(Vec::from(cleaned_buf)).unwrap();
            let deser: GameState = serde_json::from_str(&*o).unwrap();
            //println!("aquiring lock");
            let mut lock = connect_game_state.lock().unwrap();
            lock.time = deser.time;
            // unsafe { StaticGameState = deser; }
            println!("time changed");

        }

    });



    let mut count = 0;
    loop {
        {
            let local_gs = game_state.lock().unwrap();

            clear_background(WHITE);

            draw_text(format!("Server Time: {}, {}", local_gs, count).as_str(), 50., 50., 12., BLACK);
        }

        count += 1;
        // sleep(Duration::from_millis(500));
        frame_delay().await;
        next_frame().await;

    }


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