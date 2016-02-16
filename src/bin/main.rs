extern crate tub;

use tub::api;
use tub::config::{WindowConfig};
use std::path::Path;

fn main() {
    let config = WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        .. Default::default()
    };

    let owned_config = WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        size: Some((300, 300)),
        .. Default::default()
    };

    let window = api::Window::new("It's a window!", config.clone());
    let owned_window = window.new_owned_window("Owned Window", owned_config);

    window.show();
    owned_window.show();

    loop {
        for event in window.poll_events() {
            println!("Not: {:?}", event);
        }

        for event in owned_window.poll_events() {
            println!("Owned: {:?}", event);
        }
    }
}