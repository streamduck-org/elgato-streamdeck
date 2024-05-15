use std::str::{from_utf8, Utf8Error};
use std::time::Duration;
use hidapi::{HidDevice, HidError};
use crate::{Kind, StreamDeckError, StreamDeckInput};

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
        None => device.read(buf.as_mut_slice()),
    }?;

    Ok(buf)
}

/// Writes data to [HidDevice]
pub fn write_data(device: &HidDevice, payload: &[u8]) -> Result<usize, HidError> {
    #[cfg(target_os = "windows")]
    let platform = "windows";
    #[cfg(target_os = "macos")]
    let platform = "mac";
    #[cfg(target_os = "linux")]
    let platform = "linux";

    match platform {
        "windows" => Ok(device.write(payload)?),
        _ => {
            if payload[0] == 0 {
                let mut buf = vec![0u8; payload.len() + 1];
                buf[0] = 0;
                buf[1..].copy_from_slice(payload);
                return Ok(device.write(buf.as_slice())?);
            } else {
                Ok(device.write(payload)?)
            }
        }
    }
}

/// Extracts string from byte array, removing \0 symbols
pub fn extract_str(bytes: &[u8]) -> Result<String, Utf8Error> {
    Ok(from_utf8(bytes)?.replace('\0', "").to_string())
}

/*
 Ajazz's key index
 -----------------------------
| 0d | 0a | 07 | 04 | 01 | 10 |
|----|----|----|----|----|----|
| 0e | 0b | 08 | 05 | 02 | 11 |
|----|----|----|----|----|----|
| 0f | 0c | 09 | 06 | 03 | 12 |
 -----------------------------
 Elgato's key index
 -----------------------------
| 01 | 02 | 03 | 04 | 05 | 06 |
|----|----|----|----|----|----|
| 07 | 08 | 09 | 10 | 11 | 12 |
|----|----|----|----|----|----|
| 13 | 14 | 15 | 16 | 17 | 18 |
 -----------------------------
*/

/// Converts Elgato key index to Ajazz key index
pub fn elgato_to_ajazz(kind: &Kind, key: u8) -> u8 {
    if key < kind.key_count() {
        return [12, 9, 6, 3, 0, 15, 13, 10, 7, 4, 1, 16, 14, 11, 8, 5, 2, 17][key as usize];
    } else {
        return key;
    }
}

/// Converts Ajazz key index to Elgato key index
pub fn ajazz_to_elgato_input(kind: &Kind, key: u8) -> u8 {
    if key < kind.key_count() {
        return [4, 10, 16, 3, 9, 15, 2, 8, 14, 1, 7, 13, 0, 6, 12, 5, 11, 17][key as usize];
    } else {
        return key;
    }
}

/// Flips key index horizontally, for use with Original v1 Stream Deck
pub fn flip_key_index(kind: &Kind, key: u8) -> u8 {
    let col = key % kind.column_count();
    return (key - col) + ((kind.column_count() - 1) - col);
}

/// Reads button states, empty vector if no data
pub fn read_button_states(kind: &Kind, states: &Vec<u8>) -> Vec<bool> {
    if states[0] == 0 {
        return vec![];
    }

    match kind {
        Kind::Akp153 => {
            let mut bools = vec![];

            for i in 0..kind.key_count() {
                bools.push(states[(i + 1) as usize] != 0);
            }

            bools
        }

        Kind::Original => {
            let mut bools = vec![];

            for i in 0..kind.key_count() {
                let flipped_i = flip_key_index(kind, i) as usize;

                bools.push(states[flipped_i + 1] != 0);
            }

            bools
        }

        Kind::Mini | Kind::MiniMk2 => states[1..].iter().map(|s| *s != 0).collect(),

        _ => states[4..].iter().map(|s| *s != 0).collect(),
    }
}

/// Reads lcd screen input
pub fn read_lcd_input(data: &Vec<u8>) -> Result<StreamDeckInput, StreamDeckError> {
    let start_x = u16::from_le_bytes([data[6], data[7]]);
    let start_y = u16::from_le_bytes([data[8], data[9]]);

    match &data[4] {
        0x1 => Ok(StreamDeckInput::TouchScreenPress(start_x, start_y)),
        0x2 => Ok(StreamDeckInput::TouchScreenLongPress(start_x, start_y)),

        0x3 => {
            let end_x = u16::from_le_bytes([data[10], data[11]]);
            let end_y = u16::from_le_bytes([data[12], data[13]]);

            Ok(StreamDeckInput::TouchScreenSwipe((start_x, start_y), (end_x, end_y)))
        }

        _ => Err(StreamDeckError::BadData),
    }
}

/// Reads encoder input
pub fn read_encoder_input(kind: &Kind, data: &Vec<u8>) -> Result<StreamDeckInput, StreamDeckError> {
    match &data[4] {
        0x0 => Ok(StreamDeckInput::EncoderStateChange(data[5..5 + kind.encoder_count() as usize].iter().map(|s| *s != 0).collect())),

        0x1 => Ok(StreamDeckInput::EncoderTwist(
            data[5..5 + kind.encoder_count() as usize].iter().map(|s| i8::from_le_bytes([*s])).collect(),
        )),

        _ => Err(StreamDeckError::BadData),
    }
}
