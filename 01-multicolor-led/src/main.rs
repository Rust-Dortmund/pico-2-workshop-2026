// We want to run on bare metal!
#![no_std]
#![no_main]

// Dependencies
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

// Our main function - place your code in HERE:
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get access to the pin(s) we need.
    // TODO: Change this so you can access the pins for the R, G and B channel 
    // of the multicolor LED.
    let Peripherals {
        PIN_25,
        ..
    } = embassy_rp::init(Default::default());

    // Set all pins to low initially.
    let mut led = Output::new(PIN_25, Level::Low);

    // Blink the LED on and off once every 2 seconds.
    // TODO: Cycle through the colors of the multicolor LED instead.
    let mut ticker = Ticker::every(Duration::from_millis(1000));
    loop {
        info!("Heartbeat");
        led.toggle();
        ticker.next().await;
    }
}

