extern crate tub;

use tub::api;
use tub::config::{WindowConfig};
use std::path::Path;

fn main() {
    let config = WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        .. Default::default()
    };

    let window = api::Window::new("It's a window!", config.clone());

    window.show();

    loop {
        for event in window.poll_events() {
            println!("{:?}", event);
        }
    }
}