use core::{fmt::Display, marker::PhantomData, str::FromStr};

use embedded_hal::digital::OutputPin;

pub(crate) enum LedState {
    On,
    Off,
}

pub(crate) trait Led {
    type Error;

    async fn turn_on(&mut self) -> Result<(), Self::Error>;
    async fn turn_off(&mut self) -> Result<(), Self::Error>;

    async fn set_state(&mut self, state: LedState) -> Result<(), Self::Error> {
        match state {
            LedState::On => self.turn_on().await,
            LedState::Off => self.turn_off().await,
        }
    }
}

pub(crate) struct ActiveHighOutputPinLed<Pin> {
    pin: Pin,
}

impl<Pin> ActiveHighOutputPinLed<Pin>
where
    Pin: OutputPin,
{
    pub(crate) fn new(mut pin: Pin) -> Result<Self, Pin::Error> {
        pin.set_low()?;
        Ok(Self { pin })
    }
}

impl<Pin> Led for ActiveHighOutputPinLed<Pin>
where
    Pin: OutputPin,
{
    type Error = Pin::Error;

    async fn turn_on(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high()
    }

    async fn turn_off(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low()
    }
}

#[derive(Debug, Clone, Copy)]
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

pub(crate) trait TriColorLed {
    type Error;

    async fn set_color(&mut self, color: Color) -> Result<(), Self::Error>;
    async fn toggle(&mut self) -> Result<(), Self::Error>;
}

pub(crate) struct TrippleLedTriColorLed<R, G, B, Error> {
    red_led: R,
    green_led: G,
    blue_led: B,
    color: Color,
    on: bool,
    error: PhantomData<Error>,
}

impl<R, G, B, Error> TrippleLedTriColorLed<R, G, B, Error> {
    pub(crate) fn new(red_led: R, green_led: G, blue_led: B) -> Self {
        Self {
            red_led,
            green_led,
            blue_led,
            color: Color::Red,
            on: false,
            error: PhantomData,
        }
    }
}

impl<R, G, B, Error> TriColorLed for TrippleLedTriColorLed<R, G, B, Error>
where
    R: Led,
    G: Led,
    B: Led,
    Error: From<R::Error> + From<G::Error> + From<B::Error>,
{
    type Error = Error;

    async fn set_color(&mut self, color: Color) -> Result<(), Self::Error> {
        let on = self.on;
        if self.on {
            self.toggle().await?;
        }

        self.color = color;

        if on {
            self.toggle().await?;
        }

        Ok(())
    }

    async fn toggle(&mut self) -> Result<(), Self::Error> {
        let state = if self.on { LedState::Off } else { LedState::On };
        self.on = !self.on;
        match self.color {
            Color::Red => self.red_led.set_state(state).await?,
            Color::Green => self.green_led.set_state(state).await?,
            Color::Blue => self.blue_led.set_state(state).await?,
        }
        Ok(())
    }
}
