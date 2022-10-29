use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use crate::packets::packets::GameState;

mod packets;

type GameStateType = Arc<Mutex<GameState>>;

fn main() {
    println!("I am the server!");
    let server = TcpListener::bind("127.0.0.1:8111").unwrap();
    let mut game_state = Arc::new(Mutex::new(GameState::default()));
    let mut client_threads: Vec<JoinHandle<()>> = vec![];
    let copied_game_state = Arc::clone(&game_state);

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

            match response {
                Ok(r) => {
                    client_threads.push(handle_client(r,copied_game_state.clone()));
                }
                Err(e) => {}
            }
            println!("Client count: {}", client_threads.len());
        }

    });
    let copied_game_state = Arc::clone(&game_state);

    let game_thread = thread::spawn(move || {

        loop {
            let mut lock = copied_game_state.lock().unwrap();
            lock.time = SystemTime::now();
            thread::sleep(Duration::from_millis(1));
        }
    });

    connect_thread.join();
    game_thread.join();
}

fn handle_client(stream: TcpStream, game_state: GameStateType) -> JoinHandle<()> {

    thread::spawn(move || {
        let state = game_state;
        let mut client_stream = stream;

        loop {
            let lock = state.lock().unwrap();
            let ser = serde_json::to_string(lock.deref()).unwrap();
            let write = client_stream.write(ser.as_bytes());
            let flush = client_stream.flush();
            let read = client_stream.read(&mut [0 ; 128]);

            if write.is_err() || flush.is_err() || read.is_err() {
                break;
            }

            // thread::sleep(Duration::from_secs(1));
        }

    })
}