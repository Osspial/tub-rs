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

use std::default::Default;

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

#[derive(Debug, Clone, Copy)]
pub struct PixelFormat {
    pub color_bits: u8,
    pub alpha_bits: u8,
    pub depth_bits: u8,
    pub stencil_bits: u8,
    pub srgb: Option<bool>,
    pub color_buffer_float: bool,
}

impl Default for PixelFormat {
    fn default() -> PixelFormat {
        PixelFormat {
            color_bits: 32,
            alpha_bits: 8,
            depth_bits: 0,
            stencil_bits: 0,
            srgb: None,
            color_buffer_float: false
        }
    }
}