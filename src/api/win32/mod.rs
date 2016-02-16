mod internal;
use self::internal::{InternalWindow, CallbackData, WindowData, CALLBACK_DATA};

use user32;

use std::ptr;
use std::mem;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

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

                CALLBACK_DATA.with(|sender| {
                    let mut data_vector = Vec::with_capacity(4);
                    data_vector.push(CallbackData::new(internal_window.0, sx));

                    tx.send(WindowData(internal_window, rx)).unwrap();
                    *sender.borrow_mut() = Some((data_vector, tx));
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

    pub fn new_owned_window<'a>(&'o self, name: &'a str, config: WindowConfig) -> Window<'o> {
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

    pub fn poll_events(&self) -> PollEventsIter {
        PollEventsIter {
            window: self
        }
    }

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