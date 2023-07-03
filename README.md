# elgato-streamdeck
Library for interacting with Elgato Stream Decks through [hidapi](https://crates.io/crates/hidapi). 
Heavily based on [python-elgato-streamdeck](https://github.com/abcminiuser/python-elgato-streamdeck) and partially on
[streamdeck library for rust](https://github.com/ryankurte/rust-streamdeck).

This library was made as a better designed alternative to streamdeck library for Rust.
I just took code from both of the libraries and made it more so pleasant to use.

## udev rules for Linux
If you're using systemd on your system, you might have to install udev rules to allow connecting to Stream Decks from userspace.

You can do that by using following command to copy included into repo `40-streamdeck.rules` file into `udev/rules.d/`
```shell
cp 40-streamdeck.rules /etc/udev/rules.d/
```
And then reloading udev rules
```shell
sudo udevadm control --reload-rules
```
Unplugging and plugging back in the device should also help

You should also create group called "plugdev" and add yourself to it, so you get access to the devices. 
You also need to restart the user session to let user group changes to kick in

## Example
```rust
// Create instance of HidApi
let hid = new_hidapi();

// List devices and unsafely take first one
let (kind, serial) = StreamDeck::list_devices(&hid).remove(0);

// Connect to the device
let mut device = StreamDeck::connect(&hid, kind, &serial)
    .expect("Failed to connect");

// Print out some info from the device
println!(
    "Connected to '{}' with version '{}'",
    device.serial_number().unwrap(),
    device.firmware_version().unwrap()
);

// Set device brightness
device.set_brightness(35).unwrap();

// Use image-rs to load an image
let image = open("no-place-like-localhost.jpg").unwrap();

// Write it to the device
device.set_button_image(7, image).unwrap();
```

## Status
- [x] Convenient to use API for looking up devices, connecting to them and interacting with them
- [x] Reading buttons with async


## Supported Devices
Support of the devices is the same as from libraries above, I personally tested Original v2 and Plus. 
I'll just keep updating this library to match upstream libraries.

But as it stands, this library should support following devices:
- Stream Deck Original
- Stream Deck Original V2
- Stream Deck XL
- Stream Deck XL V2
- Stream Deck Mini
- Stream Deck Mini Mk2
- Stream Deck Mk2
- Stream Deck Pedal
- Stream Deck Plus (thanks to [node-elgato-stream-deck](https://github.com/Julusian/node-elgato-stream-deck))