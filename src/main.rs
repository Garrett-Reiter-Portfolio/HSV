#![no_main]
#![no_std]

mod init;
mod conversion;
mod state;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use core::cell::RefCell;
use embedded_hal::digital::OutputPin;
use libm;

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{saadc, Saadc, gpio, Timer, gpiote::Gpiote, pac::{self, interrupt}},
};
use critical_section_lock_mut::LockMut;
use critical_section::Mutex;

//10ms (1s/1000ms) at 1MHz (1_000_000 ticks per second) count rate at 50% (/2)
static FRAME: f32 = ((10 * 1_000_000 / 1000) / 2) as f32;
static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static MODE: Mutex<RefCell<state::State>> = Mutex::new(RefCell::new(state::State::H));
static TIMER_R: LockMut<Timer<pac::TIMER1>> = LockMut::new();
static TIMER_G: LockMut<Timer<pac::TIMER2>> = LockMut::new();
static TIMER_B: LockMut<Timer<pac::TIMER3>> = LockMut::new();

static PIN_R: LockMut<gpio::Pin<gpio::Output<gpio::PushPull>>> = LockMut::new();
static PIN_G: LockMut<gpio::Pin<gpio::Output<gpio::PushPull>>> = LockMut::new();
static PIN_B: LockMut<gpio::Pin<gpio::Output<gpio::PushPull>>> = LockMut::new();
const EMPTY: [[u8; 5]; 5] = [[0; 5]; 5];

fn normalize_pot(saadc_result: Result<i16, ()>) -> f32 {
    let mut normalized = (saadc_result.unwrap() as f32 / 16000.0).clamp(0.0, 1.0);
    normalized = libm::roundf(normalized * 100.0) / 100.0;
    normalized
}

fn updateHSV(mode: state::State, current_value: f32, hsv: &mut conversion::Hsv) {
    match mode {
        state::State::H => {
            hsv.h = current_value;
        }
        state::State::S => {
            hsv.s = current_value;
        }
        state::State::V => {
            hsv.v = current_value;
        }
    }
}

fn if_zero(value: f32) -> f32 {
    if (value == 0.0) {
        1.0
    }
    else {
        value
    }
}


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


#[interrupt]
fn TIMER1() {
    TIMER_R.with_lock(|timer_r| {
        timer_r.reset_event()
    });
    PIN_R.with_lock(|pin_r| {
        pin_r.set_high().unwrap()
    });
}

#[interrupt]
fn TIMER2() {
    TIMER_G.with_lock(|timer_g| {
        timer_g.reset_event()
    });
    PIN_G.with_lock(|pin_g| {
        pin_g.set_high().unwrap()
    });
}

#[interrupt]
fn TIMER3() {
    TIMER_B.with_lock(|timer_b| {
        timer_b.reset_event()
    });
    PIN_B.with_lock(|pin_b| {
        pin_b.set_high().unwrap()
    });
}


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut leds: [[u8; 5]; 5] = EMPTY;
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut hsv = conversion::Hsv {h: 0.5, s: 0.5, v: 0.5};
    let mut rgb = conversion::Rgb {r: 0.2, g: 0.33, b: 0.2, };
    let mut timer_r = Timer::new(board.TIMER1);
    let mut timer_g = Timer::new(board.TIMER2);
    let mut timer_b = Timer::new(board.TIMER3);
    let pin_r = board.edge.e09.into_push_pull_output(gpio::Level::Low).degrade();
    let pin_g = board.edge.e08.into_push_pull_output(gpio::Level::Low).degrade();
    let pin_b = board.edge.e16.into_push_pull_output(gpio::Level::Low).degrade();
    let saadc_config = saadc::SaadcConfig::default();
    let mut saadc = Saadc::new(board.ADC, saadc_config);
    let mut saadc_pin = board.edge.e02.into_floating_input();
    let mut saadc_result: Result<i16, ()>;
    let mut norm_pot: f32;

    //let _: () = pin_r;    //compiler will tell me type

    PIN_R.init(pin_r);
    PIN_G.init(pin_g);
    PIN_B.init(pin_b);

    timer_r.enable_interrupt();
    timer_r.reset_event();
    TIMER_R.init(timer_r);

    timer_g.enable_interrupt();
    timer_g.reset_event();
    TIMER_G.init(timer_g);

    timer_b.enable_interrupt();
    timer_b.reset_event();
    TIMER_B.init(timer_b);


    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
        pac::NVIC::unmask(pac::Interrupt::TIMER2);
        pac::NVIC::unmask(pac::Interrupt::TIMER3);
    }
    pac::NVIC::unpend(pac::Interrupt::TIMER1);
    pac::NVIC::unpend(pac::Interrupt::TIMER2);
    pac::NVIC::unpend(pac::Interrupt::TIMER3);


    init::init_buttons(board.GPIOTE, board.buttons);

    rprintln!("Hi from the program");
    loop {
        let mut mode = critical_section::with(|cs| *MODE.borrow(cs).borrow());
        state::update_led(&mut leds, &mut mode);
        saadc_result = saadc.read_channel(&mut saadc_pin);
        norm_pot = normalize_pot(saadc_result);

        updateHSV(mode, norm_pot,&mut hsv);
        rgb = hsv.to_rgb();

        PIN_R.with_lock(|pin_r| {
            pin_r.set_low().unwrap()
        });
        PIN_G.with_lock(|pin_g| {
            pin_g.set_low().unwrap()
        });
        PIN_B.with_lock(|pin_b| {
            pin_b.set_low().unwrap()
        });
        

        TIMER_R.with_lock(|timer_r| {
            timer_r.start(if_zero(rgb.r * FRAME) as u32);
        });
        TIMER_G.with_lock(|timer_g| {
            timer_g.start(if_zero(rgb.g * FRAME) as u32);
        });
        TIMER_B.with_lock(|timer_b| {
            timer_b.start(if_zero(rgb.b * FRAME) as u32);
        });
        

        display.show(&mut timer, leds, 10);

    }
}


