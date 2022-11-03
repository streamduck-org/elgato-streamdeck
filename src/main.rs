use elgato_streamdeck::{new_hidapi, StreamDeck};

fn main() {
    let hid = new_hidapi().expect("No hidapi");

    println!("{:?}", StreamDeck::list_devices(&hid))
}