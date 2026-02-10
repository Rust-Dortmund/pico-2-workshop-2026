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
//   1. The synchronization primitive. Since we know we're only running only single-threaded code,
//      we use a "no-op" mutex that doesn't actually do anything to protect against data races.
//      After all, if there's only a single thread, there's no one else racing!
//   2. The maximum number of `Receiver`s we want to use at the same time. Since we only want the 
//      `LedControllerRunner` to receive new color values, we only need 1.
pub(crate) type ColorWatch = Watch<NoopRawMutex, Color, 1>;
pub(crate) type ColorSender = Sender<'static, NoopRawMutex, Color, 1>;
pub(crate) type ColorReceiver = Receiver<'static, NoopRawMutex, Color, 1>;

/// Holds the required state for blinking the LED and changing its color on request.
pub(crate) struct LedControllerRunner {
    led: TriColorLed,
    receiver: ColorReceiver,
}

impl LedControllerRunner {
    pub(crate) async fn run(mut self) {
        // This runner has 2 tasks: 
        //   1. Every 0.5s, make the LED blink on or off.
        //   2. When a new LED color is requested through the web API, make the color change.
        let mut ticker = Ticker::every(Duration::from_millis(500));
        loop {
            match select(self.receiver.changed(), ticker.next()).await {
                Either::First(new_color) => self.led.set_color(new_color),
                Either::Second(()) => self.led.toggle(),
            }
        }
    }
}

/// Initializes the LED controller that drives the LED connected to the given pins.
///
/// Returns two things:
/// 
/// - A runner that needs to be polled (e.g. given to a task) in order for the LED controller to run.
/// - A [`Watch`] for passing the color to display to the LED controller.
pub(crate) fn initialize(
    red_gpio: Peri<'static, PIN_19>,
    green_gpio: Peri<'static, PIN_20>,
    blue_gpio: Peri<'static, PIN_18>,
) -> (LedControllerRunner, &'static ColorWatch) {
    // Initialize a `TriColorLed` to work with.
    let red_led = Output::new(red_gpio, Level::Low);
    let green_led = Output::new(green_gpio, Level::Low);
    let blue_led = Output::new(blue_gpio, Level::Low);
    let tri_color_led = TriColorLed::new(red_led, green_led, blue_led);

    // Create a channel for receiving new color values and store the receiving end inside `Self`.
    let watch = mk_static!(ColorWatch, ColorWatch::new_with(Color::Red));
    let led_controller_runner = LedControllerRunner {
        led: tri_color_led,
        receiver: watch.receiver().expect("we just created the watch channel, so the single receiver is still available"),
    };

    (led_controller_runner, watch)
}
