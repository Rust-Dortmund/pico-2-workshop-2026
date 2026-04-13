#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals, gpio::{Level, Output}, i2c::InterruptHandler, peripherals::I2C1
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

// The `apds9960` library can work with any type that implements the `I2C` trait from `embedded_hal_async`.
// To save us some typing, create a type alias that has the RP Pico types already filled in.
type Apds9960 = apds9960::Apds9960<embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>, apds9960::Async>;

// Bind the interrupt for the I2C bus so we can get notified if there is new data.
embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<embassy_rp::peripherals::I2C1>;
});

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
    // We don't need the blue pin for the traffic light, so just make sure to turn it off initially.
    let mut output_red = Output::new(PIN_18, Level::Low);
    let mut output_green = Output::new(PIN_19, Level::Low);
    let _output_blue = Output::new(PIN_20, Level::Low); 

    // Instantiate the I2C bus with the correct pins.
    let sda = PIN_14;
    let scl = PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(I2C1, scl, sda, Irqs, config);

    // Create and initialize the driver for the APDS9960 sensor.
    let mut sensor = Apds9960::new(bus);
    sensor.enable().await.unwrap();
    sensor.enable_proximity().await.unwrap();

    // Make the loop check the sensor's proximity value instead and have the LED react to it.
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        let proximity = sensor.read_proximity().await.unwrap();
        info!("Proximity: {}", proximity);
        match proximity {
            0..2 => {
                // Not in range, so no traffic light at all.
                output_red.set_low();
                output_green.set_low();
            }
            2..10 => {
                // Noticed something, but it's far away - show a green light.
                output_red.set_low();
                output_green.set_high();
            }
            10..200 => {
                // Getting close! Yellow light to indicate that you should stop your approach.
                output_red.set_high();
                output_green.set_high();
            }
            200.. => {
                // Too-close-for-comfort zone! Red light!
                output_red.set_high();
                output_green.set_low();
            }
        }

        ticker.next().await;
    }
}
