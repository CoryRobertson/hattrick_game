pub mod packets;

pub static PONG_PADDLE_WIDTH: f32 = 100.0;
pub static PONG_PADDLE_HEIGHT: f32 = 10.0;
pub static PONG_BALL_RADIUS: f32 = 15.0;


#[cfg(test)]
mod tests {
    use crate::packets::ClientState;
    use std::ops::Add;
    use std::time::{Duration, SystemTime};

    #[test]
    fn difference_check_test() {
        // dummy unit test, maybe add some later
        let cs1 = ClientState {
            time: SystemTime::now(),
            mouse_pos: (0.0, 0.0),
            team_id: 0
        };
        let cs2 = ClientState {
            time: cs1.time.clone(),
            mouse_pos: (0.8, 0.0),
            team_id: 0
        };
        let cs3 = ClientState {
            time: SystemTime::now().add(Duration::from_secs(2)),
            mouse_pos: (10.0, 0.0),
            team_id: 0
        };
        let cs4 = ClientState {
            time: SystemTime::now().add(Duration::from_secs(3)),
            mouse_pos: (10.0, 10.0),
            team_id: 0
        };
        assert_eq!(cs1.differ_count(&cs1), 0);
        assert_eq!(cs1.differ_count(&cs2), 1);
        assert_eq!(cs1.differ_count(&cs3), 2);
        assert_eq!(cs1.differ_count(&cs4), 3);
    }
}
