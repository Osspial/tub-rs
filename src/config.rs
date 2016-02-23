use std::default::Default;
use std::path::PathBuf;
use std::marker::{Send, Sync};

unsafe impl Send for WindowConfig {}
unsafe impl Sync for WindowConfig {}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// The window's dimensions
    pub size: Option<(i32, i32)>,

    /// Whether or not the window is a topmost window. If true, this window will
    /// always appear at the top of the Z order
    pub topmost: bool,

    /// Whether or not the window is a borderless window. Note that this will
    /// override any specified window decorations.
    pub borderless: bool,
    /// Whether or not the window can be resized by dragging on it's edge
    pub resizable: bool,
    /// Whether or not the window can be minimized
    pub maximizable: bool,
    /// Whether or not the window can be maximized
    pub minimizable: bool,
    /// Whether or not the window appears on the taskbar
    pub tool_window: bool,

    /// The initial state of the window
    pub initial_state: InitialState,

    /// The path to the window's icon
    pub icon: Option<PathBuf>
}

impl Default for WindowConfig {
    fn default() -> WindowConfig {
        WindowConfig {
            size: None,

            topmost: false,
            
            borderless: false,
            resizable: true,
            maximizable: true,
            minimizable: true,
            tool_window: false,

            initial_state: InitialState::Windowed,

            icon: None
        }
    }
}

/// The initial state of the window
#[derive(Debug, Clone, Copy)]
pub enum InitialState {
    /// The window starts as a floating window
    Windowed,
    /// The window starts minimized
    Minimized,
    /// The window starts maximized
    Maximized
}