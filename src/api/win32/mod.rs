use winapi;
use user32;
use kernel32;

use winapi::{UINT, WPARAM, LPARAM};
use winapi::windef::HWND;
use winapi::winuser::WNDCLASSEXW;

use std::ptr;
use std::mem;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use num::FromPrimitive;

use event::VKeyCode;

pub struct Window {
    window_class: WNDCLASSEXW,
    window_handle: HWND
}

impl Window {
    pub fn new<'a>(name: &'a str) -> Window {
        unsafe {
            let class_name = osstr("Window Class");

            let window_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as winapi::UINT,
                style: 0,
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

            let window_name = osstr(name);

            let window_handle = user32::CreateWindowExW(
                winapi::WS_EX_CLIENTEDGE,
                class_name.as_ptr(),
                window_name.as_ptr() as winapi::LPCWSTR,
                winapi::WS_OVERLAPPEDWINDOW,
                winapi::CW_USEDEFAULT,
                winapi::CW_USEDEFAULT,
                1280,
                720,
                ptr::null_mut(),
                ptr::null_mut(),
                kernel32::GetModuleHandleW(ptr::null()),
                ptr::null_mut()
            );

            if window_handle == ptr::null_mut() {
                panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
            }

            Window {
                window_class: window_class,
                window_handle: window_handle
            }
        }
    }

    pub fn show(&self) {
        unsafe {
            user32::ShowWindow(self.window_handle, winapi::SW_SHOW);
        }
    }

    pub fn hide(&self) {
        unsafe {
            user32::ShowWindow(self.window_handle, winapi::SW_HIDE);
        }
    }

    pub fn event_loop(&self) {
        unsafe {
            let mut msg = mem::uninitialized();

            while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                user32::TranslateMessage(&msg);
                user32::DispatchMessageW(&msg);
            }
        }
    }
}

unsafe extern "system" fn callback(hwnd: HWND, msg: UINT,
                                   wparam: WPARAM, lparam: LPARAM)
                                   -> winapi::LRESULT {
    
    match msg {
        winapi::WM_KEYDOWN  => {
            println!("{} {:?}", lparam & 0x40000000, VKeyCode::from_u64(wparam));

            0
        }

        _ => user32::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
    
}

fn osstr<'a>(s: &'a str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}