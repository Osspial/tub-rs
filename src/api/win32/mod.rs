use winapi;
use user32;
use kernel32;
use gdi32;

use winapi::{UINT, WPARAM, LPARAM};
use winapi::windef::HWND;
use winapi::winuser::WNDCLASSEXW;

use std::ptr;
use std::mem;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::collections::VecDeque;

use num::FromPrimitive;

use event::{Event, VKeyCode};

pub struct Window {
    internal: InternalWindow,
    event_queue: VecDeque<Event>
}

impl Window {
    pub fn new<'a>(name: &'a str, config: WindowConfig) -> Window {
        Window {
            internal: InternalWindow::new(name, config),
            event_queue: VecDeque::with_capacity(8)
        }
    }

    pub fn show(&self) {
        self.internal.show();
        self.internal.event_loop();
    }

    pub fn hide(&self) {
        self.internal.hide();
    }
}


pub struct WindowConfig {
    size_x: i32,
    size_y: i32,
    resizable: bool,
    maximizable: bool,
    minimizable: bool,

    title_icon: bool
}

impl WindowConfig {
    pub fn new() -> WindowConfig {
        WindowConfig {
            size_x: 1280,
            size_y: 720,
            resizable: true,
            maximizable: true,
            minimizable: true,

            title_icon: true
        }
    }

    /// Sets the size of windows generated from this config. Defaults to 1280x720.
    pub fn size(&mut self, x: i32, y: i32) -> &mut WindowConfig {
        self.size_x = x;
        self.size_y = y;

        self
    }

    /// Sets if windows generated from this config are resizable. Defaults to true.
    pub fn resizable(&mut self, resizable: bool) -> &mut WindowConfig {
        self.resizable = resizable;

        self
    }

    /// Sets if windows generated from this config can be maximized. Defaults to true.
    /// Has no effect if resizable is set to false.
    pub fn maximizable(&mut self, maximizable: bool) -> &mut WindowConfig {
        self.maximizable = maximizable;

        self
    }

    /// Sets if windows generated from this config can be minimized. Defaults to true.
    pub fn minimizable(&mut self, minimizable: bool) -> &mut WindowConfig {
        self.minimizable = minimizable;

        self
    }

    /// Sets if windows generated from this config have an icon on their title bar. Defaults
    /// to true.
    pub fn has_title_icon(&mut self, title_icon: bool) -> &mut WindowConfig {
        self.title_icon = title_icon;

        self
    }
}

struct InternalWindow( HWND );

impl InternalWindow {
    fn new<'a>(name: &'a str, config: WindowConfig) -> InternalWindow {
        unsafe {
            let class_name = register_window_class();

            let window_name = osstr(name);

            let style = {
                let mut style_temp = 0;

                if config.resizable == true {
                    style_temp |= winapi::WS_SIZEBOX;

                    if config.maximizable == true {
                        style_temp |= winapi::WS_MAXIMIZEBOX;
                    }
                }

                if config.minimizable == true {
                    style_temp |= winapi::WS_MINIMIZEBOX;
                }

                if config.title_icon == true {
                    style_temp |= winapi::WS_SYSMENU;
                }

                style_temp
            };

            let window_handle = user32::CreateWindowExW(
                winapi::WS_EX_CLIENTEDGE,
                class_name.as_ptr(),
                window_name.as_ptr() as winapi::LPCWSTR,
                style,
                winapi::CW_USEDEFAULT,
                winapi::CW_USEDEFAULT,
                config.size_x,
                config.size_y,
                ptr::null_mut(),
                ptr::null_mut(),
                kernel32::GetModuleHandleW(ptr::null()),
                ptr::null_mut()
            );

            if window_handle == ptr::null_mut() {
                panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
            }

            InternalWindow( window_handle )
        }
    }

    #[inline]
    fn show(&self) {
        unsafe {
            user32::ShowWindow(self.0, winapi::SW_SHOW);
        }
    }

    #[inline]
    fn hide(&self) {
        unsafe {
            user32::ShowWindow(self.0, winapi::SW_HIDE);
        }
    }

    fn event_loop(&self) {
        unsafe {
            let mut msg = mem::uninitialized();

            while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                user32::TranslateMessage(&msg);
                user32::DispatchMessageW(&msg);
            }
        }
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = osstr("Window Class");

    let window_class = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as winapi::UINT,
        style: winapi::CS_OWNDC | winapi::CS_VREDRAW | winapi::CS_HREDRAW,
        lpfnWndProc: Some(callback),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: kernel32::GetModuleHandleW(ptr::null()),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ptr::null_mut()
    };
    user32::RegisterClassExW(&window_class);

    class_name
}

unsafe extern "system" fn callback(hwnd: HWND, msg: UINT,
                                   wparam: WPARAM, lparam: LPARAM)
                                   -> winapi::LRESULT {
    
    match msg {
        winapi::WM_KEYDOWN  => {
            println!("{} {:?}", lparam & 0x40000000, VKeyCode::from_u64(wparam));

            0
        }

        winapi::WM_PAINT    => {
            let mut pstruct = mem::uninitialized();
            let hdc = user32::BeginPaint(hwnd, &mut pstruct);

            user32::FillRect(hdc, &pstruct.rcPaint, gdi32::CreateSolidBrush(0x000000));

            user32::EndPaint(hwnd, &pstruct);
            0
        }

        _ => user32::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    
}

fn osstr<'a>(s: &'a str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}