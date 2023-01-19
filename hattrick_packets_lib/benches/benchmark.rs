use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hattrick_packets_lib::clientstate::ClientState;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::gametypes::GameType::{PONG, TANK};
use hattrick_packets_lib::pong::PongGameState;
use hattrick_packets_lib::tank::{respawn_tank, TankBullet, TankGameState};
use hattrick_packets_lib::team::Team;
use hattrick_packets_lib::{
    distance, get_vote_count_for_number, point_distance, round_digits, round_number, Magnitude,
    GAME_HEIGHT, GAME_WIDTH,
};
use rand::Rng;
use std::time::SystemTime;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("round digits", |b| {
        b.iter(|| round_digits(black_box(&mut 9.87654321), black_box(5)))
    });
    c.bench_function("round number", |b| {
        b.iter(|| round_number(black_box(&9.87654321), black_box(5)))
    });
    c.bench_function("magnitude", |b| {
        b.iter(|| (black_box(1.23456789 as f32), black_box(9.87654321 as f32)).mag())
    });
    c.bench_function("get vote num", |b| {
        b.iter(|| {
            let mut gs = GameState::default();
            gs.client_list.insert(
                "test1".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );
            gs.client_list.insert(
                "test2".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 2,
                },
            );
            gs.client_list.insert(
                "test3".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );
            get_vote_count_for_number(1, &gs)
        })
    });
    c.bench_function("distance", |b| {
        b.iter(|| {
            distance(
                black_box(5.123),
                black_box(9.654),
                black_box(1.23456),
                black_box(7.6254),
            )
        })
    });
    c.bench_function("point distance", |b| {
        b.iter(|| {
            point_distance(
                (black_box(5.123), black_box(9.654)),
                (black_box(1.23456), black_box(7.6254)),
            )
        })
    });
    c.bench_function("pong ball step", |b| {
        b.iter(|| {
            let mut pgs = PongGameState::default();
            for _ in 0..100 {
                pgs.step_ball(&black_box(10.0));
            }
        })
    });
    c.bench_function("pong game state step", |b| {
        b.iter(|| {
            let mut gs = GameState::default();
            gs.client_list.insert(
                "test1".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );
            gs.client_list.insert(
                "test2".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 2,
                },
            );
            gs.client_list.insert(
                "test3".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );

            match &mut gs.game_type {
                PONG(pgs) => pgs.step_game_state(&gs.client_list),
                TANK(_tgs) => {
                    panic!("Error, game type was of type tank game type, this should not happen.");
                }
            }
        })
    });
    c.bench_function("tank bullet step", |b| {
        b.iter(|| {
            let mut tgs = TankGameState::default();
            for _ in 0..10 {
                tgs.bullets.push(TankBullet {
                    x: rand::thread_rng().gen_range(0.0..GAME_WIDTH),
                    y: rand::thread_rng().gen_range(0.0..GAME_HEIGHT),
                    x_vel: rand::thread_rng().gen_range(-5.0..5.0),
                    y_vel: rand::thread_rng().gen_range(-5.0..5.0),
                    bounce_count: 0,
                    team: Team::RedTeam,
                })
            }
            for _ in 0..1000 {
                for bullet in tgs.bullets.iter_mut() {
                    bullet.step(&10.0);
                }
                tgs.remove_dead_bullets();
            }
        })
    });
    c.bench_function("respawn tank", |b| {
        b.iter(|| {
            let mut gs = GameState {
                time: SystemTime::now(),
                game_type: TANK(TankGameState::default()),
                client_list: Default::default(),
                vote_running: false,
                vote_start_time: None,
            };
            gs.client_list.insert(
                "test1".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );
            gs.client_list.insert(
                "test2".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 2,
                },
            );
            gs.client_list.insert(
                "test3".to_string(),
                ClientState {
                    time: SystemTime::now(),
                    team_id: Team::RedTeam,
                    mouse_pos: (0.0, 0.0),
                    key_state: Default::default(),
                    pong_client_state: Default::default(),
                    tank_client_state: Default::default(),
                    vote_number: 1,
                },
            );
            for _ in 0..10 {
                for (_, mut client) in gs.client_list.clone() {
                    match &gs.game_type {
                        PONG(_) => {}
                        TANK(tgs) => {
                            respawn_tank(
                                &mut client.tank_client_state,
                                &tgs.bullets,
                                &mut gs.client_list,
                            );
                        }
                    }
                }
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
