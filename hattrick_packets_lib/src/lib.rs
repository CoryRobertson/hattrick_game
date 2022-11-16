pub mod clientinfo;
pub mod clientstate;
pub mod gamestate;
pub mod gametypes;
pub mod keystate;
pub mod pong;
pub mod tank;
pub mod team;

pub static GAME_WIDTH: f32 = 600.0;
pub static GAME_HEIGHT: f32 = 600.0;

/// Round a given number to a given number of digits.
pub fn round_digits(num: &mut f32, digits: u32) {
    let multiple = (10.0 as f32).powi(digits as i32);
    *num = ((*num * multiple) as f32).round() / multiple;
}

/// Round a given number to a number of digits.
pub fn round_number(num: &f32, digits: u32) -> f32 {
    let multiple = (10.0 as f32).powi(digits as i32);
    ((num * multiple) as f32).round() / multiple
}

/// Subtract two vectors and return a new vector that would be A pointing towards B.
pub fn vector_subtract(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    let x = b.0 - a.0;
    let y = b.1 - a.1;
    (x, y)
}

/// Finds the given angle from a two points
pub fn two_point_angle(point: (f32, f32), point2: (f32, f32)) -> f32 {
    (point2.1 - point.1).atan2(point2.0 - point.0).to_degrees()
}

/// Return the degree angle of travel based on the x,y,xvel,yvel of the object.
pub fn get_angle_of_travel_degrees(x: f32, y: f32, xvel: f32, yvel: f32) -> f32 {
    let next_x = x + xvel;
    let next_y = y + yvel;
    (next_y - y).atan2(next_x - x).to_degrees()
}

/// Returns the distance between two points
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let g1 = (x2 - x1).powi(2);
    let g2 = (y2 - y1).powi(2);
    (g1 + g2).sqrt()
}

#[cfg(test)]
mod tests {

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
