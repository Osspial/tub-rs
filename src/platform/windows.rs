use std::mem;

use api::win32;
use api::wgl;
use error::TubResult;
use config::{WindowConfig, PixelFormat};
use CursorType;

pub struct Window<'o>( win32::Window<'o> );

impl<'o> Window<'o> {
    pub fn new<'a>(config: WindowConfig, pixel_format: PixelFormat) -> TubResult<Window<'o>> {
        // Because this struct is just a bitwise-equivalent wrapper around a win32 window, we can
        // just transmute the reference to the result.
        unsafe{ mem::transmute(win32::Window::new(config, pixel_format)) }
    }

    pub fn new_owned<'a>(&'o self, config: WindowConfig, pixel_format: PixelFormat) -> TubResult<Window<'o>> {
        unsafe{ mem::transmute(self.0.new_owned(config, pixel_format)) }
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
        unsafe{ mem::transmute(self.0.owner()) }
    }

    #[inline]
    pub fn get_config(&self) -> &WindowConfig {
        self.0.get_config()
    }

    #[inline]
    pub fn get_pixel_format(&self) -> &PixelFormat {
        self.0.get_pixel_format()
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

pub struct GlContext<'w, 'c> ( wgl::GlContext<'w, 'c> );

impl<'w, 'c> GlContext<'w, 'c> {
    pub fn new(window: &'w Window, shared_context: Option<&'c GlContext>) -> TubResult<GlContext<'w, 'c>> {
        unsafe{ mem::transmute(wgl::GlContext::new(&window.0, mem::transmute(shared_context))) }
    }

    pub unsafe fn make_current(&self) -> TubResult<()> {
        self.0.make_current()
    }

    pub fn get_proc_address(&self, proc_name: &str) -> *const () {
        self.0.get_proc_address(proc_name)
    }

    pub fn swap_buffers(&self) {
        self.0.swap_buffers()
    }
}



pub use api::win32::PollEventsIter;
pub use api::win32::WaitEventsIter;