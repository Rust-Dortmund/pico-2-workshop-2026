#![no_std]
#![no_main]

mod gesture;
mod led_state;

use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
    i2c::InterruptHandler,
    peripherals::I2C1,
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::{gesture::GestureDetector, led_state::LedState};

// The `apds9960` library can work with any type that implements the `I2C` trait from `embedded_hal_async`.
// To save us some typing, create a type alias that has the RP Pico types already filled in.
type Apds9960 = apds9960::Apds9960<
    embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>,
    apds9960::Async,
>;

// Bind the interrupt for the I2C bus so we can get notified if there is new data.
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

    // Instantiate the output pins for controlling the LED.
    let mut output_red = Output::new(PIN_18, Level::Low);
    let mut output_green = Output::new(PIN_19, Level::Low);
    let mut output_blue = Output::new(PIN_20, Level::Low);

    // Instantiate the I2C bus with the correct pins.
    let sda = PIN_14;
    let scl = PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(I2C1, scl, sda, Irqs, config);

    // Create and initialize the driver for the APDS9960 sensor.
    let mut sensor = Apds9960::new(bus);

    // Enable the neccessary sensor components.
    sensor.enable().await.unwrap();
    // TODO: Enable gesture detection.

    // Instantiate the GetsureDetector.
    let mut detector = GestureDetector::new(sensor);

    let mut state = LedState::Off;

    loop {
        // TODO: Wait until a gesture is detected.
        // TODO: Find and apply the next LED state.
        // TODO: Wait for the gesture detection to stop to avoid accidental progressing the LED state.
        Timer::after(Duration::from_secs(1)).await;
    }
}
