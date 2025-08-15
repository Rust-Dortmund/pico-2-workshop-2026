#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, config::Config, interrupt::Priority};
use embassy_time::{Duration, Ticker};

// Panic handler
#[allow(unused_imports)]
use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(
    struct Irqs {
        // TODO: Not sure what is needed here
        //SAADC => saadc::InterruptHandler;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // Configure interrupt priorities to exclude 0 (default), 1, and 4,
    // which are reserved for the SoftDevice
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let _ = embassy_nrf::init(config);

    let mut ticker = Ticker::every(Duration::from_millis(1000));
    loop {
        defmt::info!("Heartbeat");
        ticker.next().await;
    }
    // TODO: Draw something to the display
    // TODO: Draw the buttons to the display
    // TODO: Get touch input up and running
    // TODO: On button press change button color
    // TODO: Get bluetooth up and running (pairing with other device etc..)
    // TODO: On button press send color change command to other device
    // TODO: Disable button for current color
    // TODO: Receive notifications for color change
}
