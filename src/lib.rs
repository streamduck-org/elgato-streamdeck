//! Elgato Streamdeck library
//!
//! Library for interacting with Elgato Stream Decks through [hidapi](https://crates.io/crates/hidapi).
//! Heavily based on [python-elgato-streamdeck](https://github.com/abcminiuser/python-elgato-streamdeck) and partially on
//! [streamdeck library for rust](https://github.com/ryankurte/rust-streamdeck).

#![warn(missing_docs)]

use std::str::Utf8Error;

use hidapi::{HidApi, HidDevice, HidError, HidResult};
use image::{DynamicImage, ImageError};
use crate::images::convert_image;

use crate::info::{ELGATO_VENDOR_ID, Kind};
use crate::util::extract_str;

/// Various information about Stream Deck devices
pub mod info;
/// Utility functions for working with Stream Deck devices
pub mod util;
/// Image processing functions
pub mod images;

#[cfg(test)]
mod tests;


/// Creates an instance of the HidApi
///
/// Can be used if you don't want to link hidapi crate into your project
pub fn new_hidapi() -> HidResult<HidApi> {
    HidApi::new()
}

/// Interface for a Stream Deck device
pub struct StreamDeck {
    /// Kind of the device
    kind: Kind,
    /// Connected HIDDevice
    device: HidDevice,
}

/// Static functions of the struct (looking up devices, connecting)
impl StreamDeck {
    /// Returns a list of devices as (Kind, Serial Number) that could be found using HidApi
    pub fn list_devices(hidapi: &HidApi) -> Vec<(Kind, String)> {
        hidapi.device_list()
            .filter_map(|d| {
                if d.vendor_id() != ELGATO_VENDOR_ID {
                    return None;
                }

                if let Some(serial) = d.serial_number() {
                    if !serial.chars().all(|c| c.is_alphanumeric()) {
                        return None;
                    }

                    Some((
                        Kind::from_pid(d.product_id())?,
                        serial.to_string()
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Attempts to connect to the device
    pub fn connect(hidapi: &HidApi, kind: Kind, serial: &str) -> Result<StreamDeck, StreamDeckError> {
        let device = hidapi.open_serial(ELGATO_VENDOR_ID, kind.product_id(), serial)?;

        Ok(StreamDeck {
            kind,
            device
        })
    }
}

/// Instance methods of the struct
impl StreamDeck {
    /// Returns kind of the Stream Deck
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns manufacturer string of the device
    pub fn manufacturer(&mut self) -> Result<String, StreamDeckError> {
        Ok(self.device.get_manufacturer_string()?.unwrap_or_else(|| "Unknown".to_string()))
    }

    /// Returns product string of the device
    pub fn product(&mut self) -> Result<String, StreamDeckError> {
        Ok(self.device.get_product_string()?.unwrap_or_else(|| "Unknown".to_string()))
    }

    /// Performs get_feature_report on [HidDevice], for advanced use
    pub fn get_feature_report(&mut self, report_id: u8, length: usize) -> Result<Vec<u8>, StreamDeckError> {
        let mut buff = vec![0u8; length];

        // Inserting report id byte
        buff.insert(0, report_id);

        // Getting feature report
        self.device.get_feature_report(buff.as_mut_slice())?;

        Ok(buff)
    }

    /// Performs send_feature_report on [HidDevice], for advanced use
    pub fn send_feature_report(&mut self, payload: &[u8]) -> Result<(), StreamDeckError> {
        Ok(self.device.send_feature_report(payload)?)
    }

    /// Writes data to [HidDevice], for advanced use
    pub fn write_data(&mut self, payload: &[u8]) -> Result<usize, StreamDeckError> {
        Ok(self.device.write(payload)?)
    }

    /// Returns serial number of the device
    pub fn serial_number(&mut self) -> Result<String, StreamDeckError> {
        match self.kind {
            Kind::Original | Kind::Mini => {
                let bytes = self.get_feature_report(0x03, 17)?;
                Ok(extract_str(&bytes[5..])?)
            }

            Kind::MiniMk2 => {
                let bytes = self.get_feature_report(0x03, 32)?;
                Ok(extract_str(&bytes[5..])?)
            }

            _ => {
                let bytes = self.get_feature_report(0x06, 32)?;
                Ok(extract_str(&bytes[2..])?)
            }
        }
    }

    /// Returns firmware version of the StreamDeck
    pub fn firmware_version(&mut self) -> Result<String, StreamDeckError> {
        match self.kind {
            Kind::Original | Kind::Mini | Kind::MiniMk2 => {
                let bytes = self.get_feature_report(0x04, 17)?;
                Ok(extract_str(&bytes[5..])?)
            }

            _ => {
                let bytes = self.get_feature_report(0x05, 32)?;
                Ok(extract_str(&bytes[6..])?)
            }
        }
    }

    /// Resets the device
    pub fn reset(&mut self) -> Result<(), StreamDeckError> {
        match self.kind {
            Kind::Original | Kind::Mini | Kind::MiniMk2 => {
                let mut buf = vec![0x0B, 0x63];

                buf.extend(vec![0u8; 15]);

                Ok(self.send_feature_report(buf.as_slice())?)
            }

            _ => {
                let mut buf = vec![0x03, 0x02];

                buf.extend(vec![0u8; 30]);

                Ok(self.send_feature_report(buf.as_slice())?)
            }
        }
    }

    /// Sets brightness of the device, value range is 0 - 100
    pub fn set_brightness(&mut self, percent: u8) -> Result<(), StreamDeckError> {
        let percent = percent.max(0).min(100);

        match self.kind {
            Kind::Original | Kind::Mini | Kind::MiniMk2 => {
                let mut buf = vec![
                    0x05,
                    0x55,
                    0xaa,
                    0xd1,
                    0x01,
                    percent
                ];

                buf.extend(vec![0u8; 11]);

                Ok(self.send_feature_report(buf.as_slice())?)
            }

            _ => {
                let mut buf = vec![
                    0x03,
                    0x08,
                    percent
                ];

                buf.extend(vec![0u8; 29]);

                Ok(self.send_feature_report(buf.as_slice())?)
            }
        }
    }

    /// Writes image data to Stream Deck device
    pub fn write_image(&mut self, key: u8, image_data: &[u8]) -> Result<(), StreamDeckError> {
        if key >= self.kind.key_count() {
            return Err(StreamDeckError::InvalidKeyIndex);
        }

        if !self.kind.is_visual() {
            return Err(StreamDeckError::NoScreen);
        }

        let image_report_length = match self.kind {
            Kind::Original => 8191,
            _ => 1024
        };

        let image_report_header_length = match self.kind {
            Kind::Original | Kind::Mini | Kind::MiniMk2 => 16,
            _ => 8
        };

        let image_report_payload_length = match self.kind {
            Kind::Original => image_data.len() / 2,
            _ => image_report_length - image_report_header_length
        };

        let mut page_number = 0;
        let mut bytes_remaining = image_data.len();

        while bytes_remaining > 0 {
            let this_length = bytes_remaining.min(image_report_payload_length);
            let bytes_sent = page_number * image_report_payload_length;

            // Selecting header based on device
            let mut buf: Vec<u8> = match self.kind {
                Kind::Original => vec![
                    0x02,
                    0x01,
                    (page_number + 1) as u8,
                    0,
                    if this_length == bytes_remaining { 1 } else { 0 },
                    key + 1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],

                Kind::Mini | Kind::MiniMk2 => vec![
                    0x02,
                    0x01,
                    (page_number) as u8,
                    0,
                    if this_length == bytes_remaining { 1 } else { 0 },
                    key + 1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],

                _ => vec![
                    0x02,
                    0x07,
                    key,
                    if this_length == bytes_remaining { 1 } else { 0 },
                    (this_length & 0xff) as u8,
                    (this_length >> 8) as u8,
                    (page_number & 0xff) as u8,
                    (page_number >> 8) as u8,
                ]
            };

            buf.extend(&image_data[bytes_sent .. bytes_sent + this_length]);

            // Adding padding
            buf.extend(vec![0u8; image_report_length - buf.len()]);

            self.write_data(&buf)?;

            bytes_remaining -= this_length;
            page_number += 1;
        }

        Ok(())
    }

    /// Sets specified button's image
    pub fn set_button_image(&mut self, key: u8, image: DynamicImage) -> Result<(), StreamDeckError> {
        let image_data = convert_image(self.kind, image)?;
        Ok(self.write_image(key, &image_data)?)
    }
}

/// Errors that can occur while working with Stream Decks
#[derive(Debug)]
pub enum StreamDeckError {
    /// HidApi error
    HidError(HidError),

    /// Failed to convert bytes into string
    Utf8Error(Utf8Error),

    /// Failed to encode image
    ImageError(ImageError),

    /// There's literally nowhere to write the image
    NoScreen,

    /// Key index is invalid
    InvalidKeyIndex,

    /// Unrecognized Product ID
    UnrecognizedPID,
}

impl From<HidError> for StreamDeckError {
    fn from(e: HidError) -> Self {
        Self::HidError(e)
    }
}

impl From<Utf8Error> for StreamDeckError {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

impl From<ImageError> for StreamDeckError {
    fn from(e: ImageError) -> Self {
        Self::ImageError(e)
    }
}