use std::default::Default;
use std::path::PathBuf;
use std::marker::{Send, Sync};

/// A struct that contains configuration information for any new window that's created.
/// The functions are present to allow the use of method-chaining to set the arguments,
/// which can be more concisely placed into a window creation function than just struct
/// creation. Note that those functions actually take ownership of the value instead of
/// a mutable reference - this is because they aren't intended to be used to mutate a
/// commonly-used WindowConfig but instead to mutate a one-off config used for one window,
/// like this:
/// 
/// ```
/// # use tub::config::WindowConfig;
/// # use tub::platform::Window;
/// # use std::path::Path;
/// let window = Window::new(
///     WindowConfig::new()
///         .name("It's a window!".to_owned())
///         .icon(Some(Path::new("tub.ico").to_path_buf()))
///         .size(Some((500, 500))),
///     Default::default()).unwrap();
/// ```
///
/// If you want to mutate a commonly-used WindowConfig, directly change the struct's fields
/// like so:
///
/// ```
/// # use tub::config::WindowConfig;
/// let mut window_config = WindowConfig::new();
///
/// window_config.name = "A name!".to_owned();
/// window_config.borderless = false;
/// ```
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// The window's name
    pub name: String,
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

    /// Whether or not the window can be transparent
    pub transparent: bool,

    /// The initial state of the window
    pub initial_state: InitialState,

    /// The path to the window's icon
    pub icon: Option<PathBuf>
}

unsafe impl Send for WindowConfig {}
unsafe impl Sync for WindowConfig {}

impl WindowConfig {
    /// Create a new window config. Identical to Default::default()
    #[inline]
    pub fn new() -> WindowConfig {
        Default::default()
    }

    #[inline]
    pub fn name(mut self, name: String) -> WindowConfig {
        self.name = name;
        self
    }

    #[inline]
    pub fn size(mut self, size: Option<(i32, i32)>) -> WindowConfig {
        self.size = size;
        self
    }


    #[inline]
    pub fn topmost(mut self, topmost: bool) -> WindowConfig {
        self.topmost = topmost;
        self
    }


    #[inline]
    pub fn borderless(mut self, borderless: bool) -> WindowConfig {
        self.borderless = borderless;
        self
    }

    #[inline]
    pub fn resizable(mut self, resizable: bool) -> WindowConfig {
        self.resizable = resizable;
        self
    }

    #[inline]
    pub fn maximizable(mut self, maximizable: bool) -> WindowConfig {
        self.maximizable = maximizable;
        self
    }

    #[inline]
    pub fn minimizable(mut self, minimizable: bool) -> WindowConfig {
        self.minimizable = minimizable;
        self
    }

    #[inline]
    pub fn tool_window(mut self, tool_window: bool) -> WindowConfig {
        self.tool_window = tool_window;
        self
    }


    #[inline]
    pub fn transparent(mut self, transparent: bool) -> WindowConfig {
        self.transparent = transparent;
        self
    }


    #[inline]
    pub fn initial_state(mut self, initial_state: InitialState) -> WindowConfig {
        self.initial_state = initial_state;
        self
    }

    #[inline]
    pub fn icon(mut self, icon: Option<PathBuf>) -> WindowConfig {
        self.icon = icon;
        self
    }
}


impl Default for WindowConfig {
    fn default() -> WindowConfig {
        WindowConfig {
            name: String::new(),
            size: None,

            topmost: false,
            
            borderless: false,
            resizable: true,
            maximizable: true,
            minimizable: true,
            tool_window: false,

            transparent: false,

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

/// A struct that contains information about the pixel format for the window. See
/// the WindowConfig documentation for how and when to use the methods
#[derive(Debug, Clone)]
pub struct PixelFormat {
    /// The minimum number of color bits in the framebuffer. Defaults to 24.
    pub color_bits: u8,
    /// The minimum number of alpha bits in the framebuffer. Defaults to 8.
    pub alpha_bits: u8,
    /// The minimum size of the depth buffer. Defaults to 0.
    pub depth_bits: u8,
    /// The minimum size of the stencil buffer. Defaults to 0.
    pub stencil_bits: u8,
    /// Whether or not the window should use SRGB. None is don't care, defaults to None.
    pub srgb: Option<bool>,
    /// Whether or not the window should have a floating-point color buffer. Defaults to false.
    pub color_buffer_float: bool,
    /// The minimum amount of multisampling for the window. Defaults to 0
    pub multisampling: u16,
    /// Whether or not the window should produce a hardware accelerated pixel format. None
    /// is don't care, defaults to Some(true)
    pub hardware_accel: Option<bool>,
    /// Whether or not the window should have a left and right color buffer. Defaults to false.
    pub stereoscopic: bool
}

unsafe impl Send for PixelFormat {}
unsafe impl Sync for PixelFormat {}

impl PixelFormat {
    #[inline]
    pub fn new() -> PixelFormat {
        Default::default()
    }

    #[inline]
    pub fn color_bits(mut self, color_bits: u8) -> PixelFormat {
        self.color_bits = color_bits;
        self
    }

    #[inline]
    /// Part of a healthy and balanced breakfast
    pub fn alpha_bits(mut self, alpha_bits: u8) -> PixelFormat {
        self.alpha_bits = alpha_bits;
        self
    }

    #[inline]
    pub fn depth_bits(mut self, depth_bits: u8) -> PixelFormat {
        self.depth_bits = depth_bits;
        self
    }

    #[inline]
    pub fn stencil_bits(mut self, stencil_bits: u8) -> PixelFormat {
        self.stencil_bits = stencil_bits;
        self
    }

    #[inline]
    pub fn srgb(mut self, srgb: Option<bool>) -> PixelFormat {
        self.srgb = srgb;
        self
    }

    #[inline]
    pub fn color_buffer_float(mut self, color_buffer_float: bool) -> PixelFormat {
        self.color_buffer_float = color_buffer_float;
        self
    }

    #[inline]
    pub fn multisampling(mut self, multisampling: u16) -> PixelFormat {
        self.multisampling = multisampling;
        self
    }

    #[inline]
    pub fn hardware_accel(mut self, hardware_accel: Option<bool>) -> PixelFormat {
        self.hardware_accel = hardware_accel;
        self
    }

    #[inline]
    pub fn stereoscopic(mut self, stereoscopic: bool) -> PixelFormat {
        self.stereoscopic = stereoscopic;
        self
    }
}

impl Default for PixelFormat {
    fn default() -> PixelFormat {
        PixelFormat {
            color_bits: 24,
            alpha_bits: 8,
            depth_bits: 0,
            stencil_bits: 0,
            srgb: None,
            color_buffer_float: false,
            multisampling: 0,
            hardware_accel: Some(true),
            stereoscopic: false
        }
    }
}