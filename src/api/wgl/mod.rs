pub mod gl;

use std::mem;
use std::ptr;

use winapi;
use winapi::{HDC, HGLRC, HMODULE};
use gdi32;
use kernel32;

use std::os::raw::c_void;
use std::ffi::{CString};
use std::marker::PhantomData;

use self::gl::wgl;
use self::gl::wgl_ex;
use api::osstr;
use api::win32::Window;
use error::{TubResult, TubError};
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
    pub fn new(window: &'w Window, format: PixelFormat) -> TubResult<GlContext<'w>> {
        let hdc = window.wrapper.1;

        if set_pixel_format(hdc, &format) == 0 { return Err(TubError::OsError(format!("Error: {}", ::std::io::Error::last_os_error()))) }

        let context = unsafe{ wgl::CreateContext(hdc as *const c_void) };
        if context == ptr::null_mut() { return Err(TubError::OsError(format!("Error: {}", ::std::io::Error::last_os_error()))) }

        let gl_library = unsafe{
            let name = osstr("opengl32.dll");
            let library = kernel32::LoadLibraryW(name.as_ptr());

            if library == ptr::null_mut() { return Err(TubError::OsError(format!("Error: {}", ::std::io::Error::last_os_error()))) }

            library
        };


        // Gets the current hdc and gl context to restore after loading the functions
        let (last_hdc, last_context) = unsafe{ (wgl::GetCurrentDC(), wgl::GetCurrentContext()) };

        // wglMakeCurrent requires an OpenGL context to be current, so this makes the newly created
        // one the current one. After the function pointers have been loaded, the program resets the
        // context to whatever it was previously.
        unsafe{ wgl::MakeCurrent(hdc as *const _, context as *const _) };

        // Load the wgl functions that might not be defined
        let wgl_ex_fns = wgl_ex::Wgl::load_with(|s| get_proc_address(gl_library, s) as *const _);

        // Reset the context to what it was before loading the functions
        unsafe{ wgl::MakeCurrent(last_hdc as *const _, last_context as *const _) };

        Ok(
            GlContext {
                hdc: hdc,
                context: context as HGLRC,
                gl_library: gl_library,
                phantom: PhantomData
            }
        )
    }

    pub unsafe fn make_current(&self) -> TubResult<()> {
        if wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) == 0 {
            return Err(TubError::OsError(format!("Error: {}", ::std::io::Error::last_os_error())));
        }
        Ok(())
    }

    pub fn get_proc_address(&self, proc_name: &str) -> *const () {
        get_proc_address(self.gl_library, proc_name)
    }

    pub fn swap_buffers(&self) {
        unsafe{ gdi32::SwapBuffers(self.hdc) };
    }
}

impl<'w> Drop for GlContext<'w> {
    fn drop(&mut self) {
        unsafe {
            wgl::DeleteContext(self.context as *const _);
        }
    }
}

fn get_proc_address(library: HMODULE, proc_name: &str) -> *const () {
    unsafe {
        let proc_addr = CString::new(proc_name.as_bytes()).unwrap();
        let proc_addr = wgl::GetProcAddress(proc_addr.as_ptr()) as *const _;

        match proc_addr as isize {
            0  |
            0x1|
            0x2|
            0x3|
            -1  => kernel32::GetProcAddress(library, proc_addr as *const i8) as *const (),
            _   => proc_addr
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