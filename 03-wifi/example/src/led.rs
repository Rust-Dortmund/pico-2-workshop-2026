//! This module contains abstractions for working with LEDs.

use core::{fmt::Display, str::FromStr};

use defmt::Format;
use embassy_rp::gpio::{ Level, Output };

/// A wrapper around the three individual LED colors to control them as one.
pub(crate) struct TriColorLed {
    red_led: Output<'static>,
    green_led: Output<'static>,
    blue_led: Output<'static>,
    color: Color,
    on: bool,
}

impl TriColorLed {
    pub(crate) fn new(red_led: Output<'static>, green_led: Output<'static>, blue_led: Output<'static>) -> Self {
        Self {
            red_led,
            green_led,
            blue_led,
            color: Color::Red,
            on: false,
        }
    }
}

impl TriColorLed {
    /// Change the color of the LED.
    pub(crate) fn set_color(&mut self, color: Color) {
        // Make sure the previous color is turned off before switching on the new color.
        let was_on = self.on;
        if self.on {
            self.toggle();
        }
        self.color = color;
        if was_on {
            self.toggle();
        }
    }

    /// Switch the LED from on to off or vice-versa.
    pub(crate) fn toggle(&mut self) {
        let new_level = if self.on { Level::Low } else { Level::High };
        self.on = !self.on;
        match self.color {
            Color::Red => self.red_led.set_level(new_level),
            Color::Green => self.green_led.set_level(new_level),
            Color::Blue => self.blue_led.set_level(new_level),
        }
    }
}

#[derive(Debug, Format, Clone, Copy)]
pub(crate) enum Color {
    Red,
    Green,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::Red => write!(f, "red"),
            Color::Green => write!(f, "green"),
            Color::Blue => write!(f, "blue"),
        }
    }
}

pub(crate) struct NoSuchColor;
impl FromStr for Color {
    type Err = NoSuchColor;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            _ => Err(NoSuchColor),
        }
    }
}