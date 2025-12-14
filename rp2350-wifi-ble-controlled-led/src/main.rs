#![no_std]
#![no_main]
// For picoserve.
#![feature(impl_trait_in_assoc_type)]

mod ble;
mod led;
mod led_controller;
mod mk_static;
mod webserver;

use crate::{
    ble::{Ble, BleConnectionRunner},
    led_controller::LedControllerRunner,
    webserver::WebserverRunner,
};
use cyw43::{JoinOptions, NetDriver, bluetooth::BtDriver};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
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
use static_cell::StaticCell;
use trouble_host::prelude::{DefaultPacketPool, ExternalController, Runner as BleRunner};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_cargo_bin_name!(),
    embassy_rp::binary_info::rp_program_description!(c"WiFi and BLE controlled LED example"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// Load SSID and WiFi password from environment variables at build time.
const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

const WEB_TASK_POOL_SIZE: usize = 8;
const NET_STACK_RESOURCES: usize = WEB_TASK_POOL_SIZE + 3;

bind_interrupts!(struct Irqs {
    // PIO0 is used to emulate the non-standard SPI protocol used by CYW43 (the WiFi / BLE SoC).
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
        while !self.network_stack.is_link_up() {
            Timer::after(Duration::from_millis(500)).await;
        }

        info!("Waiting to get IP address...");
        loop {
            if let Some(config) = self.network_stack.config_v4() {
                let address = config.address.address().octets();
                info!(
                    "Got IP: {}.{}.{}.{}",
                    address[0], address[1], address[2], address[3]
                );
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}

/// Task for [`PrintIpRunner`].
#[embassy_executor::task]
async fn run_print_ip(runner: PrintIpRunner) {
    runner.run().await
}

/// Task driving the LED.
#[embassy_executor::task]
async fn run_led_controller(runner: LedControllerRunner) {
    runner.run().await.expect("Infallible");
}

/// Task running the webserver.
#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn run_webserver(runner: WebserverRunner) -> ! {
    runner.run().await
}

/// Task running the BLE stack.
#[embassy_executor::task]
async fn run_ble(
    mut runner: BleRunner<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
) {
    runner.run().await.expect("BLE stack error")
}

/// Task driving the BLE connections.
#[embassy_executor::task]
async fn run_ble_connection(runner: BleConnectionRunner) {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    let mut rng = RoscRng;
    let cyw43_firmware = include_bytes!("../cyw43-firmware/43439A0.bin");
    let cyw43_clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let cyw43_bluetooth_firmware = include_bytes!("../cyw43-firmware/43439A0_btfw.bin");

    let cyw43_power = Output::new(PIN_23, Level::Low);
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

    let (led_controller_runner, watch) = led_controller::initialize(PIN_19, PIN_20, PIN_18);

    info!("Initializing CYW43");

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (network_device, bluetooth_driver, mut control, runner) = cyw43::new_with_bluetooth(
        state,
        cyw43_power,
        spi,
        cyw43_firmware,
        cyw43_bluetooth_firmware,
    )
    .await;

    // The CYW43 must be operable when we create the network stack, so we have to spawn its task before doing so.
    info!("Spawning CYW43 task");
    spawner.must_spawn(cyw43_task(runner));
    info!("Spawned CYW43 task");

    control.init(cyw43_clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    info!("Initialized CYW43");

    info!("Creating network stack");

    let network_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = rng.next_u64();
    let (network_stack, network_runner) = embassy_net::new(
        network_device,
        network_config,
        mk_static!(
            StackResources<{ NET_STACK_RESOURCES }>,
            StackResources::new()
        ),
        seed,
    );

    let print_ip_runner = PrintIpRunner { network_stack };

    info!("Created network stack");

    let Ble {
        ble_runner,
        connection_runner: ble_connection_runner,
    } = ble::initialize(bluetooth_driver, watch.sender(), watch.receiver().unwrap());

    let mut webserver_task_factory = webserver::initialize(network_stack, watch.sender());

    info!("Spawning tasks");

    spawner.must_spawn(run_led_controller(led_controller_runner));
    spawner.must_spawn(run_network(network_runner));
    spawner.must_spawn(run_print_ip(print_ip_runner));

    // Note that we are running multiple tasks for handling requests to the webserver!
    for _ in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(run_webserver(webserver_task_factory.new_runner()));
    }

    spawner.must_spawn(run_ble(ble_runner));
    spawner.must_spawn(run_ble_connection(ble_connection_runner));

    info!("Tasks spawned");

    info!("Joining network");

    control
        .join(SSID, JoinOptions::new(PASSWORD.as_bytes()))
        .await
        .unwrap();

    info!("Joined network");

    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
