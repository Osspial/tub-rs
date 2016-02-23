pub mod gl;

use std::mem;
use std::ptr;

use winapi;
use winapi::{HDC, HGLRC};
use gdi32;

use std::os::raw::c_void;
use std::marker::PhantomData;

use self::gl::wgl;
use api::win32::Window;
use PixelFormat;

pub struct GlContext<'w> {
    hdc: HDC,
    /// A handle to the OpenGL context
    context: HGLRC,
    /// Guarantees that this won't live longer than the window that created it, which would
    /// be very very bad.
    phantom: PhantomData<&'w ()>
}

impl<'w> GlContext<'w> {
    pub fn new(window: &'w Window, format: PixelFormat) -> GlContext<'w> {
        let hdc = window.internal.1;

        if set_pixel_format(hdc, &format) == 0 {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }

        let context = unsafe{ wgl::CreateContext(hdc as *const c_void) };
        if context == ptr::null_mut() {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }

        GlContext {
            hdc: hdc,
            context: context as HGLRC,
            phantom: PhantomData
        }
    }

    pub unsafe fn make_current(&self) {
        if wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) == 0 {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }
    }
}

fn set_pixel_format(hdc: HDC, format: &PixelFormat) -> i32 {
    let mut pfd: winapi::PIXELFORMATDESCRIPTOR = unsafe{ mem::zeroed() };
    pfd.nSize = mem::size_of::<winapi::PIXELFORMATDESCRIPTOR>() as winapi::WORD;
    pfd.nVersion = 1;
    pfd.dwFlags = winapi::PFD_DRAW_TO_WINDOW | winapi::PFD_SUPPORT_OPENGL | winapi::PFD_DOUBLEBUFFER;
    pfd.iPixelType = winapi::PFD_TYPE_RGBA;
    pfd.cColorBits = format.color_bits;
    pfd.cDepthBits = format.depth_bits;
    pfd.cStencilBits = format.stencil_bits;
    pfd.iLayerType = winapi::PFD_MAIN_PLANE;

    unsafe {
        let format = match gdi32::ChoosePixelFormat(hdc, &pfd) {
            0 => panic!("Could not get acceptable pixel format!"),
            f => f
        };

        gdi32::SetPixelFormat(hdc, format, &pfd)
    }
}