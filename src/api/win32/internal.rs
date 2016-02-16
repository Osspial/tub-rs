use winapi;
use user32;
use kernel32;
use gdi32;

use winapi::{UINT, WPARAM, LPARAM};
use winapi::windef::HWND;
use winapi::winuser::WNDCLASSEXW;

use std::ptr;
use std::mem;
use std::ops::Drop;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::cell::RefCell;
use std::sync::mpsc::Sender;

use num::FromPrimitive;

use config::WindowConfig;
use event::{Event, VKeyCode};

#[derive(Clone)]
pub struct InternalWindow( pub HWND );

unsafe impl Send for InternalWindow {}
unsafe impl Sync for InternalWindow {}

impl InternalWindow {
    pub fn new<'a>(name: String, config: WindowConfig) -> InternalWindow {
        unsafe {
            let class_name = register_window_class();

            let window_name = osstr(&name);

            let (style, style_ex) = {
                use config::InitialState::*;

                let mut style = winapi::WS_SYSMENU;
                let mut style_ex = 0;

                if !config.borderless && !config.tool_window {
                    if config.resizable {
                        style |= winapi::WS_SIZEBOX;

                        if config.maximizable {
                            style |= winapi::WS_MAXIMIZEBOX;
                        }
                    }

                    if config.minimizable {
                        style |= winapi::WS_MINIMIZEBOX;
                    }

                    style_ex |= winapi::WS_EX_WINDOWEDGE;
                }

                if config.tool_window {
                    style_ex |= winapi::WS_EX_TOOLWINDOW;
                }

                match config.initial_state {
                    Windowed    => (),
                    Minimized   => style |= winapi::WS_MINIMIZE,
                    Maximized   => style |= winapi::WS_MAXIMIZE
                }

                (style, style_ex)
            };

            let size =
                match config.size {
                    Some(s) => s,
                    None    => (winapi::CW_USEDEFAULT, winapi::CW_USEDEFAULT)
                };

            let window_handle = user32::CreateWindowExW(
                style_ex,
                class_name.as_ptr(),
                window_name.as_ptr() as winapi::LPCWSTR,
                style,
                winapi::CW_USEDEFAULT,
                winapi::CW_USEDEFAULT,
                size.0,
                size.1,
                ptr::null_mut(),
                ptr::null_mut(),
                kernel32::GetModuleHandleW(ptr::null()),
                ptr::null_mut()
            );

            if config.borderless {
                user32::SetWindowLongW(window_handle, -16, 0);
            }

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

    pub fn set_title(&self, title: &str) {
        unsafe {
            let title = osstr(title);
            user32::SetWindowTextW(self.0, title.as_ptr());
        }
    }

    #[inline]
    pub fn show(&self) {
        unsafe {
            user32::ShowWindow(self.0, winapi::SW_SHOW);
        }
    }

    #[inline]
    pub fn hide(&self) {
        unsafe {
            user32::ShowWindow(self.0, winapi::SW_HIDE);
        }
    }

    pub fn kill(&self) {
        unsafe {
            user32::PostMessageW(self.0, winapi::WM_DESTROY, 0, 0);
        }
    }
}

impl Drop for InternalWindow {
    fn drop(&mut self) {
        self.kill();
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

#[allow(non_upper_case_globals)]
thread_local!(pub static CALLBACK_DATA: RefCell<Option<Vec<CallbackData>>> = RefCell::new(None));

pub struct CallbackData {
    pub window: HWND,
    pub sender: Sender<Event>
}

impl CallbackData {
    #[inline]
    pub fn new(window: HWND, sender: Sender<Event>) -> CallbackData {
        CallbackData {
            window: window,
            sender: sender
        }
    }
}

fn send_event(source: HWND, event: Event) {
    CALLBACK_DATA.with(|data| {
        let vector = data.borrow();

        let vector = match *vector {
            Some(ref v) => v,
            None => return
        };

        match get_window_index(vector, source) {
            -1  => (),
            i   => {vector[i as usize].sender.send(event).ok();}
        }
    });
}

fn get_window_index(vector: &Vec<CallbackData>, window: HWND) -> isize {
    let mut index = vector.len();

    if index != 0 {
        while index != 0 {
            index -= 1;

            if vector[index].window == window {
                return index as isize;
            }
        }
    }

    -1
}

unsafe extern "system" fn callback(hwnd: HWND, msg: UINT,
                                   wparam: WPARAM, lparam: LPARAM)
                                   -> winapi::LRESULT {
    
    match msg {
        winapi::WM_KEYDOWN  => {
            use event::Event::KeyInput;
            use event::PressState;

            let press_state = {
                match lparam & 0x40000000 {
                    0 => PressState::Pressed,
                    _ => PressState::Held
                }
            };

            match VKeyCode::from_u64(wparam) {
                Some(k) => send_event(hwnd, KeyInput(press_state, k)),
                None    => ()
            }

            0
        }

        winapi::WM_KEYUP    => {
            use event::Event::KeyInput;
            use event::PressState;

            match VKeyCode::from_u64(wparam) {
                Some(k) => send_event(hwnd, KeyInput(PressState::Released, k)),
                None    => ()
            }

            0
        }

        winapi::WM_PAINT    => {
            let mut pstruct = mem::uninitialized();
            let hdc = user32::BeginPaint(hwnd, &mut pstruct);

            user32::FillRect(hdc, &pstruct.rcPaint, gdi32::CreateSolidBrush(0x000000));

            user32::EndPaint(hwnd, &pstruct);
            0
        }

        winapi::WM_DESTROY  => {
            use event::Event::Closed;

            CALLBACK_DATA.with(|data| {
                let mut vector = data.borrow_mut();

                let mut vector = match *vector {
                    Some(ref mut v) => v,
                    None        => return
                };

                match get_window_index(vector, hwnd) {
                    -1  => (),
                    i   => {vector.remove(i as usize).sender.send(Closed).ok();}
                }
            });

            user32::DestroyWindow(hwnd);
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