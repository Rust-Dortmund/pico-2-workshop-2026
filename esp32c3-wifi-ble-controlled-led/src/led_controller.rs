use core::convert::Infallible;

use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    watch::{Receiver, Sender, Watch},
};
use embassy_time::{Duration, Ticker};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::{GPIO1, GPIO2, GPIO3},
};

use crate::{
    led::{ActiveHighOutputPinLed, Color, TriColorLed, TrippleLedTriColorLed},
    mk_static,
};

pub(crate) type ColorWatch<const NUM_RECEIVERS: usize> = Watch<NoopRawMutex, Color, NUM_RECEIVERS>;
pub(crate) type ColorSender<const NUM_RECEIVERS: usize> =
    Sender<'static, NoopRawMutex, Color, NUM_RECEIVERS>;
pub(crate) type ColorReceiver<const NUM_RECEIVERS: usize> =
    Receiver<'static, NoopRawMutex, Color, NUM_RECEIVERS>;

pub(crate) struct Runner<Led, const NUM_RECEIVERS: usize> {
    led: Led,
    receiver: ColorReceiver<NUM_RECEIVERS>,
}

impl<Led, const NUM_RECEIVERS: usize> Runner<Led, NUM_RECEIVERS>
where
    Led: TriColorLed,
{
    pub(crate) async fn run(mut self) -> Result<(), Led::Error> {
        let mut ticker = Ticker::every(Duration::from_millis(500));
        loop {
            match select(self.receiver.changed(), ticker.next()).await {
                Either::First(new_color) => self.led.set_color(new_color).await?,
                Either::Second(()) => self.led.toggle().await?,
            }
        }
    }
}

pub(crate) type LedControllerRunner = Runner<
    TrippleLedTriColorLed<
        ActiveHighOutputPinLed<Output<'static>>,
        ActiveHighOutputPinLed<Output<'static>>,
        ActiveHighOutputPinLed<Output<'static>>,
        Infallible,
    >,
    2,
>;

pub(crate) fn initialize(
    red_gpio: GPIO3<'static>,
    green_gpio: GPIO2<'static>,
    blue_gpio: GPIO1<'static>,
) -> (LedControllerRunner, &'static ColorWatch<2>) {
    let red_pin = Output::new(red_gpio, Level::Low, OutputConfig::default());
    let green_pin = Output::new(green_gpio, Level::Low, OutputConfig::default());
    let blue_pin = Output::new(blue_gpio, Level::Low, OutputConfig::default());

    let red_led = ActiveHighOutputPinLed::new(red_pin).expect("Infallible");
    let green_led = ActiveHighOutputPinLed::new(green_pin).expect("Infallible");
    let blue_led = ActiveHighOutputPinLed::new(blue_pin).expect("Infallible");
    let tri_color_led = TrippleLedTriColorLed::new(red_led, green_led, blue_led);

    let watch = mk_static!(ColorWatch<2>, ColorWatch::new());
    watch.sender().send(Color::Red);

    let led_controller_runner = Runner {
        led: tri_color_led,
        receiver: watch.receiver().unwrap(),
    };

    (led_controller_runner, watch)
}
