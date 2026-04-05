//! This module handles BLE connections.

use cyw43::bluetooth::BtDriver;
use defmt::info;
use embassy_futures::select::select;
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
    led_service: LedService,
}

/// GATT service providing capabilities to read and set the LED color.
///
/// As there is no standard UUID for "tri-color RGB LEDs" defined, we use one without a standard meaning.
#[gatt_service(uuid = BluetoothUuid16::new(0x180a))]
struct LedService {
    /// Characteristic of the GATT service setting the actual color.
    ///
    /// Note that we define some descriptor that provide metadata about the characteristic.
    ///
    /// Again, there is no standard cahracteristic defined, so we use one without a standard meaning.
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 2])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "color", read, value = "LED Color")]
    #[characteristic(uuid = BluetoothUuid16::new(0x2a57), read, write, notify, value = 0)]
    color: u8,
}

/// Runner handling the BLE connection.
pub(crate) struct BleConnectionRunner {
    peripheral: Peripheral<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
    sender: ColorSender<2>,
    receiver: ColorReceiver<2>,
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
        let len = AdStructure::encode_slice(
            &[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[[0x0a, 0x18]]),
                AdStructure::CompleteLocalName(name.as_bytes()),
            ],
            &mut advertiser_data[..],
        )?;
        let advertiser = peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &advertiser_data[..len],
                    scan_data: &[],
                },
            )
            .await?;
        info!("[adv] advertising");
        let conn = advertiser.accept().await?.with_attribute_server(server)?;
        info!("[adv] connection established");
        Ok(conn)
    }

    /// Handles read access to a characteristic in a GATT service.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn handle_gatt_read<P: PacketPool>(event: ReadEvent<'_, '_, P>, server: &Server<'_>) {
        if event.handle() == server.led_service.color.handle {
            // This is for the `color` characteristic.

            // Note that this does nothing but printing the current value! See below for how the value is sent.
            let value = server.get(&server.led_service.color);
            info!("[gatt] Read Event to Color Characteristic: {:?}", value);
        }

        // Accepting and then sending the read event queries the currently cached value and returns it over BLE.
        match event.accept() {
            // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
            Ok(reply) => reply.send().await,
            Err(e) => info!("[gatt] error sending response: {:?}", e),
        };
    }

    /// Accepts a [`WriteEvent`] to a characteristic in a GATT service.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn accept_write_event<P: PacketPool>(event: WriteEvent<'_, '_, P>) {
        match event.accept() {
            // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
            Ok(reply) => reply.send().await,
            Err(e) => {
                info!("[gatt] error sending response: {:?}", e)
            }
        }
    }

    /// Handles write access to a characteristic in a GATT service.
    ///
    /// # Cancellation safety
    ///
    /// This function is cancel safe.
    async fn handle_gatt_write<P: PacketPool>(
        event: WriteEvent<'_, '_, P>,
        server: &Server<'_>,
        sender: &ColorSender<2>,
    ) {
        if event.handle() == server.led_service.color.handle {
            // This is for the `color` characteristic.

            info!(
                "[gatt] Write Event to Level Characteristic: {:?}",
                event.data()
            );

            // Decode the payload, notify the LED controller of the new color and respond to the
            // sender that the request was processed.
            // Accepting the event will implicitly update the cached value.
            // Note that sending the color here will implicitly trigger a second notify in [`notify_task`].
            // We accept this behavior so we don't have to fiddle around with two different
            // sender / receiver pairs or need a more complicated messaging infrastructure.
            match *event.data() {
                [COLOR_RED] => {
                    sender.send(Color::Red);
                    // CANCELLATION SAFETY: Documented as being cancel safe.
                    Self::accept_write_event(event).await;
                }
                [COLOR_GREEN] => {
                    sender.send(Color::Green);
                    // CANCELLATION SAFETY: Documented as being cancel safe.
                    Self::accept_write_event(event).await;
                }
                [COLOR_BLUE] => {
                    sender.send(Color::Blue);
                    // CANCELLATION SAFETY: Documented as being cancel safe.
                    Self::accept_write_event(event).await;
                }
                _ => match event.reject(AttErrorCode::OUT_OF_RANGE) {
                    // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
                    Ok(reply) => reply.send().await,
                    Err(e) => {
                        info!("[gatt] error sending response: {:?}", e)
                    }
                },
            }
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
        sender: &ColorSender<2>,
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
        receiver: &mut ColorReceiver<2>,
    ) {
        // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
        if let Some(new_color) = receiver.try_get()
            && server
                .led_service
                .color
                .notify(connection, &u8::from(new_color))
                .await
                .is_err()
        {
            info!("[notify_task] error notifying connection");
            return;
        }

        loop {
            // CANCELLATION SAFETY: `embassy_sync::watch::Receiver::changed` is not documented as being cancel safe, but
            // should be according to [this comment](https://github.com/embassy-rs/embassy/issues/5484#issuecomment-3921041927).
            // Also see [this issue](https://github.com/embassy-rs/embassy/issues/5796).
            let new_color: u8 = receiver.changed().await.into();
            // CANCELLATION SAFETY: Used this way in https://github.com/embassy-rs/trouble/blob/main/examples/apps/src/ble_bas_peripheral.rs
            if server
                .led_service
                .color
                .notify(connection, &new_color)
                .await
                .is_err()
            {
                info!("[notify_task] error notifying connection");
                break;
            }
        }
    }

    pub(crate) async fn run(mut self) {
        info!("Starting advertising and GATT service");
        loop {
            match Self::advertise("LED Trouble", &mut self.peripheral, &self.server).await {
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
    sender: ColorSender<2>,
    receiver: ColorReceiver<2>,
) -> Ble {
    let ble_controller: ExternalController<_, 10> = ExternalController::new(bluetooth_driver);

    let ble_host_resources = mk_static!(
        HostResources<DefaultPacketPool, 4, 0, 1>,
        HostResources::new()
    );

    let stack = mk_static!(
        Stack<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
        trouble_host::new(ble_controller, ble_host_resources)
    );
    let Host {
        peripheral, runner, ..
    } = stack.build();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "LED TrouBLE",
        appearance: &appearance::light_source::LED_LAMP,
    }))
    .unwrap();

    let connection_runner = BleConnectionRunner {
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
