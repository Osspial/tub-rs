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
use api::win32::wrapper::WindowWrapper;
use error::{TubResult, TubError, GlCreationError, GlCreationResult};
use config::PixelFormat;

pub struct GlContext<'w, 'c> {
    hdc: HDC,
    /// A handle to the OpenGL context
    context: HGLRC,
    gl_library: HMODULE,
    /// Guarantees that this won't live longer than the window that created it, which would
    /// be very very bad.
    window_lifetime: PhantomData<&'w ()>,
    /// Guarantees that this won't live longer than any context that this is sharing resources
    /// with.
    shared_lifetime: PhantomData<&'c ()>
}

impl<'w, 'c> GlContext<'w, 'c> {
    pub fn new(window: &'w Window, shared_context: Option<&'c GlContext>) -> GlCreationResult<GlContext<'w, 'c>> {
        unsafe {
            let hdc = window.wrapper.1;

            let (context, gl_library) = {
                let pixel_format = window.get_pixel_format();

                // Create the dummy window. We can unwrap because, if we've gotten this far, the config
                // shouldn't cause any errors.
                let dummy_window = WindowWrapper::new(window.get_config(), None).unwrap();
                let d_hdc = dummy_window.1;

                try!(set_pixel_format(d_hdc, try!(get_dummy_pixel_format(d_hdc, &pixel_format))));

                // Create the dummy context and make it current. If it cannot create the context or make it current,
                // abort this function with an error.
                let d_context = try!(DummyContext::new(d_hdc));
                try!(d_context.make_current());

                // Load the opengl library
                let gl_library = {
                    let name = osstr("opengl32.dll");
                    let library = kernel32::LoadLibraryW(name.as_ptr());

                    if library == ptr::null_mut() { return Err(GlCreationError::OsError(OsErr::last_os_error().to_string(), "Could not load opengl32.dll".to_owned())) }

                    library
                };

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
                            wgl_ex::COLOR_BITS_ARB, pixel_format.color_bits as u32,
                            wgl_ex::ALPHA_BITS_ARB, pixel_format.alpha_bits as u32,
                            wgl_ex::DEPTH_BITS_ARB, pixel_format.depth_bits as u32,
                            wgl_ex::STENCIL_BITS_ARB, pixel_format.stencil_bits as u32
                        ];

                    if pixel_format.color_buffer_float {
                        if extns.contains("WGL_ARB_pixel_format_float") {
                            attrs.push(wgl_ex::TYPE_RGBA_FLOAT_ARB);
                            attrs.push(1);
                        }
                        else {
                            return Err(GlCreationError::FloatingBufferError);
                        }
                    }

                    if let Some(srgb) = pixel_format.srgb {
                        if extns.contains("WGL_ARB_framebuffer_sRGB") {
                            attrs.push(wgl_ex::FRAMEBUFFER_SRGB_CAPABLE_ARB);
                        }
                        else if extns.contains("WGL_EXT_framebuffer_sRGB") {
                            attrs.push(wgl_ex::FRAMEBUFFER_SRGB_CAPABLE_EXT);
                        }
                        else {
                            return Err(GlCreationError::SRGBBufferError);
                        }

                        match srgb {
                            true  => attrs.push(1),
                            false => attrs.push(0)
                        }
                    }

                    if pixel_format.multisampling > 0 {
                        if extns.contains("WGL_ARB_multisample") {
                            attrs.push(wgl_ex::SAMPLE_BUFFERS_ARB);
                            attrs.push(1);
                            attrs.push(wgl_ex::SAMPLES_ARB);
                            attrs.push(pixel_format.multisampling as u32);
                        }
                        else {
                            return Err(GlCreationError::MSAABufferError);
                        }
                    }

                    if pixel_format.stereoscopic {
                        attrs.push(wgl_ex::STEREO_ARB);
                        attrs.push(1);
                    }

                    if let Some(accel) = pixel_format.hardware_accel {
                        attrs.push(wgl_ex::ACCELERATION_ARB);
                        match accel {
                            true  => attrs.push(wgl_ex::FULL_ACCELERATION_ARB),
                            false => attrs.push(wgl_ex::NO_ACCELERATION_ARB)
                        }
                    }

                    // The attributes list must end with a zero, so this makes it end with a zero
                    attrs.push(0);


                    let mut format_num = 0;
                    let mut format_count = 0;
                    wgl_ex_fns.ChoosePixelFormatARB(hdc as *const _, 
                                                    attrs.as_ptr() as *const i32, 
                                                    ptr::null(), 1, 
                                                    &mut format_num, 
                                                    &mut format_count);
                    try!(set_pixel_format(hdc, format_num));
                    
                    let shared_context_ptr = 
                        match shared_context {
                            Some(c) => c.context as *const c_void,
                            None    => ptr::null()
                        };
                    let context = 
                        wgl_ex_fns.CreateContextAttribsARB(hdc as *const _,
                                                           shared_context_ptr,
                                                           ptr::null());
                    if context == ptr::null_mut() { return Err(GlCreationError::ExtendedCreationError) }

                    (context, gl_library)
                }
                else {
                    return Err(GlCreationError::FunctionLoadError);
                }
            };


            Ok(
                GlContext {
                    hdc: hdc,
                    context: context as HGLRC,
                    gl_library: gl_library,
                    window_lifetime: PhantomData,
                    shared_lifetime: PhantomData
                }
            )
        }
        
    }

    pub unsafe fn make_current(&self) -> TubResult<()> {
        if wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) == 0 {
            return Err(TubError::OsError(format!("Context Switch Error: {}", OsErr::last_os_error().to_string())));
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

impl<'w, 'c> Drop for GlContext<'w, 'c> {
    fn drop(&mut self) {
        unsafe {
            wgl::DeleteContext(self.context as *const _);
        }
    }
}

/// A wrapper around the dummy OpenGL context. Why, you ask, does this exist? Well, there are multiple
/// instances where using the functions this context provides may fail, causing the actual OpenGL context
/// creation function to return an Err. When that happens, certain cleanup must be done and it's easiest
/// to wrap that cleanup in an RAII wrapper than it is to have a bit of boilerplate every time. That cleanup
/// is to reset the context to the one created before the function was run and to delete this context.
struct DummyContext {
    hdc: HDC,
    context: HGLRC,
    last_hdc: HDC,
    last_context: HGLRC
}

impl DummyContext {
    fn new(hdc: HDC) -> GlCreationResult<DummyContext> {
        unsafe {
            let context = wgl::CreateContext(hdc as *const c_void);
            if context == ptr::null_mut() {
                return Err(GlCreationError::OsError(OsErr::last_os_error().to_string(), "Could not create dummy context".to_owned()));
            }

            Ok(
                DummyContext {
                    hdc: hdc,
                    context: context as HGLRC,
                    last_hdc: wgl::GetCurrentDC() as HDC,
                    last_context: wgl::GetCurrentContext() as HGLRC
                }
            )
        }
    }

    unsafe fn make_current(&self) -> GlCreationResult<()> {
        match wgl::MakeCurrent(self.hdc as *const c_void, self.context as *const c_void) {
            0 => Err(GlCreationError::OsError(OsErr::last_os_error().to_string(), "Could not make dummy context current".to_owned())),
            _ => Ok(())
        }
    }
}

impl Drop for DummyContext {
    fn drop(&mut self) {
        unsafe {
            wgl::MakeCurrent(self.last_hdc as *const _, self.last_context as *const _);
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

unsafe fn get_dummy_pixel_format(hdc: HDC, pixel_format: &PixelFormat) -> GlCreationResult<i32> {
    let mut pfd: winapi::PIXELFORMATDESCRIPTOR = mem::zeroed();
    pfd.nSize = mem::size_of::<winapi::PIXELFORMATDESCRIPTOR>() as winapi::WORD;
    pfd.nVersion = 1;
    pfd.dwFlags = winapi::PFD_DRAW_TO_WINDOW | winapi::PFD_SUPPORT_OPENGL | winapi::PFD_DOUBLEBUFFER;
    pfd.iPixelType = winapi::PFD_TYPE_RGBA;
    pfd.cColorBits = pixel_format.color_bits;
    pfd.cAlphaBits = pixel_format.alpha_bits;
    pfd.cDepthBits = pixel_format.depth_bits;
    pfd.cStencilBits = pixel_format.stencil_bits;
    pfd.iLayerType = winapi::PFD_MAIN_PLANE;

    match gdi32::ChoosePixelFormat(hdc, &pfd) {
        0 => Err(GlCreationError::OsError(OsErr::last_os_error().to_string(), "Could not get Dummy Pixel Format: {}".to_owned())),
        f => Ok(f)
    }
}

/// Creates a pixel format for the dummy window, selectively taking relevant parts of the
/// pixel format to make one that resembles the actual format as closely as possible. 
unsafe fn set_pixel_format(hdc: HDC, format_num: i32) -> GlCreationResult<()> {
    let mut pfd = mem::zeroed();
    let pfd_size = mem::size_of::<winapi::PIXELFORMATDESCRIPTOR>() as u32;

    // Get the pixel format description and put it into the PFD struct. If it fails (which it really, 
    // REALLY shouldn't), return an error.
    if gdi32::DescribePixelFormat(hdc, format_num, pfd_size, &mut pfd) == 0 {
        return Err(GlCreationError::IndescribableFormatError(OsErr::last_os_error().to_string()));
    }


    match gdi32::SetPixelFormat(hdc, format_num, &pfd) {
        0 => Err(GlCreationError::OsError(OsErr::last_os_error().to_string(), "Could not set window pixel format".to_owned())),
        1 => Ok(()),
        _ => unreachable!()
    }
}