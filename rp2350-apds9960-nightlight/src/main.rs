#![no_std]
#![no_main]

use apds9960::Apds9960Async;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
    i2c::InterruptHandler,
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

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let Peripherals {
        PIN_14,
        PIN_15,
        PIN_18,
        PIN_19,
        PIN_20,
        I2C1,
        ..
    } = embassy_rp::init(Default::default());

    let mut output_red = Output::new(PIN_19, Level::Low);
    let mut output_green = Output::new(PIN_20, Level::Low);
    let mut output_blue = Output::new(PIN_18, Level::Low);

    let sda = PIN_14;
    let scl = PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(I2C1, scl, sda, Irqs, config);
    let mut sensor = Apds9960Async::new(bus);

    let device_id = sensor.read_device_id().await.unwrap();
    info!("APDS9960 Device Id: {}", device_id);

    sensor.enable().await.unwrap();
    sensor.enable_light().await.unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        let brightness = sensor.read_light_clear().await.unwrap();
        info!("Brightness: {}", brightness);
        if brightness < 2 {
            output_red.set_high();
            output_green.set_high();
            output_blue.set_high();
        } else {
            output_red.set_low();
            output_green.set_low();
            output_blue.set_low();
        }

        ticker.next().await;
    }
}
