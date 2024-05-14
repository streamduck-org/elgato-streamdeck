use std::sync::Arc;

use elgato_streamdeck::{list_devices, new_hidapi, DeviceStateUpdate, StreamDeck};
use image::open;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create instance of HidApi
    match new_hidapi() {
        Ok(hid) => {
            // Refresh device list
            for (kind, serial) in list_devices(&hid) {
                println!("{:?} {} {}", kind, serial, kind.product_id());

                // Connect to the device
                let device = StreamDeck::connect(&hid, kind, &serial).expect("Failed to connect");
                // Print out some info from the device
                println!("Connected to '{}' with version '{}'", device.serial_number().unwrap(), device.firmware_version().unwrap());

                device.set_brightness(50).unwrap();
                device.clear_button_image(0xff).unwrap();
                // Use image-rs to load an image
                let image = open("no-place-like-localhost.jpg").unwrap();

                // Write it to the device
                device.set_button_image(7, image).unwrap();

                // device.flush().unwrap();

                let reader = Arc::new(device).get_reader();

                let updates = match reader.read(Some(Duration::from_secs_f64(100.0))) {
                    Ok(updates) => updates,
                    Err(_) => break,
                };
                for update in updates {
                    match update {
                        DeviceStateUpdate::ButtonDown(key) => {
                            println!("Button {} down", key);
                        }
                        DeviceStateUpdate::ButtonUp(key) => {
                            println!("Button {} up", key);
                        }
                        DeviceStateUpdate::EncoderTwist(dial, ticks) => {
                            println!("Dial {} twisted by {}", dial, ticks);
                        }
                        DeviceStateUpdate::EncoderDown(dial) => {
                            println!("Dial {} down", dial);
                        }
                        DeviceStateUpdate::EncoderUp(dial) => {
                            println!("Dial {} up", dial);
                        }
                        _ => {
                            println!("Unknown update");
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to create HidApi instance: {}", e),
    }
}
