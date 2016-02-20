use winapi;
use user32;
use kernel32;
use gdi32;

use winapi::{UINT, DWORD, WPARAM, LPARAM};
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

use CursorType;
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
                    style |= winapi::WS_CAPTION;

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
            

            let size = match config.size {
                Some(s) => {
                    let mut size_rect = winapi::RECT {
                        left: 0,
                        top: 0,
                        right: s.0,
                        bottom: s.1
                    };

                    user32::AdjustWindowRectEx(&mut size_rect, style, 0, style_ex);
                    (size_rect.right - size_rect.left, size_rect.bottom - size_rect.top)
                }

                None => (winapi::CW_USEDEFAULT, winapi::CW_USEDEFAULT)
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

    #[inline]
    pub fn focus(&self) {
        unsafe {
            // The SetFocus method does not work across threads, but SendMessage does. As
            // such, this sends a message to the corresponding window where the focus is set
            // in the callback, which is executed in the correct thread.
            user32::SendMessageW(self.0, MSG_GAINFOCUS, 0, 0);
        }
    }

    #[inline]
    pub fn get_inner_position(&self) -> Option<(i32, i32)> {
        use winapi::POINT;

        unsafe {
            let mut point = POINT {
                x: 0,
                y: 0
            };

            match user32::ClientToScreen(self.0, &mut point) {
                0 => None,
                _ => Some((point.x as i32, point.y as i32))
            }
        }
    }

    #[inline]
    pub fn get_outer_position(&self) -> Option<(i32, i32)> {
        unsafe {
            let mut rect = mem::uninitialized();

            match user32::GetWindowRect(self.0, &mut rect) {
                0 => None,
                _ => Some((rect.left as i32, rect.top as i32))
            }
        }
    }

    #[inline]
    pub fn get_inner_size(&self) -> Option<(u32, u32)> {
        unsafe {
            let mut rect = mem::uninitialized();
            
            match user32::GetClientRect(self.0, &mut rect) {
                0 => None,
                _ => Some(((rect.right - rect.left) as u32, 
                           (rect.bottom - rect.top) as u32))
            }
        }
    }

    #[inline]
    pub fn get_outer_size(&self) -> Option<(u32, u32)> {
        unsafe {
            let mut rect = mem::uninitialized();
            
            match user32::GetWindowRect(self.0, &mut rect) {
                0 => None,
                _ => Some(((rect.right - rect.left) as u32, 
                           (rect.bottom - rect.top) as u32))
            }
        }
    }

    #[inline]
    pub fn set_position(&self, x: i32, y: i32) -> Option<()> {
        unsafe {
            let result = user32::SetWindowPos(
                self.0,
                ptr::null_mut(),
                x,
                y,
                0,
                0,
                winapi::SWP_NOSIZE | winapi::SWP_NOZORDER | winapi::SWP_NOACTIVATE
            );

            match result {
                0 => None,
                _ => Some(())
            }
        }
    }

    #[inline]
    pub fn set_inner_size(&self, x: u32, y: u32) -> Option<()> {
        unsafe {
            let mut rect = winapi::RECT {
                left: 0,
                top: 0,
                right: x as i32,
                bottom: y as i32
            };

            user32::AdjustWindowRectEx(
                &mut rect,
                user32::GetWindowLongW(self.0, -16) as u32, // Get the window's style
                0,
                user32::GetWindowLongW(self.0, -20) as u32  // Get the window's extended style
            );

            let result = user32::SetWindowPos(
                self.0,
                ptr::null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                winapi::SWP_NOMOVE | winapi::SWP_NOZORDER | winapi::SWP_NOACTIVATE
            );

            match result {
                0 => None,
                _ => Some(())
            }
        }
    }

    pub fn is_active(&self) -> bool {
        unsafe {
            let mut is_active = false;

            user32::SendMessageW(self.0, MSG_ISACTIVEWIN, &mut is_active as *mut bool as WPARAM, 0);
            is_active
        }
    }

    #[inline]
    pub fn set_cursor(&self, cursor_type: CursorType) {
        use CursorType::*;

        let cursor_type = match cursor_type {
            AppStarting     => winapi::IDC_APPSTARTING,
            Arrow           => winapi::IDC_ARROW,
            Crosshair       => winapi::IDC_CROSS,
            Hand            => winapi::IDC_HAND,
            Help            => winapi::IDC_HELP,
            Text            => winapi::IDC_IBEAM,
            Prohibited      => winapi::IDC_NO,
            ResizeAll       => winapi::IDC_SIZEALL,
            ResizeNESW      => winapi::IDC_SIZENESW,
            ResizeNWSE      => winapi::IDC_SIZENWSE,
            ResizeVertical  => winapi::IDC_SIZENS,
            ResizeHoriz     => winapi::IDC_SIZEWE,
            UpArrow         => winapi::IDC_UPARROW,
            Wait            => winapi::IDC_WAIT,
            Invisible       => ptr::null()
        };

        unsafe {
            let cursor = user32::LoadCursorW(ptr::null_mut(), cursor_type);
            user32::SendMessageW(self.0, MSG_SETCURSOR, cursor as u64, 0);
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
        style: winapi::CS_OWNDC | winapi::CS_VREDRAW | winapi::CS_HREDRAW | winapi::CS_DBLCLKS,
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



thread_local!(pub static CALLBACK_DATA: RefCell<Option<CallbackData>> = RefCell::new(None));

pub struct CallbackData {
    win_vec: Vec<WindowDataIntern>,
    /// A cached index so that the program does not have to search through all of the
    /// window vertex to get the proper window information
    win_index: usize,
    win_sender: Sender<WindowData>,
    /// The last position of the mouse. This is used to catch duplicate WM_MOUSEHOVER
    /// messages.
    last_mpos: LPARAM
}

impl CallbackData {
    #[inline]
    pub fn new(vec_window: HWND, event_sender: Sender<Event>, sender: Sender<WindowData>) -> CallbackData {
        let mut data_vector = Vec::with_capacity(4);
        data_vector.push(WindowDataIntern::new(vec_window, event_sender));

        CallbackData {
            win_vec: data_vector,
            win_index: 0,
            win_sender: sender,
            last_mpos: LPARAM::max_value()
        }
    }

    fn get_window_index(&mut self, window: HWND) -> isize {
        // If the cached index isn't less than the vector length, it cannot be valid.
        // If the cached index IS less, we can check to see if the window handle at that
        // index is equal to the given handle.
        if self.win_index < self.win_vec.len() &&
           self.win_vec[self.win_index].window == window {

            return self.win_index as isize;
        }

        let mut index = self.win_vec.len();

        if index != 0 {
            while index != 0 {
                index -= 1;

                if self.win_vec[index].window == window {
                    self.win_index = index;
                    return index as isize;
                }
            }
        }

        -1
    }
}

/// Struct that contains information about the window internal to the callback.
/// Stuff like the raw window and the event sender are only used by the callback
/// function, and as such they do not need to be exposed. 
struct WindowDataIntern {
    window: HWND,
    sender: Sender<Event>,
    cursor: winapi::HCURSOR
}

impl WindowDataIntern {
    #[inline]
    fn new(window: HWND, sender: Sender<Event>) -> WindowDataIntern {
        WindowDataIntern {
            window: window,
            sender: sender,
            cursor: unsafe{ user32::LoadCursorW(ptr::null_mut(), winapi::IDC_ARROW) }
        }
    }
}

pub struct WindowData( pub InternalWindow, pub Receiver<Event> );


pub const MSG_NEWOWNEDWINDOW: UINT = 0xADD;
pub const MSG_GAINFOCUS: UINT = 71913;
pub const MSG_SETCURSOR: UINT = 32118;
pub const MSG_ISACTIVEWIN: UINT = 0xAC20;

fn send_event(source: HWND, event: Event) {
    CALLBACK_DATA.with(|data| {
        let mut data = data.borrow_mut();

        let (index, vector) = match *data {
            Some(ref mut d) => (d.get_window_index(source), &d.win_vec),
            None => return
        };

        match index {
            -1  => (),
            i   => {vector[i as usize].sender.send(event).ok();}
        }
    });
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

        winapi::WM_SETCURSOR=> {
            CALLBACK_DATA.with(|data| {
                let mut data = data.borrow_mut();
                let (index, data) = match *data {
                    Some(ref mut d) => (d.get_window_index(hwnd) as usize, d),
                    _        => return 1
                };

                if data.last_mpos != LPARAM::max_value() {
                    user32::SetCursor(data.win_vec[index].cursor);
                    0
                }
                else {
                    user32::DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            })
        }

        winapi::WM_LBUTTONDOWN  => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Single, Left));

            0
        }

        winapi::WM_LBUTTONUP    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Released, Left));

            0
        }

        winapi::WM_LBUTTONDBLCLK    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Double, Left));

            0
        }

        winapi::WM_RBUTTONDOWN  => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Single, Right));

            0
        }

        winapi::WM_RBUTTONUP    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Released, Right));

            0
        }

        winapi::WM_RBUTTONDBLCLK    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Double, Right));

            0
        }

        winapi::WM_MBUTTONDOWN  => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Single, Middle));

            0
        }

        winapi::WM_MBUTTONUP    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Released, Middle));

            0
        }

        winapi::WM_MBUTTONDBLCLK    => {
            use event::ClickType;
            use event::MButton::*;

            send_event(hwnd, Event::MButtonInput(ClickType::Double, Middle));

            0
        }

        winapi::WM_XBUTTONDOWN  => {
            use event::ClickType;
            use event::MButton::*;

            match wparam >> 16 & 0xFFFF {
                1 => send_event(hwnd, Event::MButtonInput(ClickType::Single, Button4)),
                2 => send_event(hwnd, Event::MButtonInput(ClickType::Single, Button5)),
                _ => panic!("A new mouse button approaches...")
            }

            0
        }

        winapi::WM_XBUTTONDBLCLK    => {
            use event::ClickType;
            use event::MButton::*;

            match wparam >> 16 & 0xFFFF {
                1 => send_event(hwnd, Event::MButtonInput(ClickType::Double, Button4)),
                2 => send_event(hwnd, Event::MButtonInput(ClickType::Double, Button5)),
                _ => panic!("A new mouse button approaches...")
            }

            0
        }

        winapi::WM_XBUTTONUP    => {
            use event::ClickType;
            use event::MButton::*;

            match wparam >> 16 & 0xFFFF {
                1 => send_event(hwnd, Event::MButtonInput(ClickType::Released, Button4)),
                2 => send_event(hwnd, Event::MButtonInput(ClickType::Released, Button5)),
                _ => panic!("A new mouse button approaches...")
            }

            0
        }

        winapi::WM_MOUSEMOVE    => {
            CALLBACK_DATA.with(|data| {
                
                let mpos = {
                    let mut data = data.borrow_mut();
                    match *data {
                        Some(ref mut d) => {
                            let ret = d.last_mpos;
                            d.last_mpos = lparam;
                            ret
                        }

                        None => return 1
                    }
                };
                

                if mpos == LPARAM::max_value() {
                    send_event(hwnd, Event::MouseEnter);
                }

                if mpos != lparam {
                    let mut mouse_track = winapi::TRACKMOUSEEVENT {
                        cbSize: mem::size_of::<winapi::TRACKMOUSEEVENT>() as DWORD,
                        dwFlags: winapi::TME_HOVER | winapi::TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: winapi::HOVER_DEFAULT
                    };

                    user32::TrackMouseEvent(&mut mouse_track);
                    send_event(hwnd, Event::MouseMoved(lparam as i32 >> 16, lparam as i16 as i32));

                    0
                }
                else {1}
            })
        }

        winapi::WM_MOUSELEAVE   => {
            CALLBACK_DATA.with(|data| {
                let mut data = data.borrow_mut();
                match *data {
                    Some(ref mut d) => d.last_mpos = LPARAM::max_value(),
                    None => return
                }
            });

            send_event(hwnd, Event::MouseLeave);

            0
        }

        winapi::WM_MOUSEHOVER   => {
            send_event(hwnd, Event::MouseHover(lparam as i32 >> 16, lparam as i16 as i32));

            0
        }

        winapi::WM_SIZE     => {
            use event::ResizeType::*;

            let resize_type = match wparam {
                1 => Minimized,
                2 => Maximized,
                0 => Changed,
                _ => panic!("Unhandled popup window resize; open error on github with the breaking code")
            };

            send_event(hwnd, Event::Resized(resize_type, lparam as u32 >> 16, lparam as u32 & 0xFFFF));
            0
        }

        winapi::WM_MOVE     => {
            send_event(hwnd, Event::Moved(lparam as i32 >> 16, lparam as i16 as i32));

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

        MSG_SETCURSOR       => {
            let cursor = wparam as *mut _;

            CALLBACK_DATA.with(|data| {
                let mut data = data.borrow_mut();
                match *data {
                    Some(ref mut d) => {
                        let index = d.get_window_index(hwnd) as usize;
                        d.win_vec[index].cursor = cursor;

                        if hwnd == user32::GetActiveWindow() {
                            user32::SetCursor(cursor);
                        }

                        Some(())
                    }

                    None => None
                }
            });

            0
        }

        MSG_GAINFOCUS       => {
            user32::SetFocus(hwnd);

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
                        Some(ref mut v) => &mut v.win_vec,
                        None            => return
                    };

                    vector.push(WindowDataIntern::new(internal_window.0, tx));
                }
                
                // Get a reference to the window data sender
                let sender = match *data {
                    Some(ref r) => &r.win_sender,
                    None        => return
                };

                sender.send(WindowData(internal_window, rx)).ok();
            });

            0
        }

        MSG_ISACTIVEWIN => {
            if hwnd == user32::GetActiveWindow() {
                *(wparam as *mut bool) = true;
            }
            else {
                *(wparam as *mut bool) = false;
            }

            0
        }

        winapi::WM_DESTROY  => {
            use event::Event::Closed;

            CALLBACK_DATA.with(|data| {
                let mut data = data.borrow_mut();

                let (index, mut vector) = match *data {
                    Some(ref mut d) => (d.get_window_index(hwnd), &mut d.win_vec),
                    None        => return
                };

                // If this window's information is still in the vector, remove it from
                // the vector and send the closed message for this window. 
                match index {
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