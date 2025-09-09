use cyw43::bluetooth::BtDriver;
use defmt::info;
use embassy_futures::select::select;
use trouble_host::prelude::*;

use crate::{
    led::Color,
    led_controller::{ColorReceiver, ColorSender},
    mk_static,
};

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

#[gatt_server]
struct Server {
    led_service: LedService,
}

#[gatt_service(uuid = BluetoothUuid16::new(0x180a))]
struct LedService {
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 2])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "color", read, value = "LED Color")]
    #[characteristic(uuid = BluetoothUuid16::new(0x2a57), read, write, notify, value = 0)]
    color: u8,
}

pub(crate) struct BleConnectionRunner {
    peripheral: Peripheral<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
    sender: ColorSender<2>,
    receiver: ColorReceiver<2>,
    server: Server<'static>,
}

impl BleConnectionRunner {
    /// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
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

    async fn handle_gatt_read<P: PacketPool>(event: ReadEvent<'_, '_, P>, server: &Server<'_>) {
        if event.handle() == server.led_service.color.handle {
            let value = server.get(&server.led_service.color);
            info!("[gatt] Read Event to Color Characteristic: {:?}", value);
        }

        match event.accept() {
            Ok(reply) => reply.send().await,
            Err(e) => info!("[gatt] error sending response: {:?}", e),
        };
    }

    async fn accept_write_event<P: PacketPool>(event: WriteEvent<'_, '_, P>) {
        match event.accept() {
            Ok(reply) => reply.send().await,
            Err(e) => {
                info!("[gatt] error sending response: {:?}", e)
            }
        }
    }

    async fn handle_gatt_write<P: PacketPool>(
        event: WriteEvent<'_, '_, P>,
        server: &Server<'_>,
        sender: &ColorSender<2>,
    ) {
        if event.handle() == server.led_service.color.handle {
            info!(
                "[gatt] Write Event to Level Characteristic: {:?}",
                event.data()
            );
            match *event.data() {
                [COLOR_RED] => {
                    sender.send(Color::Red);
                    Self::accept_write_event(event).await;
                }
                [COLOR_GREEN] => {
                    sender.send(Color::Green);
                    Self::accept_write_event(event).await;
                }
                [COLOR_BLUE] => {
                    sender.send(Color::Blue);
                    Self::accept_write_event(event).await;
                }
                _ => match event.reject(AttErrorCode::OUT_OF_RANGE) {
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
    async fn gatt_events_task<P: PacketPool>(
        server: &Server<'_>,
        connection: &GattConnection<'_, '_, P>,
        sender: &ColorSender<2>,
    ) -> Result<(), trouble_host::prelude::Error> {
        let reason = loop {
            match connection.next().await {
                GattConnectionEvent::Disconnected { reason } => break reason,
                GattConnectionEvent::Gatt { event } => {
                    match event {
                        GattEvent::Read(event) => Self::handle_gatt_read(event, server).await,
                        GattEvent::Write(event) => {
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
    /// It will stop when the connection is closed by the central or an error occurs.
    async fn notify_task<P: PacketPool>(
        server: &Server<'_>,
        connection: &GattConnection<'_, '_, P>,
        receiver: &mut ColorReceiver<2>,
    ) {
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
            let new_color: u8 = receiver.changed().await.into();
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
                    // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                    let a = Self::gatt_events_task(&self.server, &connection, &self.sender);
                    let b = Self::notify_task(&self.server, &connection, &mut self.receiver);
                    // run until any task ends (usually because the connection has been closed),
                    // then return to advertising state.
                    select(a, b).await;
                }
                Err(e) => {
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    }
}

pub(crate) struct Ble {
    pub(crate) ble_runner:
        Runner<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
    pub(crate) connection_runner: BleConnectionRunner,
}

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
