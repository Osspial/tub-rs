pub mod wrapper;
pub mod os;
use self::wrapper::{WindowWrapper, CallbackData, WindowData, CALLBACK_DATA};

use winapi;
use user32;

use std::ptr;
use std::mem;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use error::TubResult;
use config::{WindowConfig, PixelFormat};
use event::Event;

enum ReceiverTagged<'o> {
    Owned(Receiver<TubResult<WindowData>>),
    Borrowed(&'o Receiver<TubResult<WindowData>>)
}

impl<'o> ReceiverTagged<'o> {
    fn get_ref(&'o self) -> &'o Receiver<TubResult<WindowData>> {
        use self::ReceiverTagged::*;

        match *self {
            Owned(ref r)    => r,
            Borrowed(r)     => r
        }
    }
}

pub struct Window<'o> {
    pub wrapper: WindowWrapper,
    event_receiver: Receiver<Event>,
    window_receiver: ReceiverTagged<'o>,
    owner: Option<&'o Window<'o>>,
    /// Used when setting the pixel format on context creation
    config: WindowConfig,
    pixel_format: PixelFormat
}

impl<'o> Window<'o> {
    /// Create a new window with the specified title and config
    pub fn new<'a>(name: &'a str, config: WindowConfig, pixel_format: PixelFormat) -> TubResult<Window<'o>> {
        // Channel for the handle to the window
        let (tx, rx) = mpsc::channel();
        let config = Arc::new(config);

        let name_raw = RawSlice(name.as_ptr(), name.len());

        let config_arc = config.clone();
        thread::spawn(move || {
            unsafe {
                use std::{slice, str};

                let name = str::from_utf8(
                    slice::from_raw_parts(name_raw.0, name_raw.1)
                ).unwrap();

                let wrapper_window = WindowWrapper::new(name, &config_arc, None);
                mem::drop(config_arc);

                // Event channel
                let (sx, rx) = mpsc::channel();

                match wrapper_window {
                    Ok(wr) => {
                        CALLBACK_DATA.with(move |sender| {
                            let callback_data = CallbackData::new(wr.0, sx, tx.clone());

                            tx.send(Ok(WindowData(wr, rx))).unwrap();
                            *sender.borrow_mut() = Some(callback_data);
                        });
                    }

                    Err(e) => {
                        tx.send(Err(e)).unwrap();
                        panic!("Window creation error: see sent result for details");
                    }
                }
                

                let mut msg = mem::uninitialized();

                while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    user32::TranslateMessage(&msg);
                    user32::DispatchMessageW(&msg);
                }
            }
        });

        let WindowData(wrapper_window, receiver) = try!(rx.recv().unwrap());

        Ok(
            Window {
                wrapper: wrapper_window,
                event_receiver: receiver,
                window_receiver: ReceiverTagged::Owned(rx),
                owner: None,
                config: Arc::try_unwrap(config).unwrap(),
                pixel_format: pixel_format
            }
        )
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
    pub fn new_owned<'a>(&'o self, name: &'a str, config: WindowConfig, pixel_format: PixelFormat) -> TubResult<Window<'o>> {
        unsafe {
            user32::SendMessageW(self.wrapper.0, wrapper::MSG_NEWOWNEDWINDOW, &name as *const _ as winapi::WPARAM, &config as *const _ as winapi::LPARAM);

            let win_data = try!(self.window_receiver.get_ref().recv().unwrap());

            Ok(
                Window {
                    wrapper: win_data.0,
                    event_receiver: win_data.1,
                    window_receiver: ReceiverTagged::Borrowed(self.window_receiver.get_ref()),
                    owner: Some(self),
                    config: config.clone(),
                    pixel_format: pixel_format
                }
            )
        }
    }

    /// Sets the cursor position relative to window space. Note that this will fail in a few
    /// conditions:
    ///
    /// * If the window isn't currently active
    /// * If the cursor is outside of the window's client area
    #[inline]
    pub fn set_cursor_pos(&self, x: i32, y: i32) {
        let cursor_in_client = {
            let size = match self.wrapper.get_inner_size() {
                Some(s) => (s.0 as i32, s.1 as i32),
                None    => return
            };
            let (cx, cy) = self::os::get_cursor_pos();

            let (xmin, ymin) = self.wrapper.get_inner_pos().unwrap();
            let (xmax, ymax) = (xmin + size.0, ymin + size.1);

            xmin < cx && cx < xmax &&
            ymin < cy && cy < ymax
        };


        if self.wrapper.is_active() && cursor_in_client {
            let pos = self.wrapper.get_inner_pos().unwrap();

            self::os::set_cursor_pos(x + pos.0, y + pos.1);
        }
    }

    /// Get a reference to this window's owner, if the window is owned.
    #[inline]
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

    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    pub fn pixel_format(&self) -> &PixelFormat {
        &self.pixel_format
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

struct RawSlice (*const u8, usize);
unsafe impl Send for RawSlice {}