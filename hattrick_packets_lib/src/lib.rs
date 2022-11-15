pub mod clientinfo;
pub mod clientstate;
pub mod gamestate;
pub mod gametypes;
pub mod keystate;
pub mod pong;
pub mod tank;
pub mod team;

pub fn round_digits(num: &mut f32, digits: u32) {
    let multiple = (10.0 as f32).powi(digits as i32);
    *num = ((*num * multiple) as f32).round() / multiple;
}

#[cfg(test)]
mod tests {
    use crate::packets::ClientState;
    use crate::packets::Team::{BlueTeam, RedTeam};
    use std::ops::Add;
    use std::time::{Duration, SystemTime};

    #[test]
    fn difference_check_test() {
        // dummy unit test, maybe add some later
        // let cs1 = ClientState {
        //     time: SystemTime::now(),
        //     pos: (0.0, 0.0),
        //     team_id: BlueTeam,
        // };
        // let cs2 = ClientState {
        //     time: cs1.time.clone(),
        //     pos: (0.8, 0.0),
        //     team_id: BlueTeam,
        // };
        // let cs3 = ClientState {
        //     time: SystemTime::now().add(Duration::from_secs(2)),
        //     pos: (10.0, 0.0),
        //     team_id: BlueTeam,
        // };
        // let cs4 = ClientState {
        //     time: SystemTime::now().add(Duration::from_secs(3)),
        //     pos: (10.0, 10.0),
        //     team_id: BlueTeam,
        // };
        // let cs5 = ClientState {
        //     time: SystemTime::now().add(Duration::from_secs(3)),
        //     pos: (10.0, 10.0),
        //     team_id: RedTeam,
        // };
        // assert_eq!(cs1.differ_count(&cs1), 0);
        // assert_eq!(cs1.differ_count(&cs2), 1);
        // assert_eq!(cs1.differ_count(&cs3), 2);
        // assert_eq!(cs1.differ_count(&cs4), 3);
        // assert_eq!(cs4.differ_count(&cs5), 1);
    }
}
