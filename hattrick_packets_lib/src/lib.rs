use crate::gamestate::GameState;
use std::ops::Deref;

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
    let multiple = 10.0_f32.powi(digits as i32);
    *num = (*num * multiple).round() / multiple;
}

// not really needed right now.
// pub fn get_vector_magnitude(vector: (f32,f32)) -> f32 {
//     (vector.0.powi(2) + vector.1.powi(2)).sqrt()
// }

pub trait Magnitude {
    fn mag(&self) -> f32;
}

impl Magnitude for (f32, f32) {
    fn mag(&self) -> f32 {
        let v: (f32, f32) = *self.deref();

        (v.0.powi(2) + v.1.powi(2)).sqrt()
    }
}

/// Round a given number to a number of digits.
pub fn round_number(num: &f32, digits: u32) -> f32 {
    let multiple = 10.0_f32.powi(digits as i32);
    (num * multiple).round() / multiple
}

pub fn get_vote_count_for_number(number: u8, game_state: &GameState) -> u32 {
    // TODO: could eventually make this run for every number, but for now I dont think I need this.
    let count = game_state
        .client_list
        .iter()
        .map(|(_, client)| client.vote_number) // map each client to their vote number, that all we care about
        .fold(0, |count, vote_number| {
            // here we reduce the iter to the count respective to that number.
            if vote_number == number {
                count + 1 // if the vote number is the number we are looking for, then add the count
            } else {
                count // if not, then dont increment the count.
            }
        });

    count
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
    use crate::{distance, round_number, two_point_angle};

    #[test]
    fn round_num_test() {
        let first_num = 10.12345;
        let rounded_num = 10.12;
        assert_eq!(round_number(&first_num, 2), rounded_num);
        let first_num = -987.654_3;
        let rounded_num = -987.654;
        assert_eq!(round_number(&first_num, 3), rounded_num);
    }

    #[test]
    fn distance_test() {
        let x1 = -1.0;
        let y1 = 0.0;
        let x2 = 1.0;
        let y2 = 0.0;
        assert_eq!(distance(x1, y1, x2, y2), 2.0);
    }

    #[test]
    fn angle_points_diagonal_test() {
        let point1 = (0.0, 0.0);
        let point2 = (0.5, 0.5);
        assert_eq!(two_point_angle(point1, point2), 45.0);
        let point2 = (0.0, 0.0);
        let point1 = (0.5, 0.5);
        assert_eq!(two_point_angle(point1, point2), 45.0 - 180.0);
    }

    #[test]
    fn angle_points_upwards_test() {
        let point1 = (0.0, 0.0);
        let point2 = (0.0, 1.0);
        assert_eq!(two_point_angle(point1, point2), 90.0);
        let point2 = (0.0, 0.0);
        let point1 = (0.0, 1.0);
        assert_eq!(two_point_angle(point1, point2), 90.0 - 180.0);
    }

    #[test]
    fn angle_points_downwards_test() {
        let point1 = (0.0, 0.0);
        let point2 = (0.0, -1.0);
        assert_eq!(two_point_angle(point1, point2), -90.0);
        let point2 = (0.0, 0.0);
        let point1 = (0.0, -1.0);
        assert_eq!(two_point_angle(point1, point2), -90.0 + 180.0);
    }

    #[test]
    fn angle_points_right_test() {
        let point1 = (0.0, 0.0);
        let point2 = (1.0, 0.0);
        assert_eq!(two_point_angle(point1, point2), 0.0);
        let point2 = (0.0, 0.0);
        let point1 = (1.0, 0.0);
        assert_eq!(two_point_angle(point1, point2), 0.0 + 180.0);
    }

    #[test]
    fn angle_points_left_test() {
        let point1 = (0.0, 0.0);
        let point2 = (-1.0, 0.0);
        assert_eq!(two_point_angle(point1, point2), 180.0);
        let point2 = (0.0, 0.0);
        let point1 = (-1.0, 0.0);
        assert_eq!(two_point_angle(point1, point2), 180.0 - 180.0);
    }
}
