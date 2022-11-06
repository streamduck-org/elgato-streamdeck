use std::iter::zip;
use std::sync::Arc;
use std::time::Duration;

use hidapi::HidApi;
use image::DynamicImage;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::{Kind, StreamDeck, StreamDeckError};
use crate::images::convert_image_async;

/// Stream Deck interface suitable to be used in async
pub struct AsyncStreamDeck {
    kind: Kind,
    device: Mutex<StreamDeck>
}

/// Static functions of the struct
impl AsyncStreamDeck {
    /// Attempts to connect to the device
    pub fn connect(hidapi: &HidApi, kind: Kind, serial: &str) -> Result<Arc<AsyncStreamDeck>, StreamDeckError> {
        let device = StreamDeck::connect(hidapi, kind, serial)?;

        Ok(Arc::new(AsyncStreamDeck {
            kind,
            device: Mutex::new(device)
        }))
    }
}

/// Instance methods of the struct
impl AsyncStreamDeck {
    /// Returns kind of the Stream Deck
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns manufacturer string of the device
    pub async fn manufacturer(&self) -> Result<String, StreamDeckError> {
        Ok(self.device.lock().await.manufacturer()?)
    }

    /// Returns product string of the device
    pub async fn product(&self) -> Result<String, StreamDeckError> {
        Ok(self.device.lock().await.product()?)
    }

    /// Returns serial number of the device
    pub async fn serial_number(&self) -> Result<String, StreamDeckError> {
        Ok(self.device.lock().await.serial_number()?)
    }

    /// Returns firmware version of the StreamDeck
    pub async fn firmware_version(&self) -> Result<String, StreamDeckError> {
        Ok(self.device.lock().await.firmware_version()?)
    }

    /// Reads button states, awaits until there's data.
    /// Poll rate determines how often button state gets checked
    pub async fn read_button_states(&self, poll_rate: f32) -> Result<Vec<bool>, StreamDeckError> {
        loop {
            let data = self.device.lock().await.read_button_states(None)?;

            if !data.is_empty() {
                return Ok(data);
            }

            sleep(Duration::from_secs_f32(1.0 / poll_rate)).await;
        }
    }

    /// Returns button state reader for this device
    pub fn get_reader(self: &Arc<Self>) -> Arc<ButtonStateReader> {
        Arc::new(ButtonStateReader {
            device: self.clone(),
            states: Mutex::new(vec![false; self.kind.key_count() as usize])
        })
    }

    /// Resets the device
    pub async fn reset(&self) -> Result<(), StreamDeckError> {
        Ok(self.device.lock().await.reset()?)
    }

    /// Sets brightness of the device, value range is 0 - 100
    pub async fn set_brightness(&self, percent: u8) -> Result<(), StreamDeckError> {
        Ok(self.device.lock().await.set_brightness(percent)?)
    }

    /// Writes image data to Stream Deck device
    pub async fn write_image(&self, key: u8, image_data: &[u8]) -> Result<(), StreamDeckError> {
        Ok(self.device.lock().await.write_image(key, image_data)?)
    }

    /// Writes image data to Stream Deck device
    pub async fn clear_button_image(&self, key: u8) -> Result<(), StreamDeckError> {
        Ok(self.device.lock().await.write_image(key, &self.kind.blank_image())?)
    }

    /// Sets specified button's image
    pub async fn set_button_image(&self, key: u8, image: DynamicImage) -> Result<(), StreamDeckError> {
        let image = convert_image_async(self.kind, image).await?;

        Ok(self.device.lock().await.write_image(key, &image)?)
    }
}

/// Button reader that keeps state of the Stream Deck and returns events instead of full states
pub struct ButtonStateReader {
    device: Arc<AsyncStreamDeck>,
    states: Mutex<Vec<bool>>
}

/// Tells what changed in button states
#[derive(Copy, Clone, Debug, Hash)]
pub enum ButtonStateUpdate {
    /// Button got pressed down
    ButtonDown(u8),
    /// Button got released
    ButtonUp(u8)
}

impl ButtonStateReader {
    /// Reads states and returns updates
    #[async_recursion::async_recursion]
    pub async fn read(&self, poll_rate: f32) -> Result<Vec<ButtonStateUpdate>, StreamDeckError> {
        let states = self.device.read_button_states(poll_rate).await?;
        let mut my_states = self.states.lock().await;

        let mut updates = vec![];

        for (index, (their, mine)) in zip(states.iter(), my_states.iter()).enumerate() {
            if *their != *mine {
               if *their {
                   updates.push(ButtonStateUpdate::ButtonDown(index as u8));
               } else {
                   updates.push(ButtonStateUpdate::ButtonUp(index as u8));
               }
            }
        }

        *my_states = states;

        drop(my_states);

        if updates.is_empty() {
            self.read(poll_rate).await
        } else {
            Ok(updates)
        }
    }
}