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

    let image = open("no-place-like-localhost.jpg").unwrap();

    device.set_button_image(7, image).unwrap();
}