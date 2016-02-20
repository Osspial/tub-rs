mod internal;
pub mod os;
use self::internal::{InternalWindow, CallbackData, WindowData, CALLBACK_DATA};

use user32;

use std::ptr;
use std::mem;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use CursorType;
use config::WindowConfig;
use event::Event;

enum ReceiverTagged<'o> {
    Owned(Receiver<WindowData>),
    Borrowed(&'o Receiver<WindowData>)
}

impl<'o> ReceiverTagged<'o> {
    fn get_ref(&'o self) -> &'o Receiver<WindowData> {
        use self::ReceiverTagged::*;

        match *self {
            Owned(ref r)    => r,
            Borrowed(r)     => r
        }
    }
}

pub struct Window<'o> {
    internal: InternalWindow,
    event_receiver: Receiver<Event>,
    window_receiver: ReceiverTagged<'o>,
    owner: Option<&'o Window<'o>>
}

impl<'o> Window<'o> {
    /// Create a new window with the specified title and config
    pub fn new<'a>(name: &'a str, config: WindowConfig) -> Window<'o> {
        // Channel for the handle to the window
        let (tx, rx) = mpsc::channel();
        let name = name.into();

        thread::spawn(move || {
            unsafe {
                let internal_window = InternalWindow::new(name, config, None);

                // Event channel
                let (sx, rx) = mpsc::channel();

                CALLBACK_DATA.with(move |sender| {
                    let callback_data = CallbackData::new(internal_window.0, sx, tx.clone());

                    tx.send(WindowData(internal_window, rx)).unwrap();
                    *sender.borrow_mut() = Some(callback_data);
                });

                let mut msg = mem::uninitialized();

                while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        });

        let WindowData(internal_window, receiver) = rx.recv().unwrap();

        Window {
            internal: internal_window,
            event_receiver: receiver,
            window_receiver: ReceiverTagged::Owned(rx),
            owner: None
        }
    }

    /// Creates a window that is owned by the calling window.
    /// 
    /// # What is different about an owned window?
    /// 
    /// Unowned windows and owned windows are quite similar, but there are a few
    /// major differences:
    /// 
    /// * Owned windows do not appear on the taskbar
    /// * Owned windows cannot live longer than their owner
    /// * Owned windows are always drawn in front of their owner
    /// * Creating an owned window does not create a new thread
    /// 
    /// The last point is mostly related to how tub handles windows internally -
    /// when creating a new unowned window, tub spins up a thread to handle receiving
    /// input from the window in a way that does not block the main program's execution.
    /// Owned windows, however, share a thread with their owner. 
    pub fn new_owned<'a>(&'o self, name: &'a str, config: WindowConfig) -> Window<'o> {
        use std::mem::transmute;

        unsafe {
            user32::SendMessageW(self.internal.0, internal::MSG_NEWOWNEDWINDOW, transmute(&name), transmute(&config));

            let win_data = self.window_receiver.get_ref().recv().unwrap();

            Window {
                internal: win_data.0,
                event_receiver: win_data.1,
                window_receiver: ReceiverTagged::Borrowed(self.window_receiver.get_ref()),
                owner: Some(self)
            }
        }
    }

    pub fn set_title(&self, title: &str) {
        self.internal.set_title(title);
    }

    #[inline]
    pub fn show(&self) {
        self.internal.show();
    }

    #[inline]
    pub fn hide(&self) {
        self.internal.hide();
    }

    /// Allow the window to take user input. Any newly created window defaults to
    /// being enabled.
    #[inline]
    pub fn enable(&self) {
        self.internal.enable();
    }

    /// Disallow the window from taking user input.
    #[inline]
    pub fn disable(&self) {
        self.internal.disable();
    }

    /// Sets input focus to this window
    pub fn focus(&self) {
        self.internal.focus();
    }

    pub fn get_inner_position(&self) -> Option<(i32, i32)> {
        self.internal.get_inner_position()
    }

    /// Gets the position of the upper-left corner of the window, including the title bar
    pub fn get_outer_position(&self) -> Option<(i32, i32)> {
        self.internal.get_outer_position()
    }

    pub fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.internal.get_inner_size()
    }

    pub fn get_outer_size(&self) -> Option<(u32, u32)> {
        self.internal.get_outer_size()
    }

    pub fn set_position(&self, x: i32, y: i32) -> Option<()> {
        self.internal.set_position(x, y)
    }

    pub fn set_inner_size(&self, x: u32, y: u32) -> Option<()> {
        self.internal.set_inner_size(x, y)
    }

    pub fn is_active(&self) -> bool {
        self.internal.is_active()
    }

    pub fn set_cursor(&self, cursor_type: CursorType) {
        self.internal.set_cursor(cursor_type);
    }

    pub fn set_cursor_pos(&self, x: i32, y: i32) {
        let cursor_in_client = {
            let size = match self.get_inner_size() {
                Some(s) => (s.0 as i32, s.1 as i32),
                None    => return
            };
            let (cx, cy) = self::os::get_cursor_pos();

            let (xmin, ymin) = self.get_inner_position().unwrap();
            let (xmax, ymax) = (xmin + size.0, ymin + size.1);

            xmin < cx && cx < xmax &&
            ymin < cy && cy < ymax
        };


        if self.is_active() && cursor_in_client {
            let pos = self.get_inner_position().unwrap();

            self::os::set_cursor_pos(x + pos.0, y + pos.1);
        }
    }

    /// Get a reference to this window's owner, if the window is owned.
    pub fn owner(&self) -> Option<&Window> {
        self.owner.clone()
    }

    /// Get a non-blocking iterator over the window's events
    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            window: self
        }
    }

    /// Get a blocking iterator over the window's events
    pub fn wait_events(&self) -> WaitEventsIter {
        WaitEventsIter {
            window: self
        }
    }
}

pub struct PollEventsIter<'w> {
    window: &'w Window<'w>
}

impl<'w> Iterator for PollEventsIter<'w> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.window.event_receiver.try_recv().ok()
    }
}

pub struct WaitEventsIter<'w> {
    window: &'w Window<'w>
}

impl<'w> Iterator for WaitEventsIter<'w> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.window.event_receiver.recv().ok()
    }
}