//! Code from this module is using [block_in_place](tokio::task::block_in_place),
//! and so they cannot be used in [current_thread](tokio::runtime::Builder::new_current_thread) runtimes

use std::iter::zip;
use std::sync::Arc;
use std::time::Duration;

use hidapi::{HidApi, HidResult};
use image::DynamicImage;
use tokio::sync::Mutex;
use tokio::task::block_in_place;
use tokio::time::sleep;

use crate::{Kind, list_devices, StreamDeck, StreamDeckError, StreamDeckInput};
use crate::images::{convert_image_async, ImageRect};

/// Actually refreshes the device list, can be safely ran inside [multi_thread](tokio::runtime::Builder::new_multi_thread) runtime
pub fn refresh_device_list_async(hidapi: &mut HidApi) -> HidResult<()> {
    block_in_place(move || {
        hidapi.refresh_devices()
    })
}

/// Returns a list of devices as (Kind, Serial Number) that could be found using HidApi,
/// can be safely ran inside [multi_thread](tokio::runtime::Builder::new_multi_thread) runtime
///
/// **WARNING:** To refresh the list, use [refresh_device_list]
pub fn list_devices_async(hidapi: &HidApi) -> Vec<(Kind, String)> {
    block_in_place(move || {
        list_devices(&hidapi)
    })
}

/// Stream Deck interface suitable to be used in async, uses [block_in_place](block_in_place)
/// so this wrapper cannot be used in [current_thread](tokio::runtime::Builder::new_current_thread) runtimes
#[derive(Clone)]
pub struct AsyncStreamDeck {
    kind: Kind,
    device: Arc<Mutex<StreamDeck>>
}

/// Static functions of the struct
impl AsyncStreamDeck {
    /// Attempts to connect to the device, can be safely ran inside [multi_thread](tokio::runtime::Builder::new_multi_thread) runtime
    pub fn connect(hidapi: &HidApi, kind: Kind, serial: &str) -> Result<AsyncStreamDeck, StreamDeckError> {
        let device = block_in_place(move || StreamDeck::connect(hidapi, kind, serial))?;

        Ok(AsyncStreamDeck {
            kind,
            device: Arc::new(Mutex::new(device))
        })
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
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.manufacturer()
        })?)
    }

    /// Returns product string of the device
    pub async fn product(&self) -> Result<String, StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.product()
        })?)
    }

    /// Returns serial number of the device
    pub async fn serial_number(&self) -> Result<String, StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.serial_number()
        })?)
    }

    /// Returns firmware version of the StreamDeck
    pub async fn firmware_version(&self) -> Result<String, StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.firmware_version()
        })?)
    }

    /// Reads button states, awaits until there's data.
    /// Poll rate determines how often button state gets checked
    pub async fn read_input(&self, poll_rate: f32) -> Result<StreamDeckInput, StreamDeckError> {
        loop {
            let device = self.device.clone();
            let lock = device.lock().await;
            let data = block_in_place(move || {
                lock.read_input(None)
            })?;

            if !data.is_empty() {
                return Ok(data);
            }

            sleep(Duration::from_secs_f32(1.0 / poll_rate)).await;
        }
    }

    /// Returns button state reader for this device
    pub fn get_reader(&self) -> Arc<DeviceStateReader> {
        Arc::new(DeviceStateReader {
            device: self.clone(),
            states: Mutex::new(DeviceState {
                buttons: vec![false; self.kind.key_count() as usize],
                encoders: vec![false; self.kind.encoder_count() as usize],
            })
        })
    }

    /// Resets the device
    pub async fn reset(&self) -> Result<(), StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.reset()
        })?)
    }

    /// Sets brightness of the device, value range is 0 - 100
    pub async fn set_brightness(&self, percent: u8) -> Result<(), StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.set_brightness(percent)
        })?)
    }

    /// Writes image data to Stream Deck device, needs to accept Arc for image data since it runs a blocking thread under the hood
    pub async fn write_image(&self, key: u8, image_data: &[u8]) -> Result<(), StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.write_image(key, image_data)
        })?)
    }

    /// Writes image data to Stream Deck device's lcd strip/screen, needs to accept Arc for image data since it runs a blocking thread under the hood
    pub async fn write_lcd(&self, x: u16, y: u16, rect: Arc<ImageRect>) -> Result<(), StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.write_lcd(x, y, rect.as_ref())
        })?)
    }

    /// Writes image data to Stream Deck device
    pub async fn clear_button_image(&self, key: u8) -> Result<(), StreamDeckError> {
        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.clear_button_image(key)
        })?)
    }

    /// Sets specified button's image
    pub async fn set_button_image(&self, key: u8, image: DynamicImage) -> Result<(), StreamDeckError> {
        let image = convert_image_async(self.kind, image)?;

        let device = self.device.clone();
        let lock = device.lock().await;
        Ok(block_in_place(move || {
            lock.write_image(key, &image)
        })?)
    }
}

/// Button reader that keeps state of the Stream Deck and returns events instead of full states
pub struct DeviceStateReader {
    device: AsyncStreamDeck,
    states: Mutex<DeviceState>
}

#[derive(Default)]
struct DeviceState {
    buttons: Vec<bool>,
    encoders: Vec<bool>
}

/// Tells what changed in button states
#[derive(Copy, Clone, Debug, Hash)]
pub enum DeviceStateUpdate {
    /// Button got pressed down
    ButtonDown(u8),

    /// Button got released
    ButtonUp(u8),

    /// Encoder got pressed down
    EncoderDown(u8),

    /// Encoder was released from being pressed down
    EncoderUp(u8),

    /// Encoder was twisted
    EncoderTwist(u8, i8),

    /// Touch screen received short press
    TouchScreenPress(u16, u16),

    /// Touch screen received long press
    TouchScreenLongPress(u16, u16),

    /// Touch screen received a swipe
    TouchScreenSwipe((u16, u16), (u16, u16)),
}

impl DeviceStateReader {
    /// Reads states and returns updates
    #[async_recursion::async_recursion]
    pub async fn read(&self, poll_rate: f32) -> Result<Vec<DeviceStateUpdate>, StreamDeckError> {
        let input = self.device.read_input(poll_rate).await?;
        let mut my_states = self.states.lock().await;

        let mut updates = vec![];

        match input {
            StreamDeckInput::ButtonStateChange(buttons) => {
                for (index, (their, mine)) in zip(buttons.iter(), my_states.buttons.iter()).enumerate() {
                    if *their != *mine {
                        if *their {
                            updates.push(DeviceStateUpdate::ButtonDown(index as u8));
                        } else {
                            updates.push(DeviceStateUpdate::ButtonUp(index as u8));
                        }
                    }
                }

                my_states.buttons = buttons;
            }

            StreamDeckInput::EncoderStateChange(encoders) => {
                for (index, (their, mine)) in zip(encoders.iter(), my_states.encoders.iter()).enumerate() {
                    if *their != *mine {
                        if *their {
                            updates.push(DeviceStateUpdate::EncoderDown(index as u8));
                        } else {
                            updates.push(DeviceStateUpdate::EncoderUp(index as u8));
                        }
                    }
                }

                my_states.encoders = encoders;
            }

            StreamDeckInput::EncoderTwist(twist) => {
                for (index, change) in twist.iter().enumerate() {
                    if *change != 0 {
                        updates.push(DeviceStateUpdate::EncoderTwist(index as u8, *change));
                    }
                }
            }

            StreamDeckInput::TouchScreenPress(x, y) => {
                updates.push(DeviceStateUpdate::TouchScreenPress(x, y));
            }

            StreamDeckInput::TouchScreenLongPress(x, y) => {
                updates.push(DeviceStateUpdate::TouchScreenLongPress(x, y));
            }

            StreamDeckInput::TouchScreenSwipe(s, e) => {
                updates.push(DeviceStateUpdate::TouchScreenSwipe(s, e));
            }

            _ => {}
        }


        drop(my_states);

        if updates.is_empty() {
            self.read(poll_rate).await
        } else {
            Ok(updates)
        }
    }
}
