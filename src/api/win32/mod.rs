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
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use num::FromPrimitive;

use config::WindowConfig;
use event::{Event, VKeyCode};

pub struct Window {
    internal: InternalWindow,
}

impl Window {
    pub fn new<'a>(name: &'a str, config: &WindowConfig) -> Window {
        let (sx, rx) = mpsc::channel();
        let name = name.into();
        let config = config.clone();

        thread::spawn(move || {
            unsafe {
                let internal_window = InternalWindow::new(name, config);
                sx.send(internal_window).unwrap();

                let mut msg = mem::uninitialized();

                while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        });

        Window {
            internal: rx.recv().unwrap(),
        }
    }

    pub fn show(&self) {
        self.internal.show();
    }

    pub fn hide(&self) {
        self.internal.hide();
    }

    pub fn kill(self) {
        unsafe {
            user32::DestroyWindow(self.internal.0);
        }
    }
}

#[derive(Clone)]
struct InternalWindow( HWND );

unsafe impl Send for InternalWindow {}
unsafe impl Sync for InternalWindow {}

impl InternalWindow {
    fn new<'a>(name: String, config: WindowConfig) -> InternalWindow {
        unsafe {
            let class_name = register_window_class();

            let window_name = osstr(&name);

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

            if let Some(p) = config.icon {
                let path = wide_path(p).as_ptr();

                let icon = user32::LoadImageW(ptr::null_mut(), path, winapi::IMAGE_ICON, 32, 32, winapi::LR_LOADFROMFILE);
                if icon != ptr::null_mut() {
                    user32::SendMessageW(window_handle, winapi::WM_SETICON, winapi::ICON_BIG as u64, icon as winapi::LPARAM);
                }
                else {
                    panic!("Could not load 32x32 icon (TODO: Make this not panic)");
                }

                let icon = user32::LoadImageW(ptr::null_mut(), path, winapi::IMAGE_ICON, 16, 16, winapi::LR_LOADFROMFILE);
                if icon != ptr::null_mut() {
                    user32::SendMessageW(window_handle, winapi::WM_SETICON, winapi::ICON_SMALL as u64, icon as winapi::LPARAM);
                }
                else {
                    panic!("Could not load 16x16 icon (TODO: Make this not panic)");
                }
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

fn wide_path(path: PathBuf) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}