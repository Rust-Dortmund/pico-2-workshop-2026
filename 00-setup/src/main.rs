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

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_cargo_bin_name!(),
    embassy_rp::binary_info::rp_program_description!(
        c"Hello world, sends a heartbeat message each second"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

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
