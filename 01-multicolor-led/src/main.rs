#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let Peripherals {
        PIN_25,
        ..
    } = embassy_rp::init(Default::default());

    let mut led = Output::new(PIN_25, Level::Low);
    let mut ticker = Ticker::every(Duration::from_millis(1000));
    loop {
        info!("Heartbeat");
        led.toggle();
        ticker.next().await;
    }
}

