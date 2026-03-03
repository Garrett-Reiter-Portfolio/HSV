
#[derive(Copy, Clone)]
pub enum State {
    H,  //hue
    S,  //saturation
    V,  //value
}

impl State {

    pub fn next(&mut self) {
        *self = match *self {
            State::H => State::S,
            State::S => State::V,
            State::V => State::H,
        }
    }

    pub fn prev(&mut self) {
        *self = match *self {
            State::H => State::V,
            State::S => State::H,
            State::V => State::S,
        }
    }
} 

pub fn update_led (leds: &mut [[u8; 5]; 5], mode: &mut State) {
    match mode {
        State::H => {
            *leds = H;
        }
        State::S => {
            *leds = S;
        }
        State::V => {
            *leds = V;
        }
    }
}

pub const H: [[u8; 5]; 5] = [
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 1, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
];
pub const S: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [0, 1, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 1, 0],
    [0, 1, 1, 1, 0],
];
pub const V: [[u8; 5]; 5] = [
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 1, 0, 1, 0],
    [0, 0, 1, 0, 0],
];

