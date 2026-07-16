#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    i2c::InterruptHandler,
    peripherals::I2C1,
    pwm::{Pwm, PwmOutput, SetDutyCycle},
};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

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

// The maximum brightness that is typically reported by the APDS-9960.
const APDS_9960_MAX_BRIGHTNESS: u16 = 1024;

// Our main function - place your code in HERE:
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get access to the pin(s), PWM slices and I2C bus peripheral.
    let Peripherals {
        PIN_14,
        PIN_15,
        PIN_18,
        PIN_19,
        PIN_20,
        I2C1,
        ..
    } = embassy_rp::init(Default::default());

    // Instead of Output, as in the previous exercise, use Pwm so we can drive the LED with variable brightness.
    let pwm_red_green = Pwm::new_output_ab(
        todo!("Choose the right PWM slice"),
        todo!("Choose the right pin"),
        todo!("Choose the right pin"),
        Default::default(),
    );
    let (pwm_red, pwm_green) = pwm_red_green.split();
    let mut pwm_red = pwm_red.unwrap();
    let mut pwm_green = pwm_green.unwrap();

    let mut pwm_blue: PwmOutput = todo!("Initialize the PWM instance for the blue LED pin");

    // Instantiate the I2C bus with the correct pins.
    let sda = PIN_14;
    let scl = PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(I2C1, scl, sda, Irqs, config);

    // Create and initialize the driver for the APDS9960 sensor.
    let mut sensor = Apds9960::new(bus);

    // TODO: Enable the neccessary sensor components.

    // Make the loop check the sensor's light value and have the LED react to it.
    let mut ticker = Ticker::every(Duration::from_millis(50));
    loop {
        // Obtain a brightness reading from the sensor.
        let brightness: u16 = todo!("Obtain a brightness reading");
        info!("Brightness: {}", brightness);

        // Compute the duty cycle from the brightness.
        let effective_duty_cycle_percent: u8 = todo!("Compute the duty cycle from brightness");

        pwm_red
            .set_duty_cycle_percent(effective_duty_cycle_percent)
            .unwrap();
        pwm_green
            .set_duty_cycle_percent(effective_duty_cycle_percent)
            .unwrap();
        pwm_blue
            .set_duty_cycle_percent(effective_duty_cycle_percent)
            .unwrap();

        ticker.next().await;
    }
}
