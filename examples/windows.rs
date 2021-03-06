extern crate tub;

use tub::platform;
use tub::config::{WindowConfig};
use tub::event::{Event, PressState, VKeyCode};
use std::path::Path;

fn main() {
    let owned_config = WindowConfig {
        name: "Owned Window".to_owned(),
        icon: Some(Path::new("tub.ico").to_path_buf()),
        size: Some((300, 300)),
        .. Default::default()
    };

    let window = platform::Window::new(
        WindowConfig::new()
            .name("It's a window!".to_owned())
            .icon(Some(Path::new("tub.ico").to_path_buf()))
            .size(Some((1280, 720))),
        Default::default()).unwrap();
    let window_context = platform::GlContext::new(&window, None).unwrap();
    unsafe{ window_context.make_current().unwrap() };

    let mut owned_window: Option<platform::Window> = None;
    let mut reset_owned = false;

    let child_window = window.new_child(
        WindowConfig::new()
            .name("Child Window".to_owned())
            .size(Some((500, 500))),
        Default::default()).unwrap();

    window.show();
    window.focus();
    child_window.show();
    loop {
        for event in window.poll_events() {
            println!("parent: {:?}", event);
            match event {
                Event::KeyInput(PressState::Pressed, VKeyCode::D)   => {
                    match owned_window {
                        None    => { 
                            let owned = window.new_owned(owned_config.clone(), Default::default()).unwrap();
                            owned.show();
                            owned.get_type().unwrap().disable();
                            owned.focus();
                            owned_window = Some(owned);
                        }

                        Some(_) => ()
                    }
                }

                Event::KeyInput(PressState::Pressed, VKeyCode::E) => {
                    window.set_cursor(tub::CursorType::Crosshair);
                }

                Event::KeyInput(PressState::Pressed, VKeyCode::I) => {
                    window.set_cursor(tub::CursorType::Invisible);
                }

                Event::KeyInput(PressState::Pressed, VKeyCode::G) => {
                    window.set_cursor(tub::CursorType::Hand);
                }

                Event::KeyInput(PressState::Released, VKeyCode::E)|
                Event::KeyInput(PressState::Released, VKeyCode::I)|
                Event::KeyInput(PressState::Released, VKeyCode::G) =>{
                    window.set_cursor(tub::CursorType::Arrow);
                }

                Event::KeyInput(PressState::Pressed, VKeyCode::C) => {
                    window.set_cursor_pos(250, 250);
                }

                Event::Closed => return,

                _ => ()
            }
        }

        if let Some(ref owned) = owned_window {
            for event in owned.poll_events() {
                println!("owned: {:?}", event);
                match event {
                    Event::KeyInput(PressState::Pressed, VKeyCode::E)   => {
                        owned.set_cursor(tub::CursorType::Crosshair);
                    }

                    Event::Closed   => {
                        owned.get_type().unwrap().enable();
                        owned.get_type().unwrap().focus();
                        reset_owned = true;
                    }

                    _ => ()
                }
                
            }
        }

        for event in child_window.poll_events() {
            println!("child: {:?}", event);
        }

        if reset_owned {
            owned_window = None;
            reset_owned = false;
        }
    }
}
