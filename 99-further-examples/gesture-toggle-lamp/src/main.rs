#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Level, Output},
    i2c::InterruptHandler, peripherals::I2C1,
};
use embassy_time::{Duration, Instant};
use {defmt_rtt as _, panic_probe as _};

type Apds9960 = apds9960::Apds9960<embassy_rp::i2c::I2c<'static, I2C1, embassy_rp::i2c::Async>, apds9960::Async>;

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_cargo_bin_name!(),
    embassy_rp::binary_info::rp_program_description!(
        c"Hello world, sends a heartbeat message each second"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[derive(Format)]
struct Gesture {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

impl Gesture {
    fn is_any(&self) -> bool {
        const THRESHOLD: u8 = 4;
        self.up > THRESHOLD
            || self.down > THRESHOLD
            || self.left > THRESHOLD
            || self.right > THRESHOLD
    }
}

enum LampState {
    Off,
    Red,
    Green,
    Blue,
    White,
}

impl LampState {
    fn proceed(self) -> Self {
        match self {
            Self::Off => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::White,
            Self::White => Self::Off,
        }
    }

    fn get_levels(&self) -> (Level, Level, Level) {
        match self {
            Self::Off => (Level::Low, Level::Low, Level::Low),
            Self::Red => (Level::High, Level::Low, Level::Low),
            Self::Green => (Level::Low, Level::High, Level::Low),
            Self::Blue => (Level::Low, Level::Low, Level::High),
            Self::White => (Level::High, Level::High, Level::High),
        }
    }
}

struct GestureDetector {
    sensor: Apds9960,
    gesture_data: [u8; 4 * (u8::MAX as usize + 1)],
}

impl GestureDetector {
    fn new(sensor: Apds9960) -> Self {
        Self {
            sensor,
            gesture_data: [0; 4 * (u8::MAX as usize + 1)],
        }
    }
}

impl GestureDetector {
    async fn any_gesture_detected(&mut self) -> bool {
        let available_gestures = usize::from(self.sensor.read_gesture_data_level().await.unwrap());
        if available_gestures == 0 {
            return false;
        }
        self.sensor
            .read_gesture_data(&mut self.gesture_data[..available_gestures * 4])
            .await
            .unwrap();

        self.gesture_data[..available_gestures * 4]
            .chunks_exact(4)
            .map(|gesture| Gesture {
                up: gesture[0],
                down: gesture[1],
                left: gesture[2],
                right: gesture[3],
            })
            .inspect(|gesture| info!("Gesture: {:?}", gesture))
            .any(|gesture| gesture.is_any())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let Peripherals {
        PIN_14,
        PIN_15,
        PIN_18,
        PIN_19,
        PIN_20,
        I2C1,
        ..
    } = embassy_rp::init(Default::default());

    let mut output_red = Output::new(PIN_18, Level::Low);
    let mut output_green = Output::new(PIN_19, Level::Low);
    let mut output_blue = Output::new(PIN_20, Level::Low);

    let sda = PIN_14;
    let scl = PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(I2C1, scl, sda, Irqs, config);
    let mut sensor = Apds9960::new(bus);

    let device_id = sensor.read_device_id().await.unwrap();
    info!("APDS9960 Device Id: {}", device_id);

    sensor.enable().await.unwrap();
    sensor.enable_gesture().await.unwrap();
    sensor.enable_gesture_mode().await.unwrap();

    let mut detector = GestureDetector::new(sensor);

    let mut state = LampState::Off;

    loop {
        if detector.any_gesture_detected().await {
            state = state.proceed();
            let (red_level, green_level, blue_level) = state.get_levels();
            output_red.set_level(red_level);
            output_green.set_level(green_level);
            output_blue.set_level(blue_level);

            let mut started = Instant::now();
            while started.elapsed() < Duration::from_secs(1) {
                if detector.any_gesture_detected().await {
                    started = Instant::now();
                }
            }
        }
    }
}
