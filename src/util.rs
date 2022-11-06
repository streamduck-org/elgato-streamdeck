use std::str::{from_utf8, Utf8Error};
use std::time::Duration;
use hidapi::{HidDevice, HidError};
use crate::Kind;

/// Performs get_feature_report on [HidDevice]
pub fn get_feature_report(device: &HidDevice, report_id: u8, length: usize) -> Result<Vec<u8>, HidError> {
    let mut buff = vec![0u8; length];

    // Inserting report id byte
    buff.insert(0, report_id);

    // Getting feature report
    device.get_feature_report(buff.as_mut_slice())?;

    Ok(buff)
}

/// Performs send_feature_report on [HidDevice]
pub fn send_feature_report(device: &HidDevice, payload: &[u8]) -> Result<(), HidError> {
    Ok(device.send_feature_report(payload)?)
}

/// Reads data from [HidDevice]. Blocking mode is used if timeout is specified
pub fn read_data(device: &HidDevice, length: usize, timeout: Option<Duration>) -> Result<Vec<u8>, HidError> {
    device.set_blocking_mode(timeout.is_some())?;

    let mut buf = vec![0u8; length];

    match timeout {
        Some(timeout) => device.read_timeout(buf.as_mut_slice(), timeout.as_millis() as i32),
        None => device.read(buf.as_mut_slice())
    }?;

    Ok(buf)
}

/// Writes data to [HidDevice]
pub fn write_data(device: &HidDevice, payload: &[u8]) -> Result<usize, HidError> {
    Ok(device.write(payload)?)
}

/// Extracts string from byte array, removing \0 symbols
pub fn extract_str(bytes: &[u8]) -> Result<String, Utf8Error> {
    Ok(from_utf8(bytes)?.replace('\0', "").to_string())
}

/// Flips key index horizontally, for use with Original v1 Stream Deck
pub fn flip_key_index(kind: Kind, key: u8) -> u8 {
    let col = key % kind.column_count();
    return (key - col) + ((kind.column_count() - 1) - col);
}