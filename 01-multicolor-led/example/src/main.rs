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
        PIN_18,
        PIN_19,
        PIN_20,
        ..
    } = embassy_rp::init(Default::default());

    let mut output_red = Output::new(PIN_19, Level::Low);
    let mut output_green = Output::new(PIN_20, Level::Low);
    let mut output_blue = Output::new(PIN_18, Level::Low);

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        info!("Heartbeat");

        output_red.set_high();
        output_green.set_low();
        output_blue.set_low();
        ticker.next().await;

        output_red.set_low();
        output_green.set_high();
        output_blue.set_low();
        ticker.next().await;

        output_red.set_low();
        output_green.set_low();
        output_blue.set_high();
        ticker.next().await;
    }
}
