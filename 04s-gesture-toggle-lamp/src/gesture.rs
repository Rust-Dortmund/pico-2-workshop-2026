//! Contains gesture detection logic and data structures.

use defmt::{Format, info};

use crate::Apds9960;

// Size of the APDS9960's gesture FIFO buffer.
const APDS_9960_GESTURE_FIFO_SIZE: usize = 32;

/// Represents a gesture dataset captured by the APDS9960.
#[derive(Format)]
pub(crate) struct GestureDataset {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

impl GestureDataset {
    /// Checks whether this gesture dataset contains just noise.
    pub(crate) fn is_noise(&self) -> bool {
        const THRESHOLD: u8 = 4;
        self.up <= THRESHOLD
            && self.down <= THRESHOLD
            && self.left <= THRESHOLD
            && self.right <= THRESHOLD
    }
}

/// Helper struct for detecting gesture input using the APDS9960.
pub(crate) struct GestureDetector {
    sensor: Apds9960,
    /// Buffer for storing raw gesture datasets obtained from the sensor.
    ///
    /// When reading the array from the beginning then four consecutive bytes resemble one raw
    /// gesture dataset.
    /// Each gesture dataset contains readings for up, down, left and right in this order with a
    /// size of one byte each.
    ///
    /// As the sensor's FIFO buffer can hold up to 32 readings this is exactly the amount of data
    /// we reserve space for.
    gesture_datasets_buffer: [u8; 4 * APDS_9960_GESTURE_FIFO_SIZE],
}

impl GestureDetector {
    pub(crate) fn new(sensor: Apds9960) -> Self {
        Self {
            sensor,
            gesture_datasets_buffer: [0; 4 * APDS_9960_GESTURE_FIFO_SIZE],
        }
    }
}

impl GestureDetector {
    /// Checks whether any gesture was detected since the last call to this function.
    pub(crate) async fn any_gesture_detected(&mut self) -> bool {
        // TODO: Read the available gesture datasets from `self.sensor` into `self.gesture_datasets_buffer`.
        // TODO: Check whether there is a gesture dataset that doesn't just hold noise. Tip: Have a look at the `GestureDataset` struct.
        todo!("Implement me!")
    }
}
