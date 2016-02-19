extern crate tub;

use tub::api;
use tub::config::{WindowConfig};
use tub::event::{Event, PressState, VKeyCode};
use std::path::Path;

fn main() {
    let config = WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        size: Some((500, 500)),
        .. Default::default()
    };

    let owned_config = WindowConfig {
        icon: Some(Path::new("tub.ico").to_path_buf()),
        size: Some((300, 300)),
        .. Default::default()
    };

    let window = api::Window::new("It's a window!", config.clone());
    window.focus();
    let mut owned_window: Option<api::Window> = None;

    let mut reset_owned = false;

    window.show();
    window.set_cursor(tub::CursorType::Crosshair);

    loop {
        for event in window.poll_events() {
            println!("{:?}", event);
            match event {
                Event::KeyInput(PressState::Pressed, VKeyCode::D)   => {
                    match owned_window {
                        None    => { 
                            let owned = window.new_owned("Owned Window", owned_config.clone());
                            owned.show();
                            owned.owner().unwrap().disable();
                            owned.focus();
                            owned_window = Some(owned);
                        },

                        Some(_) => ()
                    }
                }

                Event::KeyInput(PressState::Pressed, VKeyCode::E) => {
                    window.set_cursor(tub::CursorType::Crosshair);
                }

                Event::KeyInput(PressState::Released, VKeyCode::E) =>{
                    window.set_cursor(tub::CursorType::Arrow);
                }

                Event::Closed => return,

                _ => ()
            }
        }

        if let Some(ref owned) = owned_window {
            for event in owned.poll_events() {
                match event {
                    Event::KeyInput(PressState::Pressed, VKeyCode::E)   => {
                        owned.set_cursor(tub::CursorType::Crosshair);
                    }

                    Event::Closed   => {
                        owned.owner().unwrap().enable();
                        owned.owner().unwrap().focus();
                        reset_owned = true;
                    }

                    _ => ()
                }
                
            }
        }

        if reset_owned {
            owned_window = None;
            reset_owned = false;
        }
    }
}