//! This module handles BLE connections.

use cyw43::bluetooth::BtDriver;
use defmt::info;
use embassy_futures::select::select;
use embassy_time::Timer;
use trouble_host::prelude::*;

use crate::{
    led::Color,
    led_controller::{ColorReceiver, ColorSender},
    mk_static,
};

// Color codes for the simplistic BLE protocol.
const COLOR_RED: u8 = 0;
const COLOR_GREEN: u8 = 1;
const COLOR_BLUE: u8 = 2;

impl From<Color> for u8 {
    fn from(color: Color) -> Self {
        match color {
            Color::Red => COLOR_RED,
            Color::Green => COLOR_GREEN,
            Color::Blue => COLOR_BLUE,
        }
    }
}

/// GATT server offering a single service: Interacting with the LED color.
#[gatt_server]
struct Server {
    // TODO: Provide an LED service.
}

/// GATT service providing capabilities to read and set the LED color.
///
/// As there is no standard UUID for "tri-color RGB LEDs" defined, we use one without a standard meaning.
struct LedService {
    // TODO: Complete me!
    color: u8,
}

/// Runner handling the BLE connection.
pub(crate) struct BleConnectionRunner {
    device_name: &'static str,
    peripheral: Peripheral<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
    sender: ColorSender,
    receiver: ColorReceiver,
    server: Server<'static>,
}

impl BleConnectionRunner {
    /// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
    ///
    /// BLE devices use advertisements to signal their presence to other devices.
    async fn advertise<'values, 'server, C: Controller>(
        name: &'values str,
        peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
        server: &'server Server<'values>,
    ) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
        let mut advertiser_data = [0; 31];
        let len = todo!("create and encode advertising data");
        let advertisement = todo!("construct advertisement with correct type");
        let advertiser = todo!("send advertisement");
        info!("[adv] advertising");
        let conn = todo!("accept connetion and serve our GATT server");
        info!("[adv] connection established");
        Ok(conn)
    }

    /// Handles read access to a characteristic in a GATT service.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn handle_gatt_read<P: PacketPool>(event: ReadEvent<'_, '_, P>, server: &Server<'_>) {
        if todo!("event is for LED color") {
            // This is for the `color` characteristic.

            // Note that this does nothing but printing the current value! See below for how the value is sent.
            let value = todo!("read value");
            // info!("[gatt] Read Event to Color Characteristic: {:?}", value);
        }

        // Accepting and then sending the read event queries the currently cached value and returns it over BLE.
        todo!("accept event and send response");
    }

    /// Handles write access to a characteristic in a GATT service.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn handle_gatt_write<P: PacketPool>(
        event: WriteEvent<'_, '_, P>,
        server: &Server<'_>,
        sender: &ColorSender,
    ) {
        todo!("check event target");
        let requested_color: Option<Color> = todo!("inspect and validate event");
        match requested_color {
            Some(color) => {
                todo!("change LED color");
                todo!("send response");
            }
            None => todo!("reject"),
        }
    }

    /// Stream Events until the connection closes.
    ///
    /// This function will handle the GATT events and process them.
    /// This is how we interact with read and write requests.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn gatt_events_task<P: PacketPool>(
        server: &Server<'_>,
        connection: &GattConnection<'_, '_, P>,
        sender: &ColorSender,
    ) -> Result<(), trouble_host::prelude::Error> {
        let reason = loop {
            // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
            match connection.next().await {
                GattConnectionEvent::Disconnected { reason } => break reason,
                GattConnectionEvent::Gatt { event } => {
                    match event {
                        // CANCELLATION SAFETY: Documented as being cancel safe.
                        GattEvent::Read(event) => Self::handle_gatt_read(event, server).await,
                        GattEvent::Write(event) => {
                            // CANCELLATION SAFETY: Documented as being cancel safe.
                            Self::handle_gatt_write(event, server, sender).await
                        }
                        _ => {}
                    };
                }
                _ => {} // ignore other Gatt Connection Events
            }
        };
        info!("[gatt] disconnected: {:?}", reason);
        Ok(())
    }

    /// This task will notify the connected central of changes to characteristics.
    ///
    /// In our case there is only the color charcateristic.
    /// If the color changes then we update the cached value in the BLE server which will trigger a
    /// notify message to the connected central.
    ///
    /// This function stops when the connection is closed by the central or an error occurs.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn notify_task<P: PacketPool>(
        server: &Server<'_>,
        connection: &GattConnection<'_, '_, P>,
        receiver: &mut ColorReceiver,
    ) {
        // TODO: Get current color and notify client about it
        if false { // TODO: check for failure
            info!("[notify_task] error notifying connection");
            return;
        }

        // TODO: When the LED color changes, notify the client
        loop {
            Timer::after_millis(500).await;
        }
    }

    pub(crate) async fn run(mut self) {
        info!("Starting advertising and GATT service");
        loop {
            match Self::advertise(self.device_name, &mut self.peripheral, &self.server).await {
                Ok(connection) => {
                    // Set up tasks when the connection is established to a central, so they don't run when no one is connected.
                    let gatt_events_task =
                        Self::gatt_events_task(&self.server, &connection, &self.sender);
                    let notify_task =
                        Self::notify_task(&self.server, &connection, &mut self.receiver);
                    // Run until any task ends (usually because the connection has been closed),
                    // then return to advertising state.
                    // CANCELLATION SAFETY:
                    // - `Self::gatt_events_task` is documented as being cancel safe.
                    // - `Self::notify_task` is documented as being cancel safe.
                    select(gatt_events_task, notify_task).await;
                }
                Err(e) => {
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    }
}

/// Wrapper struct for all runners needed for BLE.
pub(crate) struct Ble {
    pub(crate) ble_runner:
        Runner<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
    pub(crate) connection_runner: BleConnectionRunner,
}

/// Initializes BLE connectivity and returns the runners that need to be polled (e.g. in tasks).
pub(crate) fn initialize(
    bluetooth_driver: BtDriver<'static>,
    sender: ColorSender,
    receiver: ColorReceiver,
    device_name: &'static str,
) -> Ble {
    let ble_controller: ExternalController<_, 10> = ExternalController::new(bluetooth_driver);

    // See https://embassy.dev/trouble/#_hostresources if you're curious about the numbers.
    let ble_host_resources = mk_static!(
        HostResources<DefaultPacketPool, 4, 0, 1>,
        HostResources::new()
    );

    let stack = mk_static!(
        Stack<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
        trouble_host::new(ble_controller, ble_host_resources).build()
    );
    let runner = stack.runner();
    let peripheral = stack.peripheral();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: device_name,
        appearance: &appearance::light_source::LED_LAMP,
    }))
    .unwrap();

    let connection_runner = BleConnectionRunner {
        device_name,
        peripheral,
        sender,
        receiver,
        server,
    };

    Ble {
        ble_runner: runner,
        connection_runner,
    }
}
