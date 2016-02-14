extern crate tub;

use tub::api;
use std::path::{Path};

fn main() {
    let config = tub::config::WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        .. Default::default()
    };

    let window = api::Window::new("A Window", &config);

    window.show();

    loop {
        window.print_event();
    }
}