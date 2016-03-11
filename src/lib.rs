extern crate num;
extern crate rand;
extern crate winapi;
extern crate user32;
extern crate gdi32;
extern crate kernel32;
extern crate dwmapi;

mod api;
pub mod platform;
pub mod event;
pub mod config;
pub mod error;

use platform::Window;

use std::fmt;
use std::ops::Deref;


pub enum CursorType {
    AppStarting,
    Arrow,
    Crosshair,
    Hand,
    Help,
    Text,
    Prohibited,
    ResizeAll,
    ResizeNESW,
    ResizeNWSE,
    ResizeVertical,
    ResizeHoriz,
    UpArrow,
    Wait,
    Invisible
}

#[derive(Clone)]
pub enum WindowType<'p> {
    /// An owned window. This type of window is always drawn on top of it's owner.
    Owned(&'p Window<'p>),
    /// A child window. This type of window is drawn *inside* of the parent window's client.
    Child(&'p Window<'p>),
    /// A top-level window. This type of window is beholden to no-one.
    Top
}

impl<'p> Deref for WindowType<'p> {
    type Target = Option<&'p Window<'p>>;

    fn deref<'a>(&'a self) -> &'a Option<&'p Window<'p>> {
        use WindowType::*;
        use std::ptr;
        use std::mem;

        unsafe {
            // Now, you might be looking at this and wondering: why? Why return a transmuted reference
            // to a pointer? Well, we can't resturn &Some(w) because we're defining Some at the same
            // time we're returning a reference to it and the compiler rightfully blows up in our face.
            // HOWEVER - what we are returning is an Option that contains a pointer. Because the Option
            // contains a pointer instead of an actual struct, rustc performs a null pointer optimization
            // on the Option defining None to just be a null pointer and Some to just be the pointer,
            // meaning that `Some(&'p Window<'p>)` and `&'p Window<'p>` are *completely identical.* (You
            // can read more about the null pointer optimization here: https://doc.rust-lang.org/nomicon/repr-rust.html)
            // Because they're identical, we can just get a reference to the window pointer and transmute
            // it to a reference to an option. In the instance that we need to return a None, we can just
            // get a reference to a null pointer (which works because ptr::null() is a const function)
            // and transmute that to a reference to None.
            match *self {
                Owned(w) |
                Child(w) => mem::transmute(&(w as *const _)),
                Top      => mem::transmute(&ptr::null::<()>())
            }
        }
    }
}

impl<'p> fmt::Debug for WindowType<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use WindowType::*;

        match *self {
            Owned(w) |
            Child(w) => 
                f.debug_tuple(
                        match *self {
                            Owned(_) => "Owned",
                            Child(_) => "Child",
                            Top      => unreachable!()
                        })
                    .field(&(w as *const _))
                    .finish(),
            Top     => write!(f, "Top")
        }
    }
}