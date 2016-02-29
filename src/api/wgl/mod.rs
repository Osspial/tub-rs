pub mod gl;

use std::mem;
use std::ptr;

use winapi;
use winapi::{HDC, HGLRC, HMODULE};
use gdi32;
use kernel32;

use std::os::raw::c_void;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::io::Error as OsErr;

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
        unsafe {
            // Gets the current hdc and gl context to restore after loading the context creation functions
            let (last_hdc, last_context) = (wgl::GetCurrentDC(), wgl::GetCurrentContext());
            let hdc = window.wrapper.1;

            let (context, gl_library) = {
                let dummy_window = try!(Window::new("A window, dummy", &window.config));
                let d_hdc = dummy_window.wrapper.1;

                try!(set_pixel_format(d_hdc, try!(get_dummy_pixel_format(d_hdc, &format))));

                // Create the dummy context
                let d_context = wgl::CreateContext(d_hdc as *const c_void);
                if d_context == ptr::null_mut() { return Err(TubError::OsError(format!("Dummy Context Creation Error: {}", OsErr::last_os_error()))) }

                // Load the opengl library
                let gl_library = {
                    let name = osstr("opengl32.dll");
                    let library = kernel32::LoadLibraryW(name.as_ptr());

                    if library == ptr::null_mut() { return Err(TubError::OsError(format!("opengl32.dll Load Error: {}", OsErr::last_os_error()))) }

                    library
                };

                // wglMakeCurrent requires an OpenGL context to be current, so this makes the newly created
                // one the current one. After the function pointers have been loaded, the program resets the
                // context to whatever it was previously.
                wgl::MakeCurrent(d_hdc as *const _, d_context as *const _);

                // Load the wgl functions that might not be defined
                let wgl_ex_fns = wgl_ex::Wgl::load_with(|s| get_proc_address(gl_library, s) as *const _);

                if wgl_ex_fns.ChoosePixelFormatARB.is_loaded() &&
                   wgl_ex_fns.CreateContextAttribsARB.is_loaded() {
                    let extns = 
                        if wgl_ex_fns.GetExtensionsStringARB.is_loaded() {
                            let extns_ptr = wgl_ex_fns.GetExtensionsStringARB(d_hdc as *const _);
                            let extns_cstr = CStr::from_ptr(extns_ptr);
                            ::std::str::from_utf8(extns_cstr.to_bytes()).unwrap()
                        }
                        else if wgl_ex_fns.GetExtensionsStringEXT.is_loaded() {
                            let extns_ptr = wgl_ex_fns.GetExtensionsStringEXT();
                            let extns_cstr = CStr::from_ptr(extns_ptr);
                            ::std::str::from_utf8(extns_cstr.to_bytes()).unwrap()
                        }
                        else { "" };


                    let mut attrs = 
                        vec![
                            wgl_ex::DRAW_TO_WINDOW_ARB, 1,
                            wgl_ex::SUPPORT_OPENGL_ARB, 1,
                            wgl_ex::DOUBLE_BUFFER_ARB, 1,
                            wgl_ex::COLOR_BITS_ARB, format.color_bits as u32,
                            wgl_ex::ALPHA_BITS_ARB, format.alpha_bits as u32,
                            wgl_ex::DEPTH_BITS_ARB, format.depth_bits as u32,
                            wgl_ex::STENCIL_BITS_ARB, format.stencil_bits as u32
                        ];

                    if format.color_buffer_float {
                        if extns.contains("WGL_ARB_pixel_format_float") {
                            attrs.push(wgl_ex::TYPE_RGBA_FLOAT_ARB);
                            attrs.push(1);
                        }
                        else {
                            return Err(TubError::ContextCreationError("Could not create floating-point color buffer".to_string()));
                        }
                    }

                    if format.srgb {
                        if extns.contains("WGL_ARB_framebuffer_sRGB") {
                            attrs.push(wgl_ex::FRAMEBUFFER_SRGB_CAPABLE_ARB);
                            attrs.push(1);
                        }
                        else if extns.contains("WGL_EXT_framebuffer_sRGB") {
                            attrs.push(wgl_ex::FRAMEBUFFER_SRGB_CAPABLE_EXT);
                            attrs.push(1);
                        }
                        else {
                            return Err(TubError::ContextCreationError("Could not create SRGB pixel format".to_string()));
                        }
                    }

                    // The attributes list must end with a zero, so this makes it end with a zero
                    attrs.push(0);


                    let mut format = 0;
                    let mut num_formats = 0;
                    wgl_ex_fns.ChoosePixelFormatARB(hdc as *const _, 
                                                    attrs.as_ptr() as *const i32, 
                                                    ptr::null(), 1, 
                                                    &mut format, 
                                                    &mut num_formats);
                    try!(set_pixel_format(hdc, format));
                    
                    let context = 
                        wgl_ex_fns.CreateContextAttribsARB(hdc as *const _,
                                                           ptr::null_mut(),
                                                           ptr::null());
                    if context == ptr::null_mut() { return Err(TubError::ContextCreationError("Could not create OpenGL context".to_string())) }

                    (context, gl_library)
                }
                else {
                    return Err(TubError::ContextCreationError("Could not load extended OpenGL context creation functions".to_string()));
                }
            };


            // Reset the context to what it was before loading the functions
            wgl::MakeCurrent(last_hdc as *const _, last_context as *const _);

            Ok(
                GlContext {
                    hdc: hdc,
                    context: context as HGLRC,
                    gl_library: gl_library,
                    phantom: PhantomData
                }
            )
        }
        
    }

    pub unsafe fn make_current(&self) -> TubResult<()> {
        if wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) == 0 {
            return Err(TubError::OsError(format!("Context Switch Error: {}", OsErr::last_os_error())));
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
        let proc_name = CString::new(proc_name.as_bytes()).unwrap();
        let proc_name = proc_name.as_ptr();
        let proc_addr = wgl::GetProcAddress(proc_name) as *const _;

        match proc_addr as isize {
            0  |
            0x1|
            0x2|
            0x3|
            -1  => kernel32::GetProcAddress(library, proc_name) as *const (),
            _   => proc_addr
        }
    }
}

unsafe fn get_dummy_pixel_format(hdc: HDC, format: &PixelFormat) -> TubResult<i32> {
    let mut pfd: winapi::PIXELFORMATDESCRIPTOR = mem::zeroed();
    pfd.nSize = mem::size_of::<winapi::PIXELFORMATDESCRIPTOR>() as winapi::WORD;
    pfd.nVersion = 1;
    pfd.dwFlags = winapi::PFD_DRAW_TO_WINDOW | winapi::PFD_SUPPORT_OPENGL | winapi::PFD_DOUBLEBUFFER;
    pfd.iPixelType = winapi::PFD_TYPE_RGBA;
    pfd.cColorBits = format.color_bits;
    pfd.cAlphaBits = format.alpha_bits;
    pfd.cDepthBits = format.depth_bits;
    pfd.cStencilBits = format.stencil_bits;
    pfd.iLayerType = winapi::PFD_MAIN_PLANE;

    match gdi32::ChoosePixelFormat(hdc, &pfd) {
        0 => Err(TubError::OsError(format!("Could not get Dummy Pixel Format: {}", OsErr::last_os_error()))),
        f => Ok(f)
    }
}

/// Creates a pixel format for the dummy window, selectively taking relevant parts of the
/// pixel format to make one that resembles the actual format as closely as possible. 
unsafe fn set_pixel_format(hdc: HDC, format_num: i32) -> TubResult<()> {
    let mut pfd = mem::zeroed();
    let pfd_size = mem::size_of::<winapi::PIXELFORMATDESCRIPTOR>() as u32;

    // Get the pixel format description and put it into the PFD struct. If it fails (which it really, 
    // REALLY shouldn't), return an error.
    if gdi32::DescribePixelFormat(hdc, format_num, pfd_size, &mut pfd) == 0 {
        return Err(TubError::OsError(format!("Indescribable Pixel Format (how? I don't know, something must have gone really wrong): {}", OsErr::last_os_error())));
    }


    match gdi32::SetPixelFormat(hdc, format_num, &pfd) {
        0 => Err(TubError::OsError(format!("Dummy Pixel Format Error: {}", OsErr::last_os_error()))),
        1 => Ok(()),
        _ => unreachable!()
    }
}