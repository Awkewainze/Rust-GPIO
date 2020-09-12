extern crate ctrlc;

use std::error::Error;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rppal::gpio::Gpio;
use rppal::gpio::Trigger;
use rppal::gpio::InputPin;

// Gpio uses BCM pin numbering.
// BCM GPIO 4 is tied to physical pin 7.
// BCM GPIO 17 is tied to physical pin 11.
const GPIO_LED: u8 = 4;
const GPIO_BUT: u8 = 17;

fn set_interrupt_handler(should_continue: Arc<AtomicBool>, pause: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        pause.swap(false, Ordering::Relaxed);
        should_continue.swap(false, Ordering::Relaxed);
    }).expect("Error setting interrupt handler");
}

fn set_button_interrupt_handler(pause: Arc<AtomicBool>) -> Result<InputPin, Box<dyn Error>> {
    let mut but_pin = Gpio::new()?.get(GPIO_BUT)?.into_input();
    but_pin.set_async_interrupt(Trigger::RisingEdge, move |_| {
        pause.swap(!pause.load(Ordering::Relaxed), Ordering::Relaxed);
    }).expect("Error setting button handler");
    Ok(but_pin)
}

fn main() -> Result<(), Box<dyn Error>> {
    let should_continue: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let pause: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut led_pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    set_interrupt_handler(should_continue.clone(), pause.clone());
    let _but_pin = set_button_interrupt_handler(pause.clone()).expect("Error getting pin");

    while should_continue.load(Ordering::Relaxed) {
        led_pin.set_high();
        thread::sleep(Duration::from_millis(500));
        led_pin.set_low();
        thread::sleep(Duration::from_millis(500));
        while pause.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(500));
        }
    }
    Ok(())
}