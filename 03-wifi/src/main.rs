#![no_std]
#![no_main]

// Required for the `picoserve` webserver.
#![feature(impl_trait_in_assoc_type)]

mod led;
mod led_controller;
mod mk_static;
mod webserver;

use crate::{
    led_controller::{ColorWatch, LedControllerRunner},
    webserver::{WebserverRunner, WebserverRunnerFactory},
};
use cyw43::{JoinOptions, NetDriver};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_rp::{
    Peripherals, bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Include the firmware for the WiFi chip.
const CYW43_FIRMWARE: &[u8; 231_077] = include_bytes!("../../cyw43-firmware/43439A0.bin");
const CYW43_CLM: &[u8; 984] = include_bytes!("../../cyw43-firmware/43439A0_clm.bin"); 

// Load SSID and WiFi password from environment variables at build time.
const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

// How many concurrent requests can we handle?
const WEB_TASK_POOL_SIZE: usize = 8;
const NET_STACK_RESOURCES: usize = WEB_TASK_POOL_SIZE + 3;

// PIO0 is used to emulate the non-standard SPI protocol used by CYW43 (the WiFi / BLE SoC onboard 
// the Pico 2).
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Task driving the CYW43 WiFi / BLE SoC.
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

/// Task driving the network stack.
#[embassy_executor::task]
async fn run_network(mut runner: embassy_net::Runner<'static, NetDriver<'static>>) {
    runner.run().await
}

/// Runner that prints the IP once a WiFi connection was established.
pub(crate) struct PrintIpRunner {
    network_stack: embassy_net::Stack<'static>,
}
impl PrintIpRunner {
    pub(crate) async fn run(self) {
        // Wait for the network to connect to _something_.
        while !self.network_stack.is_link_up() {
            Timer::after(Duration::from_millis(500)).await;
        }

        // Now wait for DHCP to happen so the Pico 2 gets assigned an IP.
        info!("Waiting to get IP address...");
        self.network_stack.wait_config_up().await;
        let config = self.network_stack.config_v4().expect("we can only do V4 and just waited for a config to be available");
        let address = config.address.address().octets();
        info!(
            "Got IP: {}.{}.{}.{}",
            address[0], address[1], address[2], address[3]
        );
    }
}

/// Task for [`PrintIpRunner`].
#[embassy_executor::task]
async fn run_print_ip(runner: PrintIpRunner) {
    runner.run().await;
}

/// Task driving the LED.
#[embassy_executor::task]
async fn run_led_controller(runner: LedControllerRunner) {
    runner.run().await;
}

/// Task running the webserver.
#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn run_webserver(runner: WebserverRunner) -> ! {
    runner.run().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get access to the pin(s) we need, and also the peripherals for the emulated PIO SPI bus.
    let Peripherals {
        PIN_18,
        PIN_19,
        PIN_20,
        PIN_23,
        PIN_24,
        PIN_25,
        PIN_29,
        PIO0,
        DMA_CH0,
        ..
    } = embassy_rp::init(Default::default());

    // Set up communication with the WiFi chip.
    let cyw43_chip_select = Output::new(PIN_25, Level::High);
    let mut pio = Pio::new(PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cyw43_chip_select,
        PIN_24,
        PIN_29,
        DMA_CH0,
    );
    info!("Initializing CYW43");
    let state = mk_static!(cyw43::State, cyw43::State::new());
    let cyw43_power = Output::new(PIN_23, Level::Low);
    let (network_device, mut control, cyw43_runner) = cyw43::new(
        state,
        cyw43_power,
        spi,
        CYW43_FIRMWARE,
    )
    .await;

    // The CYW43 must be operable when we create the network stack, so we have to spawn its task 
    // before doing so.
    info!("Spawning CYW43 task");
    spawner.must_spawn(cyw43_task(cyw43_runner));

    // Continue initializing
    control.init(CYW43_CLM).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    info!("Initialized CYW43");

    // Next layer: the `embassy_net` stack
    info!("Creating network stack");
    let network_config = embassy_net::Config::dhcpv4(Default::default());
    let (network_stack, network_runner) = embassy_net::new(
        network_device,
        network_config,
        mk_static!(
            StackResources<{ NET_STACK_RESOURCES }>,
            StackResources::new()
        ),
        RoscRng.next_u64(),
    );
    let print_ip_runner = PrintIpRunner { network_stack };
    info!("Created network stack");

    // Now our tasks:
    info!("Initializing LED controller");
    let (led_controller_runner, watch): (LedControllerRunner, &'static ColorWatch) = todo!("Initialize LED controller");

    info!("Initializing web server");
    let mut webserver_task_factory: WebserverRunnerFactory = todo!("Initialize the webserver");

    info!("Spawning tasks");
    spawner.must_spawn(run_print_ip(print_ip_runner));
    spawner.must_spawn(run_led_controller(led_controller_runner));
    spawner.must_spawn(run_network(network_runner));

    // Note that we are running multiple tasks for handling requests to the webserver!
    for _ in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(run_webserver(webserver_task_factory.new_runner()));
    }
    info!("Tasks spawned");

    // Finally, connect to the local WiFi once everything is running.
    info!("Joining network");
    todo!("Actually join WiFi network");
    info!("Joined network");

    // Not much to do in `main` anymore, since all of the networking stuff runs through the stack.
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("Hello from main!");
    }
}
