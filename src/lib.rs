//! Elgato Streamdeck library
//!
//! Library for interacting with Elgato Stream Decks through [hidapi](https://crates.io/crates/hidapi).
//! Heavily based on [python-elgato-streamdeck](https://github.com/abcminiuser/python-elgato-streamdeck) and partially on
//! [streamdeck library for rust](https://github.com/ryankurte/rust-streamdeck).

#![warn(missing_docs)]

use std::str::{from_utf8, Utf8Error};
use hidapi::{HidApi, HidDevice, HidError, HidResult};
use crate::info::{ELGATO_VENDOR_ID, Kind};
use crate::util::extract_str;

/// Various information about Stream Deck devices
pub mod info;
/// Utility functions for working with Stream Deck devices
pub mod util;

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
}

/// Errors that can occur while working with Stream Decks
#[derive(Debug)]
pub enum StreamDeckError {
    /// HidApi error
    HidError(HidError),

    /// Failed to convert bytes into string
    Utf8Error(Utf8Error),

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