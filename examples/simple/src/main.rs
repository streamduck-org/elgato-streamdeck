use std::sync::Arc;
use std::time::Duration;

use elgato_streamdeck::{list_devices, new_hidapi, DeviceStateUpdate, StreamDeck};
use image::open;

#[tokio::main]
async fn main() {
    // Create instance of HidApi
    match new_hidapi() {
        Ok(hid) => {
            // Refresh device list
            for (kind, serial) in list_devices(&hid) {
                println!("{:?} {} {}", kind, serial, kind.product_id());

                // Connect to the device
                let mut device = StreamDeck::connect(&hid, kind, &serial).expect("Failed to connect");
                // Print out some info from the device
                println!("Connected to '{}' with version '{}'", device.serial_number().unwrap(), device.firmware_version().unwrap());

                device.set_brightness(50).unwrap();
                device.clear_button_image(0xff).unwrap();
                // Use image-rs to load an image
                let image = open("no-place-like-localhost.jpg").unwrap();

                // device.set_logo_image(image.clone()).unwrap();

                device.set_brightness(50).unwrap();
                device.clear_button_image(0xff).unwrap();

                println!("Key count: {}", kind.key_count());
                // Write it to the device
                for i in 0..kind.key_count() as u8 {
                    device.set_button_image(i, image.clone()).unwrap();
                }

                // Flush
                if device.updated {
                    device.flush().unwrap();
                }

                let device = Arc::new(device);
                {
                    let reader = device.get_reader();

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

                    drop(reader);
                }

                device.shutdown().unwrap();
            }
        }
        Err(e) => eprintln!("Failed to create HidApi instance: {}", e),
    }
}
