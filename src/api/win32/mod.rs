mod internal;
use self::internal::{InternalWindow, EVENT_SENDER};

use user32;

use std::ptr;
use std::mem;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use config::WindowConfig;
use event::Event;

pub struct Window {
    internal: InternalWindow,
    event_receiver: Receiver<Event>,
}

impl Window {
    pub fn new<'a>(name: &'a str, config: WindowConfig) -> Window {
        // Channel for the handle to the window
        let (tx, rx) = mpsc::channel();
        let name = name.into();

        thread::spawn(move || {
            unsafe {
                // Event channel
                let (sx, rx) = mpsc::channel();
                EVENT_SENDER.with(|sender| {
                    *sender.borrow_mut() = Some(sx);
                });

                let internal_window = InternalWindow::new(name, config);
                tx.send((internal_window, rx)).unwrap();

                let mut msg = mem::uninitialized();

                while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        });

        let (internal_window, reciever) = rx.recv().unwrap();

        Window {
            internal: internal_window,
            event_receiver: reciever
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
    window: &'w Window
}

impl<'w> Iterator for PollEventsIter<'w> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.window.event_receiver.try_recv().ok()
    }
}

pub struct WaitEventsIter<'w> {
    window: &'w Window
}

impl<'w> Iterator for WaitEventsIter<'w> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.window.event_receiver.recv().ok()
    }
}