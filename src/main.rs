#![no_main]
#![no_std]

mod init;
mod conversion;
mod state;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use core::cell::RefCell;

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{Timer, gpiote::Gpiote, pac::interrupt},
};
//use critical_section_lock_mut::LockMut;
use critical_section::Mutex;
//use init;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static MODE: Mutex<RefCell<state::State>> = Mutex::new(RefCell::new(state::State::H));
//const EMPTY: [[u8; 5]; 5] = [[0; 5]; 5];

#[interrupt]
fn GPIOTE() {
     critical_section::with(|cs| {
         if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
             let a_pressed = gpiote.channel0().is_event_triggered();
             let b_pressed = gpiote.channel1().is_event_triggered();

             let mut mode = MODE.borrow(cs).borrow_mut();
             match (a_pressed, b_pressed) {
                 (true, false) => {
                     mode.prev();
                 }
                 (false, true) => {
                     mode.next();
                 }
                 _ => {}
             }
             gpiote.channel0().reset_events();
             gpiote.channel1().reset_events();
         } else {
             rprintln!("GPIOTE interrupt but GPIO not initialized!");
         }
     });
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut leds: [[u8; 5]; 5];
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    init::init_buttons(board.GPIOTE, board.buttons);

    rprintln!("Hi from the program");
    loop {
        let mut mode = critical_section::with(|cs| *MODE.borrow(cs).borrow());
        match mode {
            state::State::H => {
                leds = state::H;
            }
            state::State::S => {
                leds = state::S;
            }
            state::State::V => {
                leds = state::V;
            }
        }
        display.show(&mut timer, leds, 100);

    }
}


