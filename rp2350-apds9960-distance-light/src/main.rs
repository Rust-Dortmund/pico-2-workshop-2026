#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals, gpio::{Level, Output}, i2c::InterruptHandler, peripherals::I2C1
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

type Apds9960 = apds9960::Apds9960<embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>, apds9960::Async>;

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
    let mut sensor = Apds9960::new(bus);

    let device_id = sensor.read_device_id().await.unwrap();
    info!("APDS9960 Device Id: {}", device_id);

    sensor.enable().await.unwrap();
    sensor.enable_proximity().await.unwrap();

    output_blue.set_low();

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        let proximity = sensor.read_proximity().await.unwrap();
        info!("Proximity: {}", proximity);
        match proximity {
            x if (0..2).contains(&x) => {
                output_red.set_low();
                output_green.set_low();
            }
            x if (2..10).contains(&x) => {
                output_red.set_low();
                output_green.set_high();
            }
            x if (10..200).contains(&x) => {
                output_red.set_high();
                output_green.set_high();
            }
            _ => {
                output_red.set_high();
                output_green.set_low();
            }
        }

        ticker.next().await;
    }
}
