pub mod clientinfo;
pub mod clientstate;
pub mod gamestate;
pub mod gametypes;
pub mod keystate;
pub mod pong;
pub mod tank;
pub mod team;

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
pub fn get_angle_from_point_to_point(point: (f32, f32), point2: (f32, f32)) -> f32 {
    let point_to_mouse_pos_vector = vector_subtract((point.0, point.1), point2);
    get_angle_of_travel_degrees(
        point.0,
        point.1,
        point_to_mouse_pos_vector.0,
        point_to_mouse_pos_vector.1,
    )
}

/// Return the degree angle of travel based on the x,y,xvel,yvel of the object.
pub fn get_angle_of_travel_degrees(x: f32, y: f32, xvel: f32, yvel: f32) -> f32 {
    let next_x = x + xvel;
    let next_y = y + yvel;
    (next_y - y).atan2(next_x - x).to_degrees()
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
