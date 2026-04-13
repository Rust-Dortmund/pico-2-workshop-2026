#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals, gpio::{Level, Output},  peripherals::I2C1
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

// The `apds9960` library can work with any type that implements the `I2C` trait from `embedded_hal_async`.
// To save us some typing, create a type alias that has the RP Pico types already filled in.
type Apds9960 = apds9960::Apds9960<embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>, apds9960::Async>;

// TODO: Bind the interrupt for the I2C bus so we can get notified if there is new data.
//
// embassy_rp::bind_interrupts!(todo!());

// Our main function - place your code in HERE:
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get access to the pin(s) we need, and also the I2C bus peripheral.
    let Peripherals {
        PIN_14,
        PIN_15,
        PIN_18,
        PIN_19,
        PIN_20,
        I2C1,
        ..
    } = embassy_rp::init(Default::default());

    // Set all pins to low initially.
    let mut output_red = Output::new(PIN_18, Level::Low);
    let mut output_green = Output::new(PIN_19, Level::Low);
    let mut output_blue = Output::new(PIN_20, Level::Low);

    let sda = PIN_14;
    let scl = PIN_15;
    // TODO: Instantiate the I2C bus with the correct pins.

    // TODO: Create and initialize the driver for the APDS9960 sensor.

    // Change LED color every half a second.
    // TODO: Make the loop check the sensor's proximity value instead and have the LED react to it.
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
