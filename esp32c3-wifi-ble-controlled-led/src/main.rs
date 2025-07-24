//! embassy WiFi controlled LED
//!
//! This is an example of controlling an LED over WiFi.

#![no_std]
#![no_main]
// For picoserve.
#![feature(impl_trait_in_assoc_type)]

mod ble;
mod led;
mod led_controller;
mod mk_static;
mod webserver;
mod wifi;

use embassy_executor::Spawner;
use embassy_net::Runner;
use esp_alloc as _;
use esp_backtrace as _;
use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
use esp_hal::peripherals::{RADIO_CLK, TIMG0};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_wifi::{EspWifiController, ble::controller::BleConnector, init, wifi::WifiDevice};
use trouble_host::prelude::{DefaultPacketPool, ExternalController, Runner as BleRunner};

use crate::ble::{Ble, BleConnectionRunner};
use crate::led_controller::LedControllerRunner;
use crate::webserver::WebserverRunner;
use crate::wifi::{ConnectWifiRunner, PrintIpRunner};

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn run_led_controller(runner: LedControllerRunner) {
    runner.run().await.expect("Infallible");
}

#[embassy_executor::task]
async fn run_connect_wifi(runner: ConnectWifiRunner) {
    runner.run().await
}

#[embassy_executor::task]
async fn run_network(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

const WEB_TASK_POOL_SIZE: usize = 8;
const NET_STACK_RESOURCES: usize = WEB_TASK_POOL_SIZE + 3;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn run_webserver(runner: WebserverRunner) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn run_print_ip(runner: PrintIpRunner) {
    runner.run().await
}

#[embassy_executor::task]
async fn run_ble(
    mut runner: BleRunner<
        'static,
        ExternalController<BleConnector<'static>, 20>,
        DefaultPacketPool,
    >,
) {
    runner.run().await.expect("BLE stack error")
}

#[embassy_executor::task]
async fn run_ble_connection(runner: BleConnectionRunner) {
    runner.run().await
}

fn initialize_esp_wifi_controller(
    timer_group: TIMG0<'static>,
    rng: Rng,
    radio_clock: RADIO_CLK<'static>,
) -> &'static EspWifiController<'static> {
    let timg0 = TimerGroup::new(timer_group);
    &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng, radio_clock).unwrap()
    )
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let Peripherals {
        TIMG0,
        GPIO1,
        GPIO2,
        GPIO3,
        RNG,
        RADIO_CLK,
        WIFI,
        SYSTIMER,
        BT,
        ..
    } = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(size: 100 * 1024);

    let system_timer = SystemTimer::new(SYSTIMER);
    esp_hal_embassy::init(system_timer.alarm0);

    let (led_controller_runner, watch) = led_controller::initialize(GPIO3, GPIO2, GPIO1);

    let rng = Rng::new(RNG);
    let esp_wifi_controller = initialize_esp_wifi_controller(TIMG0, rng, RADIO_CLK);

    let (connect_wifi_runner, network_runner, print_ip_runner, network_stack) =
        wifi::initialize_wifi(esp_wifi_controller, WIFI, rng);

    let Ble {
        ble_runner,
        connection_runner: ble_connection_runner,
    } = ble::initialize(
        esp_wifi_controller,
        BT,
        watch.sender(),
        watch.receiver().unwrap(),
    );

    let mut webserver_task_factory = webserver::initialize(network_stack, watch.sender());

    esp_println::println!("Initialization complete!");

    spawner.must_spawn(run_led_controller(led_controller_runner));
    spawner.must_spawn(run_connect_wifi(connect_wifi_runner));
    spawner.must_spawn(run_network(network_runner));
    spawner.must_spawn(run_print_ip(print_ip_runner));

    for _ in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(run_webserver(webserver_task_factory.new_runner()));
    }

    spawner.must_spawn(run_ble(ble_runner));
    spawner.must_spawn(run_ble_connection(ble_connection_runner));

    esp_println::println!("Tasks spawned!");
}
