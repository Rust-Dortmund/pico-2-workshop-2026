use embassy_net::StackResources;
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_println::println;
use esp_wifi::{
    EspWifiController,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
};

use crate::mk_static;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

pub(crate) struct ConnectWifiRunner {
    wifi_controller: WifiController<'static>,
}

impl ConnectWifiRunner {
    pub(crate) async fn run(mut self) {
        println!("start connection task");
        println!(
            "Device capabilities: {:?}",
            self.wifi_controller.capabilities()
        );
        loop {
            if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
                // wait until we're no longer connected
                self.wifi_controller
                    .wait_for_event(WifiEvent::StaDisconnected)
                    .await;
                Timer::after(Duration::from_millis(5000)).await
            }

            if !matches!(self.wifi_controller.is_started(), Ok(true)) {
                let client_config = Configuration::Client(ClientConfiguration {
                    ssid: SSID.into(),
                    password: PASSWORD.into(),
                    ..Default::default()
                });
                self.wifi_controller
                    .set_configuration(&client_config)
                    .unwrap();
                println!("Starting wifi");
                self.wifi_controller.start_async().await.unwrap();
                println!("Wifi started!");

                println!("Scan");
                let access_points = self.wifi_controller.scan_n_async(10).await.unwrap();
                for access_point in access_points {
                    println!("{:?}", access_point);
                }
            }
            println!("About to connect...");

            match self.wifi_controller.connect_async().await {
                Ok(_) => println!("Wifi connected!"),
                Err(e) => {
                    println!("Failed to connect to wifi: {e:?}");
                    Timer::after(Duration::from_millis(5000)).await
                }
            }
        }
    }
}

pub(crate) struct PrintIpRunner {
    network_stack: embassy_net::Stack<'static>,
}

impl PrintIpRunner {
    pub(crate) async fn run(self) {
        while !self.network_stack.is_link_up() {
            Timer::after(Duration::from_millis(500)).await;
        }

        println!("Waiting to get IP address...");
        loop {
            if let Some(config) = self.network_stack.config_v4() {
                println!("Got IP: {}", config.address);
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}

// TODO: Just return one runner?
pub(crate) fn initialize_wifi(
    esp_wifi_controller: &'static EspWifiController<'static>,
    wifi_device: WIFI<'static>,
    mut rng: Rng,
) -> (
    ConnectWifiRunner,
    embassy_net::Runner<'static, WifiDevice<'static>>,
    PrintIpRunner,
    embassy_net::Stack<'static>,
) {
    let (wifi_controller, wifi_interfaces) =
        esp_wifi::wifi::new(esp_wifi_controller, wifi_device).unwrap();
    let wifi_interface = wifi_interfaces.sta;

    let network_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let (network_stack, network_runner) = embassy_net::new(
        wifi_interface,
        network_config,
        // TODO: Move to main?
        mk_static!(
            StackResources<{ super::NET_STACK_RESOURCES }>,
            StackResources::new()
        ),
        seed,
    );

    (
        ConnectWifiRunner { wifi_controller },
        network_runner,
        PrintIpRunner { network_stack },
        network_stack,
    )
}
