//! Logic for the state and state transitions for the LED.

use embassy_rp::gpio::Level;

/// Lists possible states of the LED.
pub(crate) enum LedState {
    Off,
    Red,
    Green,
    Blue,
    White,
}

impl LedState {
    /// Gets the next state of the LED based on the current one.
    pub(crate) fn proceed(self) -> Self {
        match self {
            Self::Off => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::White,
            Self::White => Self::Off,
        }
    }

    /// Gets the levels for the red, green and blue color channels.
    pub(crate) fn get_rgb_levels(&self) -> (Level, Level, Level) {
        match self {
            Self::Off => (Level::Low, Level::Low, Level::Low),
            Self::Red => (Level::High, Level::Low, Level::Low),
            Self::Green => (Level::Low, Level::High, Level::Low),
            Self::Blue => (Level::Low, Level::Low, Level::High),
            Self::White => (Level::High, Level::High, Level::High),
        }
    }
}
