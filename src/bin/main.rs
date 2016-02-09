extern crate tub;

fn main() {
    let window = tub::api::win32::Window::new("A Window");

    window.show();
    window.event_loop();
}