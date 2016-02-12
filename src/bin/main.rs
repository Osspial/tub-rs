extern crate tub;

use tub::api::win32;

fn main() {
    let config = tub::config::WindowConfig {
        icon: "tub.ico",
        .. Default::default()
    };

    let window = win32::Window::new("A Window", &config);

    window.show();
}