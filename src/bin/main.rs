extern crate tub;

use tub::api::win32;

fn main() {
    let window = win32::Window::new("A Window", win32::WindowConfig::new());

    window.show();
}