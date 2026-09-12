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
        let available_gestures = usize::from(self.sensor.read_gesture_data_level().await.unwrap());
        if available_gestures == 0 {
            return false;
        }
        self.sensor
            .read_gesture_data(&mut self.gesture_datasets_buffer[..available_gestures * 4])
            .await
            .unwrap();

        self.gesture_datasets_buffer[..available_gestures * 4]
            .chunks_exact(4)
            .map(|raw_gesture_dataset| GestureDataset {
                up: raw_gesture_dataset[0],
                down: raw_gesture_dataset[1],
                left: raw_gesture_dataset[2],
                right: raw_gesture_dataset[3],
            })
            .inspect(|gesture_dataset| info!("Gesture dataset: {:?}", gesture_dataset))
            .any(|gesture_dataset| !gesture_dataset.is_noise())
    }
}
