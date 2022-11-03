//! Elgato Streamdeck library
//!
//! Library for interacting with Elgato Stream Decks through [hidapi](https://crates.io/crates/hidapi).
//! Heavily based on [python-elgato-streamdeck](https://github.com/abcminiuser/python-elgato-streamdeck) and partially on
//! [streamdeck library for rust](https://github.com/ryankurte/rust-streamdeck).

#![warn(missing_docs)]

use hidapi::{HidApi, HidDevice, HidResult};
use crate::info::{ELGATO_VENDOR_ID, Kind};

/// Various information about Stream Deck devices
pub mod info;
/// Utility functions for working with Stream Deck devices
pub mod util;

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
}