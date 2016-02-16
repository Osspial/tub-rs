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
use std::sync::mpsc::{Sender, Receiver};

use num::FromPrimitive;

use config::WindowConfig;
use event::{Event, VKeyCode};

#[derive(Clone)]
pub struct InternalWindow( pub HWND );

unsafe impl Send for InternalWindow {}
unsafe impl Sync for InternalWindow {}

impl InternalWindow {
    #[inline]
    pub fn new<'a>(name: String, config: WindowConfig, owner: Option<HWND>) -> InternalWindow {
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
                // This parameter specifies the window's owner. If the window
                // is unowned, then it passes a null pointer to the parameter.
                owner.unwrap_or(ptr::null_mut()),
                ptr::null_mut(),
                kernel32::GetModuleHandleW(ptr::null()),
                ptr::null_mut()
            );

            if window_handle == ptr::null_mut() {
                panic!(format!("Error: {}", ::std::io::Error::last_os_error()));
            }

            // If the window should be borderless, make it borderless
            if config.borderless {
                user32::SetWindowLongW(window_handle, -16, 0);
            }

            if let Some(p) = config.icon {
                let path = wide_path(p).as_ptr();

                // Load the 32x32 icon
                let icon = user32::LoadImageW(ptr::null_mut(), path, winapi::IMAGE_ICON, 32, 32, winapi::LR_LOADFROMFILE);
                if icon != ptr::null_mut() {
                    user32::SendMessageW(window_handle, winapi::WM_SETICON, winapi::ICON_BIG as u64, icon as winapi::LPARAM);
                }
                else {
                    panic!("Could not load 32x32 icon (TODO: Make this not panic)");
                }

                // Load the 16x16 icon
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

    #[inline]
    pub fn enable(&self) {
        unsafe {
            user32::EnableWindow(self.0, winapi::TRUE);
        }
    }

    #[inline]
    pub fn disable(&self) {
        unsafe {
            user32::EnableWindow(self.0, winapi::FALSE);
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



thread_local!(pub static CALLBACK_DATA: RefCell<Option<(Vec<CallbackData>, Sender<WindowData>)>> = RefCell::new(None));

pub struct CallbackData {
    pub window: HWND,
    pub sender: Sender<Event>
}

pub struct WindowData( pub InternalWindow, pub Receiver<Event> );

impl CallbackData {
    #[inline]
    pub fn new(window: HWND, sender: Sender<Event>) -> CallbackData {
        CallbackData {
            window: window,
            sender: sender
        }
    }
}

pub const MSG_NEWOWNEDWINDOW: UINT = 0xADD;

fn send_event(source: HWND, event: Event) {
    CALLBACK_DATA.with(|data| {
        let data = data.borrow();

        let vector = match *data {
            Some(ref d) => &d.0,
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

        // Currently only draws black.
        // TODO: Make it actually draw shit
        winapi::WM_PAINT    => {
            let mut pstruct = mem::uninitialized();
            let hdc = user32::BeginPaint(hwnd, &mut pstruct);

            user32::FillRect(hdc, &pstruct.rcPaint, gdi32::CreateSolidBrush(0x000000));

            user32::EndPaint(hwnd, &pstruct);
            0
        }

        MSG_NEWOWNEDWINDOW  => {
            use std::mem::transmute;
            use std::sync::mpsc;

            // For this message, pointers to the name and window config are stored in the
            // WPARAM and LPARAM parameters, respectively. This turns them into proper pointers
            // and gets the objects from the pointers.
            let name = (*transmute::<WPARAM, &&str>(wparam)).to_string();
            let config: WindowConfig = transmute::<LPARAM, &WindowConfig>(lparam).clone();

            let internal_window = InternalWindow::new(name, config, Some(hwnd));
            let (tx, rx) = mpsc::channel();

            CALLBACK_DATA.with(|data| {
                let mut data = data.borrow_mut();

                {
                    // This block of code gets a reference to the vector of windows and pushes information
                    // about new window to the top of the vector

                    let mut vector = match *data {
                        Some(ref mut v) => &mut v.0,
                        None            => return
                    };

                    vector.push(CallbackData::new(internal_window.0, tx));
                }
                
                // Get a reference to the window data sender
                let sender = match *data {
                    Some(ref r) => &r.1,
                    None        => return
                };

                sender.send(WindowData(internal_window, rx)).ok();
            });

            0
        }

        winapi::WM_DESTROY  => {
            use event::Event::Closed;

            CALLBACK_DATA.with(|data| {
                let mut vector = data.borrow_mut();

                let mut vector = match *vector {
                    Some(ref mut v) => &mut v.0,
                    None        => return
                };

                // If this window's information is still in the vector, remove it from
                // the vector and send the closed message for this window. 
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