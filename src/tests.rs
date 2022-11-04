use std::time::Duration;
use image::open;
use crate::{new_hidapi, StreamDeck};

#[test]
fn test_list_devices() {
    let hid = new_hidapi().expect("No hidapi");

    println!("{:?}", StreamDeck::list_devices(&hid))
}

#[test]
fn test_device() {
    let hid = new_hidapi().expect("No hidapi");

    let (kind, serial) = StreamDeck::list_devices(&hid).remove(0);

    let mut device = StreamDeck::connect(&hid, kind, &serial)
        .expect("Failed to connect");

    println!(
        "Connected to '{}' with version '{}'",
        device.serial_number().unwrap(),
        device.firmware_version().unwrap()
    );

    device.set_brightness(35).unwrap();

    for i in 0..device.kind().key_count() {
        device.clear_button_image(i).unwrap();
    }

    let image = open("no-place-like-localhost.jpg").unwrap();

    device.set_button_image(7, image).unwrap();

    println!("Reading some key states...");

    for _ in 0..20 {
        println!("{:?}", device.read_button_states(Some(Duration::MAX)).unwrap())
    }
}