pub mod gl;

use std::mem;
use std::ptr;

use winapi;
use winapi::{HDC, HGLRC, HMODULE};
use gdi32;
use kernel32;

use std::os::raw::c_void;
use std::ffi::CString;
use std::marker::PhantomData;

use self::gl::wgl;
use api::osstr;
use api::win32::Window;
use PixelFormat;

pub struct GlContext<'w> {
    hdc: HDC,
    /// A handle to the OpenGL context
    context: HGLRC,
    gl_library: HMODULE,
    /// Guarantees that this won't live longer than the window that created it, which would
    /// be very very bad.
    phantom: PhantomData<&'w ()>
}

impl<'w> GlContext<'w> {
    pub fn new(window: &'w Window, format: PixelFormat) -> GlContext<'w> {
        let hdc = window.wrapper.1;

        if set_pixel_format(hdc, &format) == 0 {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }

        let context = unsafe{ wgl::CreateContext(hdc as *const c_void) };
        if context == ptr::null_mut() {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }

        let gl_library = unsafe{
            let name = osstr("opengl32.dll");
            let library = kernel32::LoadLibraryW(name.as_ptr());

            if library == ptr::null_mut() {
                panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
            }

            library
        };

        GlContext {
            hdc: hdc,
            context: context as HGLRC,
            gl_library: gl_library,
            phantom: PhantomData
        }
    }

    pub unsafe fn make_current(&self) {
        if wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) == 0 {
            panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
        }
    }

    pub fn get_proc_address(&self, proc_name: &str) -> *const () {
        unsafe {
            let proc_addr = wgl::GetProcAddress(CString::new(proc_name.as_bytes()).unwrap().as_ptr()) as *const _;

            match proc_addr as isize {
                0  |
                0x1|
                0x2|
                0x3|
                -1  => kernel32::GetProcAddress(self.gl_library, proc_addr as *const i8) as *const (),
                _   => proc_addr
            }
        }
    }

    pub fn swap_buffers(&self) {
        unsafe{ gdi32::SwapBuffers(self.hdc) };
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