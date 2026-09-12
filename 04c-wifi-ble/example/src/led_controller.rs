use embassy_futures::select::{Either, select};
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals::{PIN_18, PIN_19, PIN_20},
};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    watch::{Receiver, Sender, Watch},
};
use embassy_time::{Duration, Ticker};

use crate::{
    led::{Color, TriColorLed},
    mk_static,
};

// Type definitions for channel types that we can use to send `Color` values.
// We need to pick appropriate values for the 2 generic parameters of `Watch`:
//   1. The synchronization primitive. Since we know we're only running only single-threaded code
//      and all our tasks run on a regular embassy executor, which means that scheduling is
//      cooperative and our tasks cannot be interrupted, we use a "no-op" mutex that doesn't
//      actually do anything to protect against data races. After all, if there's only a single
//      thread, there's no one else racing!
//   2. The maximum number of `Receiver`s we want to use at the same time. Since we now have more
//      than just one receiver, we defer setting this by passing the number through via const generics.
pub(crate) type ColorWatch<const NUM_RECEIVERS: usize> = Watch<NoopRawMutex, Color, NUM_RECEIVERS>;
pub(crate) type ColorSender<const NUM_RECEIVERS: usize> =
    Sender<'static, NoopRawMutex, Color, NUM_RECEIVERS>;
pub(crate) type ColorReceiver<const NUM_RECEIVERS: usize> =
    Receiver<'static, NoopRawMutex, Color, NUM_RECEIVERS>;

pub(crate) struct Runner<const NUM_RECEIVERS: usize> {
    led: TriColorLed,
    receiver: ColorReceiver<NUM_RECEIVERS>,
}

impl<const NUM_RECEIVERS: usize> Runner<NUM_RECEIVERS>
{
    pub(crate) async fn run(mut self) {
        let mut ticker = Ticker::every(Duration::from_millis(500));
        loop {
            // CANCELLATION SAFETY:
            // - `embassy_sync::watch::Receiver::changed` is not documented as being cancel safe, but
            //   should be according to [this comment](https://github.com/embassy-rs/embassy/issues/5484#issuecomment-3921041927).
            //   Also see [this issue](https://github.com/embassy-rs/embassy/issues/5796).
            // - `embassy_time::Ticker::next` is cancel safe.
            match select(self.receiver.changed(), ticker.next()).await {
                Either::First(new_color) => self.led.set_color(new_color),
                Either::Second(()) => self.led.toggle(),
            }
        }
    }
}

pub(crate) type LedControllerRunner = Runner<2>;

/// Initializes the LED controller that drives the LED connected to the given pins.
///
/// Returns two things:
///
/// - A runner that needs to be polled (e.g. given to a task) in order for the LED controller to run.
/// - A [`Watch`] for passing the color to display to the LED controller.
pub(crate) fn initialize(
    red_gpio: Peri<'static, PIN_18>,
    green_gpio: Peri<'static, PIN_19>,
    blue_gpio: Peri<'static, PIN_20>,
) -> (LedControllerRunner, &'static ColorWatch<2>) {
    let red_led = Output::new(red_gpio, Level::Low);
    let green_led = Output::new(green_gpio, Level::Low);
    let blue_led = Output::new(blue_gpio, Level::Low);
    let tri_color_led = TriColorLed::new(red_led, green_led, blue_led);

    let watch = mk_static!(ColorWatch<2>, ColorWatch::new());
    watch.sender().send(Color::Red);

    let led_controller_runner = Runner {
        led: tri_color_led,
        receiver: watch.receiver().unwrap(),
    };

    (led_controller_runner, watch)
}
