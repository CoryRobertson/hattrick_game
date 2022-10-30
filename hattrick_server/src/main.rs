use crate::packets::packets::{ClientState, GameState};
use once_cell::unsync::Lazy;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

mod packets;

// super bad practice to do this, probably move away from this eventually.
static mut STATIC_GAME_STATE: Lazy<GameState> = Lazy::new(|| GameState {
    time: SystemTime::UNIX_EPOCH,
    x: 0.0,
    y: 0.0,
    client_list: Default::default(),
});

fn main() {
    println!("I am the server!");
    let server = TcpListener::bind("127.0.0.1:8111").unwrap();
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

            match response {
                Ok(r) => {
                    client_threads.push(handle_client(r));
                }
                Err(_) => {}
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
            unsafe {
                // basic game logic goes here
                STATIC_GAME_STATE.time = SystemTime::now();
                STATIC_GAME_STATE.x = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64()
                    .sin()
                    * 100.0;
                STATIC_GAME_STATE.y = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64()
                    .cos()
                    * 100.0;
            }
        }
    })
}

fn handle_client(stream: TcpStream) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut client_stream = stream;
        let uuid = Uuid::new_v4().to_string();
        unsafe {
            STATIC_GAME_STATE.client_list.insert(
                uuid.to_string(),
                ClientState {
                    time: SystemTime::now(),
                    mouse_pos: (0.0, 0.0),
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
                if !String::from_utf8_lossy(&[value]).contains("\0") {
                    cleaned_buf.push(value);
                }
            }

            let clean = String::from_utf8(cleaned_buf).unwrap();
            match serde_json::from_str::<ClientState>(&*clean) {
                Ok(c) => {
                    unsafe {
                        STATIC_GAME_STATE
                            .client_list
                            .insert(uuid.to_string(), c.clone())
                    };
                }
                Err(e) => {
                    println!("client disconnected: {}", e);
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
