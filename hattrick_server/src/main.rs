use hattrick_packets_lib::packets::*;
use once_cell::unsync::Lazy;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

//TODO: write a game_sub_state module that contains which game is being played, then have it transition from two different states that the client will render. Use an enum

// super bad practice to do this, probably move away from this eventually.
// if this ends up backfiring, use a RWLock instead, probably rather fruitful as multiple clients reading at same time is permitted. once a client needs to make a change to their ClientState, they can upgrade to a write lock on the rwlock
// following this, we could use a function that detects how different the client state is from the current ClientState and only update it if the difference is high enough.
static mut STATIC_GAME_STATE: Lazy<GameState> = Lazy::new(|| GameState {
    time: SystemTime::UNIX_EPOCH,
    x: 0.0,
    y: 0.0,
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
                Ok(c) => {
                    // here we can decide if we want to do anything with the client state given if it is different enough,
                    // this would allow us to only take changes if they are large enough, compressing how often we have to lock the game state, if we decide to be threadsafe.

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
