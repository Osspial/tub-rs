use std::mem;

use api::win32;
use api::wgl;
use config::WindowConfig;
use {CursorType, PixelFormat};

pub struct Window<'o>( win32::Window<'o> );

impl<'o> Window<'o> {
    pub fn new<'a>(name: &'a str, config: WindowConfig) -> Window<'o> {
        Window( win32::Window::new(name, config) )
    }

    pub fn new_owned<'a>(&'o self, name: &'a str, config: WindowConfig) -> Window<'o> {
        Window( self.0.new_owned(name, config) )
    }

    #[inline]
    pub fn set_title(&self, title: &str) {
        self.0.wrapper.set_title(title);
    }

    #[inline]
    pub fn show(&self) {
        self.0.wrapper.show();
    }

    #[inline]
    pub fn hide(&self) {
        self.0.wrapper.hide();
    }

    /// Allow the window to take user input. Any newly created window defaults to
    /// being enabled.
    #[inline]
    pub fn enable(&self) {
        self.0.wrapper.enable();
    }

    /// Disallow the window from taking user input.
    #[inline]
    pub fn disable(&self) {
        self.0.wrapper.disable();
    }

    /// Sets input focus to this window
    #[inline]
    pub fn focus(&self) {
        self.0.wrapper.focus();
    }

    #[inline]
    pub fn get_inner_pos(&self) -> Option<(i32, i32)> {
        self.0.wrapper.get_inner_pos()
    }

    /// Gets the position of the upper-left corner of the window, including the title bar
    #[inline]
    pub fn get_outer_pos(&self) -> Option<(i32, i32)> {
        self.0.wrapper.get_outer_pos()
    }

    #[inline]
    pub fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.wrapper.get_inner_size()
    }

    #[inline]
    pub fn get_outer_size(&self) -> Option<(u32, u32)> {
        self.0.wrapper.get_outer_size()
    }

    #[inline]
    pub fn set_pos(&self, x: i32, y: i32) -> Option<()> {
        self.0.wrapper.set_pos(x, y)
    }

    #[inline]
    pub fn set_inner_size(&self, x: u32, y: u32) -> Option<()> {
        self.0.wrapper.set_inner_size(x, y)
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.0.wrapper.is_active()
    }

    #[inline]
    pub fn set_cursor(&self, cursor_type: CursorType) {
        self.0.wrapper.set_cursor(cursor_type);
    }

    #[inline]
    pub fn set_cursor_pos(&self, x: i32, y: i32) {
        self.0.set_cursor_pos(x, y);
    }

    #[inline]
    pub fn owner(&self) -> Option<&Window> {
        match self.0.owner() {
            // Because this struct is just a bitwise-equivalent wrapper around a win32 window, we can
            // just transmute the reference to the wrapper window.
            Some(w) => Some(unsafe{ mem::transmute(w) }),
            None    => None
        }
    }

    #[inline]
    pub fn poll_events(&self) -> PollEventsIter {
        self.0.poll_events()
    }

    #[inline]
    pub fn wait_events(&self) -> WaitEventsIter {
        self.0.wait_events()
    }
}

pub struct GlContext<'w> ( wgl::GlContext<'w> );

impl<'w> GlContext<'w> {
    pub fn new(window: &'w Window, format: PixelFormat) -> GlContext<'w> {
        GlContext( wgl::GlContext::new(&window.0, format) )
    }

    pub unsafe fn make_current(&self) {
        self.0.make_current()
    }
}



pub use api::win32::PollEventsIter;
pub use api::win32::WaitEventsIter;