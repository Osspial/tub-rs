extern crate num;
extern crate rand;
extern crate winapi;
extern crate user32;
extern crate gdi32;
extern crate kernel32;
extern crate dwmapi;

mod api;
pub mod platform;
pub mod event;
pub mod config;
pub mod error;

pub enum CursorType {
    AppStarting,
    Arrow,
    Crosshair,
    Hand,
    Help,
    Text,
    Prohibited,
    ResizeAll,
    ResizeNESW,
    ResizeNWSE,
    ResizeVertical,
    ResizeHoriz,
    UpArrow,
    Wait,
    Invisible
}