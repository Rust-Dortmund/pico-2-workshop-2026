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
        todo!("set the LED color to `color`")
    }

    /// Switch the LED from on to off or vice-versa.
    pub(crate) fn toggle(&mut self) {
        todo!("toggle the correct output on or off")
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