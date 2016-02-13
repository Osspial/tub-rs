use std::default::Default;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// The window's size in x-coordinates
    pub size_x: i32,
    /// The window's size in y-coordinates
    pub size_y: i32,
    /// Whether or not the window can be resized by dragging on it's edge
    pub resizable: bool,
    /// Whether or not the window can be minimized
    pub maximizable: bool,
    /// Whether or not the window can be maximized
    pub minimizable: bool,

    /// Whether or not the window has an icon on the title bar
    pub title_icon: bool,

    /// The path to the window's icon
    pub icon: Option<PathBuf>
}

impl Default for WindowConfig {
    fn default() -> WindowConfig {
        WindowConfig {
            size_x: 1280,
            size_y: 720,
            resizable: true,
            maximizable: true,
            minimizable: true,

            title_icon: true,

            icon: None
        }
    }
}