use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// KeyState is a struct that contains all keys that the game listens to.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyState {
    pub w_key: bool,
    pub a_key: bool,
    pub s_key: bool,
    pub d_key: bool,
    pub space_bar: bool,
}

#[cfg(feature = "client")]
pub mod client {
    use crate::keystate::KeyState;
    use macroquad::prelude::KeyCode;

    impl KeyState {
        /// new() by default returns a key state with all inputs checked from the keyboard.
        pub fn new() -> KeyState {
            KeyState {
                w_key: macroquad::prelude::is_key_down(KeyCode::W),
                a_key: macroquad::prelude::is_key_down(KeyCode::A),
                s_key: macroquad::prelude::is_key_down(KeyCode::S),
                d_key: macroquad::prelude::is_key_down(KeyCode::D),
                space_bar: macroquad::prelude::is_key_down(KeyCode::Space),
            }
        }
    }
}

impl Default for KeyState {
    /// initialized all states to false.
    fn default() -> KeyState {
        KeyState {
            w_key: false,
            a_key: false,
            s_key: false,
            d_key: false,
            space_bar: false,
        }
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "W:{}, A:{}, S:{}, D:{}, SPACE:{}",
            self.w_key, self.a_key, self.s_key, self.d_key, self.space_bar
        )
    }
}
